//! State-changing operations: assign profiles, install/remove hooks,
//! import pre-existing audio hooks.

use anyhow::{bail, Context, Result};

use crate::aura::{self, Agent};
use crate::config::{Config, OFF};
use crate::paths::self_invocation_path;
use crate::settings::{extract_source, Settings};

fn shell_quote(s: &str) -> String {
    if s.chars().all(|c| c.is_ascii_alphanumeric() || "-_./".contains(c)) {
        s.to_string()
    } else {
        format!("'{}'", s.replace('\'', r#"'\''"#))
    }
}

fn hook_command(agent: &str, event: &str) -> String {
    format!(
        "{} play --agent {} --event {}",
        shell_quote(&self_invocation_path().to_string_lossy()),
        shell_quote(agent),
        shell_quote(event),
    )
}

/// Install (or refresh) managed hook entries for one agent. Hooks are
/// installed for the union of events across all profiles, so switching
/// profiles later is config-only.
pub fn enable(config: &Config, agent: &Agent) -> Result<Vec<String>> {
    if !agent.supports_hooks() {
        bail!("agent '{}' has kind '{}'; only claude-code hooks are supported", agent.name, agent.kind);
    }
    let events = config.all_events();
    if events.is_empty() {
        bail!("no profiles define any events yet; add one with: profile set <name> <event> <path>");
    }
    let mut settings = Settings::load(&agent.settings_path())?;
    settings.remove_managed();
    for event in &events {
        settings.add_managed(event, &hook_command(&agent.name, event));
    }
    settings.save()?;
    Ok(events.into_iter().collect())
}

/// Remove every managed hook entry for one agent.
pub fn disable(agent: &Agent) -> Result<usize> {
    let mut settings = Settings::load(&agent.settings_path())?;
    let removed = settings.remove_managed();
    if removed > 0 {
        settings.save()?;
    }
    Ok(removed)
}

/// Re-install hooks for every agent that already has managed entries —
/// used after profile edits so new events get covered.
pub fn refresh_enabled(config: &Config) -> Result<Vec<String>> {
    let mut refreshed = Vec::new();
    for agent in aura::agents()? {
        if !agent.supports_hooks() {
            continue;
        }
        let settings = Settings::load(&agent.settings_path())?;
        if settings.has_managed() && !config.all_events().is_empty() {
            enable(config, &agent)?;
            refreshed.push(agent.name.clone());
        }
    }
    Ok(refreshed)
}

/// Assign a profile (or "off") to an agent.
pub fn use_profile(config: &mut Config, agent_name: &str, profile: &str) -> Result<()> {
    let agent = aura::find_agent(agent_name)?;
    if profile != OFF && !config.profiles.contains_key(profile) {
        let known: Vec<_> = config.profiles.keys().map(String::as_str).collect();
        bail!("unknown profile '{profile}' (profiles: {}, or 'off')", known.join(", "));
    }
    config.agents.insert(agent.name, profile.to_string());
    config.save()
}

pub struct ImportOutcome {
    pub profile: String,
    pub imported: Vec<(String, String)>, // (event, source)
    pub skipped: Vec<(String, String)>,  // (event, command we couldn't parse)
    pub events_installed: Vec<String>,
}

/// Adopt an agent's existing raw audio hooks: build/extend a profile from
/// them, delete the raw entries (after backing up settings.json), assign
/// the profile, and install managed hooks in their place.
pub fn import(config: &mut Config, agent_name: &str, profile_name: &str) -> Result<ImportOutcome> {
    let agent = aura::find_agent(agent_name)?;
    if !agent.supports_hooks() {
        bail!("agent '{}' has kind '{}'; only claude-code hooks are supported", agent.name, agent.kind);
    }
    let mut settings = Settings::load(&agent.settings_path())?;
    let candidates = settings.audio_candidates();
    if candidates.is_empty() {
        bail!(
            "no unmanaged audio hooks found in {}",
            agent.settings_path().display()
        );
    }

    let mut imported = Vec::new();
    let mut skipped = Vec::new();
    let mut imported_cmds = Vec::new();
    for (event, command) in candidates {
        match extract_source(&command) {
            Some(source) => {
                imported.push((event, source));
                imported_cmds.push(command);
            }
            None => skipped.push((event, command)),
        }
    }
    if imported.is_empty() {
        bail!("found audio hooks but could not extract a source path from any of them");
    }

    let profile = config
        .profiles
        .entry(profile_name.to_string())
        .or_default();
    for (event, source) in &imported {
        profile.events.insert(event.clone(), source.clone());
    }
    config.agents.insert(agent.name.clone(), profile_name.to_string());
    config.save()?;

    // Remove exactly the raw commands we imported, nothing else.
    settings.remove_matching(|cmd| imported_cmds.iter().any(|c| c == cmd));
    settings.save().context("removing imported raw hooks")?;

    let events_installed = enable(config, &agent)?;
    Ok(ImportOutcome { profile: profile_name.to_string(), imported, skipped, events_installed })
}

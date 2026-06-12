//! Panel button actions. The aura host re-invokes us as
//! `action <id> --period <p>` when a button is clicked; we perform the
//! operation here and `main` prints the refreshed panel.
//!
//! Id grammar (':'-separated; profile names therefore must not contain
//! ':' — `sanitize_name` enforces that for names we create):
//!
//!   mute:on | mute:off
//!   agent:<agent>:<profile|off>      assign (auto-installs hooks)
//!   hooks:<agent>:remove             uninstall managed hook entries
//!   source:<profile>:<event>:dir     re-pick source via folder dialog
//!   source:<profile>:<event>:file    re-pick source via file dialog
//!   event:add:<profile>:<event>      map a new event (folder dialog)
//!   event:rm:<profile>:<event>       unmap an event
//!   profile:new                      create profile from a picked folder
//!   profile:rm:<name>                delete a profile

use anyhow::{bail, Context, Result};

use crate::aura;
use crate::config::{Config, OFF};
use crate::ops;
use crate::picker;
use crate::settings::Settings;

/// Hook events offered by the "add event" buttons.
pub const KNOWN_EVENTS: &[&str] = &["Stop", "Notification", "SubagentStop", "SessionStart"];

pub fn handle(id: &str) -> Result<()> {
    let mut config = Config::load()?;
    let parts: Vec<&str> = id.split(':').collect();
    match parts.as_slice() {
        ["mute", "on"] => {
            config.muted = true;
            config.save()
        }
        ["mute", "off"] => {
            config.muted = false;
            config.save()
        }
        ["agent", rest @ .., assign] if !rest.is_empty() => {
            let agent_name = rest.join(":");
            ops::use_profile(&mut config, &agent_name, assign)?;
            // Selecting a profile means "I want sounds": make sure the
            // hook entries exist. "off" leaves them installed but silent.
            if *assign != OFF {
                let agent = aura::find_agent(&agent_name)?;
                if !Settings::load(&agent.settings_path())?.has_managed() {
                    ops::enable(&config, &agent)?;
                }
            }
            Ok(())
        }
        ["hooks", rest @ .., "remove"] if !rest.is_empty() => {
            let agent = aura::find_agent(&rest.join(":"))?;
            ops::disable(&agent)?;
            Ok(())
        }
        ["source", profile, event, kind @ ("dir" | "file")] => {
            let title = format!("Audio for {event} ({profile})");
            let picked = match *kind {
                "dir" => picker::pick_dir(&title)?,
                _ => picker::pick_file(&title)?,
            };
            let Some(path) = picked else { return Ok(()) }; // cancelled
            set_event_source(&mut config, profile, event, path)
        }
        ["event", "add", profile, event] => {
            let Some(path) = picker::pick_dir(&format!("Audio folder for {event} ({profile})"))?
            else {
                return Ok(());
            };
            set_event_source(&mut config, profile, event, path)
        }
        ["event", "rm", profile, event] => {
            let p = config
                .profiles
                .get_mut(*profile)
                .with_context(|| format!("no profile named '{profile}'"))?;
            p.events.remove(*event);
            config.save()
        }
        ["profile", "new"] => {
            let Some(path) = picker::pick_dir("Folder of sounds for the new profile")? else {
                return Ok(());
            };
            let base = path
                .file_name()
                .map(|n| sanitize_name(&n.to_string_lossy()))
                .filter(|n| !n.is_empty())
                .unwrap_or_else(|| "profile".to_string());
            let mut name = base.clone();
            let mut i = 2;
            while config.profiles.contains_key(&name) || name == OFF {
                name = format!("{base}-{i}");
                i += 1;
            }
            config
                .profiles
                .entry(name)
                .or_default()
                .events
                .insert("Stop".to_string(), path.to_string_lossy().into_owned());
            config.save()?;
            for _ in ops::refresh_enabled(&config)? {}
            Ok(())
        }
        ["profile", "rm", name] => {
            if config.profiles.remove(*name).is_none() {
                bail!("no profile named '{name}'");
            }
            for assigned in config.agents.values_mut() {
                if assigned == name {
                    *assigned = OFF.to_string();
                }
            }
            config.save()
        }
        _ => bail!("unknown action id '{id}'"),
    }
}

fn set_event_source(
    config: &mut Config,
    profile: &str,
    event: &str,
    path: std::path::PathBuf,
) -> Result<()> {
    config
        .profiles
        .entry(profile.to_string())
        .or_default()
        .events
        .insert(event.to_string(), path.to_string_lossy().into_owned());
    config.save()?;
    // A new event may not be covered by installed hooks yet.
    for _ in ops::refresh_enabled(config)? {}
    Ok(())
}

fn sanitize_name(raw: &str) -> String {
    raw.chars()
        .map(|c| {
            if c == ':' || c.is_whitespace() {
                '-'
            } else {
                c
            }
        })
        .collect()
}

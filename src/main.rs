//! Aura plugin managing per-agent audio hook profiles.
//!
//! Two faces:
//! - Invoked by aura with `--period <p>`: prints the status panel JSON.
//! - Invoked by agent hooks with `play --agent X --event E`: plays audio.
//! - Invoked by the user with subcommands: edits profiles/assignments.

mod actions;
mod aura;
mod config;
mod ops;
mod panel;
mod paths;
mod picker;
mod play;
mod settings;

use anyhow::{bail, Context, Result};

use config::{Config, OFF};

const USAGE: &str = "\
aura-plugin-audio-hooks — per-agent audio hook profiles for aura

USAGE:
  aura-plugin-audio-hooks [--period <all|7d|30d>]   print aura panel JSON
  aura-plugin-audio-hooks status                    human-readable status
  aura-plugin-audio-hooks mute | unmute             global mute toggle (e.g. meetings)
  aura-plugin-audio-hooks use <agent> <profile|off> assign a profile to an agent
  aura-plugin-audio-hooks enable <agent>            install hook entries for an agent
  aura-plugin-audio-hooks disable <agent>           remove this plugin's hook entries
  aura-plugin-audio-hooks import <agent> [--profile <name>]
                                                    adopt existing raw audio hooks
  aura-plugin-audio-hooks profile set <name> <event> <path>
  aura-plugin-audio-hooks profile rm <name>
  aura-plugin-audio-hooks profile list
  aura-plugin-audio-hooks play --agent <a> --event <e>
                                                    (called by agent hooks)

Profiles map hook events (Stop, Notification, ...) to a directory of
audio files (random pick) or a single file. Config:
  ~/.config/aura-audio-hooks/config.toml
Only hook entries containing 'aura-plugin-audio-hooks' are ever touched
in agent settings; all other hooks are preserved.";

fn main() {
    if let Err(e) = run() {
        eprintln!("error: {e:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let cmd = args.first().map(String::as_str);
    match cmd {
        None | Some("--period") | Some("panel") => {
            println!("{}", panel::render());
            Ok(())
        }
        // Fired by aura when a panel button is clicked. Errors are
        // reported through the panel's error envelope (stdout, exit 0)
        // so the host renders them in place; the next refresh recovers.
        Some("action") => {
            let id = args.get(1).context("usage: action <id>")?;
            match actions::handle(id) {
                Ok(()) => println!("{}", panel::render()),
                Err(e) => println!("{}", panel::render_error(&format!("{e:#}"))),
            }
            Ok(())
        }
        Some("--help") | Some("-h") | Some("help") => {
            println!("{USAGE}");
            Ok(())
        }
        Some("play") => {
            let agent = flag_value(&args, "--agent").context("play requires --agent")?;
            let event = flag_value(&args, "--event").context("play requires --event")?;
            play::play(&agent, &event)
        }
        Some("status") => status(),
        Some("mute") => set_muted(true),
        Some("unmute") => set_muted(false),
        Some("use") => {
            let (agent, profile) = two_args(&args, "use <agent> <profile|off>")?;
            let mut config = Config::load()?;
            ops::use_profile(&mut config, &agent, &profile)?;
            println!("{agent} -> {profile}");
            Ok(())
        }
        Some("enable") => {
            let agent_name = one_arg(&args, "enable <agent>")?;
            let config = Config::load()?;
            let agent = aura::find_agent(&agent_name)?;
            let events = ops::enable(&config, &agent)?;
            println!(
                "installed hooks for {} in {} (events: {})",
                agent.name,
                agent.settings_path().display(),
                events.join(", ")
            );
            Ok(())
        }
        Some("disable") => {
            let agent_name = one_arg(&args, "disable <agent>")?;
            let agent = aura::find_agent(&agent_name)?;
            let removed = ops::disable(&agent)?;
            println!(
                "removed {removed} hook entries from {}",
                agent.settings_path().display()
            );
            Ok(())
        }
        Some("import") => {
            let agent = one_arg(&args, "import <agent>")?;
            anyhow::ensure!(
                !agent.starts_with('-'),
                "usage: import <agent> [--profile <name>]"
            );
            let profile = flag_value(&args, "--profile").unwrap_or_else(|| "imported".to_string());
            let mut config = Config::load()?;
            let outcome = ops::import(&mut config, &agent, &profile)?;
            println!("imported into profile '{}':", outcome.profile);
            for (event, source) in &outcome.imported {
                println!("  {event} <- {source}");
            }
            for (event, cmd) in &outcome.skipped {
                println!("  skipped {event}: could not parse source from: {cmd}");
            }
            println!(
                "hooks installed for events: {}",
                outcome.events_installed.join(", ")
            );
            Ok(())
        }
        Some("profile") => profile_cmd(&args[1..]),
        Some(other) => bail!("unknown command '{other}'\n\n{USAGE}"),
    }
}

fn profile_cmd(args: &[String]) -> Result<()> {
    let mut config = Config::load()?;
    match args.first().map(String::as_str) {
        Some("set") => {
            let [name, event, path] = &args[1..] else {
                bail!("usage: profile set <name> <event> <path>");
            };
            let expanded = paths::expand_tilde(path);
            if !expanded.exists() {
                bail!("path does not exist: {}", expanded.display());
            }
            config
                .profiles
                .entry(name.clone())
                .or_default()
                .events
                .insert(event.clone(), path.clone());
            config.save()?;
            println!("profile '{name}': {event} <- {path}");
            for agent in ops::refresh_enabled(&config)? {
                println!("refreshed hooks for {agent}");
            }
            Ok(())
        }
        Some("rm") => {
            let name = args.get(1).context("usage: profile rm <name>")?;
            if config.profiles.remove(name).is_none() {
                bail!("no profile named '{name}'");
            }
            // Anything pointing at the removed profile falls back to off.
            for assigned in config.agents.values_mut() {
                if assigned == name {
                    *assigned = OFF.to_string();
                }
            }
            config.save()?;
            println!("removed profile '{name}'");
            Ok(())
        }
        Some("list") | None => {
            if config.profiles.is_empty() {
                println!("no profiles configured");
            }
            for (name, profile) in &config.profiles {
                println!("{name}");
                for (event, source) in &profile.events {
                    println!("  {event}: {source}");
                }
            }
            Ok(())
        }
        Some(other) => bail!("unknown profile subcommand '{other}' (set | rm | list)"),
    }
}

fn status() -> Result<()> {
    let config = Config::load()?;
    println!("sound: {}", if config.muted { "MUTED" } else { "on" });
    println!(
        "player: {}",
        play::detect_player(&config)
            .map(|(p, _)| p)
            .unwrap_or_else(|| "none found!".into())
    );
    println!("config: {}", paths::plugin_config_path().display());
    println!();
    for agent in aura::agents()? {
        let profile = config
            .agents
            .get(&agent.name)
            .cloned()
            .unwrap_or_else(|| "—".into());
        let hooks = if !agent.supports_hooks() {
            "unsupported kind".to_string()
        } else {
            match settings::Settings::load(&agent.settings_path()) {
                Ok(s) => {
                    let ev = s.managed_events();
                    if ev.is_empty() {
                        "no hooks installed".into()
                    } else {
                        format!("hooks: {}", ev.join(", "))
                    }
                }
                Err(e) => format!("unreadable settings: {e}"),
            }
        };
        println!(
            "{} ({}): profile={profile}  {hooks}",
            agent.name, agent.kind
        );
    }
    Ok(())
}

fn set_muted(muted: bool) -> Result<()> {
    let mut config = Config::load()?;
    config.muted = muted;
    config.save()?;
    println!("sound {}", if muted { "muted" } else { "unmuted" });
    Ok(())
}

fn flag_value(args: &[String], flag: &str) -> Option<String> {
    args.iter()
        .position(|a| a == flag)
        .and_then(|i| args.get(i + 1).cloned())
}

fn one_arg(args: &[String], usage: &str) -> Result<String> {
    args.get(1)
        .cloned()
        .with_context(|| format!("usage: {usage}"))
}

fn two_args(args: &[String], usage: &str) -> Result<(String, String)> {
    match (args.get(1), args.get(2)) {
        (Some(a), Some(b)) => Ok((a.clone(), b.clone())),
        _ => bail!("usage: {usage}"),
    }
}

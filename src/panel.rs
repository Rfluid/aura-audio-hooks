//! The aura plugin panel. Sections "Agents" and "Profiles" are
//! interactive (`type: "controls"` — requires aura >= 0.1.27 for the
//! `indent`/`icon`/`confirm` capabilities used here); every
//! operation the CLI offers is reachable from the panel buttons. See
//! `actions.rs` for the id grammar.

use serde_json::{json, Value};

use crate::actions::KNOWN_EVENTS;
use crate::aura;
use crate::config::{Config, OFF};
use crate::paths::expand_tilde;
use crate::play::detect_player;
use crate::settings::Settings;

pub fn render() -> String {
    match build() {
        Ok(panel) => panel.to_string(),
        Err(e) => render_error(&format!("{e:#}")),
    }
}

pub fn render_error(msg: &str) -> String {
    json!({ "title": "Audio Hooks", "error": msg }).to_string()
}

fn button(id: String, label: &str, active: bool, danger: bool) -> Value {
    json!({ "id": id, "label": label, "active": active, "danger": danger })
}

/// Destructive pill: icon-only ✕ (or labeled), armed by the host until a
/// second click confirms (`confirm` capability, aura >= 0.1.27).
fn danger_button(id: String, label: &str, confirm: &str) -> Value {
    json!({
        "id": id, "label": label, "active": false, "danger": true,
        "icon": "icons/close.svg", "confirm": confirm,
    })
}

fn build() -> anyhow::Result<Value> {
    let config = Config::load()?;
    let agents = aura::agents()?;

    // ── Agents section: global mute + per-agent profile pills ────────────
    let mut agent_controls = vec![json!({
        "label": "Sound",
        "hint": if config.muted { "muted everywhere" } else { "hooks play sounds" },
        "buttons": [
            button("mute:off".into(), "On", !config.muted, false),
            button("mute:on".into(), "Muted", config.muted, false),
        ],
    })];

    for agent in &agents {
        if !agent.supports_hooks() {
            agent_controls.push(json!({
                "label": agent.name,
                "hint": format!("{} — hooks not supported", agent.kind),
                "buttons": [],
            }));
            continue;
        }
        let assigned = config.agents.get(&agent.name).map(String::as_str);
        let installed = Settings::load(&agent.settings_path())
            .map(|s| s.managed_events())
            .unwrap_or_default();
        let hint = if installed.is_empty() {
            "hooks not installed — pick a profile".to_string()
        } else {
            format!("hooks: {}", installed.join(", "))
        };

        let mut buttons: Vec<Value> = config
            .profiles
            .keys()
            .map(|p| {
                button(
                    format!("agent:{}:{}", agent.name, p),
                    p,
                    assigned == Some(p.as_str()),
                    false,
                )
            })
            .collect();
        buttons.push(button(
            format!("agent:{}:off", agent.name),
            "Off",
            assigned.is_none_or(|a| a == OFF),
            false,
        ));
        if !installed.is_empty() {
            buttons.push(danger_button(
                format!("hooks:{}:remove", agent.name),
                "hooks",
                "remove hooks?",
            ));
        }
        agent_controls.push(json!({ "label": agent.name, "hint": hint, "buttons": buttons }));
    }

    // ── Profiles section: full CRUD via pickers ───────────────────────────
    let mut profile_controls = Vec::new();
    for (name, profile) in &config.profiles {
        let assigned_to: Vec<&str> = config
            .agents
            .iter()
            .filter(|(_, p)| p.as_str() == name)
            .map(|(a, _)| a.as_str())
            .collect();
        let hint = if assigned_to.is_empty() {
            "not assigned to any agent".to_string()
        } else {
            format!("active for {}", assigned_to.join(", "))
        };
        let mut header_buttons: Vec<Value> = KNOWN_EVENTS
            .iter()
            .filter(|e| !profile.events.contains_key(**e))
            .map(|e| {
                button(
                    format!("event:add:{name}:{e}"),
                    &format!("+ {e}"),
                    false,
                    false,
                )
            })
            .collect();
        header_buttons.push(danger_button(
            format!("profile:rm:{name}"),
            "",
            &format!("delete {name}?"),
        ));
        profile_controls.push(json!({
            "label": name,
            "hint": hint,
            "buttons": header_buttons,
        }));

        // Event rows nest under their profile via `indent` so it's
        // obvious which profile each mapping belongs to.
        for (event, source) in &profile.events {
            let path = expand_tilde(source);
            let detail = if path.is_file() {
                "single file".to_string()
            } else if path.is_dir() {
                let n = std::fs::read_dir(&path)
                    .map(|d| d.flatten().count())
                    .unwrap_or(0);
                format!("{n} files")
            } else {
                "missing!".to_string()
            };
            profile_controls.push(json!({
                "label": event,
                "hint": format!("{source} · {detail}"),
                "indent": 1,
                "buttons": [
                    button(format!("source:{name}:{event}:dir"), "Folder…", false, false),
                    button(format!("source:{name}:{event}:file"), "File…", false, false),
                    danger_button(
                        format!("event:rm:{name}:{event}"),
                        "",
                        &format!("unmap {event}?"),
                    ),
                ],
            }));
        }
    }
    profile_controls.push(json!({
        "label": "New profile",
        "hint": "named after the folder; maps Stop first",
        "buttons": [button("profile:new".into(), "+ Pick folder…", false, false)],
    }));

    // ── About section ──────────────────────────────────────────────────────
    let about = vec![
        json!({
            "label": "Player",
            "value": detect_player(&config).map(|(p, _)| p).unwrap_or_else(|| "none found".into()),
        }),
        json!({ "label": "Config", "value": crate::paths::plugin_config_path().display().to_string() }),
        json!({ "label": "CLI", "value": "aura-plugin-audio-hooks --help" }),
        json!({ "label": "Version", "value": env!("CARGO_PKG_VERSION") }),
    ];

    Ok(json!({
        "title": "Audio Hooks",
        "sections": [
            {
                "id": "agents",
                "label": "Agents",
                "uses_period": false,
                "type": "controls",
                "controls": agent_controls,
            },
            {
                "id": "profiles",
                "label": "Profiles",
                "uses_period": false,
                "type": "controls",
                "controls": profile_controls,
            },
            {
                "id": "about",
                "label": "About",
                "uses_period": false,
                "type": "lines",
                "lines": about,
            },
        ],
    }))
}

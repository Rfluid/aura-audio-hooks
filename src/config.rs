//! Plugin configuration: audio profiles and per-agent assignments.
//!
//! Lives at `~/.config/aura-audio-hooks/config.toml`:
//!
//! ```toml
//! muted = false
//! # player = "ffplay"            # optional override; auto-detected otherwise
//!
//! [profiles.coder-tags.events]
//! Stop         = "~/Music/coder-tags/done"          # dir -> random pick
//! Notification = "~/Music/coder-tags/input-needed"  # file -> that file
//!
//! [agents]
//! Peh = "coder-tags"   # or "off"
//! ```

use std::collections::{BTreeMap, BTreeSet};
use std::fs;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::paths::plugin_config_path;

/// Agent assignment value meaning "no audio for this agent".
pub const OFF: &str = "off";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Profile {
    /// Hook event name (e.g. `Stop`, `Notification`) -> audio source.
    /// A source is a directory (random pick) or a single audio file.
    #[serde(default)]
    pub events: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    /// Global mute: `play` becomes a no-op for every agent.
    #[serde(default)]
    pub muted: bool,
    /// Player binary override. Auto-detected when unset.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub player: Option<String>,
    #[serde(default)]
    pub profiles: BTreeMap<String, Profile>,
    /// Agent name (as configured in aura) -> profile name or "off".
    #[serde(default)]
    pub agents: BTreeMap<String, String>,
}

impl Config {
    pub fn load() -> Result<Self> {
        let path = plugin_config_path();
        if !path.exists() {
            return Ok(Self::default());
        }
        let raw =
            fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
        toml::from_str(&raw).with_context(|| format!("parsing {}", path.display()))
    }

    pub fn save(&self) -> Result<()> {
        let path = plugin_config_path();
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir).with_context(|| format!("creating {}", dir.display()))?;
        }
        let body = toml::to_string_pretty(self).context("serializing config")?;
        fs::write(&path, body).with_context(|| format!("writing {}", path.display()))?;
        Ok(())
    }

    /// Profile assigned to an agent, if any (and not "off").
    pub fn active_profile(&self, agent: &str) -> Option<(&str, &Profile)> {
        let name = self.agents.get(agent)?;
        if name == OFF {
            return None;
        }
        self.profiles.get(name).map(|p| (name.as_str(), p))
    }

    /// Union of event names across all profiles. Hooks are installed for
    /// every event any profile knows about, so switching profiles never
    /// requires touching the agent's settings file.
    pub fn all_events(&self) -> BTreeSet<String> {
        self.profiles
            .values()
            .flat_map(|p| p.events.keys().cloned())
            .collect()
    }
}

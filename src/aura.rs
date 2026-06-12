//! Read-only view of aura's agent roster (`~/.config/aura/config.toml`).

use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::paths::{aura_config_path, expand_tilde, home_dir};

#[derive(Debug, Clone, Deserialize)]
pub struct Agent {
    pub name: String,
    pub kind: String,
    #[serde(default)]
    pub config_path: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct AuraConfig {
    #[serde(default)]
    agents: Vec<Agent>,
}

impl Agent {
    /// The agent's config directory, with aura's per-kind defaults.
    pub fn config_dir(&self) -> PathBuf {
        match &self.config_path {
            Some(p) => expand_tilde(p),
            None => match self.kind.as_str() {
                "claude-code" => home_dir().join(".claude"),
                "codex" => home_dir().join(".codex"),
                "gemini" => home_dir().join(".gemini"),
                other => home_dir().join(format!(".{other}")),
            },
        }
    }

    /// Whether we know how to manage hooks for this agent kind.
    pub fn supports_hooks(&self) -> bool {
        self.kind == "claude-code"
    }

    /// The settings file holding this agent's hooks.
    pub fn settings_path(&self) -> PathBuf {
        self.config_dir().join("settings.json")
    }
}

pub fn agents() -> Result<Vec<Agent>> {
    let path = aura_config_path();
    if !path.exists() {
        return Ok(Vec::new());
    }
    let raw = fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
    let cfg: AuraConfig =
        toml::from_str(&raw).with_context(|| format!("parsing {}", path.display()))?;
    Ok(cfg.agents)
}

pub fn find_agent(name: &str) -> Result<Agent> {
    let all = agents()?;
    all.iter()
        .find(|a| a.name == name)
        .cloned()
        .with_context(|| {
            let known: Vec<_> = all.iter().map(|a| a.name.as_str()).collect();
            format!("unknown agent '{name}' (aura agents: {})", known.join(", "))
        })
}

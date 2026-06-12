//! Filesystem location helpers. Linux-first (XDG), with sane fallbacks.

use std::path::PathBuf;

pub fn home_dir() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/"))
}

pub fn config_root() -> PathBuf {
    std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .unwrap_or_else(|| home_dir().join(".config"))
}

/// This plugin's own config file.
pub fn plugin_config_path() -> PathBuf {
    config_root().join("aura-audio-hooks").join("config.toml")
}

/// Aura's main config file (read-only for us).
pub fn aura_config_path() -> PathBuf {
    config_root().join("aura").join("config.toml")
}

/// Expand a leading `~/` to the home directory.
pub fn expand_tilde(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/") {
        home_dir().join(rest)
    } else if path == "~" {
        home_dir()
    } else {
        PathBuf::from(path)
    }
}

/// The path our hook commands should invoke. Prefers the path we were
/// invoked as (keeps symlinked dev installs stable), falling back to
/// the resolved executable.
pub fn self_invocation_path() -> PathBuf {
    let argv0 = std::env::args_os().next().map(PathBuf::from);
    match argv0 {
        Some(p) if p.is_absolute() => p,
        _ => std::env::current_exe().unwrap_or_else(|_| PathBuf::from("aura-plugin-audio-hooks")),
    }
}

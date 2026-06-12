//! Resolve agent + event to an audio file and play it, detached.
//!
//! This is the hot path invoked by agent hooks; it must stay silent and
//! exit fast. Any "do nothing" condition (muted, agent off, event not in
//! the active profile) exits 0 quietly.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{bail, Context, Result};

use crate::config::Config;
use crate::paths::expand_tilde;

const AUDIO_EXTS: &[&str] = &["ogg", "mp3", "wav", "flac", "m4a", "opus", "aiff", "aac"];

/// Players we know how to drive, in preference order.
const PLAYERS: &[(&str, &[&str])] = &[
    ("ffplay", &["-nodisp", "-autoexit", "-loglevel", "quiet"]),
    ("pw-play", &[]),
    ("paplay", &[]),
    ("mpv", &["--no-video", "--really-quiet"]),
];

pub fn play(agent: &str, event: &str) -> Result<()> {
    let config = Config::load()?;
    if config.muted {
        return Ok(());
    }
    let Some((_, profile)) = config.active_profile(agent) else {
        return Ok(());
    };
    let Some(source) = profile.events.get(event) else {
        return Ok(());
    };
    let file = match resolve_file(&expand_tilde(source)) {
        Some(f) => f,
        None => return Ok(()), // missing/empty source: stay silent in the hook path
    };
    spawn_player(&config, &file)
}

/// Pick the file to play: the source itself, or a random audio file
/// inside it when it is a directory.
pub fn resolve_file(source: &Path) -> Option<PathBuf> {
    if source.is_file() {
        return Some(source.to_path_buf());
    }
    let mut files: Vec<PathBuf> = std::fs::read_dir(source)
        .ok()?
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.is_file()
                && p.extension()
                    .and_then(|e| e.to_str())
                    .is_some_and(|e| AUDIO_EXTS.contains(&e.to_ascii_lowercase().as_str()))
        })
        .collect();
    if files.is_empty() {
        return None;
    }
    files.sort();
    Some(files.swap_remove(pseudo_random() % files.len()))
}

/// Cheap randomness for shuffle-play; no need for a rand dependency.
fn pseudo_random() -> usize {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos() as u64)
        .unwrap_or(0);
    let mut x = nanos ^ (std::process::id() as u64) << 17 ^ 0x9e37_79b9_7f4a_7c15;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    x as usize
}

pub fn detect_player(config: &Config) -> Option<(String, &'static [&'static str])> {
    if let Some(custom) = &config.player {
        let args = PLAYERS
            .iter()
            .find(|(name, _)| name == custom)
            .map(|(_, args)| *args)
            .unwrap_or(&[]);
        return Some((custom.clone(), args));
    }
    PLAYERS
        .iter()
        .find(|(name, _)| on_path(name))
        .map(|(name, args)| (name.to_string(), *args))
}

fn on_path(bin: &str) -> bool {
    std::env::var_os("PATH")
        .is_some_and(|paths| std::env::split_paths(&paths).any(|dir| dir.join(bin).is_file()))
}

fn spawn_player(config: &Config, file: &Path) -> Result<()> {
    let (player, args) = detect_player(config)
        .context("no audio player found (tried ffplay, pw-play, paplay, mpv)")?;
    let spawned = Command::new(&player)
        .args(args)
        .arg(file)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
    match spawned {
        Ok(_) => Ok(()), // intentionally not waited on — playback outlives us
        Err(e) => bail!("spawning {player}: {e}"),
    }
}

//! Resolve agent + event to an audio file and play it, detached.
//!
//! This is the hot path invoked by agent hooks; it must stay silent and
//! exit fast. Any "do nothing" condition (muted, agent off, event not in
//! the active profile) exits 0 quietly.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{bail, Context, Result};

use crate::config::Config;
use crate::paths::{expand_tilde, state_dir};

const AUDIO_EXTS: &[&str] = &["ogg", "mp3", "wav", "flac", "m4a", "opus", "aiff", "aac"];

/// Claude Code can redeliver the same completion (observed:
/// `SubagentStop` firing twice, seconds apart, for one subagent). Dedup
/// markers older than this are pruned as stale rather than a real
/// resend still in flight.
const MARKER_MAX_AGE: Duration = Duration::from_secs(24 * 3600);

/// Players we know how to drive, in preference order.
const PLAYERS: &[(&str, &[&str])] = &[
    ("ffplay", &["-nodisp", "-autoexit", "-loglevel", "quiet"]),
    ("pw-play", &[]),
    ("paplay", &[]),
    ("mpv", &["--no-video", "--really-quiet"]),
];

pub fn play(agent: &str, event: &str) -> Result<()> {
    let payload = read_hook_payload();
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
    if already_handled(agent, event, payload.as_deref()) {
        return Ok(());
    }
    let file = match resolve_file(&expand_tilde(source)) {
        Some(f) => f,
        None => return Ok(()), // missing/empty source: stay silent in the hook path
    };
    spawn_player(&config, &file)
}

/// The JSON Claude Code sends on stdin for hook invocations — carries
/// `agent_id`/`prompt_id`, which is what lets us tell a genuine resend
/// of the same completion apart from a second, distinct one landing
/// close in time. `is_terminal` guards against hanging when `play` is
/// run by hand from a shell.
fn read_hook_payload() -> Option<String> {
    use std::io::{IsTerminal, Read};
    if std::io::stdin().is_terminal() {
        return None;
    }
    let mut buf = String::new();
    std::io::stdin().read_to_string(&mut buf).ok()?;
    (!buf.trim().is_empty()).then_some(buf)
}

/// Whether this exact completion was already handled. Keyed by the most
/// specific id available in the hook payload — `agent_id` for
/// `SubagentStop`, `prompt_id` for `Stop` — so two distinct completions
/// landing close together are never confused with a resend of the same
/// one, however far apart the resend arrives.
fn already_handled(agent: &str, event: &str, payload: Option<&str>) -> bool {
    let dir = state_dir();
    let _ = std::fs::create_dir_all(&dir);
    prune_stale_markers(&dir);

    let key = dedup_key(payload).unwrap_or_else(|| sanitize(agent));
    let marker = dir.join(format!("seen-{}-{}", sanitize(event), sanitize(&key)));
    if marker.exists() {
        return true;
    }
    let _ = std::fs::write(&marker, b"");
    false
}

fn dedup_key(payload: Option<&str>) -> Option<String> {
    let value: serde_json::Value = serde_json::from_str(payload?).ok()?;
    value
        .get("agent_id")
        .or_else(|| value.get("prompt_id"))
        .and_then(|v| v.as_str())
        .map(str::to_string)
}

fn prune_stale_markers(dir: &Path) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let now = SystemTime::now();
    for entry in entries.flatten() {
        let path = entry.path();
        let is_marker = path
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.starts_with("seen-"));
        if !is_marker {
            continue;
        }
        let stale = entry
            .metadata()
            .and_then(|m| m.modified())
            .is_ok_and(|modified| {
                now.duration_since(modified)
                    .is_ok_and(|age| age > MARKER_MAX_AGE)
            });
        if stale {
            let _ = std::fs::remove_file(path);
        }
    }
}

fn sanitize(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect()
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

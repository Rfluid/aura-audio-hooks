//! Surgical edits to a claude-code agent's `settings.json` hooks.
//!
//! Hook entries we own are identified by their command invoking this
//! binary (`… aura-plugin-audio-hooks play …`). Everything else in the
//! file — other hooks, plugins, any unknown keys — is preserved verbatim.
//!
//! Claude hook layout:
//! ```jsonc
//! { "hooks": { "Stop": [ { "matcher"?: "...", "hooks": [ {"type":"command","command":"...","async":true} ] } ] } }
//! ```

use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde_json::{json, Map, Value};

/// Substring identifying hook commands managed by this plugin.
pub const MARKER: &str = "aura-plugin-audio-hooks";

const BACKUP_SUFFIX: &str = ".aura-audio-hooks.bak";

pub struct Settings {
    path: std::path::PathBuf,
    root: Value,
}

impl Settings {
    pub fn load(path: &Path) -> Result<Self> {
        let root = if path.exists() {
            let raw =
                fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
            serde_json::from_str(&raw).with_context(|| format!("parsing {}", path.display()))?
        } else {
            Value::Object(Map::new())
        };
        anyhow::ensure!(root.is_object(), "{} is not a JSON object", path.display());
        Ok(Self {
            path: path.to_path_buf(),
            root,
        })
    }

    /// Write the file back, keeping a one-time backup of the pre-plugin state.
    pub fn save(&self) -> Result<()> {
        let backup = self.path.with_file_name(format!(
            "{}{BACKUP_SUFFIX}",
            self.path.file_name().unwrap_or_default().to_string_lossy()
        ));
        if self.path.exists() && !backup.exists() {
            fs::copy(&self.path, &backup)
                .with_context(|| format!("backing up to {}", backup.display()))?;
        }
        if let Some(dir) = self.path.parent() {
            fs::create_dir_all(dir)?;
        }
        let body = serde_json::to_string_pretty(&self.root)?;
        fs::write(&self.path, body).with_context(|| format!("writing {}", self.path.display()))?;
        Ok(())
    }

    fn hooks_obj(&self) -> Option<&Map<String, Value>> {
        self.root.get("hooks")?.as_object()
    }

    fn hooks_obj_mut(&mut self) -> &mut Map<String, Value> {
        let obj = self.root.as_object_mut().expect("validated on load");
        obj.entry("hooks".to_string())
            .or_insert_with(|| Value::Object(Map::new()))
            .as_object_mut()
            .expect("hooks coerced to object")
    }

    /// Event names that currently carry a hook command managed by us.
    pub fn managed_events(&self) -> Vec<String> {
        let Some(hooks) = self.hooks_obj() else {
            return Vec::new();
        };
        hooks
            .iter()
            .filter(|(_, groups)| {
                groups.as_array().is_some_and(|gs| {
                    gs.iter()
                        .any(|g| group_commands(g).iter().any(|c| c.contains(MARKER)))
                })
            })
            .map(|(event, _)| event.clone())
            .collect()
    }

    /// Whether the file has any hooks managed by us.
    pub fn has_managed(&self) -> bool {
        !self.managed_events().is_empty()
    }

    /// Drop every hook command containing `MARKER`; prune emptied groups
    /// and event arrays. Returns the number of commands removed.
    pub fn remove_managed(&mut self) -> usize {
        self.remove_matching(|cmd| cmd.contains(MARKER))
    }

    /// Drop every hook command for which `pred` returns true. Other
    /// commands sharing a group are kept in place.
    pub fn remove_matching(&mut self, pred: impl Fn(&str) -> bool) -> usize {
        let mut removed = 0;
        let hooks = self.hooks_obj_mut();
        let events: Vec<String> = hooks.keys().cloned().collect();
        for event in events {
            let Some(groups) = hooks.get_mut(&event).and_then(Value::as_array_mut) else {
                continue;
            };
            for group in groups.iter_mut() {
                let Some(inner) = group.get_mut("hooks").and_then(Value::as_array_mut) else {
                    continue;
                };
                inner.retain(|h| {
                    let is_match = h.get("command").and_then(Value::as_str).is_some_and(&pred);
                    if is_match {
                        removed += 1;
                    }
                    !is_match
                });
            }
            groups.retain(|g| {
                g.get("hooks")
                    .and_then(Value::as_array)
                    .is_none_or(|inner| !inner.is_empty())
            });
            if groups.is_empty() {
                hooks.remove(&event);
            }
        }
        removed
    }

    /// Append a managed hook command under `event`.
    pub fn add_managed(&mut self, event: &str, command: &str) {
        debug_assert!(command.contains(MARKER));
        let hooks = self.hooks_obj_mut();
        let groups = hooks
            .entry(event.to_string())
            .or_insert_with(|| Value::Array(Vec::new()));
        if let Some(groups) = groups.as_array_mut() {
            groups.push(json!({
                "hooks": [{ "type": "command", "command": command, "async": true }]
            }));
        }
    }

    /// Unmanaged hook commands that look like audio playback, as
    /// `(event, command)` pairs — candidates for `import`.
    pub fn audio_candidates(&self) -> Vec<(String, String)> {
        const PLAYERS: &[&str] = &[
            "ffplay", "pw-play", "paplay", "mpv", "aplay", "afplay", "ogg123",
        ];
        let Some(hooks) = self.hooks_obj() else {
            return Vec::new();
        };
        let mut out = Vec::new();
        for (event, groups) in hooks {
            let Some(groups) = groups.as_array() else {
                continue;
            };
            for group in groups {
                for cmd in group_commands(group) {
                    if !cmd.contains(MARKER) && PLAYERS.iter().any(|p| cmd.contains(p)) {
                        out.push((event.clone(), cmd));
                    }
                }
            }
        }
        out
    }
}

fn group_commands(group: &Value) -> Vec<String> {
    group
        .get("hooks")
        .and_then(Value::as_array)
        .map(|inner| {
            inner
                .iter()
                .filter_map(|h| h.get("command").and_then(Value::as_str))
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

/// Pull the audio source out of a raw player command. Handles globbed
/// directory picks like `ffplay … "$(ls /path/dir/*.ogg | shuf -n1)" …`
/// (returns `/path/dir`) and plain file arguments (`/path/file.ogg`).
pub fn extract_source(command: &str) -> Option<String> {
    let start = command.find('/')?;
    let tail = &command[start..];
    let end = tail
        .char_indices()
        .find(|(_, c)| matches!(c, '"' | '\'' | '*' | '|' | ')') || c.is_whitespace())
        .map(|(i, _)| i)
        .unwrap_or(tail.len());
    let globbed = tail[end..].starts_with('*');
    let mut path = &tail[..end];
    if globbed {
        // `/dir/*.ogg` cuts to `/dir/`; `/dir/sound-*.ogg` cuts to
        // `/dir/sound-` — either way the directory is what we want.
        path = match path.rfind('/') {
            Some(0) => "/",
            Some(i) => &path[..i],
            None => path,
        };
    } else {
        path = path.trim_end_matches('/');
        if path.is_empty() {
            path = "/";
        }
    }
    if path.len() <= 1 || path.contains('$') {
        return None;
    }
    Some(path.to_string())
}

#[cfg(test)]
mod tests {
    use super::extract_source;

    #[test]
    fn extracts_dir_from_glob_command() {
        let cmd = r#"ffplay -nodisp -autoexit "$(ls /tmp/*.ogg | shuf -n1)" 2>/dev/null &"#;
        assert_eq!(extract_source(cmd).as_deref(), Some("/tmp"));
    }

    #[test]
    fn extracts_plain_file() {
        let cmd = "paplay /tmp/ding.ogg";
        assert_eq!(extract_source(cmd).as_deref(), Some("/tmp/ding.ogg"));
    }
}

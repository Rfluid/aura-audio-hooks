---
title: Configuration
status: stable
version: 0.2.0
last_updated: 2026-06-12
source_refs:
  - src/config.rs
owner: "@rfluid"
tags: [configuration, docs]
---

# Configuration

Everything lives in one file:

```
~/.config/aura-audio-hooks/config.toml     (respects $XDG_CONFIG_HOME)
```

The panel buttons and the CLI both edit this file; hand-editing is
equally fine — it is re-read on every invocation.

## Reference

```toml
# Global mute. `play` becomes a no-op everywhere. The panel's Sound
# toggle and `mute`/`unmute` flip this.
muted = false

# Optional player override. Auto-detected when unset, in order:
# ffplay, pw-play, paplay, mpv.
# player = "ffplay"

# A profile maps hook events to an audio source. A source is either a
# directory (a random audio file inside is picked per event) or a
# single file (always played).
[profiles.coder-tags.events]
Stop         = "/home/me/Music/coder-tags/done"
Notification = "/home/me/Music/coder-tags/input-needed"

[profiles.minimal.events]
Stop = "/home/me/Music/ding.ogg"

# Which profile each aura agent uses. Keys are agent names exactly as
# they appear in aura's config.toml ([[agents]] name). "off" silences
# the agent without uninstalling its hook entries.
[agents]
Peh      = "coder-tags"
Personal = "minimal"
```

## Semantics worth knowing

- **Directory sources** pick uniformly among files with extensions:
  ogg, mp3, wav, flac, m4a, opus, aiff, aac.
- **Unknown agents / profiles** in `[agents]` are harmless: `play`
  exits silently when the lookup fails.
- **Profile names** must not contain `:` (the panel action id grammar
  uses it as a separator). Names created from the UI are sanitized.
- Deleting a profile from the UI re-points agents that used it to
  `"off"`.

## What is *not* here

Hook installation state lives in each agent's `settings.json`, not in
this file — see [architecture.md](architecture.md). Aura agent
definitions live in `~/.config/aura/config.toml` and are read-only to
this plugin.

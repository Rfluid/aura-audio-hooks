---
title: CLI reference
status: stable
version: 0.2.0
last_updated: 2026-06-12
source_refs:
  - src/main.rs
owner: "@rfluid"
tags: [cli, docs]
---

# CLI reference

The binary doubles as a CLI (symlinked to `~/.local/bin` by
`install.sh`). Every panel operation has a CLI equivalent.

## State

```bash
aura-plugin-audio-hooks status          # agents, profiles, hook state
aura-plugin-audio-hooks profile list
```

## Toggles

```bash
aura-plugin-audio-hooks mute            # meeting mode (global)
aura-plugin-audio-hooks unmute
aura-plugin-audio-hooks use <agent> <profile|off>
```

`use` validates both names; picking a profile from the *panel* also
auto-installs hook entries when missing — the CLI `use` does not
(pair it with `enable`).

## Hook management

```bash
aura-plugin-audio-hooks enable <agent>    # install managed hook entries
aura-plugin-audio-hooks disable <agent>   # remove them (others preserved)
aura-plugin-audio-hooks import <agent> [--profile <name>]
```

`import` adopts pre-existing raw audio hooks (ffplay/paplay/mpv/...):
extracts the folder or file each one plays, builds/extends a profile,
removes the raw entries (after a one-time settings backup), and installs
managed entries instead. Unparseable commands are reported and left
untouched.

## Profile editing

```bash
aura-plugin-audio-hooks profile set <name> <event> <path>   # dir or file
aura-plugin-audio-hooks profile rm <name>
```

`profile set` refreshes hook entries for every enabled agent so new
events are covered immediately.

## Plumbing (called by other programs)

```bash
aura-plugin-audio-hooks --period <all|7d|30d>      # panel JSON (aura)
aura-plugin-audio-hooks action <id> --period <p>   # panel button (aura)
aura-plugin-audio-hooks play --agent <a> --event <e>   # agent hooks
```

Test button actions headlessly through aura itself:

```bash
aura plugin run "Audio Hooks" --action mute:on
```

The action id grammar is documented in [panel.md](panel.md).

---
title: Architecture
status: stable
version: 0.2.0
last_updated: 2026-06-12
source_refs:
  - src/main.rs
  - src/play.rs
  - src/settings.rs
owner: "@rfluid"
tags: [architecture, docs]
---

# Architecture

One binary, `aura-plugin-audio-hooks`, plays three roles depending on
how it is invoked:

| Invoked by      | As                              | Does                                  |
| --------------- | ------------------------------- | ------------------------------------- |
| aura (panel)    | `--period <p>`                  | prints the panel JSON                 |
| aura (button)   | `action <id> --period <p>`      | applies the change, prints fresh panel|
| agent hooks     | `play --agent <a> --event <e>`  | resolves + plays a sound, detached    |
| the user        | subcommands (`mute`, `use`, …)  | same operations as the panel buttons  |

## The indirection that makes it work

Agent hooks never reference audio files directly. Each managed hook
entry in an agent's `settings.json` is the same fixed command:

```json
{ "type": "command",
  "command": ".../aura-plugin-audio-hooks play --agent Peh --event Stop",
  "async": true }
```

`play` resolves *agent → assigned profile → event → source* against the
plugin's own config at call time. Consequences:

- **Switching profiles, muting, and editing paths are pure config
  changes** — agent settings are never rewritten for them.
- Hooks are installed for the **union of events across all profiles**,
  so a profile switch can never leave an event uncovered. `play` exits
  silently for events the active profile doesn't map.
- Hook entries change only on `enable` / `disable` / `import`.

## Non-interference contract

Only hook commands containing the binary's name (`aura-plugin-audio-hooks`)
are ever added, modified, or removed (`src/settings.rs::MARKER`). All
other hooks — matchers, rtk instrumentation, anything user-defined —
round-trip byte-equivalent through `serde_json::Value`. The first edit
to each settings file writes a one-time backup
(`settings.json.aura-audio-hooks.bak`).

## Module map

| Module        | Responsibility                                            |
| ------------- | --------------------------------------------------------- |
| `main.rs`     | CLI dispatch                                              |
| `config.rs`   | plugin config (profiles, assignments, mute) load/save     |
| `aura.rs`     | read-only view of aura's `[[agents]]` roster              |
| `settings.rs` | surgical hook edits in agent `settings.json`              |
| `ops.rs`      | enable/disable/import/use/refresh state transitions       |
| `play.rs`     | source resolution, random pick, player spawn              |
| `panel.rs`    | aura panel JSON (interactive `controls` sections)         |
| `actions.rs`  | panel button id grammar + dispatch                        |
| `picker.rs`   | zenity/kdialog native folder & file dialogs               |
| `paths.rs`    | XDG/home path helpers                                     |

## Agent support

Hook management is implemented for `claude-code` agents (hooks live in
`<config_dir>/settings.json`). `codex` and `gemini` agents are listed in
the panel as unsupported; their hook mechanisms differ and can be added
behind `aura::Agent::supports_hooks`.

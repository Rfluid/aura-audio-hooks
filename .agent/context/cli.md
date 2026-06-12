---
title: CLI quick context
status: current
version: 0.1.0
last_updated: 2026-06-12
last_verified: 2026-06-12
source_refs: [docs/cli.md, src/main.rs]
owner: "@rfluid"
tags: [context, cli]
---

# CLI quick context

Full reference: [docs/cli.md](../../docs/cli.md). The shapes an agent
most often needs:

```bash
aura-plugin-audio-hooks status                       # current state
aura-plugin-audio-hooks mute | unmute
aura-plugin-audio-hooks use <agent> <profile|off>
aura-plugin-audio-hooks enable|disable <agent>
aura-plugin-audio-hooks import <agent> --profile <name>
aura-plugin-audio-hooks profile set <name> <event> <path>
aura-plugin-audio-hooks profile rm <name> | profile list
aura-plugin-audio-hooks play --agent <a> --event <e> # what hooks call
```

Debug loops:

```bash
just panel                       # render panel JSON locally
just action mute:on              # fire a button action headlessly
aura plugin run "Audio Hooks" --action <id>   # through the host
```

Gotchas:

- `use` from the CLI does **not** auto-install hooks (the panel's
  agent buttons do); pair with `enable`.
- Picker-backed actions (`source:*`, `event:add:*`, `profile:new`)
  open a zenity/kdialog dialog — they hang headless sessions; test
  non-picker ids instead.

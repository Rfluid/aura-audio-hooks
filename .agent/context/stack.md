---
title: Technical stack
status: current
version: 0.1.0
last_updated: 2026-06-12
last_verified: 2026-06-12
source_refs: [Cargo.toml]
owner: "@rfluid"
tags: [context, architecture]
---

# Technical stack

- **Language**: Rust 2021, single binary crate (`aura-plugin-audio-hooks`).
- **Dependencies** (deliberately minimal — the panel must answer aura's
  500 ms budget): `anyhow`, `serde`, `serde_json`, `toml`. No async
  runtime, no clap, no rand (xorshift seeded from clock+pid).
- **Host protocol**: aura plugin wire contract — JSON panel on stdout,
  `action <id>` re-invocations for button clicks. Interactive `controls`
  sections require an aura build with the `indent`/`icon`/`confirm`
  button capabilities (see `../aura/docs/plugin-authoring.md`).
- **External tools** (runtime, optional): `ffplay`/`pw-play`/`paplay`/
  `mpv` for playback; `zenity`/`kdialog` for native pickers.
- **Files touched**: own config (`~/.config/aura-audio-hooks/config.toml`),
  agent `settings.json` (managed entries only), aura's config read-only.
- Sibling repo: `../aura` (the host; gpui tray app + aura-core).

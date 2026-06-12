---
title: Glossary
status: current
version: 0.1.0
last_updated: 2026-06-12
last_verified: 2026-06-12
source_refs: []
owner: "@rfluid"
tags: [context, glossary]
---

# Glossary

- **Agent** — an AI coding assistant aura tracks (`[[agents]]` in
  aura's config): claude-code, codex, gemini. Identified by its aura
  display name (e.g. "Peh").
- **Hook event** — a lifecycle moment the agent fires commands on
  (`Stop`, `Notification`, `SubagentStop`, `SessionStart`).
- **Profile** — a named map of hook events → audio sources. The unit
  the user switches between (e.g. `coder-tags` vs `minimal`).
- **Source** — what a profile maps an event to: a directory (random
  pick per event) or a single audio file.
- **Managed entry** — a hook command in an agent's `settings.json` that
  invokes this binary; the only kind we create or delete.
- **Marker** — the binary name substring identifying managed entries.
- **Assignment** — which profile an agent currently uses (`[agents]` in
  our config); `off` means assigned-silent.
- **Mute** — global kill-switch (`muted = true`), orthogonal to
  assignments; nothing plays anywhere.
- **Panel / controls** — aura's plugin UI; `controls` sections hold
  clickable pill buttons that re-invoke the plugin with an action id.
- **Import** — adopting pre-existing raw audio hooks (e.g. inline
  `ffplay … | shuf` commands) into a profile + managed entries.

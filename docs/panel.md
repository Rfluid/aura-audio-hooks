---
title: Panel UI
status: stable
version: 0.2.0
last_updated: 2026-06-12
source_refs:
  - src/panel.rs
  - src/actions.rs
owner: "@rfluid"
tags: [ui, docs]
---

# Panel UI

The aura modal shows an **Audio Hooks** pill (purple speaker icon).
The panel uses aura's interactive `controls` sections (aura ≥ 0.1.27)
— every operation is clickable; nothing requires the terminal.

## Agents tab

- **Sound** — global **On / Muted** toggle. Mute before a meeting; no
  agent settings are touched.
- **One row per aura agent** — a pill per profile plus **Off**; the
  active assignment is highlighted. Picking a profile installs the
  agent's hook entries automatically if missing (hint line shows which
  events are installed). The red **✕ hooks** pill uninstalls them
  (two-click confirm).
- Codex/Gemini rows are informational (hooks unsupported).

## Profiles tab

- **Profile row** — hint shows which agents use it. Buttons:
  **+ \<Event\>** for each unmapped known event (Stop, Notification,
  SubagentStop, SessionStart) opens a folder picker; red **✕** deletes
  the profile (confirm).
- **Event rows** nest under their profile (indented, guide bar). Hint
  shows the source path and file count. **Folder…** / **File…** re-pick
  the source via a native dialog (zenity/kdialog); red **✕** unmaps the
  event (confirm).
- **New profile → + Pick folder…** creates a profile named after the
  chosen folder, mapping `Stop` first.

While a picker dialog is open the panel shows a spinner and aura
suspends its focus-loss auto-dismiss.

## Action id grammar

Ids are opaque to aura; this plugin parses them as `:`-separated
segments (hence no `:` in profile names):

```
mute:on | mute:off
agent:<agent>:<profile|off>     assign (auto-installs hooks)
hooks:<agent>:remove            uninstall managed hook entries
source:<profile>:<event>:dir    re-pick source (folder dialog)
source:<profile>:<event>:file   re-pick source (file dialog)
event:add:<profile>:<event>     map a new event (folder dialog)
event:rm:<profile>:<event>      unmap an event
profile:new                     create profile from picked folder
profile:rm:<name>               delete a profile
```

Errors are returned through the panel `error` envelope; the next
refresh recovers. A cancelled picker is a no-op.

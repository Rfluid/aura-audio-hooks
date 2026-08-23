---
title: Stop/SubagentStop hook payload and redelivery
status: current
version: 0.1.0
last_updated: 2026-08-23
last_verified: 2026-08-23
source_refs: [src/play.rs]
owner: "@rfluid"
tags: [fact, hooks, play]
---

# Stop/SubagentStop hook payload and redelivery

Claude Code sends a JSON payload on **stdin** to every hook command,
including managed ones (`play --agent A --event E` never reads args
for this). Confirmed fields via a live debug capture: `session_id`,
`prompt_id`, `transcript_path`, `cwd`, `permission_mode`,
`hook_event_name`, `stop_hook_active`, `last_assistant_message`. On
`SubagentStop` only: `agent_id` (unique per subagent), `agent_type`,
`agent_transcript_path`.

Two things easy to misdiagnose as bugs:

1. **`Stop` fires once per assistant turn of the main session, not
   once per conversation.** It has nothing to do with subagents. If
   you launch N subagents across M of your own turns, you'll hear
   `Stop` M times — that's expected, not a leak.
2. **`SubagentStop` can be redelivered for the same subagent**
   (observed: two firings ~2.7s apart for one subagent) *and* other,
   unrelated subagents in the same session (spawned by other enabled
   plugins/background tooling, not something this session's user
   explicitly launched) legitimately fire their own `SubagentStop`
   too. Don't assume "more bells than subagents you launched" means a
   duplicate-fire bug — check `agent_id` first.

`play` now reads the stdin payload and dedupes by `agent_id` (falls
back to `prompt_id`, then to a sanitized agent name if no payload is
available, e.g. manual CLI invocation) instead of a time window — a
time window can't tell "same completion resent" apart from "two
different completions landing close together," and will wrongly eat
one of two subagents that finish within the window of each other.
Markers live under `~/.config/aura-audio-hooks/state/seen-*` and are
pruned after 24h.

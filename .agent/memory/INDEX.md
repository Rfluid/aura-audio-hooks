---
title: Memory index
status: current
version: 0.1.0
last_updated: 2026-08-23
last_verified: 2026-08-23
source_refs: []
owner: "@rfluid"
tags: [memory, index]
---

# Memory index

Durable, dated notes agents leave for future sessions. One file per
entry, named `YYYY-MM-DD-slug.md`.

- `facts/` — verified, non-obvious truths about the system or its
  environment (e.g. host quirks, protocol edge cases).
- `lessons/` — post-mortems: what went wrong and what to do instead.
- `patterns/` — recurring solutions worth reusing.

Add an entry when you learn something the code alone won't teach the
next session. Keep entries short; link source files.

## Entries

- `facts/2026-08-23-hook-redelivery-and-payload.md` — Stop fires per
  main-turn (not per subagent); SubagentStop payload has `agent_id`
  and can redeliver/come from unrelated subagents; dedup by id, not
  time.

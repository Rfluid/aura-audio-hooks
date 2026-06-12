---
title: Conventions
status: current
version: 0.2.0
last_updated: 2026-06-12
last_verified: 2026-06-12
source_refs: []
owner: "@rfluid"
tags: [context, conventions]
---

# Conventions

- **Never touch hooks we don't own.** A hook entry is ours iff its
  command contains the binary name (`settings.rs::MARKER`). Everything
  else round-trips untouched. Tests and reviews should treat any change
  to this invariant as a bug.
- **`play` is the hot path**: silent no-op on every soft failure
  (muted, unassigned, unmapped event, empty folder). It must never
  print to the agent's hook output or block.
- **State transitions live in `ops.rs`**; `main.rs` and `actions.rs`
  are thin dispatchers. UI buttons and CLI subcommands must reach the
  same code paths.
- **Action ids** are `:`-separated (`docs/panel.md`); profile names are
  sanitized to never contain `:`.
- Errors in panel/action paths go through the JSON `error` envelope
  (exit 0), not stderr — aura renders the envelope inline.
- Commit style: conventional commits (`feat:`, `fix:`, `docs:` …),
  mirroring the aura repo.
- Run `./scripts/pre-pr.sh` (or `just pre-pr`) before any PR: fmt,
  clippy `-D warnings`, tests, release build.

# Agent guide — aura-audio-hooks

An [aura](../aura) plugin managing per-agent audio hook profiles. One
Rust binary, three faces: aura panel (interactive `controls`), agent
hook executor (`play`), and CLI. Read this first, then pull what you
need from the workspace below.

## Workspace map

| Where | What |
| ----- | ---- |
| [.agent/context/stack.md](.agent/context/stack.md) | tech stack, host-protocol version requirements |
| [.agent/context/conventions.md](.agent/context/conventions.md) | invariants and style — **read before editing** |
| [.agent/context/glossary.md](.agent/context/glossary.md) | domain terms (profile, source, managed entry, …) |
| [.agent/context/cli.md](.agent/context/cli.md) | command shapes + debug loops |
| [.agent/memory/](.agent/memory/INDEX.md) | dated facts / lessons / patterns from past sessions |
| [.agent/prompts/](.agent/prompts/), [.agent/skills/](.agent/skills/), [.agent/workflows/](.agent/workflows/) | reusable prompts, how-to skills, larger plans |
| [docs/](docs/architecture.md) | human docs: architecture, configuration, cli, panel |

## Hard invariants

1. Only hook entries whose command contains `aura-plugin-audio-hooks`
   may be created, edited, or removed in agent `settings.json` files.
2. `play` never blocks, never prints on soft failure.
3. Panel buttons and CLI subcommands share the same `ops.rs` paths.

## Dev loop

```bash
just panel            # render the panel JSON
just action <id>      # fire a button action headlessly
just pre-pr           # fmt + clippy -D warnings + test + build
./install.sh --link   # symlink install; rebuilds go live immediately
```

The host protocol (controls sections, action invocations, budgets) is
documented in `../aura/docs/plugin-authoring.md`. This plugin's action
id grammar is in [docs/panel.md](docs/panel.md).

When you learn something non-obvious, leave a dated note under
`.agent/memory/` and list it in the INDEX.

<p align="center">
  <img src="assets/icon.svg" width="72" alt="Audio Hooks logo" />
</p>

<h1 align="center">aura-audio-hooks</h1>

<p align="center">
  Per-agent <b>audio hook profiles</b> for <a href="../aura">aura</a> —
  switch the sounds your agents make, per agent, from the tray. Mute
  everything with one click when a meeting starts.
</p>

---

## What it does

You keep folders of sounds ("done", "needs input", …). This plugin
turns them into named **profiles** and lets you choose, per aura agent,
which profile plays on which hook event — entirely from aura's plugin
panel:

- **Toggle profiles per agent** — one pill per profile, plus **Off**.
- **Global mute** — meeting mode; nothing plays anywhere.
- **Edit everything in the UI** — add/remove event mappings, re-pick
  folders or files via native dialogs, create and delete profiles.
- **Plays nice with other hooks** — only its own hook entries are ever
  touched; your rtk/lint/whatever hooks are preserved byte-for-byte.

## How it works

Agent hooks run a fixed command — `aura-plugin-audio-hooks play --agent
Peh --event Stop` — and the plugin resolves *agent → profile → event →
folder* at play time, picking a random audio file. Because the hook
command never changes, profile switches and muting are pure config
edits; agent `settings.json` files are only written when installing,
removing, or importing hooks (with a one-time backup).

Details: [docs/architecture.md](docs/architecture.md).

## Install

Requires a local [aura](../aura) build with interactive plugin controls, and `cargo`.

```bash
./install.sh          # build + register with aura + CLI symlink
./install.sh --link   # dev mode: symlink, rebuilds go live immediately
```

Have existing inline audio hooks (`ffplay … | shuf` style)? Adopt them:

```bash
aura-plugin-audio-hooks import Peh --profile coder-tags
```

## Updating

From the checkout, pull and reinstall:

```bash
git pull
just update           # = just uninstall + just install
just update --link    # same, but dev-mode symlink install
```

`uninstall` runs first so nothing stale survives — the old binary,
icon, sidecar, aura registration, and CLI symlink are all removed
before the fresh build is registered. **Your profiles and config are
preserved** (`~/.config/aura-audio-hooks/`), but agent hook entries
are removed and re-installed, so re-enable profiles per agent from
the panel (or `aura-plugin-audio-hooks use <agent> <profile>`).

If you installed with `--link`, you usually don't need any of this:
the plugin runs from `target/release`, so `just build` alone makes
rebuilds go live.

## Uninstall

```bash
./uninstall.sh            # removes managed hook entries from agent
                          # settings, deregisters from aura, deletes
                          # the binary + icon + sidecar + CLI symlink
./uninstall.sh --purge    # also deletes ~/.config/aura-audio-hooks
                          # (profiles, agent mappings, mute state)
```

Only hook entries owned by this plugin are touched in agent
`settings.json` files — every other hook is preserved (a one-time
backup was written next to each `settings.json` on first edit).
Without `--purge`, profiles and config survive for a future
reinstall.

## Use

**From aura** — open the modal → Plugins → **Audio Hooks**:

| Tab | What you can do |
| --- | --- |
| Agents | global Sound On/Muted; per agent pick a profile or Off (hooks auto-install); remove hooks (two-click confirm) |
| Profiles | per profile: map new events, delete; per event: re-pick Folder…/File… via native dialog, unmap |

**From the terminal** — every operation has a CLI twin:

```bash
aura-plugin-audio-hooks status
aura-plugin-audio-hooks mute | unmute
aura-plugin-audio-hooks use <agent> <profile|off>
aura-plugin-audio-hooks profile set <name> <event> <path>
```

Full reference: [docs/cli.md](docs/cli.md) ·
panel guide & action grammar: [docs/panel.md](docs/panel.md) ·
config format: [docs/configuration.md](docs/configuration.md).

## Configuration

One TOML file, also hand-editable:

```toml
# ~/.config/aura-audio-hooks/config.toml
muted = false

[profiles.coder-tags.events]
Stop         = "/home/me/Music/coder-tags/done"           # dir -> random
Notification = "/home/me/Music/coder-tags/input-needed"

[agents]
Peh = "coder-tags"
```

## Develop

```bash
just            # list tasks
just panel      # render the panel JSON aura would show
just action mute:on
just pre-pr     # fmt + clippy -D warnings + test + build
```

Agent contributors: start at [AGENTS.md](AGENTS.md) (workspace under
`.agent/`).

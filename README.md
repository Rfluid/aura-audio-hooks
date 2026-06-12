# aura-audio-hooks

An [aura](../aura) plugin that manages **per-agent audio hook profiles**:
named sets of sounds played on agent hook events (`Stop`, `Notification`,
…), switchable per agent with one command — including a global mute for
meetings.

## How it works

Instead of embedding raw `ffplay …` commands in each agent's
`settings.json`, the plugin installs one managed hook entry per event:

```json
{ "type": "command", "command": "…/aura-plugin-audio-hooks play --agent Peh --event Stop", "async": true }
```

`play` resolves *agent → active profile → event → source* from this
plugin's own config at `~/.config/aura-audio-hooks/config.toml` and plays
a random file from the source directory (or the file itself). Because the
hook command never changes, **switching profiles and muting are pure
config changes** — agent settings are only touched by `enable` / `disable`
/ `import`.

Only hook entries whose command contains `aura-plugin-audio-hooks` are
ever added or removed; all other hooks (matchers, rtk, anything) are
preserved verbatim. The first edit to a settings file writes a one-time
backup next to it (`settings.json.aura-audio-hooks.bak`).

Agents are discovered from aura's `~/.config/aura/config.toml`
(`[[agents]]`, honoring `config_path`). Only `claude-code` agents are
supported for hook management; other kinds show as unsupported.

## Config

```toml
muted = false
# player = "ffplay"   # optional; auto-detects ffplay, pw-play, paplay, mpv

[profiles.coder-tags.events]
Stop         = "/home/me/Music/coder-tags/done"           # dir -> random pick
Notification = "/home/me/Music/coder-tags/input-needed"   # file -> that file

[agents]
Peh      = "coder-tags"
Personal = "off"
```

## UI (aura ≥ 0.1.26)

Every operation is available from the plugin panel in the aura modal
(`type: "controls"` sections — see `docs/plugin-authoring.md` in aura):

- **Agents tab** — global Sound On/Muted toggle; per agent, one pill per
  profile plus **Off** (picking a profile auto-installs the hook entries
  if missing) and a danger **✕ hooks** pill that uninstalls them.
- **Profiles tab** — per profile: **+ \<event\>** pills to map a new
  event, **Delete**; per event: **Folder…** / **File…** open a native
  picker (zenity/kdialog) to change the source, **✕** unmaps it.
  **New profile → + Pick folder…** creates a profile named after the
  chosen folder with its `Stop` event mapped.

Button clicks re-invoke this binary as `action <id> --period <p>`; it
applies the change and prints the refreshed panel. Test headlessly:

```bash
aura plugin run "Audio Hooks" --action mute:on
```

## CLI

```bash
aura-plugin-audio-hooks status                      # what's configured where
aura-plugin-audio-hooks mute / unmute               # meeting mode
aura-plugin-audio-hooks use <agent> <profile|off>   # switch an agent's profile
aura-plugin-audio-hooks profile set <name> <event> <path>
aura-plugin-audio-hooks profile rm <name>
aura-plugin-audio-hooks profile list
aura-plugin-audio-hooks enable <agent>              # install managed hook entries
aura-plugin-audio-hooks disable <agent>             # remove them (others untouched)
aura-plugin-audio-hooks import <agent> [--profile <name>]
                                                    # adopt existing raw audio hooks
```

`import` finds unmanaged hook commands that invoke an audio player,
extracts the folder/file they play, builds a profile from them, removes
the raw entries (backup first), and installs managed entries instead.

Hooks are installed for the **union of events across all profiles**, so a
profile that adds a new event auto-refreshes every enabled agent.

## Install / update

```bash
cargo build --release
aura plugin add ./target/release/aura-plugin-audio-hooks \
    --name "Audio Hooks" --color "#8b5cf6"
# optional, for the CLI anywhere:
ln -sf ~/.config/aura/plugins/aura-plugin-audio-hooks ~/.local/bin/
```

The aura modal then shows an **Audio Hooks** pill with Overview / Agents /
Profiles tabs (read-only — aura panels have no input; use the CLI to
change state, the panel refreshes on modal open).

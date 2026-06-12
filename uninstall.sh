#!/usr/bin/env bash
# Remove the Audio Hooks plugin.
#
#   ./uninstall.sh           remove hook entries, plugin binary, icon, symlink
#   ./uninstall.sh --purge   also delete ~/.config/aura-audio-hooks (profiles)
#
# Agent settings only lose hook entries owned by this plugin; every other
# hook is preserved (and a one-time backup was written next to each
# settings.json on first edit).
set -euo pipefail

NAME="Audio Hooks"
BIN=aura-plugin-audio-hooks
BIN_DIR="${HOME}/.local/bin"
CONFIG_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/aura-audio-hooks"

PLUGINS_DIR="$(aura plugin dir 2>/dev/null || echo "${XDG_CONFIG_HOME:-$HOME/.config}/aura/plugins")"
INSTALLED="$PLUGINS_DIR/$BIN"

# Pull our hook entries out of every agent's settings.json first, while
# the binary still exists to do it.
if [[ -x "$INSTALLED" ]]; then
    echo "▸ Removing managed hook entries from agent settings"
    "$INSTALLED" status 2>/dev/null | awk -F'[ (]' '/profile=/ {print $1}' | while read -r agent; do
        "$INSTALLED" disable "$agent" 2>/dev/null || true
    done
fi

if command -v aura >/dev/null; then
    aura plugin remove "$NAME" 2>/dev/null || true
fi
rm -f "$PLUGINS_DIR/$BIN.svg" "$PLUGINS_DIR/$BIN.toml" "$INSTALLED"
rm -f "$BIN_DIR/$BIN"
echo "▸ Removed plugin binary, sidecar, icon, and CLI symlink"

if [[ "${1:-}" == "--purge" ]]; then
    rm -rf "$CONFIG_DIR"
    echo "▸ Purged $CONFIG_DIR"
else
    echo "Profiles kept at $CONFIG_DIR (pass --purge to delete)"
fi

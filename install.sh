#!/usr/bin/env bash
# Build and register the Audio Hooks plugin with a local aura install.
#
#   ./install.sh            build + register + CLI symlink
#   ./install.sh --link     symlink the binary into aura's plugins dir
#                           instead of copying (dev loop: rebuilds are
#                           picked up without re-running this script)
set -euo pipefail

cd "$(dirname "$0")"

NAME="Audio Hooks"
COLOR="#8b5cf6"
BIN=aura-plugin-audio-hooks
BIN_DIR="${HOME}/.local/bin"

if ! command -v aura >/dev/null; then
    echo "error: aura not found on PATH — install aura first" >&2
    exit 1
fi
if ! command -v cargo >/dev/null; then
    echo "error: cargo not found on PATH" >&2
    exit 1
fi

LINK_FLAG=""
if [[ "${1:-}" == "--link" ]]; then
    LINK_FLAG="--link"
fi

echo "▸ Building release binary"
cargo build --release

PLUGINS_DIR="$(aura plugin dir)"
mkdir -p "$PLUGINS_DIR"

# The icon must outlive this checkout for copy-installs, so it ships next
# to the binary inside aura's plugins dir.
ICON_DEST="$PLUGINS_DIR/$BIN.svg"
install -m 644 assets/icon.svg "$ICON_DEST"
echo "▸ Installed icon to $ICON_DEST"

aura plugin add "target/release/$BIN" \
    --name "$NAME" \
    --color "$COLOR" \
    --icon "$ICON_DEST" \
    $LINK_FLAG

# CLI convenience: mute/profile switches callable from anywhere.
mkdir -p "$BIN_DIR"
ln -sf "$PLUGINS_DIR/$BIN" "$BIN_DIR/$BIN"
echo "▸ Symlinked CLI to $BIN_DIR/$BIN"

echo
echo "Done. Open the aura modal → Plugins → $NAME."
echo "Adopt existing raw audio hooks with:  $BIN import <agent> --profile <name>"

# aura-audio-hooks — common dev/install tasks
# Run `just` with no args for a list.

# Default target
default:
    @just --list

# ── Build ─────────────────────────────────────────────────────────────────────

# Build in release mode
build:
    cargo build --release

# ── Tests / lint ──────────────────────────────────────────────────────────────

test:
    cargo test

lint:
    cargo clippy -- -D warnings
    cargo fmt --check

fix:
    cargo fmt

# Everything CI would check, in one shot
pre-pr:
    ./scripts/pre-pr.sh

# ── Dev loop ──────────────────────────────────────────────────────────────────

# Print the panel JSON the plugin emits (what aura renders)
panel:
    cargo run --release --quiet -- --period all | python3 -m json.tool

# Fire a panel button action headlessly, e.g. `just action mute:on`
action id:
    cargo run --release --quiet -- action {{id}} | python3 -m json.tool

# Human-readable state across agents and profiles
status:
    cargo run --release --quiet -- status

# ── Install ───────────────────────────────────────────────────────────────────

# Build + register with aura (icon, name, color) + CLI symlink
install:
    ./install.sh

uninstall:
    ./uninstall.sh

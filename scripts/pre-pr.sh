#!/usr/bin/env bash
# Everything CI would check, in one shot. Mirrors aura's scripts/pre-pr.sh.
set -euo pipefail

cd "$(dirname "$0")/.."

echo "▸ cargo fmt --check"
cargo fmt --check

echo "▸ cargo clippy -D warnings"
cargo clippy --all-targets -- -D warnings

echo "▸ cargo test"
cargo test

echo "▸ cargo build --release"
cargo build --release

echo
echo "All green."

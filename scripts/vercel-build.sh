#!/usr/bin/env bash
# Vercel build command. The build image has no Rust, so install what build.sh
# needs, then run it. Slow (wasm-pack compiles from source), fine for a weekly
# deploy. Safe to run locally: anything already installed is skipped.
set -euo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/common.sh"
cd "$ROOT"

if ! command -v cargo >/dev/null 2>&1; then
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs |
    sh -s -- -y --profile minimal
fi
# rustup puts cargo here, and this shell did not have it on PATH yet.
if [ -f "$HOME/.cargo/env" ]; then
  source "$HOME/.cargo/env"
fi

need rustup "Install Rust with rustup, wasm-pack needs it to add the wasm target."
rustup target add wasm32-unknown-unknown

if ! command -v wasm-pack >/dev/null 2>&1; then
  cargo install --locked wasm-pack
fi

"$ROOT/scripts/build.sh"

#!/usr/bin/env bash
# Compile the Rust engine to wasm and drop the JS glue in web/pkg.
set -euo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/common.sh"
cd "$ROOT"

need wasm-pack "Install it with: cargo install wasm-pack"

wasm-pack build "$WASM_CRATE" \
  --target web \
  --out-dir "$ROOT/$PKG_DIR" \
  --out-name physics_wasm \
  "$@"

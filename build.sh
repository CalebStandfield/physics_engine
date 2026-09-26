#!/usr/bin/env bash
# Build the wasm module into web/pkg.
set -euo pipefail
cd "$(dirname "$0")"
wasm-pack build crates/physics-wasm --target web --out-dir ../../web/pkg --out-name physics_wasm "$@"

#!/usr/bin/env bash
# Serve the frontend. ES modules need a real origin, file:// will not work.
set -euo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/common.sh"
cd "$ROOT"

need python3 "Install Python 3, or serve $WEB_DIR with any static file server."
[ -f "$PKG_DIR/physics_wasm.js" ] || die "$PKG_DIR is empty. Run ./run.sh or scripts/build.sh first."

echo "serving http://localhost:$PORT"
exec python3 -m http.server "$PORT" --directory "$WEB_DIR"

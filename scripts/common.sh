# Shared settings. Sourced, not run.

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WASM_CRATE="crates/physics-wasm"
WEB_DIR="web"
PKG_DIR="$WEB_DIR/pkg"
PORT="${PORT:-8080}"

die() {
  echo "error: $*" >&2
  exit 1
}

need() {
  command -v "$1" >/dev/null 2>&1 || die "$1 not found. $2"
}

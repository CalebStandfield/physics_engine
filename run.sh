#!/usr/bin/env bash
# Build the engine, then serve the frontend. One command, start to browser.
#
#   ./run.sh              build (release) and serve on :8080
#   ./run.sh --dev        build unoptimized, faster to compile
#   ./run.sh --no-build   skip the build, serve what is already in web/pkg
#   PORT=3000 ./run.sh    serve somewhere else
set -euo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/scripts/common.sh"

build=1
build_args=()

for arg in "$@"; do
  case "$arg" in
    --no-build) build=0 ;;
    --dev) build_args+=(--dev) ;;
    -h | --help)
      # Print the header comment above, minus the shebang.
      awk 'NR > 1 && /^#/ { sub(/^# ?/, ""); print; next } NR > 1 { exit }' \
        "${BASH_SOURCE[0]}"
      exit 0
      ;;
    *) die "unknown option '$arg'. Try --help." ;;
  esac
done

if [ "$build" -eq 1 ]; then
  "$ROOT/scripts/build.sh" ${build_args[@]+"${build_args[@]}"}
fi

exec "$ROOT/scripts/serve.sh"

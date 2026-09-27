#!/usr/bin/env bash
# Build the engine, then serve the frontend. One command, start to browser.
#
#   ./run.sh              build (release) and serve on :8080, Ctrl-C to stop
#   ./run.sh start        build and serve in the background
#   ./run.sh stop         stop the background server
#   ./run.sh restart      stop it, rebuild, start it again
#   ./run.sh status       say whether it is running, and where
#   ./run.sh --dev        build unoptimized, faster to compile
#   ./run.sh --no-build   skip the build, serve what is already in web/pkg
#   PORT=3000 ./run.sh    serve somewhere else
#
# The flags combine with the commands: ./run.sh start --dev
set -euo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/scripts/common.sh"

cmd=foreground
build=1
build_args=()

for arg in "$@"; do
  case "$arg" in
    start | stop | restart | status) cmd="$arg" ;;
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

# Nothing to build for a command that only prods a server that is already up.
case "$cmd" in
  stop | status) build=0 ;;
esac

# Free the port before rebuilding, so a failed build does not leave the old
# server running while you think it restarted.
if [ "$cmd" = restart ]; then
  "$ROOT/scripts/serve.sh" stop
fi

if [ "$build" -eq 1 ]; then
  "$ROOT/scripts/build.sh" ${build_args[@]+"${build_args[@]}"}
fi

case "$cmd" in
  foreground) exec "$ROOT/scripts/serve.sh" ;;
  restart) exec "$ROOT/scripts/serve.sh" start ;;
  *) exec "$ROOT/scripts/serve.sh" "$cmd" ;;
esac

#!/usr/bin/env bash
# Serve the frontend. ES modules need a real origin, file:// will not work.
#
#   scripts/serve.sh          serve in the foreground, Ctrl-C to stop
#   scripts/serve.sh start    serve in the background
#   scripts/serve.sh stop     stop the background server
#   scripts/serve.sh status   say whether it is running, and where
#
# Everything is keyed by $PORT, so two ports are two independent servers. The
# pid and the log live in .run/, which is not tracked.
set -euo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/common.sh"
cd "$ROOT"

RUN_DIR="$ROOT/.run"
PID_FILE="$RUN_DIR/server-$PORT.pid"
LOG_FILE="$RUN_DIR/server-$PORT.log"
URL="http://localhost:$PORT"

# Pid of our background server on this port, empty if there is not one. A
# recorded pid that has died, or that some unrelated process has since been
# given, counts as no server, and the stale file goes away.
running_pid() {
  [ -f "$PID_FILE" ] || return 0

  local pid
  pid="$(cat "$PID_FILE" 2>/dev/null || true)"
  if [ -n "$pid" ] && ps -p "$pid" -o command= 2>/dev/null | grep -q "http\.server"; then
    echo "$pid"
  else
    rm -f "$PID_FILE"
  fi
}

# Whatever is listening on the port, ours or not. Empty if the port is free, or
# if there is no lsof to ask.
port_holder() {
  command -v lsof >/dev/null 2>&1 || return 0
  # lsof exits 1 on no match, which pipefail would turn into a failed call.
  lsof -ti "tcp:$PORT" -sTCP:LISTEN 2>/dev/null | head -1 || true
}

# Everything the server needs before it is worth starting one.
check_ready() {
  need python3 "Install Python 3, or serve $WEB_DIR with any static file server."
  [ -f "$PKG_DIR/physics_wasm.js" ] ||
    die "$PKG_DIR is empty. Run ./run.sh or scripts/build.sh first."
}

cmd_foreground() {
  check_ready
  echo "serving $URL"
  exec python3 -m http.server "$PORT" --directory "$WEB_DIR"
}

cmd_start() {
  local pid holder
  pid="$(running_pid)"
  if [ -n "$pid" ]; then
    echo "already serving $URL (pid $pid)"
    return 0
  fi

  holder="$(port_holder)"
  if [ -n "$holder" ]; then
    die "port $PORT is taken by pid $holder and it is not ours. Stop it, or set PORT."
  fi

  check_ready
  mkdir -p "$RUN_DIR"
  nohup python3 -m http.server "$PORT" --directory "$WEB_DIR" >"$LOG_FILE" 2>&1 &
  echo $! >"$PID_FILE"

  # Give it a moment to fall over on its own before claiming it is up.
  sleep 0.4
  pid="$(running_pid)"
  if [ -z "$pid" ]; then
    echo "--- $LOG_FILE" >&2
    tail -5 "$LOG_FILE" >&2 || true
    die "the server did not stay up."
  fi

  echo "serving $URL (pid $pid, log $LOG_FILE)"
}

cmd_stop() {
  local pid
  pid="$(running_pid)"
  if [ -z "$pid" ]; then
    echo "nothing of ours is serving :$PORT"
    return 0
  fi

  kill "$pid" 2>/dev/null || true
  for _ in $(seq 1 25); do
    ps -p "$pid" >/dev/null 2>&1 || break
    sleep 0.2
  done
  # Still there after five seconds: stop asking.
  if ps -p "$pid" >/dev/null 2>&1; then
    kill -9 "$pid" 2>/dev/null || true
  fi

  rm -f "$PID_FILE"
  echo "stopped :$PORT (pid $pid)"
}

cmd_status() {
  local pid holder
  pid="$(running_pid)"
  if [ -n "$pid" ]; then
    echo "serving $URL (pid $pid, log $LOG_FILE)"
    return 0
  fi

  holder="$(port_holder)"
  if [ -n "$holder" ]; then
    echo "not ours, but pid $holder is listening on :$PORT"
  else
    echo "not serving :$PORT"
  fi
}

case "${1-}" in
  "") cmd_foreground ;;
  start) cmd_start ;;
  stop) cmd_stop ;;
  status) cmd_status ;;
  *) die "unknown command '$1'. Try start, stop or status." ;;
esac

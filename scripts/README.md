# scripts

Build and serve. `run.sh` at the project root is the single entry point and calls these.

| File | What it does |
| --- | --- |
| `common.sh` | Sourced, not run. Sets `ROOT`, `WASM_CRATE`, `WEB_DIR`, `PKG_DIR`, `PORT` (default 8080). Helpers `die` and `need`. |
| `build.sh` | `wasm-pack build crates/physics-wasm --target web` into `web/pkg`, out-name `physics_wasm`. Extra args pass through (e.g. `--dev`). Needs `wasm-pack`. |
| `serve.sh` | `python3 -m http.server $PORT --directory web`, in the foreground or in the background. Fails if `web/pkg` is empty. Takes `start`, `stop` or `status`; no argument means foreground. |

ES modules need a real origin, so `file://` will not work. Serve it.

## run.sh

    ./run.sh              build release, serve on :8080, Ctrl-C to stop
    ./run.sh start        build and serve in the background
    ./run.sh stop         stop the background server
    ./run.sh restart      stop it, rebuild, start it again
    ./run.sh status       is it running, and where
    ./run.sh --dev        unoptimized build, faster to compile
    ./run.sh --no-build   serve whatever is already in web/pkg
    PORT=3000 ./run.sh    different port

Flags combine with commands: `./run.sh start --dev`. `stop` and `status` never build.

## Background server

`start` writes `.run/server-$PORT.pid` and `.run/server-$PORT.log` (`.run/` is not tracked).
Everything is keyed by the port, so two ports are two independent servers and `PORT=3000
./run.sh stop` stops that one and nothing else.

- `start` refuses a port something else is already listening on, rather than racing it.
- A pid file left behind by a server that died, or whose pid something unrelated now has,
  counts as not running and gets cleaned up. `stop` asks nicely, then kills after five seconds.
- `status` distinguishes our server from a stranger holding the port, which is the usual
  reason a page will not pick up a rebuild.

`--help` prints the header comment out of the script itself, so the usage block at the top
of `run.sh` is the help text. Keep them the same thing.

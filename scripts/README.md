# scripts

Build and serve. `run.sh` at the project root is the single entry point and calls these.

| File | What it does |
| --- | --- |
| `common.sh` | Sourced, not run. Sets `ROOT`, `WASM_CRATE`, `WEB_DIR`, `PKG_DIR`, `PORT` (default 8080). Helpers `die` and `need`. |
| `build.sh` | `wasm-pack build crates/physics-wasm --target web` into `web/pkg`, out-name `physics_wasm`. Extra args pass through (e.g. `--dev`). Needs `wasm-pack`. |
| `serve.sh` | `python3 -m http.server $PORT --directory web`. Fails if `web/pkg` is empty. |

ES modules need a real origin, so `file://` will not work. Serve it.

## run.sh

    ./run.sh              build release, serve on :8080
    ./run.sh --dev        unoptimized build, faster to compile
    ./run.sh --no-build   serve whatever is already in web/pkg
    PORT=3000 ./run.sh    different port

`--help` prints the header comment out of the script itself, so the usage block at the top
of `run.sh` is the help text. Keep them the same thing.

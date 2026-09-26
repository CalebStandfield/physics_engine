#!/usr/bin/env bash
# Serve the frontend. ES modules need a real origin, file:// will not work.
set -euo pipefail
cd "$(dirname "$0")"
python3 -m http.server 8080 --directory web

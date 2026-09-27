#!/usr/bin/env python3
"""Static file server for development.

`python3 -m http.server` with two changes that matter when you reload the page
after every edit:

  - Nothing is cached. The stock server sends only Last-Modified, which leaves a
    browser free to guess how long a file stays fresh, and Chrome guesses
    generously for ES modules and wasm. A rebuild then does not show up until a
    hard reload, which looks exactly like the build being broken.
  - .wasm is served as application/wasm, which streaming compilation needs.

Usage: devserver.py PORT DIRECTORY
"""

import sys
from functools import partial
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer


class DevHandler(SimpleHTTPRequestHandler):
    extensions_map = {
        **SimpleHTTPRequestHandler.extensions_map,
        ".js": "text/javascript",
        ".mjs": "text/javascript",
        ".wasm": "application/wasm",
    }

    def end_headers(self):
        # no-store rather than no-cache: do not keep a copy at all, so there is
        # nothing to revalidate and nothing to serve stale.
        self.send_header("Cache-Control", "no-store, must-revalidate")
        self.send_header("Expires", "0")
        super().end_headers()


def main():
    if len(sys.argv) != 3:
        print(__doc__.strip().splitlines()[-1], file=sys.stderr)
        return 2

    port = int(sys.argv[1])
    handler = partial(DevHandler, directory=sys.argv[2])

    with ThreadingHTTPServer(("", port), handler) as httpd:
        try:
            httpd.serve_forever()
        except KeyboardInterrupt:
            pass
    return 0


if __name__ == "__main__":
    sys.exit(main())

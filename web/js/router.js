// The URL as the view: `/` is the scenario list, `/<id>` is that scenario.
//
// The ids come from the engine's own catalog, so `/incline` and `/spring` are
// not routes anybody wrote down: a scenario added in Rust gets its path with no
// change here. An id that is not in the catalog falls back to the list.
//
// Everything is one page, so the server has to answer these paths with
// `index.html`. `scripts/devserver.py` does; any other host needs the same
// fallback.

// Path -> scenario id, or null for the list. `known` is the catalog, so a stale
// or made-up path lands on the list instead of a broken stage.
export function readRoute(known) {
  const id = location.pathname.replace(/^\/+|\/+$/g, "");
  return known.includes(id) ? id : null;
}

export function routeOf(id) {
  return id ? `/${id}` : "/";
}

// Point the URL at a view without reloading. `replace` rewrites the current
// entry instead of adding one, for the redirect a bad path lands on.
export function setRoute(id, { replace = false } = {}) {
  const path = routeOf(id);
  if (path === location.pathname) return;
  history[replace ? "replaceState" : "pushState"]({ id: id ?? null }, "", path);
}

// Back and forward. The callback gets the id the URL now names, or null.
export function onRoute(known, show) {
  window.addEventListener("popstate", () => show(readRoute(known)));
}

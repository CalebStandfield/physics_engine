# web

HTML/Canvas frontend. Renders what the engine reports and reads/writes parameters.
No physics in JS: if the frontend needs a physics answer it asks wasm.

Served by `scripts/serve.sh`, which sends `Cache-Control: no-store` so a rebuild shows up on a
plain reload. `pkg/` is wasm-pack output, generated, do not edit.

## Files

| File | What is in it |
| --- | --- |
| `index.html` | Two views: the landing page (`home`) and the stage (`stage`). Icons are named with `data-icon` and swapped for Phosphor glyphs (fill weight) at boot by `js/ui/icons.js`. Element ids the JS looks up: `home`, `cards`, `stage`, `tabs`, `btn-home`, `scene`, `fbd`, `controls`, `solver`, `solver-block`, `legend`, `readout`, `caption`, `boot`, `btn-start`, `btn-stop`, `btn-reset`, `mode-controls`, `mode-fbd`, `panel-controls`, `panel-fbd`, `btn-reset-controls`, `btn-clock`, `view-fbd`, `view-time`, `timescale`. |
| `styles.css` | Dark theme, tokens in `:root`. Panels float over the canvas. Narrow-screen rules at the bottom under `@media (max-width: 1180px)`. |
| `js/engine.js` | `Sim`, the wrapper over the wasm `Engine`. Module init, run/pause, and one snapshot cached per frame so every panel shares one boundary crossing. `MAX_FRAME = 0.1 s` clamps a long stall. `spec(key)` hands a panel one parameter's range. `stillShot(id)` is a one-off frame of any scenario at its defaults, for the landing page cards. `timeScale` scales the time handed to the engine each frame, declared by the `TIME_SCALE` spec here; `resetParams()` puts every parameter back to its declared default without touching the clock. |
| `js/router.js` | `readRoute`, `routeOf`, `setRoute`, `onRoute`. The URL as the view: `/` is the list, `/<scenario id>` is that scenario. Ids come from the catalog, so the routes are not written down anywhere. |
| `js/memory.js` | `ParamMemory`. Slider values kept per scenario for as long as the page is open. |
| `js/app.js` | Wiring, the two-view switch, and the `requestAnimationFrame` loop. Paused frames still redraw, so slider changes show up right away. Owns the simple/advanced mode. |
| `js/camera.js` | `Camera`. World meters -> canvas pixels, and the only place world +y up flips to canvas +y down. |
| `js/theme.js` | `COLOR` tokens plus `forceStyle(kind)`, `guideStyle(kind)`, `NET_STYLE`. Every canvas color comes from here. |
| `js/render/` | Canvas drawing. See `js/render/README.md`. |
| `js/ui/` | DOM panels and the icon set. See `js/ui/README.md`. |

## Two views

The page opens on a landing page: a title block (name, one line, two chips) and one card per
scenario, each with a still render of that scenario. Clicking a card loads it and swaps to the
stage; the brand on the left of the rail goes back and pauses whatever was running, same rule
as switching scenarios. The scenario tabs only show on the stage, since the landing page is
already the list.

Which view is up is in the URL: `/` is the list and `/<scenario id>` (`/incline`, `/spring`) is
one scenario, so a scenario can be linked to and back/forward work. The ids come from the
catalog, so a scenario added in Rust gets its route with no change here, and a path naming no
scenario lands on the list and rewrites itself. Every click goes through `open(id)`, which sets
the URL and then shows it; `show(id)` alone is for the URL having already changed (a reload, or
the back button). One page behind several URLs means the server has to answer them all with
`index.html`: `scripts/devserver.py` falls back to the page for any path that names no file and
has no extension, and any other host serving this needs the same fallback.

Cards are built from `scenario_catalog()` in `js/ui/home.js`, so a scenario added in Rust gets
a card with no change here. The grid is `auto-fit`, so two scenarios sit side by side and four
fall into a 2x2. Previews are drawn once and again on resize; the render loop does nothing
while the landing page is up.

## Camera

- `observe(frame, guides)` grows an accumulated fit box over the body and every guide point.
  `guides` defaults to all of them; a renderer that hides some (the card previews) passes the
  ones it actually draws, so the view leaves no room for lines nobody sees. The box
  only grows between `reset()` calls, otherwise an oscillating body makes the view breathe.
- `fit(width, height, inset)` lerps toward the target view (`LERP = 0.12`, `PAD = 0.14`).
  `inset` is the region the floating panels cover, so the scene centers in what is visible.
- `toPx(p)` is the conversion. `scale` is pixels per meter.

## Simple and advanced

Each panel has its own switch and its own mode: one at the top of the controls panel, one at
the top of the free body diagram panel, and they are independent. Trimming the controls does
not trim the readout. `simple` shows the controls or readout rows the engine marked basic, and
hides the solver; `advanced` shows everything. Which rows are basic is the engine's call,
carried on `ParamSpec.tier` and `Derived.tier`, so nothing in the frontend names a parameter.
Both modes are page state and survive a scenario switch.

## Unfolding a readout row

A readout row the engine explained carries a formula and the terms in it, so it unfolds: the
formula in symbols, then one row per term, and a term that was itself explained unfolds the same
way, down to the parameters. `js/ui/readout.js` walks whatever tree came over the boundary and
names none of it. Which rows unfold, what their formulas say and where the chain ends are all
the engine's call.

Panels only grow downward. `.panel-controls` is pinned 20px from the top rather than centered,
and `.panel-fbd` sits directly under the legend, so switching modes moves the bottom edge and
nothing else. The height change is tweened by `js/ui/accordion.js`.

## What a scenario switch keeps

- Slider values: kept per scenario in `js/memory.js`, in memory only. Switching back restores
  what you had; a reload is a clean slate. Nothing bleeds between scenarios.
- The run state: always paused. Starting one scenario should not hand a running clock to the
  next one, so `Sim.loadScenario` clears `running` and `app.js` puts the buttons back.
- Integrator and step size: kept, by the wasm `loadScenario`.
- Everything else resets: the scenario is rebuilt, so the clock, the energy ledger and the
  camera fit box all start over.

## Keyed on kind, never on scenario

The scene, the free body diagram and the legend read `ForceVector.kind` and `Guide.kind` out
of the frame and look up a style. Nothing in the render path names a scenario. `js/ui/readout.js`
has the one exception: its `COMPARISONS` table is keyed by scenario id, and it only names which
two engine numbers to pair up. The percentage itself comes from wasm.

# web

HTML/Canvas frontend. Renders what the engine reports and reads/writes parameters.
No physics in JS: if the frontend needs a physics answer it asks wasm.

Served by `scripts/serve.sh`, which sends `Cache-Control: no-store` so a rebuild shows up on a
plain reload. `pkg/` is wasm-pack output, generated, do not edit.

## Files

| File | What is in it |
| --- | --- |
| `index.html` | Element ids the JS looks up: `tabs`, `scene`, `fbd`, `controls`, `solver`, `solver-block`, `legend`, `readout`, `caption`, `boot`, `btn-start`, `btn-stop`, `btn-reset`, `mode-controls`, `mode-fbd`, `panel-controls`, `panel-fbd`. |
| `styles.css` | Dark theme, tokens in `:root`. Panels float over the canvas. Narrow-screen rules at the bottom under `@media (max-width: 1180px)`. |
| `js/engine.js` | `Sim`, the wrapper over the wasm `Engine`. Module init, run/pause, and one snapshot cached per frame so every panel shares one boundary crossing. `MAX_FRAME = 0.1 s` clamps a long stall. `spec(key)` hands a panel one parameter's range. |
| `js/memory.js` | `ParamMemory`. Slider values kept per scenario for as long as the page is open. |
| `js/app.js` | Wiring and the `requestAnimationFrame` loop. Paused frames still redraw, so slider changes show up right away. Owns the simple/advanced mode. |
| `js/camera.js` | `Camera`. World meters -> canvas pixels, and the only place world +y up flips to canvas +y down. |
| `js/theme.js` | `COLOR` tokens plus `forceStyle(kind)`, `guideStyle(kind)`, `NET_STYLE`. Every canvas color comes from here. |
| `js/render/` | Canvas drawing. See `js/render/README.md`. |
| `js/ui/` | DOM panels. See `js/ui/README.md`. |

## Camera

- `observe(frame)` grows an accumulated fit box over the body and every guide point. The box
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

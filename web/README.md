# web

HTML/Canvas frontend. Renders what the engine reports and reads/writes parameters.
No physics in JS: if the frontend needs a physics answer it asks wasm.

Served by `scripts/serve.sh`. `pkg/` is wasm-pack output, generated, do not edit.

## Files

| File | What is in it |
| --- | --- |
| `index.html` | Element ids the JS looks up: `tabs`, `scene`, `fbd`, `controls`, `solver`, `legend`, `readout`, `caption`, `boot`, `btn-start`, `btn-stop`, `btn-reset`. |
| `styles.css` | Dark theme, tokens in `:root`. Panels float over the canvas. Narrow-screen rules at the bottom under `@media (max-width: 1180px)`. |
| `js/engine.js` | `Sim`, the wrapper over the wasm `Engine`. Module init, run/pause, and one snapshot cached per frame so every panel shares one boundary crossing. `MAX_FRAME = 0.1 s` clamps a long stall. |
| `js/app.js` | Wiring and the `requestAnimationFrame` loop. Paused frames still redraw, so slider changes show up right away. |
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

## Keyed on kind, never on scenario

The scene, the free body diagram and the legend read `ForceVector.kind` and `Guide.kind` out
of the frame and look up a style. Nothing in the render path names a scenario. `js/ui/readout.js`
has the one exception: its `COMPARISONS` table is keyed by scenario id, and it only names which
two engine numbers to pair up. The percentage itself comes from wasm.

# web/js/ui

DOM panels. All of them are rebuilt from engine data, so a new scenario needs no changes here.

| File | What is in it |
| --- | --- |
| `topbar.js` | `buildTabs`, `setActiveTab`, `setCaption`. Scenario tabs from `scenario_catalog()`. |
| `controls.js` | `buildControls` (one slider per `ParamSpec` the mode shows), `buildSolver` (integrator picker and step-size picker). |
| `legend.js` | `buildLegend`. Rows for the forces in the current frame plus one per scenery kind. |
| `readout.js` | `Readout`. Clock, the scenario's `derived` rows, energy lost, measured period, and the measured-vs-predicted percentages. |
| `mode.js` | `buildModeToggle`, `setActiveMode`, and `shows(mode, tier)`, the one place the simple/advanced rule lives. |
| `format.js` | `num(value, step)` and `unitSuffix(unit)`. Keeps columns from jittering as values cross zero. |
| `inset.js` | `overlayInset(canvas, overlays)`. How much of the canvas the floating panels cover, per side. |

## Details worth knowing

- `controls.js`: sliders come straight from `sim.schema()`, nothing names a parameter. Keys
  matching `RESET_ONLY` (`/^initial_/`) get an "on reset" tag. `DT_CHOICES` is 1/240, 1/480, 1/960 s.
  In `simple` mode the advanced-tier specs are skipped; the tier is the engine's, from the
  control catalog in `physics-scenarios`.
- `mode.js`: two toggles, one mode. `shows(mode, tier)` is the whole rule, used by both
  `controls.js` and `readout.js`, so the two panels can never disagree about what basic means.
- `legend.js`: skips the rebuild when the row set has not changed (`root.dataset.key`).
  `GUIDE_NAMES` maps guide kind to a display name, falling back to the guide's own label.
- `readout.js`: rows carry a tier like the controls do. The engine tiers its own `derived`
  rows; the clock is basic, and energy lost, measured period and the comparisons are advanced.
  `COMPARISONS` is keyed by scenario id and is data, not logic. It names which
  two numbers to pair; both come from the engine and the percentage comes from
  `percent_difference` in wasm. Cells are created once per row shape and then only have their
  text swapped, so the panel does not thrash per frame.
- `format.js`: derives decimal places from the slider step, switches to exponential below 1e-3
  or at/above 1e6, and prints `--` for anything not finite.
- `inset.js`: assigns each overlay to a side by comparing its rect to the canvas midlines.
  Nothing here knows which panel is which. `GAP = 16 px`.

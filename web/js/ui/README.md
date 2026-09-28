# web/js/ui

DOM panels. All of them are rebuilt from engine data, so a new scenario needs no changes here.

| File | What is in it |
| --- | --- |
| `home.js` | `buildCards`. The landing page grid, one card per catalog entry, returning each card's canvas so the caller can draw a preview into it. |
| `icons.js` | `ICONS`, `icon(name)`, `hydrateIcons(root)`. Phosphor Icons (fill weight) vendored as path data. |
| `topbar.js` | `buildTabs`, `setActiveTab`, `setCaption`. Scenario tabs from `scenario_catalog()`. |
| `controls.js` | `buildControls` (one slider per `ParamSpec` the mode shows), `buildSolver` (integrator picker and step-size picker). |
| `legend.js` | `buildLegend`. Rows for the forces in the current frame plus one per scenery kind. |
| `readout.js` | `Readout`. Clock, the scenario's `derived` rows, energy lost, measured period, and the measured-vs-predicted percentages. |
| `mode.js` | `buildModeToggle`, `setActiveMode`, and `shows(mode, tier)`, the one place the simple/advanced rule lives. |
| `accordion.js` | `resizeAround(panel, change)`. Tweens a panel's height across a rebuild so it folds instead of snapping. |
| `format.js` | `num(value, step)` and `unitSuffix(unit)`. Keeps columns from jittering as values cross zero. |
| `inset.js` | `overlayInset(canvas, overlays)`. How much of the canvas the floating panels cover, per side. |

## Details worth knowing

- `home.js`: cards come from `scenario_catalog()`, so a new scenario shows up with no change
  here. The grid is CSS `auto-fit`, so two cards sit side by side and four fall into a 2x2.
  The title block above them is static markup in `index.html`; only the cards are built here.
- `icons.js`: the page has no network at runtime, so nothing loads from a CDN. Each entry is
  one Phosphor glyph's `d` on a 256x256 box, copied from `@phosphor-icons/core` (MIT), all of
  them the `fill` weight so the rail matches the portfolio. Static
  markup names an icon with `<i data-icon="house-simple">` and `hydrateIcons(document)` swaps
  the slots for real `<svg>`s at boot; JS that builds its own DOM calls `icon(name)`. Glyphs
  are `fill: currentColor` and sized by the rule around them, so `.ph` never picks a color.
- `controls.js`: sliders come straight from `sim.schema()`, nothing names a parameter. Keys
  matching `RESET_ONLY` (`/^initial_/`) get an "on reset" tag. `DT_CHOICES` is 1/240, 1/480, 1/960 s.
  In `simple` mode the advanced-tier specs are skipped; the tier is the engine's, from the
  control catalog in `physics-scenarios`.
- `mode.js`: two toggles, one per panel, each with its own mode. `shows(mode, tier)` is the
  whole rule, used by both `controls.js` and `readout.js`, so neither panel invents its own
  idea of what basic means.
- `accordion.js`: measures, runs the change, measures again, then animates the box between the
  two heights (220 ms, ease-out) with the Web Animations API. The content swap is instant; only
  the panel's height is tweened, and no inline height is left behind. It animates even under
  `prefers-reduced-motion`; the comment in the file says how to hand that setting back.
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

# physics-wasm

Thin `wasm-bindgen` layer over the engine. Marshaling only. Every number here comes
out of `physics-core` or `physics-scenarios`, nothing in this crate decides any physics.

`src/lib.rs` is the whole crate. Built by `scripts/build.sh` into `web/pkg/`.

## Free functions

- `scenario_ids()` -> `string[]`
- `scenario_catalog()` -> `[{ id, name, description, coordinate_label }]`
- `integrators()` -> `[{ id, name }]`
- `percent_difference(a, b)`, `percent_error(measured, accepted)` (undefined when accepted is 0)

## class Engine

Constructed with a scenario id. Holds a `Simulation` across calls.

| JS name | Notes |
| --- | --- |
| `loadScenario(id)` | Swaps scenario, keeps integrator and step size. |
| `scenarioId`, `scenarioName`, `description`, `coordinateLabel` | Getters. |
| `schema()` | `ParamSpec[]`: key, label, unit, min, max, default, step, tier (`"basic"` or `"advanced"`), marks (`{value, label}[]`, the landmarks on the range). |
| `getParam(key)`, `setParam(key, value)` | Writes take effect next step. `reset()` to restart from them. Out-of-range throws. |
| `integratorId`, `setIntegrator(id)` | Unknown id throws. |
| `dt` (get/set), `setHistory(capacity, stride)` | |
| `reset()`, `step()`, `advance(elapsed)`, `runFor(duration)` | `advance` returns fixed steps run. |
| `time`, `position`, `velocity`, `measuredPeriod` | Getters. `measuredPeriod` is undefined until one cycle. |
| `snapshot()` | State, net force, accel, energy lost, steps, measured period, cycles, `frame` (body, axis, `pose`, forces, guides), `derived` (each row tiered like a param, each carrying `symbol` and, unless it is a parameter, `from: {equation, terms}`, terms being rows of the same shape). |
| `history()`, `historyLength` | Flat `[t, x, v, ...]`, oldest first. One typed array instead of thousands of objects. |

## Rules

- No physics logic in this file. If the frontend needs a number, add it to a scenario's
  `derived()` or to `physics-core`, not here.
- Nothing names a concrete scenario or parameter. Adding a scenario needs no change here.
- Serialization goes through `serde-wasm-bindgen`; errors come back as `JsError`.

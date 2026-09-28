# physics-core

Scenario-agnostic simulation core. Knows how to step one degree of freedom
forward given the forces on it. Knows nothing about springs or inclines.

## Files

| File | What is in it |
| --- | --- |
| `src/math.rs` | `Vec2` (dot, cross, rotated, perpendicular, normalized, component_along), `G = 9.80665`. World +y is up. |
| `src/state.rs` | `State { t, x, v }`. `x` is a generalized coordinate, meaning set by the scenario. `kinetic_energy`, `is_finite`. |
| `src/system.rs` | `System` trait: `accel(x, v, t)`. The only thing integrators see. `ConstantAccel` for tests. |
| `src/integrator.rs` | `Integrator` trait plus `ExplicitEuler`, `SemiImplicitEuler` (default), `VelocityVerlet`. `all()`, `by_id()`, `default_integrator()`. |
| `src/scenario.rs` | `Scenario` trait (the plug-in point), `ForceVector`, `Guide`, `Frame`, `BodyPose`, `Derived`, `Derivation`, and `ScenarioSystem` (the one place `F = ma` is divided out). |
| `src/params.rs` | Self-describing parameters: `ParamSpec`, `Tier`, `Mark`, `ParamDef`, the `bind!` macro, `get`/`set`/`specs`/`defaults`, `ParamError`. |
| `src/record.rs` | `Recorder`, fixed-capacity ring buffer with a stride. `to_flat()` gives `[t, x, v, ...]` for the JS boundary. Default 4096 samples, stride 1. |
| `src/analysis.rs` | `PeriodDetector` (interpolated level crossings), `period_of`, `extent`/`Extent`, `percent_difference`, `percent_error`. |
| `src/sim.rs` | `Simulation` driver and `Snapshot`. Owns scenario + integrator + clock. |

## Things that matter

- `Scenario` is object-safe, so the registry hands back `Box<dyn Scenario>`.
- `bind!(Ty, field, spec)` ties a `ParamSpec` to a struct field with plain fn pointers, so
  reading a parameter mid-step is a field access, not a string lookup. The spec is an
  expression: scenarios take theirs from the catalog in `physics-scenarios`, and
  `with_default` / `with_label` adjust it without touching the range.
- `ParamSpec::marks` are the values on a range worth pointing at: Earth gravity, 45 degrees on
  a ramp. The UI marks them on the track so they are easy to hit exactly. Naming what is
  notable about a range is the scenario's call, which is why they live with the spec. They are
  write-only across serde (a borrowed slice cannot outlive the input it was read from).
- `Tier` is on both `ParamSpec` and `Derived`. It is the engine saying which knobs and which
  readout rows a short panel should show, so the frontend never has to name a parameter.
- `Derived::explain(equation, terms)` says where a number came from: the formula in symbols,
  plus the terms in it, which are `Derived` themselves. So a derived row is a tree that bottoms
  out at parameters and state, and the readout can unfold it one step at a time. `sym` names the
  symbol a quantity is written with, and the equation is written out of those symbols. Building
  the tree is the scenario's job, since the scenario is what knows the formula.
- `Frame::pose` is `BodyPose { angle, support }`, drawing only: how the body is turned, and
  which way is out of whatever it is resting on. `frame.body` stays on the surface, so a
  renderer lifts the drawn box itself.
- Integrators take `System`, not `Scenario`. They cannot see parameters or geometry.
- `net_force(x, v, t)` must be pure: integrators call it at trial points, more than once per step.
- `Scenario::constrain(previous, state)` runs after every step, for anything a force law
  cannot express (a block stopping at the end of a ramp, static friction pinning a body).
- `Scenario::max_stable_dt()` lets a stiff setup ask the driver to subdivide. `Simulation::step`
  cuts the step into equal pieces so a step is always exactly `dt` of simulated time.
  Capped by `MAX_SUBDIVISIONS = 256`.
- Energy ledger: `energy_lost` accumulates `dissipated_power * h` per piece, plus any kinetic
  energy `constrain` removed. It is the only path-dependent quantity in the engine.
- `advance(elapsed)` takes whole fixed steps and carries the leftover. `MAX_SUBSTEPS = 2000`
  caps catch-up, past which the backlog is dropped. `DEFAULT_DT = 1/480 s`.
- `run_for(duration)` ignores the real-time accumulator. Tests and offline runs use it.

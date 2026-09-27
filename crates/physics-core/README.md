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
| `src/scenario.rs` | `Scenario` trait (the plug-in point), `ForceVector`, `Guide`, `Frame`, `Derived`, and `ScenarioSystem` (the one place `F = ma` is divided out). |
| `src/params.rs` | Self-describing parameters: `ParamSpec`, `ParamDef`, the `param!` macro, `get`/`set`/`specs`/`defaults`, `ParamError`. |
| `src/record.rs` | `Recorder`, fixed-capacity ring buffer with a stride. `to_flat()` gives `[t, x, v, ...]` for the JS boundary. Default 4096 samples, stride 1. |
| `src/analysis.rs` | `PeriodDetector` (interpolated level crossings), `period_of`, `extent`/`Extent`, `percent_difference`, `percent_error`. |
| `src/sim.rs` | `Simulation` driver and `Snapshot`. Owns scenario + integrator + clock. |

## Things that matter

- `Scenario` is object-safe, so the registry hands back `Box<dyn Scenario>`.
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

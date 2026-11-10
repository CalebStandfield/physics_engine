# physics-core

Scenario-agnostic simulation core. Knows how to step one degree of freedom
forward given the forces on it. Knows nothing about springs, inclines or pivots.
The coordinate is generalized: a distance with a mass and a force, or an angle with a
moment of inertia and a torque. The core only ever divides one by the other.

## Files

| File | What is in it |
| --- | --- |
| `src/math.rs` | `Vec2` (dot, cross, rotated, perpendicular, normalized, component_along), `agm` (arithmetic-geometric mean, for the exact pendulum period), `G = 9.80665`. World +y is up. |
| `src/state.rs` | `State { t, x, v }`. `x` is a generalized coordinate, meaning set by the scenario. `kinetic_energy`, `is_finite`. |
| `src/system.rs` | `System` trait: `accel(x, v, t)`. The only thing integrators see. `ConstantAccel` for tests. |
| `src/integrator.rs` | `Integrator` trait plus `ExplicitEuler`, `SemiImplicitEuler` (default), `VelocityVerlet`. `all()`, `by_id()`, `default_integrator()`. |
| `src/scenario.rs` | `Scenario` trait (the plug-in point), `ForceVector`, `Couple`, `Guide`, `Frame`, `BodyPose`, `BodyShape`, `Derived`, `Derivation`, and `ScenarioSystem` (the one place `F = ma` / `tau = I alpha` is divided out). |
| `src/params.rs` | Self-describing parameters: `ParamSpec`, `Tier`, `Widget`, `Mark`, `ParamDef`, the `bind!` macro, `get`/`set`/`specs`/`defaults`, `ParamError`. |
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
- `Scenario::inertia()` is mass (kg) for a distance coordinate, moment of inertia about the
  pivot (kg m^2) for an angle. `net_force` is a force (N) or a torque (N m, ccw +) to match.
  `State::kinetic_energy(inertia)` is then `m v^2 / 2` or `I omega^2 / 2` with no special case.
- `ForceVector::at` is where a force acts, `None` meaning the body's center. `Frame::pivot` is
  the point a rotating body turns about. `Frame::generalized_force()` reduces the drawn forces
  to torque about the pivot (sum of r x F) when there is one, the component along `axis`
  otherwise; a contract test holds it equal to `net_force` for every scenario.
- `Frame::couples` are pure torques (`Couple { label, kind, torque, at }`, N m ccw +), for
  something with no line of action like friction at a nail. `at` is only where to draw it.
  `generalized_force()` adds them in when there is a pivot.
- `ParamSpec::widget` is `Slider` (default) or `Switch { on }` (`with_switch(on)`): a slider a
  simple panel shows as a checkbox, checked while above `min`, checking sets `on`, unchecking
  sets `min`. Still one `f64`, so binding and validation do not care which it is.
- `Frame::shape` is `BodyShape::Block` (point mass, fixed px size) or `Rod { length, width }`
  (to scale, centered on `body`, lying along `pose.angle`). Drawing only.
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
- `set_param(key, value)` resets too while `steps == 0`, so before Start the state always
  matches the controls; once the run moves, starting values wait for `reset()`.
- `run_for(duration)` ignores the real-time accumulator. Tests and offline runs use it.

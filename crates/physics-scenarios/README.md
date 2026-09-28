# physics-scenarios

The two concrete scenarios. Each implements `physics_core::Scenario`: force law,
geometry, derived quantities. No time-stepping lives here.

## Files

| File | What is in it |
| --- | --- |
| `src/spring.rs` | `SpringScenario`, `SpringParams`. Block hanging from a vertical spring. |
| `src/incline.rs` | `InclineScenario`, `InclineParams`. Block on a ramp with friction. |
| `src/controls.rs` | The control catalog: every slider any scenario can offer, with its range, step, unit, tier and marks. `control(key)`, `all()`. |
| `src/registry.rs` | id -> scenario factory. `ids()`, `create(id)`, `default_scenario()`. |
| `examples/report.rs` | Prints engine numbers next to the hand calculation for both scenarios. Anything quoted in the write-up comes from here. `cargo run -p physics-scenarios --example report`. |
| `tests/engine.rs` | End-to-end checks. The contract tests at the bottom loop over the registry, so a new scenario inherits them. |

## Spring

Coordinate `x` is stretch past natural length, measured down. Axis is world `-y`,
anchor at the origin, block at `y = -(L0 + x)`.

- `F = m g - k x - b v`, zero at `x0 = m g / k` (`equilibrium_stretch`).
- `ideal_period` `T = 2 pi sqrt(m / k)`, `angular_frequency` `sqrt(k / m)`,
  `damping_ratio` `b / (2 sqrt(m k))`, `damped_period` `T / sqrt(1 - z^2)` (None once `z >= 1`).
- Controls: `mass` (default 0.25), `stiffness`, `natural_length`, `initial_displacement`,
  `initial_velocity` (relabeled "down +"), `damping`, `gravity`.
- `max_stable_dt` = half of `min(2 / omega, 2 m / b)`.
- Guides: `Ceiling` (anchor), `Natural length` and `Equilibrium` (reference), `Spring`.

## Incline

Coordinate `x` is distance down the slope from the top. High end at the origin,
axis `(cos t, -sin t)`, outward normal `(sin t, cos t)`.

- `N = m g cos(theta)`, drive = `m g sin(theta) + applied_force`.
- Sliding: friction is `mu_k N` opposing velocity. At rest: friction cancels the drive up to
  `mu_s N`. Breaking loose caps friction at the drive size, so `mu_k > mu_s` cannot push a
  block backwards out of rest. `REST_SPEED = 1e-6` is the at-rest threshold.
- `ideal_sliding_accel` `g (sin theta - mu_k cos theta)`. Holds still while `tan theta <= mu_s`.
- `constrain` parks the block at rest on a mid-step velocity sign flip, and stops it at
  either end of the finite ramp.
- Controls: `mass` (default 1.5), `angle_deg`, `mu_static`, `mu_kinetic`, `length`,
  `initial_position`, `initial_velocity` (relabeled "downhill +"), `applied_force`, `gravity`.
- Guides: `Slope` (surface), `Rise` and `Run` (reference), `Slope direction` (axis).
- `pose` turns the drawn block by `-theta` and points `support` out of the surface, so the block
  lies along the ramp and sits on it. `frame.body` stays on the slope line.

## Controls

A scenario does not invent a slider. It picks keys out of `controls.rs` and binds them to its
own fields with `bind!`, so a control two scenarios share has the same range, step and unit in
both: `mass` is 0.01-100 kg everywhere, whoever is asking. Only the default and the label are
per-scenario, through `with_default` and `with_label`. A registry test checks every schema entry
still matches the catalog, so the ranges cannot drift apart again.

A control can also carry marks, through `with_marks`: the values on its range worth pointing at,
which the UI puts on the track as clickable ticks. Gravity marks the Moon, Mars and Earth, the
incline angle marks 15/30/45/60 degrees, and the ranges that straddle zero mark it (at rest,
equilibrium, no applied force). A test keeps every mark inside its own range.

Each control also carries a `Tier`. Basic is the short list a simple panel shows (`mass`,
`stiffness`, `initial_displacement`, `angle_deg`, `mu_static`, `mu_kinetic`); everything else,
gravity and the initial-condition knobs included, is advanced. `Derived` rows carry the same
tier, so one switch in the UI trims both panels.

## Derivations

Every derived row a scenario reports says how it was computed, through
`Derived::explain(equation, terms)`: the formula in symbols and the values in it, each of which
may carry its own formula. `a = F_net / m` opens onto the net force, which opens onto the
weight component and friction, which open onto the mass, gravity and the angle, and it stops at
parameters and state.

Both scenarios build those trees in one block of small `*_term` methods, so the same node is
written once and reused wherever it appears. The numbers in them come from the physics methods,
never from a second copy of the formula, so a tree cannot drift from what is simulated.
Friction's tree names whichever of its three cases is in effect (sliding, held, breaking loose),
which is how the readout can say why the number is what it is. A contract test walks every tree
in every scenario and checks that each formula names a symbol for every term under it.

## Adding a scenario

One new module implementing `Scenario`, one `use` and one entry in `SCENARIOS` in
`registry.rs`. Reuse the control keys that already exist and add new ones to `controls.rs`. Nothing in the core, the bindings or the JS changes. The frontend reads
the catalog and the parameter schema at runtime, so it gets a landing page card, a tab,
sliders, a legend and a readout for free.

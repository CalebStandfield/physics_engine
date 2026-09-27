# physics-scenarios

The two concrete scenarios. Each implements `physics_core::Scenario`: force law,
geometry, derived quantities. No time-stepping lives here.

## Files

| File | What is in it |
| --- | --- |
| `src/spring.rs` | `SpringScenario`, `SpringParams`. Block hanging from a vertical spring. |
| `src/incline.rs` | `InclineScenario`, `InclineParams`. Block on a ramp with friction. |
| `src/registry.rs` | id -> scenario factory. `ids()`, `create(id)`, `default_scenario()`. |
| `examples/report.rs` | Prints engine numbers next to the hand calculation for both scenarios. Anything quoted in the write-up comes from here. `cargo run -p physics-scenarios --example report`. |
| `tests/engine.rs` | End-to-end checks. The contract tests at the bottom loop over the registry, so a new scenario inherits them. |

## Spring

Coordinate `x` is stretch past natural length, measured down. Axis is world `-y`,
anchor at the origin, block at `y = -(L0 + x)`.

- `F = m g - k x - b v`, zero at `x0 = m g / k` (`equilibrium_stretch`).
- `ideal_period` `T = 2 pi sqrt(m / k)`, `angular_frequency` `sqrt(k / m)`,
  `damping_ratio` `b / (2 sqrt(m k))`, `damped_period` `T / sqrt(1 - z^2)` (None once `z >= 1`).
- Params: `mass`, `stiffness`, `natural_length`, `initial_displacement`,
  `initial_velocity`, `damping`, `gravity`.
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
- Params: `mass`, `angle_deg`, `mu_static`, `mu_kinetic`, `length`, `initial_position`,
  `initial_velocity`, `applied_force`, `gravity`.
- Guides: `Slope` (surface), `Rise` and `Run` (reference), `Slope direction` (axis).

## Adding a scenario

One new module implementing `Scenario`, one `use` and one entry in `SCENARIOS` in
`registry.rs`. Nothing in the core, the bindings or the JS changes. The frontend reads
the catalog and the parameter schema at runtime, so it gets tabs, sliders, a legend and a
readout for free.

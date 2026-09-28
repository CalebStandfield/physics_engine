//! Block hanging from a vertical spring.
//!
//! Coordinate: `x` is the spring's stretch past its natural length, in meters,
//! measured downward. So the axis points down (world `-y`), the anchor sits at
//! the world origin, and the block hangs at `y = -(L0 + x)`.
//!
//! Forces along that axis:
//!   weight   = +m g        (down, along the axis)
//!   spring   = -k x        (Hooke's law, pulls back toward natural length)
//!   damping  = -b v        (air drag / internal loss, zero by default)
//!
//! so the net force is `F = m g - k x - b v`, which is zero at the equilibrium
//! stretch `x0 = m g / k`. Writing `u = x - x0` turns that into `F = -k u`, the
//! plain Hooke's-law oscillator, with period `T = 2 pi sqrt(m / k)`.

use physics_core::bind;
use physics_core::math::Vec2;
use physics_core::params::{self, ParamDef, ParamError, ParamSpec};
use physics_core::scenario::{BodyPose, Derived, ForceVector, Frame, Guide, Scenario};
use physics_core::state::State;

use crate::controls::control;

/// Tunable inputs, all SI.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct SpringParams {
    /// Hanging mass, kg.
    pub mass: f64,
    /// Spring constant, N/m.
    pub stiffness: f64,
    /// Unstretched spring length, m. Geometry only, it does not affect motion.
    pub natural_length: f64,
    /// How far the block is pulled past equilibrium to start, m. Positive is
    /// further down.
    pub initial_displacement: f64,
    /// Starting velocity, m/s, positive downward.
    pub initial_velocity: f64,
    /// Linear damping, N per m/s. Zero is an ideal frictionless spring.
    pub damping: f64,
    /// Gravitational field strength, m/s^2.
    pub gravity: f64,
}

fn defs() -> Vec<ParamDef<SpringParams>> {
    vec![
        bind!(SpringParams, mass, control("mass").with_default(0.25)),
        bind!(SpringParams, stiffness, control("stiffness")),
        bind!(SpringParams, natural_length, control("natural_length")),
        bind!(SpringParams, initial_displacement, control("initial_displacement")),
        bind!(
            SpringParams,
            initial_velocity,
            control("initial_velocity").with_label("Initial velocity (down +)")
        ),
        bind!(SpringParams, damping, control("damping")),
        bind!(SpringParams, gravity, control("gravity")),
    ]
}

/// Mass on a vertical spring.
pub struct SpringScenario {
    params: SpringParams,
    defs: Vec<ParamDef<SpringParams>>,
}

impl SpringScenario {
    pub fn new() -> Self {
        let defs = defs();
        let params = params::defaults(&defs);
        Self { params, defs }
    }

    pub fn params(&self) -> SpringParams {
        self.params
    }

    /// Stretch at which the spring force balances the weight, m.
    pub fn equilibrium_stretch(&self) -> f64 {
        self.params.mass * self.params.gravity / self.params.stiffness
    }

    /// Ideal period of an undamped massless spring, `T = 2 pi sqrt(m / k)`, s.
    pub fn ideal_period(&self) -> f64 {
        std::f64::consts::TAU * (self.params.mass / self.params.stiffness).sqrt()
    }

    /// Undamped angular frequency `sqrt(k / m)`, rad/s.
    pub fn angular_frequency(&self) -> f64 {
        (self.params.stiffness / self.params.mass).sqrt()
    }

    /// Period the block actually oscillates at once damping is included,
    /// `T / sqrt(1 - z^2)`, s. `None` once damping is strong enough that it
    /// stops oscillating at all.
    pub fn damped_period(&self) -> Option<f64> {
        let z = self.damping_ratio();
        if z < 1.0 {
            Some(self.ideal_period() / (1.0 - z * z).sqrt())
        } else {
            None
        }
    }

    /// `b / (2 sqrt(m k))`. Below 1 the block still oscillates.
    pub fn damping_ratio(&self) -> f64 {
        self.params.damping / (2.0 * (self.params.mass * self.params.stiffness).sqrt())
    }

    /// Spring force along the axis (down positive): `-k x`, N.
    fn spring_force(&self, x: f64) -> f64 {
        -self.params.stiffness * x
    }

    /// Weight along the axis (down positive): `+m g`, N.
    fn weight(&self) -> f64 {
        self.params.mass * self.params.gravity
    }

    /// Damping force along the axis: `-b v`, N.
    fn damping_force(&self, v: f64) -> f64 {
        -self.params.damping * v
    }

    /// World position of the block for a given stretch.
    fn body_position(&self, x: f64) -> Vec2 {
        Vec2::new(0.0, -(self.params.natural_length + x))
    }

    // ---- derivation terms ----
    //
    // One method per node of the tree the readout unfolds. A parameter or a
    // piece of state is a bare value; everything else names its formula and the
    // terms under it, which are these same methods. The numbers come from the
    // physics methods above, so the tree cannot drift from what is simulated.

    fn mass_term(&self) -> Derived {
        Derived::new("mass", "Mass", "kg", self.params.mass).sym("m")
    }

    fn gravity_term(&self) -> Derived {
        Derived::new("gravity", "Gravity", "m/s^2", self.params.gravity).sym("g")
    }

    fn stiffness_term(&self) -> Derived {
        Derived::new("stiffness", "Spring constant", "N/m", self.params.stiffness).sym("k")
    }

    fn damping_term(&self) -> Derived {
        Derived::new("damping", "Damping", "N s/m", self.params.damping).sym("b")
    }

    fn natural_length_term(&self) -> Derived {
        Derived::new("natural_length", "Natural length", "m", self.params.natural_length)
            .sym("L_0")
    }

    fn stretch_term(&self, x: f64) -> Derived {
        Derived::new("stretch", "Stretch past natural length", "m", x).sym("x")
    }

    fn velocity_term(&self, v: f64) -> Derived {
        Derived::new("velocity", "Velocity (down +)", "m/s", v).sym("v")
    }

    fn weight_term(&self) -> Derived {
        Derived::new("weight", "Weight", "N", self.weight())
            .sym("W")
            .explain("m g", vec![self.mass_term(), self.gravity_term()])
    }

    fn equilibrium_stretch_term(&self) -> Derived {
        // The stretch at which the spring pulls back exactly as hard as gravity
        // pulls down, so `k x0 = W`.
        Derived::new("equilibrium_stretch", "Equilibrium stretch", "m", self.equilibrium_stretch())
            .sym("x_0")
            .explain("W / k", vec![self.weight_term(), self.stiffness_term()])
    }

    fn ideal_period_term(&self) -> Derived {
        Derived::new("ideal_period", "Ideal period (undamped)", "s", self.ideal_period())
            .sym("T")
            .explain(
                "2 pi sqrt(m / k)",
                vec![self.mass_term(), self.stiffness_term()],
            )
    }

    fn damping_ratio_term(&self) -> Derived {
        Derived::new("damping_ratio", "Damping ratio", "", self.damping_ratio())
            .sym("z")
            .explain(
                "b / (2 sqrt(m k))",
                vec![self.damping_term(), self.mass_term(), self.stiffness_term()],
            )
    }

    /// Height of the block, measured from the ceiling anchor, m. Negative, since
    /// the block hangs below it.
    fn height_term(&self, x: f64) -> Derived {
        Derived::new("height", "Height below the anchor", "m", self.body_position(x).y)
            .sym("y")
            .explain(
                "-(L_0 + x)",
                vec![self.natural_length_term(), self.stretch_term(x)],
            )
    }

    fn kinetic_energy_term(&self, state: &State) -> Derived {
        Derived::new("kinetic_energy", "Kinetic energy", "J", state.kinetic_energy(self.params.mass))
            .sym("KE")
            .explain(
                "m v^2 / 2",
                vec![self.mass_term(), self.velocity_term(state.v)],
            )
    }

    fn spring_energy_term(&self, x: f64) -> Derived {
        Derived::new("spring_energy", "Spring energy", "J", 0.5 * self.params.stiffness * x * x)
            .sym("PE_spring")
            .explain(
                "k x^2 / 2",
                vec![self.stiffness_term(), self.stretch_term(x)],
            )
    }

    fn gravitational_energy_term(&self, x: f64) -> Derived {
        Derived::new(
            "gravitational_energy",
            "Gravitational energy",
            "J",
            self.params.mass * self.params.gravity * self.body_position(x).y,
        )
        .sym("PE_grav")
        .explain(
            "m g y",
            vec![self.mass_term(), self.gravity_term(), self.height_term(x)],
        )
    }
}

impl Default for SpringScenario {
    fn default() -> Self {
        Self::new()
    }
}

/// Down. Positive `State::x` moves the block this way.
const AXIS: Vec2 = Vec2::new(0.0, -1.0);

impl Scenario for SpringScenario {
    fn id(&self) -> &'static str {
        "spring"
    }

    fn name(&self) -> &'static str {
        "Block hanging from a spring"
    }

    fn description(&self) -> &'static str {
        "A block hangs from a vertical spring. Gravity pulls it down, the spring pulls it \
         back, and the block oscillates about the stretch where those two balance."
    }

    fn coordinate_label(&self) -> &'static str {
        "Stretch past natural length (down +)"
    }

    fn schema(&self) -> Vec<ParamSpec> {
        params::specs(&self.defs)
    }

    fn get_param(&self, key: &str) -> Option<f64> {
        params::get(&self.defs, &self.params, key)
    }

    fn set_param(&mut self, key: &str, value: f64) -> Result<(), ParamError> {
        params::set(&self.defs, &mut self.params, key, value)
    }

    fn mass(&self) -> f64 {
        self.params.mass
    }

    fn initial_state(&self) -> State {
        State::new(
            0.0,
            self.equilibrium_stretch() + self.params.initial_displacement,
            self.params.initial_velocity,
        )
    }

    fn net_force(&self, x: f64, v: f64, _t: f64) -> f64 {
        self.weight() + self.spring_force(x) + self.damping_force(v)
    }

    fn dissipated_power(&self, state: &State) -> f64 {
        // |(-b v) * v| = b v^2.
        self.params.damping * state.v * state.v
    }

    fn max_stable_dt(&self) -> Option<f64> {
        // Explicit schemes need the step under 2 / omega for the spring, and
        // under 2 m / b for the damping. Half that, for margin.
        let by_stiffness = 2.0 / self.angular_frequency();
        let limit = if self.params.damping > 0.0 {
            by_stiffness.min(2.0 * self.params.mass / self.params.damping)
        } else {
            by_stiffness
        };
        Some(0.5 * limit)
    }

    fn equilibrium(&self) -> Option<f64> {
        Some(self.equilibrium_stretch())
    }

    fn frame(&self, state: &State) -> Frame {
        let mut forces = vec![
            ForceVector::new("Weight", "gravity", AXIS * self.weight()),
            ForceVector::new("Spring force", "spring", AXIS * self.spring_force(state.x)),
        ];
        if self.params.damping > 0.0 {
            forces.push(ForceVector::new(
                "Damping",
                "damping",
                AXIS * self.damping_force(state.v),
            ));
        }

        let half = 0.12;
        let natural_y = -self.params.natural_length;
        let equilibrium_y = -(self.params.natural_length + self.equilibrium_stretch());
        let guides = vec![
            Guide::new(
                "Ceiling",
                "anchor",
                vec![Vec2::new(-half * 1.5, 0.0), Vec2::new(half * 1.5, 0.0)],
            ),
            Guide::new(
                "Natural length",
                "reference",
                vec![
                    Vec2::new(-half, natural_y),
                    Vec2::new(half, natural_y),
                ],
            ),
            Guide::new(
                "Equilibrium",
                "reference",
                vec![
                    Vec2::new(-half, equilibrium_y),
                    Vec2::new(half, equilibrium_y),
                ],
            ),
            Guide::new(
                "Spring",
                "spring",
                vec![Vec2::ZERO, self.body_position(state.x)],
            ),
        ];

        Frame {
            body: self.body_position(state.x),
            axis: AXIS,
            // Hanging free: nothing to sit on, nothing to tilt against.
            pose: BodyPose::default(),
            forces,
            guides,
        }
    }

    fn derived(&self, state: &State) -> Vec<Derived> {
        let mut out = vec![
            self.equilibrium_stretch_term().advanced(),
            Derived::new(
                "displacement",
                "Displacement from equilibrium",
                "m",
                state.x - self.equilibrium_stretch(),
            )
            .sym("u")
            .explain(
                "x - x_0",
                vec![self.stretch_term(state.x), self.equilibrium_stretch_term()],
            ),
            self.ideal_period_term(),
            Derived::new("spring_force", "Spring force", "N", self.spring_force(state.x))
                .sym("F_spring")
                .explain(
                    "-k x",
                    vec![self.stiffness_term(), self.stretch_term(state.x)],
                ),
            self.weight_term().advanced(),
            self.kinetic_energy_term(state).advanced(),
            self.spring_energy_term(state.x).advanced(),
            self.gravitational_energy_term(state.x).advanced(),
            Derived::new(
                "mechanical_energy",
                "Total mechanical energy",
                "J",
                state.kinetic_energy(self.params.mass)
                    + 0.5 * self.params.stiffness * state.x * state.x
                    + self.params.mass * self.params.gravity * self.body_position(state.x).y,
            )
            .sym("E")
            .explain(
                "KE + PE_spring + PE_grav",
                vec![
                    self.kinetic_energy_term(state),
                    self.spring_energy_term(state.x),
                    self.gravitational_energy_term(state.x),
                ],
            )
            .advanced(),
        ];

        if self.params.damping > 0.0 {
            out.push(self.damping_ratio_term().advanced());
            // Damping stretches the period, so the undamped formula is not
            // what a stopwatch would read here.
            if let Some(damped) = self.damped_period() {
                out.push(
                    Derived::new("damped_period", "Damped period", "s", damped)
                        .sym("T_d")
                        .explain(
                            "T / sqrt(1 - z^2)",
                            vec![self.ideal_period_term(), self.damping_ratio_term()],
                        )
                        .advanced(),
                );
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use physics_core::sim::Simulation;

    fn spring(mass: f64, stiffness: f64) -> SpringScenario {
        let mut s = SpringScenario::new();
        s.set_param("mass", mass).unwrap();
        s.set_param("stiffness", stiffness).unwrap();
        s
    }

    #[test]
    fn equilibrium_is_where_weight_balances_the_spring() {
        let s = spring(0.25, 20.0);
        let x0 = s.equilibrium_stretch();
        assert!((x0 - 0.25 * physics_core::G / 20.0).abs() < 1e-12);
        assert!(s.net_force(x0, 0.0, 0.0).abs() < 1e-12);
    }

    #[test]
    fn released_at_equilibrium_it_stays_put() {
        let mut s = spring(0.25, 20.0);
        s.set_param("initial_displacement", 0.0).unwrap();
        let mut sim = Simulation::new(Box::new(s));
        sim.run_for(3.0);
        let x0 = SpringScenario::new().equilibrium_stretch();
        assert!((sim.state().x - x0).abs() < 1e-6);
        assert!(sim.state().v.abs() < 1e-6);
    }

    #[test]
    fn undamped_motion_repeats_after_one_ideal_period() {
        let s = spring(0.25, 20.0);
        let period = s.ideal_period();
        let start = s.initial_state();
        let mut sim = Simulation::new(Box::new(s));
        sim.set_dt(1e-5);
        sim.run_for(period);
        assert!((sim.state().x - start.x).abs() < 1e-4, "x drifted");
        assert!((sim.state().v - start.v).abs() < 1e-3, "v drifted");
    }

    #[test]
    fn damping_settles_the_block_at_equilibrium() {
        let mut s = spring(0.25, 20.0);
        s.set_param("damping", 3.0).unwrap();
        let x0 = s.equilibrium_stretch();
        let mut sim = Simulation::new(Box::new(s));
        sim.run_for(20.0);
        assert!((sim.state().x - x0).abs() < 1e-4);
        assert!(sim.state().v.abs() < 1e-4);
        assert!(sim.energy_lost() > 0.0, "damping should remove energy");
    }

    #[test]
    fn undamped_mechanical_energy_holds() {
        let s = spring(0.25, 20.0);
        let energy = |sim: &Simulation| {
            sim.snapshot()
                .derived
                .iter()
                .find(|d| d.key == "mechanical_energy")
                .unwrap()
                .value
        };
        let mut sim = Simulation::new(Box::new(s));
        let before = energy(&sim);
        sim.run_for(10.0);
        let after = energy(&sim);
        assert!((after - before).abs() < 1e-3, "{before} -> {after}");
    }

    #[test]
    fn drawn_forces_add_up_to_the_net_force() {
        let mut s = spring(0.4, 35.0);
        s.set_param("damping", 1.2).unwrap();
        let state = State::new(0.0, 0.2, -0.7);
        let frame = s.frame(&state);
        let scalar = s.net_force(state.x, state.v, state.t);
        assert!((frame.net_force() - AXIS * scalar).len() < 1e-12);
        assert_eq!(frame.forces.len(), 3);
    }

    #[test]
    fn schema_defaults_round_trip() {
        let s = SpringScenario::new();
        for spec in s.schema() {
            assert_eq!(s.get_param(spec.key), Some(spec.default), "{}", spec.key);
        }
    }
}

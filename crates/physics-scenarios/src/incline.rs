//! Block on an inclined plane.
//!
//! Coordinate: `x` is distance measured down the slope from the top of the
//! ramp, in meters. The ramp's high end sits at the world origin and the
//! surface runs down and to the right, so the axis is
//! `(cos theta, -sin theta)` and the outward surface normal is
//! `(sin theta, cos theta)`.
//!
//! Gravity splits into a piece along the slope and a piece into it:
//!   parallel      = m g sin(theta)     (drives the block downhill)
//!   perpendicular = m g cos(theta)     (pressed into the surface)
//!
//! The surface pushes back with exactly the perpendicular piece, so
//! `N = m g cos(theta)`. Along the slope the block feels that parallel piece
//! plus any applied push, resisted by friction:
//!   sliding: friction = mu_k N, opposing the velocity
//!   at rest: friction cancels the drive, up to a limit of mu_s N. Past that
//!            limit the block breaks loose and kinetic friction takes over.
//!
//! So a block sliding freely downhill accelerates at
//! `a = g (sin theta - mu_k cos theta)`, and it does not move at all while
//! `tan(theta) <= mu_s`.

use physics_core::math::Vec2;
use physics_core::param;
use physics_core::params::{self, ParamDef, ParamError, ParamSpec};
use physics_core::scenario::{Derived, ForceVector, Frame, Guide, Scenario};
use physics_core::state::State;

/// Speeds under this count as at rest, m/s. Below it the static friction test
/// applies instead of the kinetic one.
const REST_SPEED: f64 = 1e-6;

/// Tunable inputs, all SI except the angle.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct InclineParams {
    /// Block mass, kg.
    pub mass: f64,
    /// Incline angle above horizontal, degrees.
    pub angle_deg: f64,
    /// Coefficient of static friction.
    pub mu_static: f64,
    /// Coefficient of kinetic friction.
    pub mu_kinetic: f64,
    /// Length of the ramp surface, m.
    pub length: f64,
    /// Where the block starts, m down the slope from the top.
    pub initial_position: f64,
    /// Starting velocity, m/s, positive down-slope.
    pub initial_velocity: f64,
    /// Extra steady push along the slope, N, positive down-slope.
    pub applied_force: f64,
    /// Gravitational field strength, m/s^2.
    pub gravity: f64,
}

fn defs() -> Vec<ParamDef<InclineParams>> {
    vec![
        param!(InclineParams, mass, "mass", "Mass", "kg", 0.01, 100.0, 1.5, 0.01),
        param!(InclineParams, angle_deg, "angle_deg", "Incline angle", "deg", 0.0, 89.0, 25.0, 0.5),
        param!(InclineParams, mu_static, "mu_static", "Static friction coefficient", "", 0.0, 2.0, 0.35, 0.01),
        param!(InclineParams, mu_kinetic, "mu_kinetic", "Kinetic friction coefficient", "", 0.0, 2.0, 0.25, 0.01),
        param!(InclineParams, length, "length", "Ramp length", "m", 0.1, 20.0, 2.0, 0.05),
        param!(InclineParams, initial_position, "initial_position", "Start distance from top", "m", 0.0, 20.0, 0.0, 0.05),
        param!(InclineParams, initial_velocity, "initial_velocity", "Initial velocity (downhill +)", "m/s", -10.0, 10.0, 0.0, 0.05),
        param!(InclineParams, applied_force, "applied_force", "Applied force along slope", "N", -200.0, 200.0, 0.0, 0.5),
        param!(InclineParams, gravity, "gravity", "Gravity", "m/s^2", 0.1, 30.0, physics_core::G, 0.01),
    ]
}

/// Block on a ramp.
pub struct InclineScenario {
    params: InclineParams,
    defs: Vec<ParamDef<InclineParams>>,
}

impl InclineScenario {
    pub fn new() -> Self {
        let defs = defs();
        let params = params::defaults(&defs);
        Self { params, defs }
    }

    pub fn params(&self) -> InclineParams {
        self.params
    }

    /// Incline angle in radians.
    pub fn angle(&self) -> f64 {
        self.params.angle_deg.to_radians()
    }

    /// Unit vector pointing down the slope.
    pub fn axis(&self) -> Vec2 {
        let a = self.angle();
        Vec2::new(a.cos(), -a.sin())
    }

    /// Unit vector pointing out of the surface.
    pub fn normal_direction(&self) -> Vec2 {
        let a = self.angle();
        Vec2::new(a.sin(), a.cos())
    }

    /// Weight of the block, N.
    pub fn weight(&self) -> f64 {
        self.params.mass * self.params.gravity
    }

    /// Gravity's component along the slope, N, positive downhill.
    pub fn gravity_parallel(&self) -> f64 {
        self.weight() * self.angle().sin()
    }

    /// Gravity's component into the surface, N.
    pub fn gravity_perpendicular(&self) -> f64 {
        self.weight() * self.angle().cos()
    }

    /// Normal force, N. Equal to the perpendicular weight component, since
    /// nothing accelerates the block away from the surface.
    pub fn normal_force(&self) -> f64 {
        self.gravity_perpendicular()
    }

    /// Everything pushing the block along the slope before friction, N.
    pub fn drive_force(&self) -> f64 {
        self.gravity_parallel() + self.params.applied_force
    }

    /// Largest force static friction can hold, `mu_s N`, N.
    pub fn max_static_friction(&self) -> f64 {
        self.params.mu_static * self.normal_force()
    }

    /// Size of kinetic friction while sliding, `mu_k N`, N.
    pub fn kinetic_friction(&self) -> f64 {
        self.params.mu_kinetic * self.normal_force()
    }

    /// Acceleration of a block sliding freely downhill,
    /// `g (sin theta - mu_k cos theta)`, m/s^2. The textbook answer, for
    /// comparison against what the simulation actually produces.
    pub fn ideal_sliding_accel(&self) -> f64 {
        self.params.gravity
            * (self.angle().sin() - self.params.mu_kinetic * self.angle().cos())
    }

    /// True while static friction can hold the block where it is.
    pub fn holds_at_rest(&self) -> bool {
        self.drive_force().abs() <= self.max_static_friction()
    }

    /// Friction force along the slope at a trial point, N, signed downhill.
    fn friction_force(&self, v: f64) -> f64 {
        if v.abs() > REST_SPEED {
            // Sliding: constant size, always opposing the motion.
            -v.signum() * self.kinetic_friction()
        } else if self.holds_at_rest() {
            // Stuck: friction matches the drive exactly and nothing moves.
            -self.drive_force()
        } else {
            // Breaking loose: it starts sliding the way the drive points.
            -self.drive_force().signum() * self.kinetic_friction()
        }
    }

    /// World position of the block for a distance down the slope.
    fn body_position(&self, x: f64) -> Vec2 {
        self.axis() * x
    }

    /// True once the block has run off the end of the ramp surface.
    fn past_end(&self, x: f64) -> bool {
        x >= self.params.length
    }
}

impl Default for InclineScenario {
    fn default() -> Self {
        Self::new()
    }
}

impl Scenario for InclineScenario {
    fn id(&self) -> &'static str {
        "incline"
    }

    fn name(&self) -> &'static str {
        "Block on an incline"
    }

    fn description(&self) -> &'static str {
        "A block sits on a ramp. Gravity splits into a part along the slope and a part \
         pressing into it; the surface pushes back and friction resists sliding."
    }

    fn coordinate_label(&self) -> &'static str {
        "Distance down the slope (downhill +)"
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
            self.params.initial_position.min(self.params.length),
            self.params.initial_velocity,
        )
    }

    fn net_force(&self, _x: f64, v: f64, _t: f64) -> f64 {
        self.drive_force() + self.friction_force(v)
    }

    fn constrain(&self, previous: &State, state: &mut State) {
        // Kinetic friction cannot reverse a block, it can only stop one. If the
        // step flipped the sign of the velocity, the block passed through rest
        // inside the step: park it there and let static friction decide.
        let reversed = previous.v * state.v < 0.0;
        if reversed && self.holds_at_rest() {
            state.v = 0.0;
            state.x = previous.x;
        }

        // The ramp is finite. Running off either end stops the block.
        if self.past_end(state.x) {
            state.x = self.params.length;
            state.v = state.v.min(0.0);
        } else if state.x < 0.0 {
            state.x = 0.0;
            state.v = state.v.max(0.0);
        }
    }

    fn dissipated_power(&self, state: &State) -> f64 {
        // |friction * v|. Zero while stuck, since nothing is moving.
        (self.friction_force(state.v) * state.v).abs()
    }

    fn frame(&self, state: &State) -> Frame {
        let axis = self.axis();
        let normal = self.normal_direction();
        let body = self.body_position(state.x);

        let mut forces = vec![
            ForceVector::new("Weight", "gravity", Vec2::new(0.0, -self.weight())),
            ForceVector::new("Normal force", "normal", normal * self.normal_force()),
            ForceVector::new(
                "Friction",
                "friction",
                axis * self.friction_force(state.v),
            ),
        ];
        if self.params.applied_force != 0.0 {
            forces.push(ForceVector::new(
                "Applied force",
                "applied",
                axis * self.params.applied_force,
            ));
        }

        let top = Vec2::ZERO;
        let bottom = axis * self.params.length;
        let corner = Vec2::new(bottom.x, top.y);
        let guides = vec![
            Guide::new("Slope", "surface", vec![top, bottom]),
            Guide::new("Ground", "surface", vec![bottom, corner]),
            Guide::new("Rise", "reference", vec![corner, top]),
            Guide::new(
                "Slope direction",
                "axis",
                vec![body, body + axis * (self.params.length * 0.15)],
            ),
        ];

        Frame {
            body,
            axis,
            forces,
            guides,
        }
    }

    fn derived(&self, state: &State) -> Vec<Derived> {
        let friction = self.friction_force(state.v);
        let net = self.net_force(state.x, state.v, state.t);
        let sliding = state.v.abs() > REST_SPEED;

        vec![
            Derived::new("weight", "Weight", "N", self.weight()),
            Derived::new(
                "gravity_parallel",
                "Gravity along slope",
                "N",
                self.gravity_parallel(),
            ),
            Derived::new(
                "gravity_perpendicular",
                "Gravity into slope",
                "N",
                self.gravity_perpendicular(),
            ),
            Derived::new("normal_force", "Normal force", "N", self.normal_force()),
            Derived::new("friction_force", "Friction force", "N", friction),
            Derived::new(
                "max_static_friction",
                "Static friction limit",
                "N",
                self.max_static_friction(),
            ),
            Derived::new("net_force", "Net force along slope", "N", net),
            Derived::new(
                "acceleration",
                "Acceleration along slope",
                "m/s^2",
                net / self.params.mass,
            ),
            Derived::new(
                "ideal_sliding_accel",
                "Textbook sliding acceleration",
                "m/s^2",
                self.ideal_sliding_accel(),
            ),
            Derived::new(
                "kinetic_energy",
                "Kinetic energy",
                "J",
                state.kinetic_energy(self.params.mass),
            ),
            Derived::new(
                "distance_remaining",
                "Distance left on ramp",
                "m",
                (self.params.length - state.x).max(0.0),
            ),
            Derived::new("sliding", "Sliding", "", if sliding { 1.0 } else { 0.0 }),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use physics_core::sim::Simulation;

    fn incline(angle_deg: f64, mu_s: f64, mu_k: f64) -> InclineScenario {
        let mut s = InclineScenario::new();
        s.set_param("angle_deg", angle_deg).unwrap();
        s.set_param("mu_static", mu_s).unwrap();
        s.set_param("mu_kinetic", mu_k).unwrap();
        s.set_param("length", 20.0).unwrap();
        s
    }

    #[test]
    fn gravity_splits_into_components_that_rebuild_the_weight() {
        let s = incline(25.0, 0.35, 0.25);
        let along = s.gravity_parallel();
        let into = s.gravity_perpendicular();
        assert!((along.hypot(into) - s.weight()).abs() < 1e-12);
    }

    #[test]
    fn frictionless_slide_matches_g_sin_theta() {
        let s = incline(30.0, 0.0, 0.0);
        let expected = s.params.gravity * s.angle().sin();
        let mut sim = Simulation::new(Box::new(s));
        sim.step_n(10);
        assert!((sim.accel() - expected).abs() < 1e-12);
    }

    #[test]
    fn sliding_acceleration_matches_the_textbook_value() {
        let s = incline(35.0, 0.3, 0.2);
        let expected = s.ideal_sliding_accel();
        assert!(expected > 0.0);
        let mut sim = Simulation::new(Box::new(s));
        sim.set_dt(1e-5);
        sim.run_for(1.0);
        // v = a t from rest, and x = a t^2 / 2.
        assert!((sim.state().v - expected).abs() < 1e-3);
        assert!((sim.state().x - 0.5 * expected).abs() < 1e-3);
        assert!((sim.accel() - expected).abs() < 1e-9);
    }

    #[test]
    fn a_shallow_rough_ramp_holds_the_block() {
        // tan(10 deg) = 0.176, well under mu_s = 0.8.
        let s = incline(10.0, 0.8, 0.7);
        assert!(s.holds_at_rest());
        let mut sim = Simulation::new(Box::new(s));
        sim.run_for(5.0);
        assert_eq!(sim.state().v, 0.0);
        assert_eq!(sim.state().x, 0.0);
        assert_eq!(sim.energy_lost(), 0.0);
    }

    #[test]
    fn it_breaks_loose_once_tan_theta_passes_mu_s() {
        let holding = incline(20.0, 0.40, 0.30);
        let slipping = incline(25.0, 0.40, 0.30);
        // tan(20 deg) = 0.364 < 0.40 < 0.466 = tan(25 deg).
        assert!(holding.holds_at_rest());
        assert!(!slipping.holds_at_rest());
    }

    #[test]
    fn friction_stops_an_uphill_block_instead_of_reversing_it() {
        // Shallow enough that static friction can hold it once it stops.
        let mut s = incline(5.0, 0.8, 0.7);
        s.set_param("initial_position", 10.0).unwrap();
        s.set_param("initial_velocity", -2.0).unwrap();
        let mut sim = Simulation::new(Box::new(s));
        sim.run_for(5.0);
        assert_eq!(sim.state().v, 0.0, "block should be parked, not sliding back");
        assert!(sim.state().x < 10.0, "it should have moved uphill");
        assert!(sim.energy_lost() > 0.0);
    }

    #[test]
    fn the_block_stops_at_the_end_of_the_ramp() {
        let mut s = incline(40.0, 0.1, 0.05);
        s.set_param("length", 1.0).unwrap();
        let mut sim = Simulation::new(Box::new(s));
        sim.run_for(5.0);
        assert!((sim.state().x - 1.0).abs() < 1e-12);
        assert_eq!(sim.state().v, 0.0);
    }

    #[test]
    fn an_applied_push_can_drive_it_uphill() {
        let mut s = incline(15.0, 0.3, 0.2);
        s.set_param("initial_position", 5.0).unwrap();
        s.set_param("applied_force", -20.0).unwrap();
        let mut sim = Simulation::new(Box::new(s));
        sim.run_for(1.0);
        assert!(sim.state().v < 0.0);
        assert!(sim.state().x < 5.0);
    }

    #[test]
    fn drawn_forces_add_up_to_the_net_force_along_the_slope() {
        let mut s = incline(30.0, 0.3, 0.2);
        s.set_param("applied_force", 4.0).unwrap();
        let state = State::new(0.0, 0.5, 1.0);
        let frame = s.frame(&state);
        let net_vector = frame.net_force();
        let scalar = s.net_force(state.x, state.v, state.t);
        assert!((net_vector.component_along(s.axis()) - scalar).abs() < 1e-12);
        // Nothing left over pushing into or out of the surface.
        assert!(net_vector.component_along(s.normal_direction()).abs() < 1e-12);
        assert_eq!(frame.forces.len(), 4);
    }
}

//! Meter stick swinging on a pivot: a physical pendulum.
//!
//! Coordinate: `x` is the stick's angle from hanging straight down, radians,
//! counterclockwise positive, and `v` is its angular velocity. The pivot sits at
//! the world origin, and the center of mass is a distance `d` from it along the
//! stick, at `d (sin theta, -cos theta)`.
//!
//! The stick is a uniform rigid rod of mass `M` and length `L`, so:
//!   moment of inertia about its center  I_cm = M L^2 / 12
//!   about the pivot (parallel axis)     I    = I_cm + M d^2
//!   gravity torque about the pivot      tau  = -M g d sin(theta)
//!
//! The pivot force acts at the pivot, so it makes no torque there, and
//! `tau = I alpha` (gravity torque plus friction torque, when there is any) is
//! the whole equation of motion.
//!
//! Energy loss is optional, off by default. It is friction where the stick
//! rubs on the nail, a torque `tau_f = -b omega`. It is set as `p`, the percent
//! of the swing's energy lost per second: with light friction the swing energy
//! falls as `E (1 - p / 100)^t`, and a torque `-b omega` does that when
//! `b = -I ln(1 - p / 100)`. That rate is the smoothed decay at small angles,
//! averaged over whole swings. Within one swing the loss is fastest at the
//! bottom, so a reading mid-swing wobbles around it, more so as `p` grows or
//! the swing gets wide. For small angles
//! `sin(theta) ~ theta`, which turns it into the same restoring law as a spring,
//! with period `T_0 = 2 pi sqrt(I / (M g d))`.
//!
//! `T_0` against `d` is not monotonic. `T_0^2` is proportional to
//! `L^2 / (12 d) + d`, two terms whose product is fixed, so their sum is
//! smallest when they are equal: `d = L / sqrt(12)`. That is the pivot that
//! swings with the shortest period, `T_min = 2 pi sqrt(2 d / g)` there.

use physics_core::bind;
use physics_core::math::{agm, Vec2};
use physics_core::params::{self, ParamDef, ParamError, ParamSpec};
use physics_core::scenario::{
    BodyPose, BodyShape, Couple, Derived, ForceVector, Frame, Guide, Scenario,
};
use physics_core::state::State;

use crate::controls::control;

/// Width of the drawn stick as a fraction of its length. Drawing only; a meter
/// stick is a few centimeters wide.
const ROD_ASPECT: f64 = 0.03;

/// Tunable inputs, all SI except the release angle.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct PendulumParams {
    /// Mass of the stick, kg. The period does not depend on it.
    pub mass: f64,
    /// Length of the stick, m.
    pub rod_length: f64,
    /// Where the hole is, measured from the center of the stick, m. Capped at
    /// the end of the stick, `L / 2`, by `pivot_distance`.
    pub pivot_offset: f64,
    /// Angle from straight down at release, degrees, counterclockwise positive.
    pub initial_angle_deg: f64,
    /// Angular velocity at release, rad/s, counterclockwise positive.
    pub initial_angular_velocity: f64,
    /// Gravitational field strength, m/s^2.
    pub gravity: f64,
    /// Percent of the swing's energy friction at the nail takes each second.
    /// Zero is a frictionless nail.
    pub energy_loss: f64,
}

fn defs() -> Vec<ParamDef<PendulumParams>> {
    vec![
        bind!(PendulumParams, mass, control("mass").with_default(0.15)),
        bind!(PendulumParams, rod_length, control("rod_length")),
        bind!(PendulumParams, pivot_offset, control("pivot_offset")),
        bind!(PendulumParams, initial_angle_deg, control("initial_angle_deg")),
        bind!(
            PendulumParams,
            initial_angular_velocity,
            control("initial_angular_velocity").with_label("Initial angular velocity (ccw +)")
        ),
        bind!(PendulumParams, energy_loss, control("energy_loss")),
        bind!(PendulumParams, gravity, control("gravity")),
    ]
}

/// Uniform stick swinging about a pivot through a hole in it.
pub struct PendulumScenario {
    params: PendulumParams,
    defs: Vec<ParamDef<PendulumParams>>,
}

impl PendulumScenario {
    pub fn new() -> Self {
        let defs = defs();
        let params = params::defaults(&defs);
        Self { params, defs }
    }

    pub fn params(&self) -> PendulumParams {
        self.params
    }

    /// Distance from the pivot to the center of mass, m. The slider can ask for
    /// a hole past the end of the stick, which does not exist, so it is capped
    /// at `L / 2`.
    pub fn pivot_distance(&self) -> f64 {
        self.params.pivot_offset.min(0.5 * self.params.rod_length)
    }

    fn pivot_is_capped(&self) -> bool {
        self.params.pivot_offset > 0.5 * self.params.rod_length
    }

    /// `M L^2 / 12`, kg m^2.
    pub fn center_inertia(&self) -> f64 {
        self.params.mass * self.params.rod_length * self.params.rod_length / 12.0
    }

    /// Parallel axis theorem, `I_cm + M d^2`, kg m^2.
    pub fn pivot_inertia(&self) -> f64 {
        let d = self.pivot_distance();
        self.center_inertia() + self.params.mass * d * d
    }

    /// `M g d`, the torque per unit `sin(theta)`, N m.
    fn restoring_strength(&self) -> f64 {
        self.params.mass * self.params.gravity * self.pivot_distance()
    }

    /// Gravity torque about the pivot, `-M g d sin(theta)`, N m.
    pub fn torque(&self, theta: f64) -> f64 {
        -self.restoring_strength() * theta.sin()
    }

    fn has_friction(&self) -> bool {
        self.params.energy_loss > 0.0
    }

    /// The friction constant that loses `p` percent of the swing energy per
    /// second, `-I ln(1 - p / 100)`, N m s. Zero with no loss set.
    pub fn damping(&self) -> f64 {
        -self.pivot_inertia() * (1.0 - self.params.energy_loss / 100.0).ln()
    }

    /// Friction torque at the nail, `-b omega`, N m. Zero with friction off.
    pub fn friction_torque(&self, omega: f64) -> f64 {
        -self.damping() * omega
    }

    /// Small-angle period, `2 pi sqrt(I / (M g d))`, s.
    pub fn ideal_period(&self) -> f64 {
        std::f64::consts::TAU * (self.pivot_inertia() / self.restoring_strength()).sqrt()
    }

    /// Pivot distance with the shortest period, `L / sqrt(12)`, m.
    pub fn shortest_period_pivot(&self) -> f64 {
        self.params.rod_length / 12f64.sqrt()
    }

    /// Period at that pivot, `2 pi sqrt(2 d_min / g)`, s.
    pub fn shortest_period(&self) -> f64 {
        std::f64::consts::TAU * (2.0 * self.shortest_period_pivot() / self.params.gravity).sqrt()
    }

    fn initial_angle(&self) -> f64 {
        self.params.initial_angle_deg.to_radians()
    }

    /// Largest angle the swing reaches, radians, from energy conservation at
    /// release: `cos(theta_max) = cos(theta_0) - I omega_0^2 / (2 M g d)`. `None`
    /// when the stick has enough energy to go over the top instead.
    pub fn amplitude(&self) -> Option<f64> {
        let w0 = self.params.initial_angular_velocity;
        let c = self.initial_angle().cos()
            - self.pivot_inertia() * w0 * w0 / (2.0 * self.restoring_strength());
        (c > -1.0).then(|| c.acos())
    }

    /// Exact period at the actual amplitude, `T_0 / AGM(1, cos(theta_max / 2))`,
    /// s. Same as `T_0` in the small-angle limit, longer the wider the swing.
    pub fn amplitude_period(&self) -> Option<f64> {
        self.amplitude()
            .map(|a| self.ideal_period() / agm(1.0, (0.5 * a).cos()))
    }

    /// Angular acceleration, `(tau + tau_f) / I`, rad/s^2.
    pub fn angular_accel(&self, theta: f64, omega: f64) -> f64 {
        (self.torque(theta) + self.friction_torque(omega)) / self.pivot_inertia()
    }

    /// Pivot force component along the stick, toward the pivot, N. It has to
    /// hold the stick up and also supply the centripetal force on the center of
    /// mass: `M d omega^2 + M g cos(theta)`.
    pub fn pivot_force_along(&self, theta: f64, omega: f64) -> f64 {
        let m = self.params.mass;
        m * self.pivot_distance() * omega * omega + m * self.params.gravity * theta.cos()
    }

    /// Pivot force component across the stick, along the direction the center
    /// moves for increasing `theta`, N: `M d alpha + M g sin(theta)`.
    pub fn pivot_force_across(&self, theta: f64, omega: f64) -> f64 {
        let m = self.params.mass;
        m * self.pivot_distance() * self.angular_accel(theta, omega)
            + m * self.params.gravity * theta.sin()
    }

    /// Unit vector from the center of mass toward the pivot.
    fn inward(theta: f64) -> Vec2 {
        Vec2::new(-theta.sin(), theta.cos())
    }

    /// Unit vector the center of mass moves along for increasing `theta`.
    fn tangent(theta: f64) -> Vec2 {
        Vec2::new(theta.cos(), theta.sin())
    }

    fn center(&self, theta: f64) -> Vec2 {
        Self::inward(theta) * -self.pivot_distance()
    }

    // ---- derivation terms ----
    //
    // Same layout as the other scenarios: one method per node, numbers from the
    // physics methods above, so the tree cannot drift from what is simulated.

    fn mass_term(&self) -> Derived {
        Derived::new("mass", "Mass", "kg", self.params.mass).sym("M")
    }

    fn gravity_term(&self) -> Derived {
        Derived::new("gravity", "Gravity", "m/s^2", self.params.gravity).sym("g")
    }

    fn length_term(&self) -> Derived {
        Derived::new("rod_length", "Stick length", "m", self.params.rod_length).sym("L")
    }

    fn pivot_term(&self) -> Derived {
        let row = Derived::new("pivot_distance", "Pivot distance from center", "m", self.pivot_distance())
            .sym("d");
        if !self.pivot_is_capped() {
            return row;
        }
        // The slider asked for a hole past the end; say where it actually is.
        row.explain(
            "min(d_set, L / 2)",
            vec![
                Derived::new("pivot_offset", "Requested pivot distance", "m", self.params.pivot_offset)
                    .sym("d_set"),
                self.length_term(),
            ],
        )
    }

    fn angle_term(&self, theta: f64) -> Derived {
        Derived::new("angle_rad", "Angle from vertical", "rad", theta).sym("theta")
    }

    fn angular_velocity_term(&self, omega: f64) -> Derived {
        Derived::new("angular_velocity", "Angular velocity (ccw +)", "rad/s", omega).sym("omega")
    }

    fn center_inertia_term(&self) -> Derived {
        Derived::new("center_inertia", "Moment of inertia about center", "kg m^2", self.center_inertia())
            .sym("I_cm")
            .explain("M L^2 / 12", vec![self.mass_term(), self.length_term()])
    }

    fn inertia_term(&self) -> Derived {
        Derived::new("moment_of_inertia", "Moment of inertia about pivot", "kg m^2", self.pivot_inertia())
            .sym("I")
            .explain(
                "I_cm + M d^2",
                vec![self.center_inertia_term(), self.mass_term(), self.pivot_term()],
            )
    }

    fn torque_term(&self, theta: f64) -> Derived {
        Derived::new("torque", "Gravity torque about pivot", "N m", self.torque(theta))
            .sym("tau")
            .explain(
                "-M g d sin(theta)",
                vec![
                    self.mass_term(),
                    self.gravity_term(),
                    self.pivot_term(),
                    self.angle_term(theta),
                ],
            )
    }

    fn damping_term(&self) -> Derived {
        Derived::new("pivot_damping", "Friction constant", "N m s", self.damping())
            .sym("b")
            .explain(
                "-I ln(1 - p / 100)",
                vec![
                    self.inertia_term(),
                    Derived::new("energy_loss", "Energy loss", "%/s", self.params.energy_loss)
                        .sym("p"),
                ],
            )
    }

    fn friction_torque_term(&self, omega: f64) -> Derived {
        Derived::new("friction_torque", "Pivot friction torque", "N m", self.friction_torque(omega))
            .sym("tau_f")
            .explain(
                "-b omega",
                vec![self.damping_term(), self.angular_velocity_term(omega)],
            )
    }

    fn angular_accel_term(&self, theta: f64, omega: f64) -> Derived {
        let row = Derived::new(
            "angular_accel",
            "Angular acceleration",
            "rad/s^2",
            self.angular_accel(theta, omega),
        )
        .sym("alpha");
        if self.has_friction() {
            row.explain(
                "(tau + tau_f) / I",
                vec![
                    self.torque_term(theta),
                    self.friction_torque_term(omega),
                    self.inertia_term(),
                ],
            )
        } else {
            row.explain("tau / I", vec![self.torque_term(theta), self.inertia_term()])
        }
    }

    fn ideal_period_term(&self) -> Derived {
        Derived::new("ideal_period", "Small-angle period", "s", self.ideal_period())
            .sym("T_0")
            .explain(
                "2 pi sqrt(I / (M g d))",
                vec![
                    self.inertia_term(),
                    self.mass_term(),
                    self.gravity_term(),
                    self.pivot_term(),
                ],
            )
    }

    fn shortest_pivot_term(&self) -> Derived {
        Derived::new(
            "shortest_period_pivot",
            "Pivot for the shortest period",
            "m",
            self.shortest_period_pivot(),
        )
        .sym("d_min")
        .explain("L / sqrt(12)", vec![self.length_term()])
    }

    fn amplitude_term(&self, amplitude: f64) -> Derived {
        Derived::new("amplitude", "Amplitude at release", "rad", amplitude)
            .sym("theta_max")
            .explain(
                "acos(cos(theta_0) - I omega_0^2 / (2 M g d))",
                vec![
                    Derived::new("initial_angle", "Release angle", "rad", self.initial_angle())
                        .sym("theta_0"),
                    Derived::new(
                        "initial_angular_velocity",
                        "Initial angular velocity",
                        "rad/s",
                        self.params.initial_angular_velocity,
                    )
                    .sym("omega_0"),
                    self.inertia_term(),
                    self.mass_term(),
                    self.gravity_term(),
                    self.pivot_term(),
                ],
            )
    }

    /// Height of the center of mass relative to the pivot, m. Negative below it.
    fn height_term(&self, theta: f64) -> Derived {
        Derived::new("height", "Center height above pivot", "m", self.center(theta).y)
            .sym("y")
            .explain("-d cos(theta)", vec![self.pivot_term(), self.angle_term(theta)])
    }

    fn kinetic_energy_term(&self, state: &State) -> Derived {
        Derived::new(
            "kinetic_energy",
            "Rotational kinetic energy",
            "J",
            state.kinetic_energy(self.pivot_inertia()),
        )
        .sym("KE")
        .explain(
            "I omega^2 / 2",
            vec![self.inertia_term(), self.angular_velocity_term(state.v)],
        )
    }

    fn potential_energy_term(&self, theta: f64) -> Derived {
        Derived::new(
            "gravitational_energy",
            "Gravitational energy",
            "J",
            self.params.mass * self.params.gravity * self.center(theta).y,
        )
        .sym("PE")
        .explain(
            "M g y",
            vec![self.mass_term(), self.gravity_term(), self.height_term(theta)],
        )
    }
}

impl Default for PendulumScenario {
    fn default() -> Self {
        Self::new()
    }
}

impl Scenario for PendulumScenario {
    fn id(&self) -> &'static str {
        "pendulum"
    }

    fn name(&self) -> &'static str {
        "Meter stick on a pivot"
    }

    fn description(&self) -> &'static str {
        "A stick hangs on a nail through a hole in it. Gravity acts at its center, the \
         torque about the pivot swings it back, and where the hole is sets the period."
    }

    fn coordinate_label(&self) -> &'static str {
        "Angle from vertical (ccw +)"
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

    fn inertia(&self) -> f64 {
        self.pivot_inertia()
    }

    fn initial_state(&self) -> State {
        State::new(0.0, self.initial_angle(), self.params.initial_angular_velocity)
    }

    fn net_force(&self, x: f64, v: f64, _t: f64) -> f64 {
        self.torque(x) + self.friction_torque(v)
    }

    fn dissipated_power(&self, state: &State) -> f64 {
        // |(-b omega) * omega| = b omega^2.
        self.damping() * state.v * state.v
    }

    fn max_stable_dt(&self) -> Option<f64> {
        // Same bound as the spring, with the small-angle angular frequency
        // sqrt(M g d / I) and the friction in place of damping: under
        // 2 / omega and under 2 I / b, halved for margin.
        let omega = (self.restoring_strength() / self.pivot_inertia()).sqrt();
        let by_gravity = 2.0 / omega;
        let b = self.damping();
        let limit = if b > 0.0 {
            by_gravity.min(2.0 * self.pivot_inertia() / b)
        } else {
            by_gravity
        };
        Some(0.5 * limit)
    }

    fn equilibrium(&self) -> Option<f64> {
        Some(0.0)
    }

    fn frame(&self, state: &State) -> Frame {
        let theta = state.x;
        let pivot = Vec2::ZERO;
        let weight = Vec2::new(0.0, -self.params.mass * self.params.gravity);
        let pivot_force = Self::inward(theta) * self.pivot_force_along(theta, state.v)
            + Self::tangent(theta) * self.pivot_force_across(theta, state.v);

        let reach = self.pivot_distance() + 0.5 * self.params.rod_length;
        let guides = vec![
            Guide::new(
                "Vertical",
                "reference",
                vec![pivot, Vec2::new(0.0, -reach)],
            ),
            Guide::new("Pivot", "pivot", vec![pivot]),
        ];

        Frame {
            body: self.center(theta),
            axis: Self::tangent(theta),
            shape: BodyShape::Rod {
                length: self.params.rod_length,
                width: ROD_ASPECT * self.params.rod_length,
            },
            pivot: Some(pivot),
            // The rod's long side points from the pivot end down through the
            // center, which is straight down (-90 deg) at theta = 0.
            pose: BodyPose {
                angle: theta - std::f64::consts::FRAC_PI_2,
                support: Vec2::ZERO,
            },
            forces: vec![
                ForceVector::new("Weight", "gravity", weight),
                ForceVector::new("Pivot force", "pivot", pivot_force).acting_at(pivot),
            ],
            couples: if self.has_friction() {
                vec![Couple::new(
                    "Pivot friction",
                    "friction",
                    self.friction_torque(state.v),
                    pivot,
                )]
            } else {
                Vec::new()
            },
            guides,
        }
    }

    fn derived(&self, state: &State) -> Vec<Derived> {
        let theta = state.x;
        let mut out = vec![
            Derived::new("angle", "Angle from vertical", "deg", theta.to_degrees())
                .sym("theta")
                .explain("theta (rad) * 180 / pi", vec![self.angle_term(theta)]),
            self.inertia_term(),
            self.torque_term(theta),
            self.ideal_period_term(),
            self.shortest_pivot_term(),
            self.center_inertia_term().advanced(),
            self.angular_velocity_term(state.v).advanced(),
            self.angular_accel_term(theta, state.v).advanced(),
            Derived::new("shortest_period", "Shortest possible period", "s", self.shortest_period())
                .sym("T_min")
                .explain(
                    "2 pi sqrt(2 d_min / g)",
                    vec![self.shortest_pivot_term(), self.gravity_term()],
                )
                .advanced(),
        ];

        if self.has_friction() {
            out.push(self.friction_torque_term(state.v));
        }

        if let Some(amplitude) = self.amplitude() {
            out.push(self.amplitude_term(amplitude).advanced());
            if let Some(period) = self.amplitude_period() {
                out.push(
                    Derived::new("amplitude_period", "Period at release amplitude", "s", period)
                        .sym("T")
                        .explain(
                            "T_0 / AGM(1, cos(theta_max / 2))",
                            vec![self.ideal_period_term(), self.amplitude_term(amplitude)],
                        )
                        .advanced(),
                );
            }
        }

        out.extend([
            Derived::new(
                "centripetal_accel",
                "Centripetal accel of center",
                "m/s^2",
                self.pivot_distance() * state.v * state.v,
            )
            .sym("a_c")
            .explain(
                "d omega^2",
                vec![self.pivot_term(), self.angular_velocity_term(state.v)],
            )
            .advanced(),
            Derived::new(
                "pivot_force_along",
                "Pivot force along stick",
                "N",
                self.pivot_force_along(theta, state.v),
            )
            .sym("F_along")
            .explain(
                "M d omega^2 + M g cos(theta)",
                vec![
                    self.mass_term(),
                    self.pivot_term(),
                    self.angular_velocity_term(state.v),
                    self.gravity_term(),
                    self.angle_term(theta),
                ],
            )
            .advanced(),
            Derived::new(
                "pivot_force_across",
                "Pivot force across stick",
                "N",
                self.pivot_force_across(theta, state.v),
            )
            .sym("F_across")
            .explain(
                "M d alpha + M g sin(theta)",
                vec![
                    self.mass_term(),
                    self.pivot_term(),
                    self.angular_accel_term(theta, state.v),
                    self.gravity_term(),
                    self.angle_term(theta),
                ],
            )
            .advanced(),
            self.kinetic_energy_term(state).advanced(),
            self.potential_energy_term(theta).advanced(),
            Derived::new(
                "mechanical_energy",
                "Total mechanical energy",
                "J",
                state.kinetic_energy(self.pivot_inertia())
                    + self.params.mass * self.params.gravity * self.center(theta).y,
            )
            .sym("E")
            .explain(
                "KE + PE",
                vec![
                    self.kinetic_energy_term(state),
                    self.potential_energy_term(theta),
                ],
            )
            .advanced(),
        ]);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use physics_core::sim::Simulation;

    fn stick(length: f64, pivot: f64, release_deg: f64) -> PendulumScenario {
        let mut s = PendulumScenario::new();
        s.set_param("rod_length", length).unwrap();
        s.set_param("pivot_offset", pivot).unwrap();
        s.set_param("initial_angle_deg", release_deg).unwrap();
        s
    }

    fn measured_period(s: PendulumScenario) -> f64 {
        let periods = 12.0 * s.ideal_period();
        let mut sim = Simulation::new(Box::new(s));
        sim.set_dt(1e-4);
        sim.run_for(periods);
        sim.measured_period().expect("it should swing")
    }

    #[test]
    fn hanging_straight_down_it_stays_put() {
        let mut sim = Simulation::new(Box::new(stick(1.0, 0.3, 0.0)));
        sim.run_for(5.0);
        assert!(sim.state().x.abs() < 1e-12);
        assert!(sim.state().v.abs() < 1e-12);
    }

    #[test]
    fn parallel_axis_inertia() {
        let s = stick(1.0, 0.5, 10.0);
        // Pivot at the end: M L^2 / 12 + M L^2 / 4 = M L^2 / 3.
        let m = s.params().mass;
        assert!((s.pivot_inertia() - m / 3.0).abs() < 1e-12);
    }

    #[test]
    fn a_hole_past_the_end_is_the_end() {
        let s = stick(1.0, 0.9, 10.0);
        assert_eq!(s.pivot_distance(), 0.5);
    }

    #[test]
    fn small_swings_match_the_small_angle_period() {
        for d in [0.05, 0.2, 0.5] {
            let s = stick(1.0, d, 2.0);
            let predicted = s.ideal_period();
            let measured = measured_period(s);
            assert!(
                (measured - predicted).abs() / predicted < 5e-4,
                "d = {d}: {measured} vs {predicted}"
            );
        }
    }

    #[test]
    fn wide_swings_match_the_exact_period() {
        for release in [30.0, 90.0, 150.0] {
            let s = stick(1.0, 0.5, release);
            let predicted = s.amplitude_period().unwrap();
            let measured = measured_period(s);
            assert!(
                (measured - predicted).abs() / predicted < 5e-4,
                "{release} deg: {measured} vs {predicted}"
            );
        }
    }

    #[test]
    fn the_shortest_period_is_at_l_over_root_12() {
        let base = stick(1.0, 0.5, 2.0);
        let d_min = base.shortest_period_pivot();
        let at = |d: f64| stick(1.0, d, 2.0).ideal_period();
        assert!((at(d_min) - base.shortest_period()).abs() < 1e-12);
        assert!(at(d_min) < at(d_min - 0.01));
        assert!(at(d_min) < at(d_min + 0.01));
    }

    #[test]
    fn energy_holds_without_friction() {
        let energy = |sim: &Simulation| sim.snapshot().derived_value("mechanical_energy").unwrap();
        let s = stick(1.0, 0.3, 60.0);
        // Measured against the energy the swing actually trades back and forth,
        // M g d (1 - cos(theta_0)), since E itself sits on an arbitrary zero.
        let swing = s.restoring_strength() * (1.0 - s.initial_angle().cos());
        let mut sim = Simulation::new(Box::new(s));
        sim.set_dt(1e-4);
        let before = energy(&sim);
        sim.run_for(10.0);
        let after = energy(&sim);
        assert!((after - before).abs() < 1e-3 * swing, "{before} -> {after}");
    }

    #[test]
    fn the_pivot_force_is_what_moves_the_center() {
        // Weight plus pivot force = M a_cm, with a_cm = d alpha along the
        // tangent plus d omega^2 toward the pivot.
        let s = stick(1.0, 0.3, 40.0);
        let (theta, omega) = (0.4, -1.3);
        let frame = s.frame(&State::new(0.0, theta, omega));
        let d = s.pivot_distance();
        let accel = PendulumScenario::tangent(theta) * (d * s.angular_accel(theta, omega))
            + PendulumScenario::inward(theta) * (d * omega * omega);
        assert!((frame.net_force() - accel * s.params().mass).len() < 1e-12);
    }

    fn with_friction(mut s: PendulumScenario) -> PendulumScenario {
        s.set_param("energy_loss", 5.0).unwrap();
        s
    }

    #[test]
    fn friction_is_off_until_switched_on() {
        let s = stick(1.0, 0.3, 30.0);
        assert_eq!(s.friction_torque(2.0), 0.0);
        assert_eq!(s.dissipated_power(&State::new(0.0, 0.0, 2.0)), 0.0);
        assert!(s.frame(&State::new(0.0, 0.0, 2.0)).couples.is_empty());
    }

    #[test]
    fn friction_dies_the_swing_down_and_books_the_loss() {
        let s = with_friction(stick(1.0, 0.3, 30.0));
        let energy = |sim: &Simulation| sim.snapshot().derived_value("mechanical_energy").unwrap();
        let mut sim = Simulation::new(Box::new(s));
        sim.set_dt(1e-4);
        let before = energy(&sim);
        sim.run_for(30.0);
        let after = energy(&sim);
        assert!(after < before, "friction should take energy out");
        // Whatever left the swing is in the ledger.
        assert!(
            ((before - after) - sim.energy_lost()).abs() < 1e-3 * (before - after),
            "lost {} but booked {}",
            before - after,
            sim.energy_lost()
        );
        // Released at 30 deg; well under half that by now.
        assert!(sim.state().x.abs() < 15f64.to_radians());
    }

    #[test]
    fn the_loss_percent_is_what_the_swing_loses_per_second() {
        // Small swing, light loss: the swing energy above the bottom should
        // fall by p percent a second. Sampled a whole number of periods apart
        // so the in-cycle wobble of the loss rate cancels.
        for p in [2.0, 5.0, 10.0] {
            let mut s = stick(1.0, 0.3, 5.0);
            s.set_param("energy_loss", p).unwrap();
            let floor = -s.restoring_strength();
            let periods = 10.0;
            let span = periods * s.ideal_period();
            let mut sim = Simulation::new(Box::new(s));
            sim.set_dt(1e-4);
            let swing = |sim: &Simulation| {
                sim.snapshot().derived_value("mechanical_energy").unwrap() - floor
            };
            let before = swing(&sim);
            sim.run_for(span);
            let per_second = (swing(&sim) / before).powf(1.0 / span);
            let lost = 100.0 * (1.0 - per_second);
            assert!((lost - p).abs() < 0.02 * p, "asked for {p} %/s, lost {lost} %/s");
        }
    }

    #[test]
    fn friction_shows_up_as_a_couple_at_the_pivot() {
        let s = with_friction(stick(1.0, 0.3, 30.0));
        let state = State::new(0.0, 0.4, -1.3);
        let frame = s.frame(&state);
        assert_eq!(frame.couples.len(), 1);
        assert!(frame.couples[0].torque > 0.0, "opposes clockwise spin");
        let scalar = s.net_force(state.x, state.v, state.t);
        assert!((frame.generalized_force() - scalar).abs() < 1e-12);
    }

    #[test]
    fn enough_spin_goes_over_the_top() {
        let mut s = stick(1.0, 0.5, 0.0);
        s.set_param("initial_angular_velocity", 15.0).unwrap();
        assert!(s.amplitude().is_none());
        assert!(s.amplitude_period().is_none());
    }
}

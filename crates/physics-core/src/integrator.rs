//! Time-stepping schemes. Each one advances a state by a fixed `dt` using only
//! the acceleration function, so they are interchangeable at runtime.

use crate::state::State;
use crate::system::System;

/// A recurrence that turns the state at time `t` into the state at `t + dt`.
pub trait Integrator {
    /// Short stable name, used by the bindings to pick a scheme.
    fn id(&self) -> &'static str;

    /// Human-readable name for the UI.
    fn name(&self) -> &'static str;

    fn step(&self, sys: &dyn System, s: &State, dt: f64) -> State;
}

/// Explicit (forward) Euler. Updates position from the *old* velocity.
///
/// Kept because it is the obvious first thing to write and it visibly gains
/// energy on an oscillator, which makes it worth showing next to the others.
#[derive(Debug, Clone, Copy, Default)]
pub struct ExplicitEuler;

impl Integrator for ExplicitEuler {
    fn id(&self) -> &'static str {
        "explicit-euler"
    }

    fn name(&self) -> &'static str {
        "Explicit Euler"
    }

    fn step(&self, sys: &dyn System, s: &State, dt: f64) -> State {
        let a = sys.accel(s.x, s.v, s.t);
        State {
            t: s.t + dt,
            x: s.x + s.v * dt,
            v: s.v + a * dt,
        }
    }
}

/// Semi-implicit (symplectic) Euler. Updates velocity first, then moves the
/// position with the *new* velocity. Same cost as explicit Euler, but it keeps
/// oscillator energy bounded instead of letting it grow. This is the default.
#[derive(Debug, Clone, Copy, Default)]
pub struct SemiImplicitEuler;

impl Integrator for SemiImplicitEuler {
    fn id(&self) -> &'static str {
        "semi-implicit-euler"
    }

    fn name(&self) -> &'static str {
        "Semi-implicit Euler"
    }

    fn step(&self, sys: &dyn System, s: &State, dt: f64) -> State {
        let a = sys.accel(s.x, s.v, s.t);
        let v = s.v + a * dt;
        State {
            t: s.t + dt,
            x: s.x + v * dt,
            v,
        }
    }
}

/// Velocity Verlet. Averages the acceleration at the start and end of the step,
/// which makes it exact for constant acceleration and second-order accurate in
/// general.
///
/// Acceleration here can depend on velocity (damping, friction), so the
/// end-of-step acceleration is evaluated against a velocity predicted by the
/// start-of-step one.
#[derive(Debug, Clone, Copy, Default)]
pub struct VelocityVerlet;

impl Integrator for VelocityVerlet {
    fn id(&self) -> &'static str {
        "velocity-verlet"
    }

    fn name(&self) -> &'static str {
        "Velocity Verlet"
    }

    fn step(&self, sys: &dyn System, s: &State, dt: f64) -> State {
        let a0 = sys.accel(s.x, s.v, s.t);
        let x = s.x + s.v * dt + 0.5 * a0 * dt * dt;
        let v_predicted = s.v + a0 * dt;
        let a1 = sys.accel(x, v_predicted, s.t + dt);
        State {
            t: s.t + dt,
            x,
            v: s.v + 0.5 * (a0 + a1) * dt,
        }
    }
}

/// Every integrator the engine ships, in the order a UI should list them.
pub fn all() -> Vec<Box<dyn Integrator>> {
    vec![
        Box::new(SemiImplicitEuler),
        Box::new(ExplicitEuler),
        Box::new(VelocityVerlet),
    ]
}

/// Look one up by `id`. Unknown ids give `None` so callers can fall back.
pub fn by_id(id: &str) -> Option<Box<dyn Integrator>> {
    all().into_iter().find(|i| i.id() == id)
}

/// The scheme used when nobody picks one.
pub fn default_integrator() -> Box<dyn Integrator> {
    Box::new(SemiImplicitEuler)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::system::ConstantAccel;

    /// Free fall from rest: x = x0 + v0 t + a t^2 / 2.
    fn analytic_drop(x0: f64, v0: f64, a: f64, t: f64) -> f64 {
        x0 + v0 * t + 0.5 * a * t * t
    }

    fn run(integ: &dyn Integrator, sys: &dyn System, dt: f64, steps: usize) -> State {
        let mut s = State::new(0.0, 0.0, 0.0);
        for _ in 0..steps {
            s = integ.step(sys, &s, dt);
        }
        s
    }

    #[test]
    fn verlet_is_exact_for_constant_acceleration() {
        let sys = ConstantAccel(-9.80665);
        let s = run(&VelocityVerlet, &sys, 0.01, 100);
        let expected = analytic_drop(0.0, 0.0, -9.80665, 1.0);
        assert!((s.x - expected).abs() < 1e-9, "got {}, want {expected}", s.x);
        assert!((s.v - (-9.80665)).abs() < 1e-9);
    }

    #[test]
    fn euler_variants_converge_as_dt_shrinks() {
        let sys = ConstantAccel(-9.80665);
        let expected = analytic_drop(0.0, 0.0, -9.80665, 1.0);
        for integ in [
            &ExplicitEuler as &dyn Integrator,
            &SemiImplicitEuler as &dyn Integrator,
        ] {
            let coarse = (run(integ, &sys, 0.01, 100).x - expected).abs();
            let fine = (run(integ, &sys, 0.001, 1000).x - expected).abs();
            // First order: ten times smaller dt is roughly ten times less error.
            assert!(fine < coarse / 5.0, "{} did not converge", integ.id());
        }
    }

    /// Undamped oscillator, a = -(k/m) x. Explicit Euler pumps energy into it,
    /// semi-implicit Euler does not.
    struct Oscillator {
        omega_squared: f64,
    }

    impl System for Oscillator {
        fn accel(&self, x: f64, _v: f64, _t: f64) -> f64 {
            -self.omega_squared * x
        }
    }

    #[test]
    fn semi_implicit_bounds_oscillator_energy() {
        let sys = Oscillator { omega_squared: 4.0 };
        let start = State::new(0.0, 1.0, 0.0);
        let energy = |s: &State| 0.5 * s.v * s.v + 0.5 * 4.0 * s.x * s.x;
        let e0 = energy(&start);

        let mut semi = start;
        let mut explicit = start;
        for _ in 0..1000 {
            semi = SemiImplicitEuler.step(&sys, &semi, 0.01);
            explicit = ExplicitEuler.step(&sys, &explicit, 0.01);
        }

        assert!((energy(&semi) - e0).abs() / e0 < 0.02);
        assert!(energy(&explicit) / e0 > 1.1, "explicit Euler should drift up");
    }

    #[test]
    fn lookup_by_id_round_trips() {
        for integ in all() {
            assert_eq!(by_id(integ.id()).unwrap().id(), integ.id());
        }
        assert!(by_id("nope").is_none());
        assert_eq!(default_integrator().id(), "semi-implicit-euler");
    }
}

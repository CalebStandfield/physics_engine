//! The only thing an integrator is allowed to see.

use crate::state::State;

/// Something whose acceleration can be evaluated at an arbitrary trial point.
///
/// Integrators take this rather than a `Scenario` so they stay ignorant of
/// parameters, force labels, and geometry. Anything that can answer "what is
/// the acceleration at this position, velocity and time" can be stepped.
pub trait System {
    /// Acceleration in m/s^2 along the scenario axis.
    ///
    /// Must be a pure function of its arguments: integrators evaluate it at
    /// trial points that are not the current state and may do so more than
    /// once per step.
    fn accel(&self, x: f64, v: f64, t: f64) -> f64;

    /// Convenience for evaluating at the state itself.
    fn accel_at(&self, s: &State) -> f64 {
        self.accel(s.x, s.v, s.t)
    }
}

/// Constant-acceleration system. Useful as a reference in tests and as the
/// simplest possible example of the trait.
pub struct ConstantAccel(pub f64);

impl System for ConstantAccel {
    fn accel(&self, _x: f64, _v: f64, _t: f64) -> f64 {
        self.0
    }
}

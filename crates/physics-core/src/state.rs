//! Motion state of a one-degree-of-freedom system.

use serde::{Deserialize, Serialize};

/// Where the body is along its scenario axis and how fast it is going.
///
/// `x` is a generalized coordinate in meters: stretch past natural length for
/// the spring, distance down-slope for the incline. Each scenario maps it back
/// to world space in its `frame()`.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct State {
    /// Simulated time since reset, seconds.
    pub t: f64,
    /// Position along the scenario axis, meters.
    pub x: f64,
    /// Velocity along the scenario axis, m/s.
    pub v: f64,
}

impl State {
    pub const fn new(t: f64, x: f64, v: f64) -> Self {
        Self { t, x, v }
    }

    /// State at t = 0 with the given position and velocity.
    pub const fn at_rest(x: f64) -> Self {
        Self::new(0.0, x, 0.0)
    }

    /// Kinetic energy of a body of mass `mass`, joules.
    pub fn kinetic_energy(&self, mass: f64) -> f64 {
        0.5 * mass * self.v * self.v
    }

    /// True once every field is finite. A blown-up integration shows up here.
    pub fn is_finite(&self) -> bool {
        self.t.is_finite() && self.x.is_finite() && self.v.is_finite()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kinetic_energy_matches_half_m_v_squared() {
        let s = State::new(0.0, 0.0, 3.0);
        assert!((s.kinetic_energy(2.0) - 9.0).abs() < 1e-12);
    }

    #[test]
    fn divergence_is_detectable() {
        assert!(State::new(0.0, 1.0, 2.0).is_finite());
        assert!(!State::new(0.0, f64::INFINITY, 0.0).is_finite());
    }
}

//! The plug-in point. A scenario supplies the force law and the geometry; the
//! core supplies the stepping, recording and analysis.

use serde::{Deserialize, Serialize};

use crate::math::Vec2;
use crate::params::{ParamError, ParamSpec};
use crate::state::State;

/// One labeled force acting on the body, in world space, newtons.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ForceVector {
    /// Free-body-diagram label, e.g. `"Weight"` or `"Normal force"`.
    pub label: String,
    /// Short tag for styling, e.g. `"gravity"`, `"normal"`, `"friction"`.
    pub kind: String,
    /// The force itself, world-space newtons.
    pub vector: Vec2,
}

impl ForceVector {
    pub fn new(label: &str, kind: &str, vector: Vec2) -> Self {
        Self {
            label: label.to_string(),
            kind: kind.to_string(),
            vector,
        }
    }

    /// Magnitude in newtons.
    pub fn magnitude(&self) -> f64 {
        self.vector.len()
    }
}

/// A labeled polyline of static geometry: the incline surface, the ground, the
/// spring's ceiling anchor. Generic on purpose, so the renderer never needs to
/// know which scenario it is drawing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Guide {
    pub label: String,
    pub kind: String,
    pub points: Vec<Vec2>,
}

impl Guide {
    pub fn new(label: &str, kind: &str, points: Vec<Vec2>) -> Self {
        Self {
            label: label.to_string(),
            kind: kind.to_string(),
            points,
        }
    }
}

/// Everything needed to draw the system at one instant, in world meters.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Frame {
    /// Center of the body.
    pub body: Vec2,
    /// Unit vector pointing along increasing `State::x`.
    pub axis: Vec2,
    /// Every force on the body, for the free-body diagram.
    pub forces: Vec<ForceVector>,
    /// Static scenery.
    pub guides: Vec<Guide>,
}

impl Frame {
    /// Vector sum of the drawn forces. Should agree with the scalar
    /// `Scenario::net_force` projected on `axis`; `frame_matches_net_force`
    /// checks that for each scenario.
    pub fn net_force(&self) -> Vec2 {
        self.forces
            .iter()
            .fold(Vec2::ZERO, |sum, f| sum + f.vector)
    }
}

/// A computed quantity worth showing next to the animation: period, normal
/// force, energy, and so on.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Derived {
    pub key: String,
    pub label: String,
    pub unit: String,
    pub value: f64,
}

impl Derived {
    pub fn new(key: &str, label: &str, unit: &str, value: f64) -> Self {
        Self {
            key: key.to_string(),
            label: label.to_string(),
            unit: unit.to_string(),
            value,
        }
    }
}

/// A physical situation the engine can simulate.
///
/// Object-safe, so the registry can hand back `Box<dyn Scenario>` and the
/// bindings never mention a concrete type. Implementors own their parameters
/// and their geometry; they do not own time-stepping.
pub trait Scenario {
    /// Stable machine name, e.g. `"spring"`.
    fn id(&self) -> &'static str;

    /// Display name, e.g. `"Mass on a vertical spring"`.
    fn name(&self) -> &'static str;

    /// One or two sentences of plain-language description for the UI.
    fn description(&self) -> &'static str;

    /// What `State::x` means here, e.g. `"Stretch past natural length"`.
    fn coordinate_label(&self) -> &'static str;

    /// The tunable parameters, in the order a UI should show them.
    fn schema(&self) -> Vec<ParamSpec>;

    fn get_param(&self, key: &str) -> Option<f64>;

    fn set_param(&mut self, key: &str, value: f64) -> Result<(), ParamError>;

    /// Mass of the body, kg. Constant over a run.
    fn mass(&self) -> f64;

    /// State the simulation starts and resets to.
    fn initial_state(&self) -> State;

    /// Net force along the axis at a trial point, newtons. Signed: positive
    /// means along `Frame::axis`.
    ///
    /// Takes loose components rather than a `State` because integrators
    /// evaluate it away from the current state.
    fn net_force(&self, x: f64, v: f64, t: f64) -> f64;

    /// Applied after every step, to enforce things a force law cannot express:
    /// a block stopping at the bottom of a ramp, static friction pinning a body
    /// in place. Default is a free system with nothing to enforce.
    fn constrain(&self, _state: &mut State) {}

    /// World-space geometry and the free-body diagram at this state.
    fn frame(&self, state: &State) -> Frame;

    /// Computed quantities to display. Default is none.
    fn derived(&self, _state: &State) -> Vec<Derived> {
        Vec::new()
    }

    /// Rate at which non-conservative forces (friction, damping) are removing
    /// energy from the body, watts, never negative. The driver multiplies this
    /// by `dt` each step to track total energy lost, which is the only
    /// path-dependent quantity in the engine. Default is a lossless system.
    fn dissipated_power(&self, _state: &State) -> f64 {
        0.0
    }

    /// Position the oscillation detector should measure crossings about, in
    /// scenario coordinates. `None` for a system that does not oscillate.
    fn equilibrium(&self) -> Option<f64> {
        None
    }
}

/// Adapts a scenario into the bare acceleration function the integrators want.
/// This is the only place `F = ma` is divided out.
pub struct ScenarioSystem<'a>(pub &'a dyn Scenario);

impl crate::system::System for ScenarioSystem<'_> {
    fn accel(&self, x: f64, v: f64, t: f64) -> f64 {
        self.0.net_force(x, v, t) / self.0.mass()
    }
}

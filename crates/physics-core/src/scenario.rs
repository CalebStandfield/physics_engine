//! The plug-in point. A scenario supplies the force law and the geometry; the
//! core supplies the stepping, recording and analysis.

use serde::{Deserialize, Serialize};

use crate::math::Vec2;
use crate::params::{ParamError, ParamSpec, Tier};
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
    /// World point the force acts at. `None` means the body's center
    /// (`Frame::body`), which is all a point mass ever needs. An extended body
    /// sets it, since where a force acts decides the torque it makes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub at: Option<Vec2>,
}

impl ForceVector {
    pub fn new(label: &str, kind: &str, vector: Vec2) -> Self {
        Self {
            label: label.to_string(),
            kind: kind.to_string(),
            vector,
            at: None,
        }
    }

    /// Act at a world point instead of the body's center.
    pub fn acting_at(mut self, point: Vec2) -> Self {
        self.at = Some(point);
        self
    }

    /// Where the force acts, given where the body is.
    pub fn origin(&self, body: Vec2) -> Vec2 {
        self.at.unwrap_or(body)
    }

    /// Magnitude in newtons.
    pub fn magnitude(&self) -> f64 {
        self.vector.len()
    }
}

/// A pure torque on the body, with no single line of action: friction where a
/// stick rubs on its nail, say. Only means something for a body with a pivot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Couple {
    /// Free-body-diagram label, e.g. `"Pivot friction"`.
    pub label: String,
    /// Short tag for styling, same set as `ForceVector::kind`.
    pub kind: String,
    /// The torque, N m, counterclockwise positive.
    pub torque: f64,
    /// Where to draw it. A couple acts on the whole body; this is just where
    /// the curved arrow goes.
    pub at: Vec2,
}

impl Couple {
    pub fn new(label: &str, kind: &str, torque: f64, at: Vec2) -> Self {
        Self {
            label: label.to_string(),
            kind: kind.to_string(),
            torque,
            at,
        }
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

/// How the body sits, for drawing only. No physics reads this.
///
/// A block on a ramp is a point mass to the solver, but drawing it as an
/// axis-aligned square lying across the slope line looks wrong, so a scenario
/// says here how the box should be turned and which way is "up" out of whatever
/// it is resting on.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct BodyPose {
    /// Rotation of the body, radians counterclockwise from world +x. Zero draws
    /// it axis-aligned.
    pub angle: f64,
    /// Unit vector out of the surface the body rests on. A renderer lifts the
    /// drawn box half its height along this, so the surface line touches the
    /// bottom of the box instead of cutting through its middle. Zero when
    /// nothing supports the body.
    pub support: Vec2,
}

/// What the body looks like, for drawing only. No physics reads this.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum BodyShape {
    /// A point mass, drawn as a fixed-size block.
    #[default]
    Block,
    /// A thin rigid rod centered on `Frame::body`, lying along `BodyPose::angle`.
    /// Drawn to scale, meters.
    Rod { length: f64, width: f64 },
}

/// Everything needed to draw the system at one instant, in world meters.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Frame {
    /// Center of the body (center of mass, for an extended one).
    pub body: Vec2,
    /// Unit vector the body's center moves along as `State::x` increases.
    pub axis: Vec2,
    /// How the body is turned and seated. Drawing only.
    pub pose: BodyPose,
    /// What the body looks like. Drawing only.
    pub shape: BodyShape,
    /// The fixed point a rotating body turns about. `None` for a body that
    /// translates, where `State::x` is a distance and not an angle.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pivot: Option<Vec2>,
    /// Every force on the body, for the free-body diagram.
    pub forces: Vec<ForceVector>,
    /// Pure torques on the body. Empty for anything without a pivot.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub couples: Vec<Couple>,
    /// Static scenery.
    pub guides: Vec<Guide>,
}

impl Frame {
    /// Vector sum of the drawn forces, N. For a point mass this is the net force;
    /// for an extended body it is mass times the acceleration of the center of
    /// mass.
    pub fn net_force(&self) -> Vec2 {
        self.forces
            .iter()
            .fold(Vec2::ZERO, |sum, f| sum + f.vector)
    }

    /// The drawn forces reduced to the one number the coordinate feels: the sum
    /// of torques about the pivot when there is one (N m, counterclockwise
    /// positive, couples included), otherwise the net force along `axis` (N).
    /// Should agree with `Scenario::net_force`; a contract test checks that for
    /// every scenario.
    pub fn generalized_force(&self) -> f64 {
        match self.pivot {
            Some(pivot) => {
                let from_forces: f64 = self
                    .forces
                    .iter()
                    .map(|f| (f.origin(self.body) - pivot).cross(f.vector))
                    .sum();
                from_forces + self.couples.iter().map(|c| c.torque).sum::<f64>()
            }
            None => self.net_force().component_along(self.axis),
        }
    }
}

/// How a value was put together: the formula, in symbols, plus the terms it is
/// built from.
///
/// The terms are `Derived` themselves, so a derivation is a tree: the normal
/// force comes from the perpendicular weight component, which comes from the
/// weight and the angle, which comes from the mass and gravity. It bottoms out
/// at parameters and state, the numbers nothing else explains.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Derivation {
    /// The formula in symbols, e.g. `"m g sin(theta)"`. Written with the
    /// `symbol` of each term below, so the two read together.
    pub equation: String,
    /// The values the formula is built out of, in the order they appear in it.
    pub terms: Vec<Derived>,
}

/// A computed quantity worth showing next to the animation: period, normal
/// force, energy, and so on.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Derived {
    pub key: String,
    pub label: String,
    pub unit: String,
    pub value: f64,
    /// Whether a short readout should show this row. Same split as the
    /// parameters, so one switch in the UI drives both.
    pub tier: Tier,
    /// How this row is written in a formula, e.g. `"m"` or `"F_net"`. Empty for
    /// a quantity with no standard symbol.
    pub symbol: String,
    /// Where the number came from, when it came from other numbers. `None` for a
    /// parameter or a piece of state, which is where every chain ends.
    pub from: Option<Derivation>,
}

impl Derived {
    /// A row worth showing in a short readout.
    pub fn new(key: &str, label: &str, unit: &str, value: f64) -> Self {
        Self {
            key: key.to_string(),
            label: label.to_string(),
            unit: unit.to_string(),
            value,
            tier: Tier::Basic,
            symbol: String::new(),
            from: None,
        }
    }

    /// Demote the row to the long readout.
    pub fn advanced(mut self) -> Self {
        self.tier = Tier::Advanced;
        self
    }

    /// Name the symbol this quantity is written with in a formula.
    pub fn sym(mut self, symbol: &str) -> Self {
        self.symbol = symbol.to_string();
        self
    }

    /// Say what the number is made of: the formula, and the terms in it. Each
    /// term may carry its own derivation, which is what makes the readout able
    /// to keep unfolding.
    pub fn explain(mut self, equation: &str, terms: Vec<Derived>) -> Self {
        self.from = Some(Derivation {
            equation: equation.to_string(),
            terms,
        });
        self
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

    /// Inertia along the coordinate. Constant over a run. Mass in kg when
    /// `State::x` is a distance, moment of inertia about the pivot in kg m^2
    /// when it is an angle. Either way `net_force / inertia` is the
    /// acceleration of the coordinate and `inertia v^2 / 2` is the kinetic
    /// energy, which is all the core ever does with it.
    fn inertia(&self) -> f64;

    /// State the simulation starts and resets to.
    fn initial_state(&self) -> State;

    /// Net generalized force at a trial point: the force along `Frame::axis`
    /// in N for a distance coordinate, the torque about the pivot in N m for an
    /// angle (counterclockwise positive). Signed: positive pushes `State::x` up.
    ///
    /// Takes loose components rather than a `State` because integrators
    /// evaluate it away from the current state.
    fn net_force(&self, x: f64, v: f64, t: f64) -> f64;

    /// Applied after every step, to enforce things a force law cannot express:
    /// a block stopping at the bottom of a ramp, static friction pinning a body
    /// that just came to rest. Gets the state from before the step too, so a
    /// scenario can notice a velocity that flipped sign mid-step. Default is a
    /// free system with nothing to enforce.
    fn constrain(&self, _previous: &State, _state: &mut State) {}

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

    /// Largest step this scenario can be integrated with before the explicit
    /// schemes go unstable, seconds. `None` means any step is fine.
    ///
    /// A stiff spring or heavy damping can blow up at a step that is perfectly
    /// comfortable for everything else, and the parameters are user-facing
    /// sliders, so the scenario reports its own limit and the driver quietly
    /// subdivides to respect it.
    fn max_stable_dt(&self) -> Option<f64> {
        None
    }

    /// Position the oscillation detector should measure crossings about, in
    /// scenario coordinates. `None` for a system that does not oscillate.
    fn equilibrium(&self) -> Option<f64> {
        None
    }
}

/// Adapts a scenario into the bare acceleration function the integrators want.
/// This is the only place `F = ma` (or `tau = I alpha`) is divided out.
pub struct ScenarioSystem<'a>(pub &'a dyn Scenario);

impl crate::system::System for ScenarioSystem<'_> {
    fn accel(&self, x: f64, v: f64, t: f64) -> f64 {
        self.0.net_force(x, v, t) / self.0.inertia()
    }
}

//! Scenario-agnostic simulation core.
//!
//! Knows how to step a one-degree-of-freedom body forward given the forces on
//! it. Knows nothing about springs or inclines: those live in
//! `physics-scenarios` and plug in through the `Scenario` trait.

pub mod analysis;
pub mod integrator;
pub mod math;
pub mod params;
pub mod record;
pub mod scenario;
pub mod sim;
pub mod state;
pub mod system;

pub use analysis::{percent_difference, percent_error, PeriodDetector};
pub use integrator::Integrator;
pub use math::{Vec2, G};
pub use params::{ParamError, ParamSpec};
pub use record::Recorder;
pub use sim::{Simulation, Snapshot};
pub use scenario::{Derived, ForceVector, Frame, Guide, Scenario};
pub use state::State;
pub use system::System;

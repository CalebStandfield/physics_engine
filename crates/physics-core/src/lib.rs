//! Scenario-agnostic simulation core.
//!
//! Knows how to step a one-degree-of-freedom body forward given the forces on
//! it. Knows nothing about springs or inclines: those live in
//! `physics-scenarios` and plug in through the `Scenario` trait.

pub mod math;
pub mod state;
pub mod system;
pub mod integrator;

pub use math::{Vec2, G};
pub use state::State;
pub use system::System;
pub use integrator::Integrator;

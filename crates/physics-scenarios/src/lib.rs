//! Concrete scenarios. Each one implements `physics_core::Scenario` and is
//! reachable through `registry`.

pub mod registry;
pub mod spring;

pub use registry::{create, default_scenario, ids};
pub use spring::SpringScenario;

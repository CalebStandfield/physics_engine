//! Concrete scenarios. Each one implements `physics_core::Scenario` and is
//! reachable through `registry`.

pub mod controls;
pub mod incline;
pub mod pendulum;
pub mod registry;
pub mod spring;

pub use controls::control;
pub use registry::{create, default_scenario, ids};
pub use incline::InclineScenario;
pub use pendulum::PendulumScenario;
pub use spring::SpringScenario;

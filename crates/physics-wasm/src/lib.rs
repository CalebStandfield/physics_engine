//! WebAssembly bindings.
//!
//! Marshaling only. Every number here comes from `physics-core` or
//! `physics-scenarios`; nothing in this file decides any physics. The frontend
//! reads the scenario catalog and the parameter schema at runtime, so adding a
//! scenario needs no change here or in the JS.

use physics_core::analysis;
use physics_core::integrator;
use physics_core::record::Recorder;
use physics_core::sim::Simulation;
use physics_scenarios::registry;
use serde::Serialize;
use wasm_bindgen::prelude::*;

/// A scenario the UI can offer, without building it first.
#[derive(Serialize)]
struct CatalogEntry {
    id: &'static str,
    name: &'static str,
    description: &'static str,
    coordinate_label: &'static str,
}

/// An integrator the UI can offer.
#[derive(Serialize)]
struct IntegratorEntry {
    id: &'static str,
    name: &'static str,
}

fn to_js<T: Serialize>(value: &T) -> Result<JsValue, JsError> {
    serde_wasm_bindgen::to_value(value).map_err(|e| JsError::new(&e.to_string()))
}

/// Ids of every scenario the engine ships.
#[wasm_bindgen]
pub fn scenario_ids() -> Vec<String> {
    registry::ids().into_iter().map(String::from).collect()
}

/// Id, name and description of every scenario, for building a menu.
#[wasm_bindgen]
pub fn scenario_catalog() -> Result<JsValue, JsError> {
    let entries: Vec<_> = registry::ids()
        .into_iter()
        .filter_map(registry::create)
        .map(|s| CatalogEntry {
            id: s.id(),
            name: s.name(),
            description: s.description(),
            coordinate_label: s.coordinate_label(),
        })
        .collect();
    to_js(&entries)
}

/// Every available time-stepping scheme.
#[wasm_bindgen]
pub fn integrators() -> Result<JsValue, JsError> {
    let entries: Vec<_> = integrator::all()
        .iter()
        .map(|i| IntegratorEntry {
            id: i.id(),
            name: i.name(),
        })
        .collect();
    to_js(&entries)
}

/// Percent difference between two values, relative to their average.
#[wasm_bindgen]
pub fn percent_difference(a: f64, b: f64) -> f64 {
    analysis::percent_difference(a, b)
}

/// Percent error of a measurement against an accepted value.
#[wasm_bindgen]
pub fn percent_error(measured: f64, accepted: f64) -> f64 {
    analysis::percent_error(measured, accepted)
}

/// A running simulation, held across calls from JS.
#[wasm_bindgen]
pub struct Engine {
    sim: Simulation,
}

#[wasm_bindgen]
impl Engine {
    /// Build an engine running the scenario with this id.
    #[wasm_bindgen(constructor)]
    pub fn new(scenario_id: &str) -> Result<Engine, JsError> {
        let scenario = registry::create(scenario_id)
            .ok_or_else(|| JsError::new(&format!("unknown scenario '{scenario_id}'")))?;
        Ok(Engine {
            sim: Simulation::new(scenario),
        })
    }

    /// Swap to a different scenario, keeping the integrator and step size.
    #[wasm_bindgen(js_name = loadScenario)]
    pub fn load_scenario(&mut self, scenario_id: &str) -> Result<(), JsError> {
        let scenario = registry::create(scenario_id)
            .ok_or_else(|| JsError::new(&format!("unknown scenario '{scenario_id}'")))?;
        self.sim.set_scenario(scenario);
        Ok(())
    }

    #[wasm_bindgen(getter, js_name = scenarioId)]
    pub fn scenario_id(&self) -> String {
        self.sim.scenario().id().to_string()
    }

    #[wasm_bindgen(getter, js_name = scenarioName)]
    pub fn scenario_name(&self) -> String {
        self.sim.scenario().name().to_string()
    }

    #[wasm_bindgen(getter)]
    pub fn description(&self) -> String {
        self.sim.scenario().description().to_string()
    }

    /// What the scenario's position coordinate means, for axis labels.
    #[wasm_bindgen(getter, js_name = coordinateLabel)]
    pub fn coordinate_label(&self) -> String {
        self.sim.scenario().coordinate_label().to_string()
    }

    /// The scenario's parameters: key, label, unit, range, default, step.
    pub fn schema(&self) -> Result<JsValue, JsError> {
        to_js(&self.sim.scenario().schema())
    }

    #[wasm_bindgen(js_name = getParam)]
    pub fn get_param(&self, key: &str) -> Option<f64> {
        self.sim.scenario().get_param(key)
    }

    /// Write a parameter. Takes effect on the next step; call `reset` to
    /// restart from the new value.
    #[wasm_bindgen(js_name = setParam)]
    pub fn set_param(&mut self, key: &str, value: f64) -> Result<(), JsError> {
        self.sim
            .scenario_mut()
            .set_param(key, value)
            .map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(getter, js_name = integratorId)]
    pub fn integrator_id(&self) -> String {
        self.sim.integrator().id().to_string()
    }

    #[wasm_bindgen(js_name = setIntegrator)]
    pub fn set_integrator(&mut self, id: &str) -> Result<(), JsError> {
        let integrator = integrator::by_id(id)
            .ok_or_else(|| JsError::new(&format!("unknown integrator '{id}'")))?;
        self.sim.set_integrator(integrator);
        Ok(())
    }

    /// Fixed physics timestep, seconds.
    #[wasm_bindgen(getter)]
    pub fn dt(&self) -> f64 {
        self.sim.dt()
    }

    #[wasm_bindgen(setter)]
    pub fn set_dt(&mut self, dt: f64) {
        self.sim.set_dt(dt);
    }

    /// Resize the recording: how many samples to keep, and how many steps
    /// between kept samples.
    #[wasm_bindgen(js_name = setHistory)]
    pub fn set_history(&mut self, capacity: usize, stride: usize) {
        self.sim.set_recorder(Recorder::new(capacity, stride));
    }

    /// Back to the scenario's initial state, clock and recording cleared.
    pub fn reset(&mut self) {
        self.sim.reset();
    }

    /// One fixed step.
    pub fn step(&mut self) {
        self.sim.step();
    }

    /// Advance by real elapsed seconds. Returns the number of fixed steps run.
    pub fn advance(&mut self, elapsed: f64) -> usize {
        self.sim.advance(elapsed)
    }

    /// Run for a span of simulated seconds, ignoring real time. For offline
    /// runs feeding the comparison plots.
    #[wasm_bindgen(js_name = runFor)]
    pub fn run_for(&mut self, duration: f64) {
        self.sim.run_for(duration);
    }

    #[wasm_bindgen(getter)]
    pub fn time(&self) -> f64 {
        self.sim.state().t
    }

    #[wasm_bindgen(getter)]
    pub fn position(&self) -> f64 {
        self.sim.state().x
    }

    #[wasm_bindgen(getter)]
    pub fn velocity(&self) -> f64 {
        self.sim.state().v
    }

    /// Period measured from the run itself, seconds. Undefined until one full
    /// cycle has gone by, or for a system that does not oscillate.
    #[wasm_bindgen(getter, js_name = measuredPeriod)]
    pub fn measured_period(&self) -> Option<f64> {
        self.sim.measured_period()
    }

    /// State, forces, geometry and derived quantities at this instant.
    pub fn snapshot(&self) -> Result<JsValue, JsError> {
        to_js(&self.sim.snapshot())
    }

    /// Recorded samples as one flat `[t, x, v, t, x, v, ...]` array, oldest
    /// first. One typed array beats thousands of objects crossing the boundary.
    pub fn history(&self) -> Vec<f64> {
        self.sim.recorder().to_flat()
    }

    /// Number of recorded samples, i.e. `history().length / 3`.
    #[wasm_bindgen(getter, js_name = historyLength)]
    pub fn history_length(&self) -> usize {
        self.sim.recorder().len()
    }
}

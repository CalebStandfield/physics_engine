//! Drives a scenario forward in fixed timesteps.

use serde::{Deserialize, Serialize};

use crate::analysis::PeriodDetector;
use crate::integrator::{default_integrator, Integrator};
use crate::record::Recorder;
use crate::scenario::{Derived, Frame, Scenario, ScenarioSystem};
use crate::state::State;
use crate::system::System;

/// Default physics step, seconds. Small enough that 60 fps rendering does
/// several substeps per frame.
pub const DEFAULT_DT: f64 = 1.0 / 480.0;

/// Ceiling on substeps per `advance` call, so a stalled tab that reports a huge
/// elapsed time cannot lock the engine up trying to catch up.
pub const MAX_SUBSTEPS: usize = 2000;

/// Everything about the system at one instant, ready to serialize to the
/// frontend.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Snapshot {
    pub state: State,
    /// Net force along the axis, newtons.
    pub net_force: f64,
    /// Acceleration along the axis, m/s^2.
    pub accel: f64,
    /// Energy removed by friction and damping since the last reset, joules.
    pub energy_lost: f64,
    /// Fixed steps taken since the last reset.
    pub steps: u64,
    /// Period measured from the run itself, seconds. `None` for a system that
    /// does not oscillate, or before one full cycle has gone by.
    pub measured_period: Option<f64>,
    /// Complete cycles seen since the last reset.
    pub cycles: usize,
    pub frame: Frame,
    pub derived: Vec<Derived>,
}

impl Snapshot {
    /// One derived quantity by key, if the scenario reports it.
    pub fn derived_value(&self, key: &str) -> Option<f64> {
        self.derived.iter().find(|d| d.key == key).map(|d| d.value)
    }
}

/// Owns a scenario, an integrator, and the clock that connects them.
///
/// Real elapsed time goes in, whole fixed steps come out, and the leftover is
/// carried to the next call. That keeps the physics reproducible no matter how
/// uneven the frame rate is.
pub struct Simulation {
    scenario: Box<dyn Scenario>,
    integrator: Box<dyn Integrator>,
    dt: f64,
    state: State,
    accumulator: f64,
    steps: u64,
    energy_lost: f64,
    recorder: Recorder,
    detector: Option<PeriodDetector>,
}

impl Simulation {
    /// Fresh simulation at the scenario's initial state, semi-implicit Euler.
    pub fn new(scenario: Box<dyn Scenario>) -> Self {
        let state = scenario.initial_state();
        let mut recorder = Recorder::default();
        recorder.push(state);
        let detector = scenario.equilibrium().map(PeriodDetector::about);
        Self {
            scenario,
            integrator: default_integrator(),
            dt: DEFAULT_DT,
            state,
            accumulator: 0.0,
            steps: 0,
            energy_lost: 0.0,
            recorder,
            detector,
        }
    }

    pub fn scenario(&self) -> &dyn Scenario {
        &*self.scenario
    }

    /// Mutable access for parameter writes. Does not reset: a parameter change
    /// mid-run takes effect on the next step, which is what a live slider
    /// should do. Call `reset` for a clean start.
    pub fn scenario_mut(&mut self) -> &mut dyn Scenario {
        &mut *self.scenario
    }

    pub fn integrator(&self) -> &dyn Integrator {
        &*self.integrator
    }

    pub fn set_integrator(&mut self, integrator: Box<dyn Integrator>) {
        self.integrator = integrator;
    }

    pub fn dt(&self) -> f64 {
        self.dt
    }

    /// Set the fixed step. Ignores non-positive or non-finite values.
    pub fn set_dt(&mut self, dt: f64) {
        if dt.is_finite() && dt > 0.0 {
            self.dt = dt;
        }
    }

    pub fn state(&self) -> State {
        self.state
    }

    pub fn steps(&self) -> u64 {
        self.steps
    }

    pub fn energy_lost(&self) -> f64 {
        self.energy_lost
    }

    pub fn recorder(&self) -> &Recorder {
        &self.recorder
    }

    pub fn recorder_mut(&mut self) -> &mut Recorder {
        &mut self.recorder
    }

    /// Back to the scenario's initial state, clock and recording cleared.
    /// Parameters are kept.
    pub fn reset(&mut self) {
        self.state = self.scenario.initial_state();
        self.accumulator = 0.0;
        self.steps = 0;
        self.energy_lost = 0.0;
        self.recorder.clear();
        self.recorder.push(self.state);
        // Rebuilt rather than cleared: a parameter change can move the level
        // the crossings are counted about.
        self.detector = self.scenario.equilibrium().map(PeriodDetector::about);
    }

    /// Period measured from the run so far, seconds.
    pub fn measured_period(&self) -> Option<f64> {
        self.detector.as_ref().and_then(|d| d.period())
    }

    /// Complete oscillations seen since the last reset.
    pub fn cycles(&self) -> usize {
        self.detector.as_ref().map_or(0, |d| d.cycles())
    }

    /// One fixed step. The integrator moves the state, then the scenario gets
    /// to enforce anything the force law could not express.
    pub fn step(&mut self) {
        let dt = self.dt;
        let mut next = {
            let system = ScenarioSystem(&*self.scenario);
            self.integrator.step(&system, &self.state, dt)
        };
        self.scenario.constrain(&self.state, &mut next);

        // Friction and damping are path-dependent, so the loss is accumulated
        // here rather than recomputed from the state.
        self.energy_lost += self.scenario.dissipated_power(&self.state) * dt;

        self.state = next;
        self.steps += 1;
        self.recorder.observe(next);
        if let Some(detector) = self.detector.as_mut() {
            detector.observe(next);
        }
    }

    /// Take `count` fixed steps.
    pub fn step_n(&mut self, count: usize) {
        for _ in 0..count {
            self.step();
        }
    }

    /// Advance by real elapsed seconds. Returns how many fixed steps ran.
    ///
    /// Leftover time under one full step is carried over, so the simulated
    /// clock tracks the real one without ever taking a partial step.
    pub fn advance(&mut self, elapsed: f64) -> usize {
        if !elapsed.is_finite() || elapsed <= 0.0 {
            return 0;
        }
        self.accumulator += elapsed;
        let mut taken = 0;
        while self.accumulator >= self.dt && taken < MAX_SUBSTEPS {
            self.step();
            self.accumulator -= self.dt;
            taken += 1;
        }
        if taken == MAX_SUBSTEPS {
            // Too far behind to catch up; drop the backlog instead of
            // compounding it into the next call.
            self.accumulator = 0.0;
        }
        taken
    }

    /// Run for `duration` simulated seconds, ignoring the real-time
    /// accumulator. This is what tests and offline analysis use.
    pub fn run_for(&mut self, duration: f64) {
        if !duration.is_finite() || duration <= 0.0 {
            return;
        }
        let steps = (duration / self.dt).round() as usize;
        self.step_n(steps);
    }

    /// Net force along the axis at the current state, newtons.
    pub fn net_force(&self) -> f64 {
        self.scenario
            .net_force(self.state.x, self.state.v, self.state.t)
    }

    /// Acceleration along the axis at the current state, m/s^2.
    pub fn accel(&self) -> f64 {
        ScenarioSystem(&*self.scenario).accel_at(&self.state)
    }

    /// Full picture of the current instant.
    pub fn snapshot(&self) -> Snapshot {
        Snapshot {
            state: self.state,
            net_force: self.net_force(),
            accel: self.accel(),
            energy_lost: self.energy_lost,
            steps: self.steps,
            measured_period: self.measured_period(),
            cycles: self.cycles(),
            frame: self.scenario.frame(&self.state),
            derived: self.scenario.derived(&self.state),
        }
    }
}

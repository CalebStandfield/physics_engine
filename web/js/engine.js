// Thin wrapper over the wasm Engine: module init, run/pause state, and a
// snapshot cached once per frame so four panels share one boundary crossing.
//
// No physics lives here. Every number comes back out of wasm.

import init, {
  Engine,
  scenario_catalog,
  integrators,
  percent_difference,
} from "../pkg/physics_wasm.js";

import { ParamMemory } from "./memory.js";

const MAX_FRAME = 0.1; // s of real time fed to the engine in one go

// Everything a landing-page card needs to draw one scenario: a still frame at
// that scenario's defaults, plus the mass the block is drawn with. The engine
// is thrown away, nothing is stepped, and the running Sim is left alone.
// `init()` has to have finished, so call this after `Sim.boot`.
export function stillShot(id) {
  const engine = new Engine(id);
  return {
    frame: engine.snapshot().frame,
    body: {
      mass: engine.getParam("mass"),
      range: engine.schema().find((s) => s.key === "mass"),
    },
  };
}

export class Sim {
  constructor(engine, catalog, integratorList) {
    this.engine = engine;
    this.catalog = catalog;
    this.integrators = integratorList;
    this.running = false;
    this.memory = new ParamMemory();
    this.snap = engine.snapshot();
  }

  static async boot(scenarioId) {
    await init();
    const catalog = scenario_catalog();
    const list = integrators();
    const id = scenarioId ?? catalog[0].id;
    return new Sim(new Engine(id), catalog, list);
  }

  // ---- scenario ----

  get id() {
    return this.engine.scenarioId;
  }

  get entry() {
    return this.catalog.find((c) => c.id === this.id);
  }

  // Swapping scenarios always leaves you paused: starting one scenario should
  // not hand a running clock to the next one. Slider values are kept per
  // scenario across the swap.
  loadScenario(id) {
    this.memory.save(this.id, this);
    this.engine.loadScenario(id);
    this.memory.restore(id, this);
    this.running = false;
    this.refresh();
  }

  schema() {
    return this.engine.schema();
  }

  // One parameter's spec, for a panel that needs a range rather than a value.
  spec(key) {
    return this.schema().find((s) => s.key === key);
  }

  getParam(key) {
    return this.engine.getParam(key);
  }

  setParam(key, value) {
    this.engine.setParam(key, value);
    this.refresh();
  }

  // ---- solver ----

  get integratorId() {
    return this.engine.integratorId;
  }

  setIntegrator(id) {
    this.engine.setIntegrator(id);
  }

  get dt() {
    return this.engine.dt;
  }

  setDt(dt) {
    this.engine.dt = dt;
  }

  // ---- transport ----

  play() {
    this.running = true;
  }

  pause() {
    this.running = false;
  }

  reset() {
    this.engine.reset();
    this.refresh();
  }

  // Step by real elapsed seconds and re-read state. Returns true when the
  // engine actually moved.
  tick(elapsed) {
    if (!this.running) return false;
    this.engine.advance(Math.min(elapsed, MAX_FRAME));
    this.refresh();
    return true;
  }

  refresh() {
    this.snap = this.engine.snapshot();
  }

  // ---- readout helpers ----

  derived(key) {
    const row = this.snap.derived.find((d) => d.key === key);
    return row ? row.value : undefined;
  }

  percentDifference(a, b) {
    return percent_difference(a, b);
  }
}

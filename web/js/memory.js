// Slider values, remembered per scenario for as long as the page is open.
//
// Switching scenarios rebuilds the scenario inside wasm, which puts every
// parameter back to its declared default, so without this you lose your setup
// every time you look at the other one. Each scenario keeps its own values;
// nothing bleeds between them.
//
// Deliberately a plain Map: no storage backend, so a reload is a clean slate.

export class ParamMemory {
  constructor() {
    this.byScenario = new Map();
  }

  // Snapshot whatever the scenario currently has set.
  save(id, sim) {
    const values = {};
    for (const spec of sim.schema()) values[spec.key] = sim.getParam(spec.key);
    this.byScenario.set(id, values);
  }

  // Put back what this scenario had, if we have ever seen it. Driven by the
  // live schema, so a key the scenario no longer has is simply not restored.
  restore(id, sim) {
    const values = this.byScenario.get(id);
    if (!values) return;

    for (const spec of sim.schema()) {
      const value = values[spec.key];
      if (Number.isFinite(value)) sim.setParam(spec.key, value);
    }
  }
}

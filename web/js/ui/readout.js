// Numeric readout: whatever the scenario reports as derived, plus the clock,
// the energy ledger, and the measured-vs-predicted comparisons.
//
// The comparison list is data, not logic: it only names which two numbers to
// pair up. Both numbers come from the engine, and the percentage itself is
// computed by the engine's `percent_difference`.

import { num, unitSuffix } from "./format.js";

const COMPARISONS = {
  spring: [
    {
      label: "Period vs ideal",
      measured: (sim) => sim.snap.measured_period,
      predicted: "ideal_period",
    },
  ],
  incline: [
    {
      label: "Accel vs ideal",
      measured: (sim) => sim.derived("acceleration"),
      predicted: "ideal_sliding_accel",
    },
  ],
};

export class Readout {
  constructor(root) {
    this.root = root;
    this.cells = new Map();
  }

  update(sim) {
    const rows = [
      { key: "_t", label: "Time", unit: "s", value: sim.snap.state.t },
      ...sim.snap.derived,
      { key: "_lost", label: "Energy lost", unit: "J", value: sim.snap.energy_lost },
    ];

    if (sim.snap.measured_period !== undefined) {
      rows.push({
        key: "_period",
        label: "Measured period",
        unit: "s",
        value: sim.snap.measured_period,
      });
    }

    for (const cmp of COMPARISONS[sim.id] ?? []) {
      const measured = cmp.measured(sim);
      const predicted = sim.derived(cmp.predicted);
      if (measured === undefined || predicted === undefined) continue;
      rows.push({
        key: `_cmp_${cmp.predicted}`,
        label: cmp.label,
        unit: "%",
        value: sim.percentDifference(measured, predicted),
      });
    }

    this.render(rows);
  }

  render(rows) {
    const key = rows.map((r) => r.key).join("|");
    if (this.shape !== key) {
      this.shape = key;
      this.cells.clear();
      this.root.replaceChildren();
      for (const row of rows) {
        const k = document.createElement("div");
        k.className = "k";
        k.textContent = row.label;
        k.title = row.label;

        const v = document.createElement("div");
        v.className = "v";

        this.root.append(k, v);
        this.cells.set(row.key, v);
      }
    }

    for (const row of rows) {
      const cell = this.cells.get(row.key);
      if (!cell) continue;
      cell.replaceChildren();
      cell.append(document.createTextNode(num(row.value)));
      const suffix = unitSuffix(row.unit);
      if (suffix) {
        const u = document.createElement("span");
        u.className = "u";
        u.textContent = suffix;
        cell.append(u);
      }
    }
  }
}

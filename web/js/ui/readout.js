// Numeric readout: whatever the scenario reports as derived, plus the clock,
// the energy ledger, and the measured-vs-predicted comparisons.
//
// A row the engine explained (`from`: a formula plus the terms in it) unfolds:
// the formula in symbols, then one row per term, and any term that was itself
// explained unfolds the same way. So `a = F_net / m` opens onto the net force,
// which opens onto the weight component and friction, which open onto the mass,
// gravity and the angle. Nothing here knows any of those names: it walks
// whatever tree the engine handed over.
//
// The comparison list is data, not logic: it only names which two numbers to
// pair up. Both numbers come from the engine, and the percentage itself is
// computed by the engine's `percent_difference`.

import { num, unitSuffix } from "./format.js";
import { icon } from "./icons.js";
import { ADVANCED, shows } from "./mode.js";

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
  // `resize` wraps a DOM change, so the panel can animate around a row folding
  // open. Default is to just make the change.
  constructor(root, resize = (change) => change()) {
    this.root = root;
    this.resize = resize;
    // Paths of the rows the user has opened, e.g. `/acceleration/net_force`.
    // Paths rather than keys, so the same term under two parents opens on its
    // own, and so the state survives every rebuild.
    this.open = new Set();
    this.cells = new Map();
    this.rows = [];
  }

  // Rows carry the same tier the controls do, so one switch trims both panels.
  // The engine tiers its own derived rows; the ones added here say so inline.
  update(sim, mode) {
    const rows = [
      { key: "_t", label: "Time", unit: "s", value: sim.snap.state.t },
      ...sim.snap.derived,
      {
        key: "_lost",
        label: "Energy lost",
        unit: "J",
        value: sim.snap.energy_lost,
        tier: ADVANCED,
      },
    ];

    if (sim.snap.measured_period !== undefined) {
      rows.push({
        key: "_period",
        label: "Measured period",
        unit: "s",
        value: sim.snap.measured_period,
        tier: ADVANCED,
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
        tier: ADVANCED,
      });
    }

    this.rows = rows.filter((r) => shows(mode, r.tier));
    this.render();
  }

  // Everything on screen, in the order it is drawn: a row, then, when it is
  // open, its formula and one entry per term. Recursive, so depth is whatever
  // the engine's tree is.
  *walk(rows, prefix = "", depth = 0) {
    for (const row of rows) {
      const path = `${prefix}/${row.key}`;
      const open = Boolean(row.from) && this.open.has(path);
      yield { kind: "row", row, path, depth, open };
      if (!open) continue;
      yield { kind: "eq", row, path, depth: depth + 1 };
      yield* this.walk(row.from.terms, path, depth + 1);
    }
  }

  // The DOM is rebuilt when the set of visible entries changes, and only has its
  // numbers rewritten otherwise, so the panel does not thrash per frame.
  render() {
    const entries = [...this.walk(this.rows)];
    const shape = entries
      .map((e) => `${e.kind}:${e.path}:${e.row.from?.equation ?? ""}`)
      .join("|");

    if (this.shape !== shape) {
      this.shape = shape;
      this.build(entries);
    }

    for (const { kind, row, path } of entries) {
      if (kind !== "row") continue;
      const cell = this.cells.get(path);
      if (cell) this.writeValue(cell, row);
    }
  }

  build(entries) {
    this.cells.clear();
    this.root.replaceChildren();

    for (const entry of entries) {
      const node =
        entry.kind === "eq" ? this.equationLine(entry) : this.valueRow(entry);
      node.style.setProperty("--depth", entry.depth);
      this.root.append(node);
    }
  }

  // One `symbol = formula` line, the thing the terms under it add up to.
  equationLine({ row }) {
    const line = document.createElement("div");
    line.className = "eq";

    if (row.symbol) {
      const sym = document.createElement("span");
      sym.className = "sym";
      sym.textContent = row.symbol;
      line.append(sym, document.createTextNode(" = "));
    }
    line.append(document.createTextNode(row.from.equation));
    return line;
  }

  // Label and value. A row the engine explained is a button that folds it open.
  valueRow({ row, path, depth, open }) {
    const expandable = Boolean(row.from);
    const item = document.createElement(expandable ? "button" : "div");
    item.className = "row" + (open ? " open" : "");
    if (expandable) {
      item.type = "button";
      item.setAttribute("aria-expanded", String(open));
      item.append(icon("caret-right"));
      item.addEventListener("click", () => this.toggle(path));
    } else {
      // Blank of the same width, so labels line up with the ones that have a
      // chevron in front of them.
      const gap = document.createElement("span");
      gap.className = "twist-gap";
      item.append(gap);
    }

    const k = document.createElement("span");
    k.className = "k";
    // Nested rows lead with their symbol, so the formula above them reads.
    if (depth > 0 && row.symbol) {
      const sym = document.createElement("span");
      sym.className = "sym";
      sym.textContent = row.symbol;
      k.append(sym, " ");
    }
    k.append(row.label);
    k.title = row.label;

    const v = document.createElement("span");
    v.className = "v";

    item.append(k, v);
    this.cells.set(path, v);
    return item;
  }

  writeValue(cell, row) {
    cell.replaceChildren(document.createTextNode(num(row.value)));
    const suffix = unitSuffix(row.unit);
    if (suffix) {
      const u = document.createElement("span");
      u.className = "u";
      u.textContent = suffix;
      cell.append(u);
    }
  }

  toggle(path) {
    if (this.open.has(path)) this.open.delete(path);
    else this.open.add(path);
    this.resize(() => this.render());
  }
}

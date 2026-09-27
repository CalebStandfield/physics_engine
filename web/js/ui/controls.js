// Sliders, generated from the scenario's own parameter schema. Nothing in here
// names a scenario or a parameter, so a new scenario gets a full control panel
// for free.

import { num, unitSuffix } from "./format.js";
import { shows } from "./mode.js";

// Parameters that only take effect when the run restarts.
const RESET_ONLY = /^initial_/;

const DT_CHOICES = [
  { label: "1/240 s", value: 1 / 240 },
  { label: "1/480 s", value: 1 / 480 },
  { label: "1/960 s", value: 1 / 960 },
];

// `mode` decides how much of the schema shows. Which parameters are basic is the
// engine's call, carried on each spec's `tier`.
export function buildControls(root, sim, mode, onChange) {
  root.replaceChildren();

  for (const spec of sim.schema()) {
    if (!shows(mode, spec.tier)) continue;
    root.append(paramRow(spec, sim, onChange));
  }
}

function paramRow(spec, sim, onChange) {
  const wrap = document.createElement("div");
  wrap.className = "ctl";

  const head = document.createElement("div");
  head.className = "ctl-head";

  const left = document.createElement("span");
  left.className = "ctl-label";
  left.textContent = spec.label;
  head.append(left);

  if (RESET_ONLY.test(spec.key)) {
    const tag = document.createElement("span");
    tag.className = "ctl-tag";
    tag.textContent = "on reset";
    tag.title = "Applies the next time you press Reset";
    left.append(" ", tag);
  }

  const value = document.createElement("span");
  value.className = "ctl-value";
  head.append(value);

  const slider = document.createElement("input");
  slider.type = "range";
  slider.min = spec.min;
  slider.max = spec.max;
  slider.step = spec.step;
  slider.value = sim.getParam(spec.key) ?? spec.default;

  const show = (v) => {
    value.textContent = num(v, spec.step) + unitSuffix(spec.unit);
  };
  show(Number(slider.value));

  slider.addEventListener("input", () => {
    const v = Number(slider.value);
    sim.setParam(spec.key, v);
    show(v);
    onChange(spec.key);
  });

  wrap.append(head, slider);
  return wrap;
}

export function buildSolver(root, sim) {
  root.replaceChildren();

  root.append(
    select(
      sim.integrators.map((i) => ({ label: i.name, value: i.id })),
      sim.integratorId,
      (v) => sim.setIntegrator(v),
    ),
  );

  const gap = document.createElement("div");
  gap.style.height = "8px";
  root.append(gap);

  root.append(
    select(
      DT_CHOICES.map((c) => ({ label: `Step ${c.label}`, value: String(c.value) })),
      String(nearestDt(sim.dt)),
      (v) => sim.setDt(Number(v)),
    ),
  );
}

function nearestDt(dt) {
  let best = DT_CHOICES[0].value;
  for (const c of DT_CHOICES) {
    if (Math.abs(c.value - dt) < Math.abs(best - dt)) best = c.value;
  }
  return best;
}

function select(options, current, onPick) {
  const el = document.createElement("select");
  for (const o of options) {
    const opt = document.createElement("option");
    opt.value = o.value;
    opt.textContent = o.label;
    if (o.value === current) opt.selected = true;
    el.append(opt);
  }
  el.addEventListener("change", () => onPick(el.value));
  return el;
}

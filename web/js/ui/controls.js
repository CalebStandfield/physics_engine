// Sliders, generated from the scenario's own parameter schema. Nothing in here
// names a scenario or a parameter, so a new scenario gets a full control panel
// for free.

import { SIMPLE, shows } from "./mode.js";
import { sliderRow } from "./slider.js";
import { toggleRow } from "./toggle.js";

// Starting values. Before Start (or right after Reset) they apply at once;
// once the run has moved they wait for the next Reset. The engine does that,
// this only tags them.
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
    root.append(paramRow(spec, sim, mode, onChange));
  }
}

// A `switch` parameter is a slider that the simple panel shows as a checkbox:
// checked while the value is above its minimum, checking it sets the engine's
// `on` value, unchecking puts it back to the minimum. The advanced panel gets
// the full slider. Which parameters do this is the engine's call.
function paramRow(spec, sim, mode, onChange) {
  const get = () => sim.getParam(spec.key) ?? spec.default;
  const changed = () => onChange(spec.key);

  const row =
    spec.widget?.kind === "switch" && mode === SIMPLE
      ? toggleRow(spec, {
          checked: () => get() > spec.min,
          setChecked: (on) => sim.setParam(spec.key, on ? spec.widget.on : spec.min),
          onChange: changed,
        })
      : sliderRow(spec, {
          get,
          set: (v) => sim.setParam(spec.key, v),
          onChange: changed,
        });

  if (RESET_ONLY.test(spec.key)) {
    const tag = document.createElement("span");
    tag.className = "ctl-tag";
    tag.textContent = "on reset";
    tag.title = "Applies right away before Start; once running, the next time you press Reset";
    row.label.append(" ", tag);
  }

  return row.el;
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

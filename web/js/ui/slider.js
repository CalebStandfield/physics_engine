// One numeric control: a slider, an editable value box, and the landmarks on
// its track.
//
// Driven entirely by a spec (`label`, `unit`, `min`, `max`, `step`, `marks`) and
// a get/set pair, so the same row serves a scenario parameter and anything else
// with a range. Nothing here names a parameter.
//
// The box and the slider are two views of one value: whatever comes in gets
// clamped to `[min, max]` and written through `set`, and both views are redrawn
// from what was actually written.

import { num, unitSuffix } from "./format.js";

// A mark counts as hit inside half a step, so clicking one lights it up even
// though the slider rounds.
const HIT = 0.5;

// Steps a log track is cut into. A log slider has no natural increment, since
// the same drag means a different amount at each end, so it gets a fixed number
// of stops instead.
const LOG_STOPS = 240;

// How a value maps onto its track. Linear unless the spec asks for `log`, which
// spaces by ratio rather than by difference: half speed and double speed then
// sit the same distance either side of one. The box, the marks and the engine
// all stay in real units; only the track is warped.
const LINEAR = { to: (v) => v, from: (v) => v };
const LOG = { to: Math.log, from: Math.exp };

// The number at the front of whatever was typed. The box shows its unit, so the
// unit is what usually comes back with the edit; anything after the number is
// dropped. No leading number at all means the edit is unreadable.
const NUMBER = /^[+-]?(?:\d+\.?\d*|\.\d+)(?:[eE][+-]?\d+)?/;

export function sliderRow(spec, { get, set, onChange = () => {} } = {}) {
  const wrap = document.createElement("div");
  wrap.className = "ctl";

  const head = document.createElement("div");
  head.className = "ctl-head";

  const label = document.createElement("span");
  label.className = "ctl-label";
  label.textContent = spec.label;

  const box = document.createElement("input");
  box.className = "ctl-value";
  box.type = "text";
  box.inputMode = "decimal";
  box.spellcheck = false;
  box.setAttribute("aria-label", spec.label);
  box.title = `${num(spec.min, spec.step)} to ${num(spec.max, spec.step)}`;

  head.append(label, box);

  const track = spec.scale === "log" ? LOG : LINEAR;
  const lo = track.to(spec.min);
  const hi = track.to(spec.max);

  const slider = document.createElement("input");
  slider.type = "range";
  slider.min = lo;
  slider.max = hi;
  slider.step = track === LOG ? (hi - lo) / LOG_STOPS : spec.step;

  const marks = markTrack(spec, track, (v) => commit(v));

  const show = (v) => {
    slider.value = track.to(v);
    box.value = num(v, spec.step) + unitSuffix(spec.unit);
    for (const el of marks?.children ?? []) {
      el.classList.toggle("hit", Math.abs(Number(el.dataset.value) - v) <= spec.step * HIT);
    }
  };

  // The one way a value gets in. Returns what was kept, so a rejected edit can
  // fall back to it.
  const commit = (value) => {
    const v = clamp(value, spec.min, spec.max);
    set(v);
    show(v);
    onChange(v);
    return v;
  };

  show(get());

  // Dragging lands on the spec's own grid. Typing does not: the box is there to
  // say a number the slider cannot reach.
  slider.addEventListener("input", () => {
    const v = track.from(Number(slider.value));
    commit(Math.round(v / spec.step) * spec.step);
  });

  // Typed values are only read when the field is done being edited, so a
  // half-typed number never reaches the engine. Anything unreadable snaps back.
  box.addEventListener("change", () => {
    const typed = NUMBER.exec(box.value.trim());
    if (typed) commit(Number(typed[0]));
    else show(get());
  });
  box.addEventListener("keydown", (e) => {
    if (e.key === "Escape") {
      show(get());
      box.blur();
    }
  });
  box.addEventListener("focus", () => box.select());

  wrap.append(head, slider);
  if (marks) wrap.append(marks);

  return { el: wrap, label, show, commit };
}

// Landmarks under the track, each one a button that jumps to its value. Placed
// where the slider thumb sits for that value: the track is inset by half a thumb
// at each end, so the percentage is taken across the shortened span.
function markTrack(spec, track, onPick) {
  if (!spec.marks?.length) return null;

  const row = document.createElement("div");
  row.className = "ctl-marks";
  const lo = track.to(spec.min);
  const span = track.to(spec.max) - lo;

  for (const m of spec.marks) {
    const frac = (track.to(m.value) - lo) / span;
    if (!Number.isFinite(frac) || frac < 0 || frac > 1) continue;

    const tick = document.createElement("button");
    tick.className = "ctl-mark";
    tick.type = "button";
    tick.dataset.value = m.value;
    tick.title = `${m.label} (${num(m.value, spec.step)}${unitSuffix(spec.unit)})`;
    tick.setAttribute("aria-label", tick.title);
    tick.style.left = `calc(var(--thumb) / 2 + ${frac} * (100% - var(--thumb)))`;
    tick.addEventListener("click", () => onPick(m.value));
    row.append(tick);
  }

  return row.children.length ? row : null;
}

function clamp(v, min, max) {
  return Math.min(max, Math.max(min, v));
}

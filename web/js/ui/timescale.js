// How fast the clock runs: slow motion through to fast forward.
//
// Shares the slider row with the parameter panel, so it gets the same editable
// box and the same landmarks. The scale itself lives on the Sim; this only
// reads and writes it.

import { TIME_SCALE } from "../engine.js";
import { sliderRow } from "./slider.js";

export function buildTimeScale(root, sim) {
  root.replaceChildren();

  const row = sliderRow(TIME_SCALE, {
    get: () => sim.timeScale,
    set: (v) => {
      sim.timeScale = v;
    },
  });

  const note = document.createElement("p");
  note.className = "note";
  note.textContent =
    "Simulated seconds per real second. Only the clock changes: same step " +
    "size, same run, just watched faster or slower.";

  root.append(row.el, note);
}

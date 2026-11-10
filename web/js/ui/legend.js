// Legend, rebuilt from what is actually on screen: the forces the engine is
// reporting right now plus the scenery kinds in the frame.

import { forceStyle, guideStyle, NET_STYLE, COLOR } from "../theme.js";
import { isRod } from "../render/shape.js";

const GUIDE_NAMES = {
  surface: "Surface",
  anchor: "Anchor",
  spring: "Spring",
  reference: "Reference line",
  axis: "Positive direction",
  pivot: "Pivot",
};

export function buildLegend(root, frame) {
  const rows = [];

  for (const f of frame.forces) {
    rows.push({ text: f.label, ...forceStyle(f.kind) });
  }
  for (const c of frame.couples ?? []) {
    rows.push({ text: c.label, ...forceStyle(c.kind), turn: true });
  }
  rows.push({ text: "Net force", ...NET_STYLE });

  const seen = new Set();
  for (const g of frame.guides) {
    if (seen.has(g.kind)) continue;
    seen.add(g.kind);
    const style = guideStyle(g.kind);
    rows.push({ text: GUIDE_NAMES[g.kind] ?? g.label, color: style.color, dash: style.dash });
  }

  if (isRod(frame)) {
    rows.push({ text: "Stick", color: COLOR.orange, dash: [], body: "rod" });
    rows.push({ text: "Center of mass", color: COLOR.orange, dash: [], body: "dot" });
  } else {
    rows.push({ text: "Block", color: COLOR.orange, dash: [], body: "block" });
  }

  const key = rows.map((r) => r.text + r.color).join("|");
  if (root.dataset.key === key) return;
  root.dataset.key = key;

  root.replaceChildren();
  for (const row of rows) {
    const line = document.createElement("div");
    line.className = "legend-row";
    line.append(swatch(row), text(row.text));
    root.append(line);
  }
}

function swatch(row) {
  const el = document.createElementNS("http://www.w3.org/2000/svg", "svg");
  el.setAttribute("class", "swatch");
  el.setAttribute("viewBox", "0 0 22 10");

  if (row.body === "dot") {
    const dot = document.createElementNS(el.namespaceURI, "circle");
    dot.setAttribute("cx", "11");
    dot.setAttribute("cy", "5");
    dot.setAttribute("r", "2.5");
    dot.setAttribute("fill", row.color);
    el.append(dot);
    return el;
  }

  if (row.body) {
    // A block is a near-square, a rod a long thin bar.
    const [x, y, w, h] = row.body === "rod" ? [1, 3, 20, 4] : [5, 1, 12, 8];
    const rect = document.createElementNS(el.namespaceURI, "rect");
    rect.setAttribute("x", String(x));
    rect.setAttribute("y", String(y));
    rect.setAttribute("width", String(w));
    rect.setAttribute("height", String(h));
    rect.setAttribute("rx", "2");
    rect.setAttribute("fill", COLOR.bodyFill);
    rect.setAttribute("stroke", row.color);
    rect.setAttribute("stroke-width", "1.5");
    el.append(rect);
    return el;
  }

  const path = document.createElementNS(el.namespaceURI, "path");
  // A torque gets a little arc instead of a straight line.
  path.setAttribute("d", row.turn ? "M6 8 A5 5 0 1 1 16 8" : "M1 5 H21");
  path.setAttribute("fill", "none");
  path.setAttribute("stroke", row.color);
  path.setAttribute("stroke-width", "2");
  if (row.dash && row.dash.length) path.setAttribute("stroke-dasharray", row.dash.join(" "));
  el.append(path);
  return el;
}

function text(value) {
  const span = document.createElement("span");
  span.textContent = value;
  return span;
}

// Legend, rebuilt from what is actually on screen: the forces the engine is
// reporting right now plus the scenery kinds in the frame.

import { forceStyle, guideStyle, NET_STYLE, COLOR } from "../theme.js";

const GUIDE_NAMES = {
  surface: "Surface",
  anchor: "Anchor",
  spring: "Spring",
  reference: "Reference line",
  axis: "Positive direction",
};

export function buildLegend(root, frame) {
  const rows = [];

  for (const f of frame.forces) {
    rows.push({ text: f.label, ...forceStyle(f.kind) });
  }
  rows.push({ text: "Net force", ...NET_STYLE });

  const seen = new Set();
  for (const g of frame.guides) {
    if (seen.has(g.kind)) continue;
    seen.add(g.kind);
    const style = guideStyle(g.kind);
    rows.push({ text: GUIDE_NAMES[g.kind] ?? g.label, color: style.color, dash: style.dash });
  }

  rows.push({ text: "Block", color: COLOR.orange, dash: [], block: true });

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

  if (row.block) {
    const rect = document.createElementNS(el.namespaceURI, "rect");
    rect.setAttribute("x", "5");
    rect.setAttribute("y", "1");
    rect.setAttribute("width", "12");
    rect.setAttribute("height", "8");
    rect.setAttribute("rx", "2");
    rect.setAttribute("fill", COLOR.bodyFill);
    rect.setAttribute("stroke", row.color);
    rect.setAttribute("stroke-width", "1.5");
    el.append(rect);
    return el;
  }

  const path = document.createElementNS(el.namespaceURI, "path");
  path.setAttribute("d", "M1 5 H21");
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

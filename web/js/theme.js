// Color tokens and the kind -> style maps. Everything drawn on a canvas reads
// its colors from here, so the scene and the free body diagram never disagree.

export const COLOR = {
  bg: "#0c0e11",
  text: "#e9e6e0",
  muted: "#8b919c",
  line: "#242932",
  orange: "#e8955c",
  orangeDim: "#a8683c",
  bodyFill: "#0a0c0f",
};

// Force styling, keyed by ForceVector.kind coming out of the engine.
const FORCE_STYLES = {
  gravity: { color: COLOR.orange, dash: [] },
  spring: { color: "#f3b483", dash: [] },
  normal: { color: COLOR.text, dash: [] },
  friction: { color: "#c9803f", dash: [] },
  applied: { color: "#f6d3b0", dash: [] },
  damping: { color: COLOR.muted, dash: [] },
};

const FORCE_FALLBACK = { color: COLOR.text, dash: [] };

export function forceStyle(kind) {
  return FORCE_STYLES[kind] ?? FORCE_FALLBACK;
}

// The net force is drawn by the frontend, not reported as a vector, so it gets
// its own entry rather than living in the force map.
export const NET_STYLE = { color: COLOR.orange, dash: [5, 4] };

// Scenery styling, keyed by Guide.kind.
const GUIDE_STYLES = {
  surface: { color: COLOR.text, width: 2.5, dash: [], hatch: true },
  anchor: { color: COLOR.text, width: 2.5, dash: [], hatch: true },
  spring: { color: COLOR.text, width: 2, dash: [], hatch: false },
  reference: { color: COLOR.muted, width: 1, dash: [5, 5], hatch: false },
  axis: { color: COLOR.orange, width: 1.5, dash: [], hatch: false },
};

const GUIDE_FALLBACK = { color: COLOR.muted, width: 1, dash: [3, 4], hatch: false };

export function guideStyle(kind) {
  return GUIDE_STYLES[kind] ?? GUIDE_FALLBACK;
}

// Coil polyline for a guide of kind "spring".
//
// The engine only reports the two endpoints; the zigzag is decoration, so it
// lives entirely on this side. Coil count is fixed so the spring visibly
// stretches and compresses instead of just adding loops.

const COILS = 14;
const LEAD = 0.14; // straight lead-in at each end, as a fraction of length
const MAX_AMPLITUDE = 16; // px, half-width of the zigzag

export function coilPoints(a, b) {
  const dx = b.x - a.x;
  const dy = b.y - a.y;
  const len = Math.hypot(dx, dy);
  if (len < 1e-6) return [a, b];

  const ux = dx / len;
  const uy = dy / len;
  const nx = -uy;
  const ny = ux;

  const lead = len * LEAD;
  const coiled = len - 2 * lead;
  // Squeeze the zigzag as the spring compresses so it never folds over itself.
  const amp = Math.min(MAX_AMPLITUDE, (coiled / COILS) * 1.6);

  const start = { x: a.x + ux * lead, y: a.y + uy * lead };
  const pts = [a, start];

  for (let i = 0; i < COILS; i++) {
    const t = (i + 0.5) / COILS;
    const side = i % 2 === 0 ? 1 : -1;
    pts.push({
      x: start.x + ux * coiled * t + nx * amp * side,
      y: start.y + uy * coiled * t + ny * amp * side,
    });
  }

  pts.push({ x: a.x + ux * (lead + coiled), y: a.y + uy * (lead + coiled) });
  pts.push(b);
  return pts;
}

// How much of the canvas the floating panels are covering.
//
// Each overlay is assigned to the side it sits on, so the scene can be fitted
// into what is actually visible. Nothing here knows which panel is which.

const GAP = 16; // px of breathing room between a panel and the scene

export function overlayInset(canvas, overlays) {
  const c = canvas.getBoundingClientRect();
  const midX = c.left + c.width / 2;
  const midY = c.top + c.height / 2;
  const inset = { left: 0, right: 0, top: 0, bottom: 0 };

  for (const el of overlays) {
    const r = el.getBoundingClientRect();
    if (r.width === 0 || r.height === 0) continue;

    if (r.right < midX) {
      inset.left = Math.max(inset.left, r.right - c.left + GAP);
    } else if (r.left > midX) {
      inset.right = Math.max(inset.right, c.right - r.left + GAP);
    } else if (r.bottom < midY) {
      inset.top = Math.max(inset.top, r.bottom - c.top + GAP);
    } else {
      inset.bottom = Math.max(inset.bottom, c.bottom - r.top + GAP);
    }
  }

  return inset;
}

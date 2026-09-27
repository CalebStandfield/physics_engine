// Small canvas primitives shared by the scene and the free body diagram.
// Everything here takes pixel coordinates; converting from meters is the
// camera's job.

export function polyline(ctx, pts, { color, width = 1, dash = [] }) {
  if (pts.length < 2) return;
  ctx.save();
  ctx.strokeStyle = color;
  ctx.lineWidth = width;
  ctx.lineJoin = "round";
  ctx.lineCap = "round";
  ctx.setLineDash(dash);
  ctx.beginPath();
  ctx.moveTo(pts[0].x, pts[0].y);
  for (let i = 1; i < pts.length; i++) ctx.lineTo(pts[i].x, pts[i].y);
  ctx.stroke();
  ctx.restore();
}

// Tick marks on one side of a segment, the usual "this is solid ground" mark.
export function hatch(ctx, a, b, { color, spacing = 11, length = 9 }) {
  const dx = b.x - a.x;
  const dy = b.y - a.y;
  const len = Math.hypot(dx, dy);
  if (len < 1e-6) return;

  const ux = dx / len;
  const uy = dy / len;
  // Normal, rotated so the ticks trail behind the segment direction.
  const nx = uy;
  const ny = -ux;

  ctx.save();
  ctx.strokeStyle = color;
  ctx.lineWidth = 1;
  ctx.beginPath();
  for (let d = spacing / 2; d < len; d += spacing) {
    const px = a.x + ux * d;
    const py = a.y + uy * d;
    ctx.moveTo(px, py);
    ctx.lineTo(px + (nx - ux) * length * 0.7, py + (ny - uy) * length * 0.7);
  }
  ctx.stroke();
  ctx.restore();
}

export function arrow(ctx, from, to, { color, width = 2, dash = [], head = 8 }) {
  const dx = to.x - from.x;
  const dy = to.y - from.y;
  const len = Math.hypot(dx, dy);
  if (len < 0.5) return;

  const ux = dx / len;
  const uy = dy / len;
  const h = Math.min(head, len * 0.45);
  const base = { x: to.x - ux * h, y: to.y - uy * h };

  polyline(ctx, [from, base], { color, width, dash });

  ctx.save();
  ctx.fillStyle = color;
  ctx.beginPath();
  ctx.moveTo(to.x, to.y);
  ctx.lineTo(base.x - uy * h * 0.42, base.y + ux * h * 0.42);
  ctx.lineTo(base.x + uy * h * 0.42, base.y - ux * h * 0.42);
  ctx.closePath();
  ctx.fill();
  ctx.restore();
}

// `angle` is a canvas rotation about the center, radians, positive clockwise
// (canvas y points down, so a world angle comes in negated).
export function roundedRect(ctx, cx, cy, w, h, r, { fill, stroke, width = 2, angle = 0 }) {
  const x = cx - w / 2;
  const y = cy - h / 2;
  const rr = Math.min(r, w / 2, h / 2);
  ctx.save();
  if (angle) {
    ctx.translate(cx, cy);
    ctx.rotate(angle);
    ctx.translate(-cx, -cy);
  }
  ctx.beginPath();
  ctx.moveTo(x + rr, y);
  ctx.arcTo(x + w, y, x + w, y + h, rr);
  ctx.arcTo(x + w, y + h, x, y + h, rr);
  ctx.arcTo(x, y + h, x, y, rr);
  ctx.arcTo(x, y, x + w, y, rr);
  ctx.closePath();
  if (fill) {
    ctx.fillStyle = fill;
    ctx.fill();
  }
  if (stroke) {
    ctx.strokeStyle = stroke;
    ctx.lineWidth = width;
    ctx.stroke();
  }
  ctx.restore();
}

export function label(ctx, text, x, y, { color, align = "left", baseline = "middle", size = 11 }) {
  ctx.save();
  ctx.fillStyle = color;
  ctx.font = `${size}px ui-monospace, "SF Mono", Menlo, monospace`;
  ctx.textAlign = align;
  ctx.textBaseline = baseline;
  ctx.fillText(text, x, y);
  ctx.restore();
}

// The block, drawn the same way in the scene and in the free body diagram.
//
// `border` is painted inward: an orange rect with a smaller fill-colored rect on
// top of it, rather than a stroke. So a heavier block reads as a thicker border
// without its footprint changing, and a border of `side / 2` is a solid orange
// block with no fill left.
export function block(ctx, cx, cy, side, { fill, stroke, border = 2, angle = 0, radius = 4 }) {
  const b = Math.max(0, Math.min(border, side / 2));
  roundedRect(ctx, cx, cy, side, side, radius, { fill: stroke, angle });
  const inner = side - 2 * b;
  if (inner > 0.5) {
    roundedRect(ctx, cx, cy, inner, inner, Math.max(1, radius - b), { fill, angle });
  }
}

// Resize a canvas to its CSS box at device resolution. Returns the CSS size,
// which is what every drawing routine works in.
export function fitCanvas(canvas, ctx) {
  const dpr = window.devicePixelRatio || 1;
  const rect = canvas.getBoundingClientRect();
  const w = Math.max(1, Math.round(rect.width));
  const h = Math.max(1, Math.round(rect.height));
  if (canvas.width !== w * dpr || canvas.height !== h * dpr) {
    canvas.width = w * dpr;
    canvas.height = h * dpr;
  }
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  ctx.clearRect(0, 0, w, h);
  return { width: w, height: h };
}

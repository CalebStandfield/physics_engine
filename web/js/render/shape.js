// What the body looks like, read off `frame.shape`. Drawing only.
//
// A point mass is a fixed-size block. A rod is drawn to scale, so its ends are
// real world points the camera has to keep on screen.

import { roundedRect } from "./draw.js";

const BLOCK = { kind: "block" };

export function shapeOf(frame) {
  return frame.shape ?? BLOCK;
}

export function isRod(frame) {
  return shapeOf(frame).kind === "rod";
}

// Unit vector along the body's long side, world space.
function along(frame) {
  const a = frame.pose?.angle ?? 0;
  return { x: Math.cos(a), y: Math.sin(a) };
}

// World points the body covers: its center, plus both ends of a rod.
export function bodyPoints(frame) {
  if (!isRod(frame)) return [frame.body];
  const half = shapeOf(frame).length / 2;
  const u = along(frame);
  return [
    frame.body,
    { x: frame.body.x + u.x * half, y: frame.body.y + u.y * half },
    { x: frame.body.x - u.x * half, y: frame.body.y - u.y * half },
  ];
}

const TICK_SPACING = 0.1; // m, the marks on a meter stick
const MIN_WIDTH = 5; // px, so a thin rod still reads

// A rod centered at `center` (px), `scale` px per meter. Ticks every 10 cm and
// a dot at the center of mass, which is where the weight arrow starts.
export function rod(ctx, center, frame, scale, { fill, stroke, tick, dot }) {
  const { length, width } = shapeOf(frame);
  const lengthPx = length * scale;
  const widthPx = Math.max(MIN_WIDTH, width * scale);
  // World angle is counterclockwise, canvas angle is clockwise.
  const angle = -(frame.pose?.angle ?? 0);

  roundedRect(ctx, center.x, center.y, lengthPx, widthPx, 2, {
    fill,
    stroke,
    width: 1.5,
    angle,
  });

  const ux = Math.cos(angle);
  const uy = Math.sin(angle);
  const nx = -uy;
  const ny = ux;
  const count = Math.floor(length / TICK_SPACING + 1e-9);
  const spacingPx = TICK_SPACING * scale;
  if (spacingPx >= 4) {
    ctx.save();
    ctx.strokeStyle = tick;
    ctx.lineWidth = 1;
    ctx.beginPath();
    for (let i = 1; i < count; i++) {
      const s = -lengthPx / 2 + i * spacingPx;
      const px = center.x + ux * s;
      const py = center.y + uy * s;
      const h = (i % 5 === 0 ? 0.45 : 0.25) * widthPx;
      ctx.moveTo(px + nx * (widthPx / 2), py + ny * (widthPx / 2));
      ctx.lineTo(px + nx * (widthPx / 2 - h), py + ny * (widthPx / 2 - h));
    }
    ctx.stroke();
    ctx.restore();
  }

  ctx.save();
  ctx.fillStyle = dot;
  ctx.beginPath();
  ctx.arc(center.x, center.y, Math.max(2.5, widthPx * 0.28), 0, Math.PI * 2);
  ctx.fill();
  ctx.restore();
}

// The nail a rod hangs on: a ring with a filled center.
export function pivot(ctx, at, { color, fill }) {
  ctx.save();
  ctx.fillStyle = fill;
  ctx.strokeStyle = color;
  ctx.lineWidth = 2;
  ctx.beginPath();
  ctx.arc(at.x, at.y, 5, 0, Math.PI * 2);
  ctx.fill();
  ctx.stroke();
  ctx.fillStyle = color;
  ctx.beginPath();
  ctx.arc(at.x, at.y, 1.8, 0, Math.PI * 2);
  ctx.fill();
  ctx.restore();
}

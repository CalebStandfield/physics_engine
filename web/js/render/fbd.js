// Free body diagram: the body alone, every force on it, drawn to scale.
//
// Same data as the scene, isolated and labeled with magnitudes. The net arrow
// is the vector sum of the arrows already on screen, which is addition of
// numbers the engine handed us, not a physics decision made here.
//
// A point mass is a block with every arrow starting at it. An extended body
// (a rod) is drawn at its pose, with each arrow starting where the force acts,
// since that is what decides the torque. Its net arrow starts at the center of
// mass, which is the point the sum of the forces accelerates.

import { COLOR, forceStyle, NET_STYLE } from "../theme.js";
import { arrow, turnArrow, block, label, fitCanvas } from "./draw.js";
import { borderWidth } from "./mass.js";
import { isRod, rod, shapeOf } from "./shape.js";

const BODY = 26; // px
const MARGIN = 46; // px kept clear for labels
const MIN_ARROW = 18; // px, so a small-but-real force still reads
const ROD_FRACTION = 0.5; // of the smaller canvas side, for a rod's length

export class FbdRenderer {
  constructor(canvas) {
    this.canvas = canvas;
    this.ctx = canvas.getContext("2d");
  }

  // `body` is the same `{ mass, range }` the scene gets, so the two blocks carry
  // the same border. The diagram keeps the block axis-aligned: it is the body on
  // its own, cut out of whatever it was sitting on.
  draw(frame, body) {
    const { width, height } = fitCanvas(this.canvas, this.ctx);
    const cx = width / 2;
    const cy = height / 2;
    const forces = frame.forces;
    const extended = isRod(frame);

    // Where each arrow starts, px. A rod is scaled so its length is a fixed
    // share of the canvas, and world points map through that same scale.
    let start = (_f, ux, uy) => ({ x: cx + ux * (BODY / 2), y: cy + uy * (BODY / 2) });
    let toPx = () => ({ x: cx, y: cy });

    if (extended) {
      const scale = (Math.min(width, height) * ROD_FRACTION) / shapeOf(frame).length;
      rod(this.ctx, { x: cx, y: cy }, frame, scale, {
        fill: COLOR.bodyFill,
        stroke: COLOR.orange,
        tick: COLOR.orangeDim,
        dot: COLOR.orange,
      });
      toPx = (p) => ({
        x: cx + (p.x - frame.body.x) * scale,
        y: cy - (p.y - frame.body.y) * scale,
      });
      start = (f) => (f.at ? toPx(f.at) : { x: cx, y: cy });
    } else {
      block(this.ctx, cx, cy, BODY, {
        fill: COLOR.bodyFill,
        stroke: COLOR.orange,
        border: borderWidth(BODY, body.mass, body.range),
        radius: 3,
      });
    }

    const max = forces.reduce(
      (m, f) => Math.max(m, Math.hypot(f.vector.x, f.vector.y)),
      0,
    );
    if (max < 1e-9) {
      label(this.ctx, "no forces", cx, cy + BODY, {
        color: COLOR.muted,
        align: "center",
      });
      return;
    }

    // A rod's arrows can start off-center, so they get less room.
    const full = (Math.min(width, height) / 2 - MARGIN) * (extended ? 0.7 : 1);
    const scale = full / max;

    let sumX = 0;
    let sumY = 0;

    for (const f of forces) {
      const mag = Math.hypot(f.vector.x, f.vector.y);
      sumX += f.vector.x;
      sumY += f.vector.y;
      if (mag < 1e-9) continue;

      const ux = f.vector.x / mag;
      const uy = -f.vector.y / mag; // world +y up, canvas +y down
      const len = Math.max(MIN_ARROW, mag * scale);
      const from = start(f, ux, uy);
      const to = { x: from.x + ux * len, y: from.y + uy * len };

      const style = forceStyle(f.kind);
      arrow(this.ctx, from, to, { ...style, width: 2 });
      label(this.ctx, `${f.label} ${mag.toFixed(2)} N`, to.x + ux * 6, to.y + uy * 6, {
        color: style.color,
        align: ux < -0.3 ? "right" : ux > 0.3 ? "left" : "center",
        baseline: uy > 0.3 ? "top" : uy < -0.3 ? "bottom" : "middle",
        size: 10,
      });
    }

    // Pure torques: a curved arrow at the point they are drawn at, with the
    // size in the label since an arc has no length to scale.
    for (const c of frame.couples ?? []) {
      if (Math.abs(c.torque) < 1e-9) continue;
      const at = toPx(c.at);
      const style = forceStyle(c.kind);
      turnArrow(this.ctx, at, 12, c.torque > 0, { color: style.color, width: 1.5, head: 6 });
      // Just the size: the legend names it, and the full label clips on a
      // narrow panel.
      label(this.ctx, `${Math.abs(c.torque).toFixed(4)} N m`, at.x + 16, at.y + 14, {
        color: style.color,
        baseline: "top",
        size: 10,
      });
    }

    const netMag = Math.hypot(sumX, sumY);
    if (netMag > 1e-6) {
      const ux = sumX / netMag;
      const uy = -sumY / netMag;
      const len = Math.max(MIN_ARROW, netMag * scale);
      arrow(
        this.ctx,
        { x: cx, y: cy },
        { x: cx + ux * len, y: cy + uy * len },
        { ...NET_STYLE, width: 1.5 },
      );
    }
  }
}

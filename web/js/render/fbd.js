// Free body diagram: the block alone, every force on it, drawn to scale.
//
// Same data as the scene, isolated and labeled with magnitudes. The net arrow
// is the vector sum of the arrows already on screen, which is addition of
// numbers the engine handed us, not a physics decision made here.

import { COLOR, forceStyle, NET_STYLE } from "../theme.js";
import { arrow, roundedRect, label, fitCanvas } from "./draw.js";

const BODY = 26; // px
const MARGIN = 46; // px kept clear for labels
const MIN_ARROW = 18; // px, so a small-but-real force still reads

export class FbdRenderer {
  constructor(canvas) {
    this.canvas = canvas;
    this.ctx = canvas.getContext("2d");
  }

  draw(forces) {
    const { width, height } = fitCanvas(this.canvas, this.ctx);
    const cx = width / 2;
    const cy = height / 2;

    roundedRect(this.ctx, cx, cy, BODY, BODY, 3, {
      fill: COLOR.bodyFill,
      stroke: COLOR.orange,
      width: 2,
    });

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

    const full = Math.min(width, height) / 2 - MARGIN;
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
      const from = { x: cx + ux * (BODY / 2), y: cy + uy * (BODY / 2) };
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

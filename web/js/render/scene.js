// The animated scene.
//
// Draws whatever the engine put in `frame`, keyed only on `kind`. It has no
// idea which scenario is running, so a new scenario needs no change here.

import { COLOR, guideStyle, forceStyle } from "../theme.js";
import { polyline, hatch, arrow, roundedRect, label, fitCanvas } from "./draw.js";
import { coilPoints } from "./spring.js";

// Painting order by guide kind. Lower draws first.
const DEPTH = { reference: 0, surface: 1, anchor: 1, spring: 2, axis: 3 };

const MAX_FORCE_PX = 110; // longest force arrow in the scene
const REF_MASS = 1.0; // kg the body size is calibrated against

// Room left around the geometry for force arrows and their labels.
const ARROW_ROOM = MAX_FORCE_PX + 40;

function pad(inset = {}) {
  return {
    left: (inset.left ?? 0) + ARROW_ROOM,
    right: (inset.right ?? 0) + ARROW_ROOM,
    top: (inset.top ?? 0) + ARROW_ROOM,
    bottom: (inset.bottom ?? 0) + ARROW_ROOM,
  };
}

// Where a reference line's label goes: just past whichever end of the line is
// farthest from the block. Always anchoring the same end buries the label
// under the body on any line that starts there.
const LABEL_OFFSET = 9; // px past the end of the line

function labelAnchor(pts, bodyPx) {
  const a = pts[0];
  const b = pts[pts.length - 1];
  const dist = (p) => Math.hypot(p.x - bodyPx.x, p.y - bodyPx.y);
  const far = dist(b) >= dist(a) ? b : a;
  const near = far === b ? a : b;

  const dx = far.x - near.x;
  const dy = far.y - near.y;
  const len = Math.hypot(dx, dy) || 1;
  const ux = dx / len;
  const uy = dy / len;

  return {
    x: far.x + ux * LABEL_OFFSET,
    y: far.y + uy * LABEL_OFFSET,
    align: ux > 0.3 ? "left" : ux < -0.3 ? "right" : "center",
    baseline: uy > 0.3 ? "top" : uy < -0.3 ? "bottom" : "middle",
  };
}

export class SceneRenderer {
  constructor(canvas, camera) {
    this.canvas = canvas;
    this.ctx = canvas.getContext("2d");
    this.camera = camera;
  }

  // `mass` only sets how big the block looks; it changes no physics.
  // `inset` is the area the overlay panels cover, in CSS pixels.
  draw(frame, mass, inset) {
    const { width, height } = fitCanvas(this.canvas, this.ctx);
    this.camera.observe(frame);
    this.camera.fit(width, height, pad(inset));
    if (!this.camera.ready) return;

    const px = (p) => this.camera.toPx(p);
    const guides = [...frame.guides].sort(
      (a, b) => (DEPTH[a.kind] ?? 0) - (DEPTH[b.kind] ?? 0),
    );

    const center = px(frame.body);
    for (const guide of guides) this.drawGuide(guide, px, center);

    const side = this.bodySize(width, height, mass);
    roundedRect(this.ctx, center.x, center.y, side, side, 4, {
      fill: COLOR.bodyFill,
      stroke: COLOR.orange,
      width: 2,
    });

    this.drawForces(frame.forces, center, side);
  }

  bodySize(width, height, mass) {
    const base = Math.min(width, height) * 0.07;
    const m = Number.isFinite(mass) && mass > 0 ? mass : REF_MASS;
    const growth = Math.cbrt(m / REF_MASS);
    return Math.max(18, Math.min(64, base * Math.max(0.6, Math.min(1.8, growth))));
  }

  drawGuide(guide, px, bodyPx) {
    const style = guideStyle(guide.kind);
    const pts = guide.points.map(px);
    if (pts.length < 2) return;

    if (guide.kind === "spring") {
      polyline(this.ctx, coilPoints(pts[0], pts[pts.length - 1]), style);
      return;
    }

    if (guide.kind === "axis") {
      arrow(this.ctx, pts[0], pts[pts.length - 1], { ...style, head: 7 });
      return;
    }

    polyline(this.ctx, pts, style);

    if (style.hatch) {
      for (let i = 0; i < pts.length - 1; i++) {
        hatch(this.ctx, pts[i], pts[i + 1], { color: style.color });
      }
    }

    if (guide.kind === "reference") {
      const at = labelAnchor(pts, bodyPx);
      label(this.ctx, guide.label, at.x, at.y, {
        color: COLOR.muted,
        align: at.align,
        baseline: at.baseline,
      });
    }
  }

  drawForces(forces, center, side) {
    const max = forces.reduce(
      (m, f) => Math.max(m, Math.hypot(f.vector.x, f.vector.y)),
      0,
    );
    if (max < 1e-9) return;

    const perNewton = MAX_FORCE_PX / max;
    const offset = side / 2;

    for (const f of forces) {
      const mag = Math.hypot(f.vector.x, f.vector.y);
      if (mag < 1e-9) continue;

      // World y is up, canvas y is down.
      const ux = f.vector.x / mag;
      const uy = -f.vector.y / mag;
      const from = { x: center.x + ux * offset, y: center.y + uy * offset };
      const len = mag * perNewton;
      const to = { x: from.x + ux * len, y: from.y + uy * len };

      const style = forceStyle(f.kind);
      arrow(this.ctx, from, to, { ...style, width: 2 });
      label(this.ctx, f.label, to.x + ux * 7, to.y + uy * 7, {
        color: style.color,
        align: ux < -0.3 ? "right" : ux > 0.3 ? "left" : "center",
        baseline: uy > 0.3 ? "top" : uy < -0.3 ? "bottom" : "middle",
      });
    }
  }
}

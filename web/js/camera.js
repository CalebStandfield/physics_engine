// World meters to canvas pixels.
//
// The engine works in world coordinates with +y up; canvases have +y down, so
// this is the single place the flip happens. The fit box only ever grows
// between resets, otherwise an oscillating body would make the view breathe.

const PAD = 0.14; // fraction of the box added on every side
const LERP = 0.12; // how fast the view chases a changed box

export class Camera {
  constructor() {
    this.reset();
  }

  reset() {
    this.box = null; // accumulated world bounds
    this.view = null; // what we are actually drawing with
  }

  // Grow the fit box to hold every point in this frame.
  observe(frame) {
    const pts = [frame.body];
    for (const guide of frame.guides) pts.push(...guide.points);
    if (pts.length === 0) return;

    let box = this.box ?? {
      minX: Infinity,
      minY: Infinity,
      maxX: -Infinity,
      maxY: -Infinity,
    };
    for (const p of pts) {
      box = {
        minX: Math.min(box.minX, p.x),
        minY: Math.min(box.minY, p.y),
        maxX: Math.max(box.maxX, p.x),
        maxY: Math.max(box.maxY, p.y),
      };
    }
    this.box = box;
  }

  // Recompute the projection for a canvas of this size, in CSS pixels.
  fit(width, height) {
    if (!this.box) return;

    const w = Math.max(this.box.maxX - this.box.minX, 1e-6);
    const h = Math.max(this.box.maxY - this.box.minY, 1e-6);
    const target = {
      cx: (this.box.minX + this.box.maxX) / 2,
      cy: (this.box.minY + this.box.maxY) / 2,
      scale: Math.min(width / (w * (1 + 2 * PAD)), height / (h * (1 + 2 * PAD))),
    };

    if (!this.view) {
      this.view = { ...target };
    } else {
      this.view.cx += (target.cx - this.view.cx) * LERP;
      this.view.cy += (target.cy - this.view.cy) * LERP;
      this.view.scale += (target.scale - this.view.scale) * LERP;
    }

    this.width = width;
    this.height = height;
  }

  get ready() {
    return this.view != null;
  }

  // Pixels per meter.
  get scale() {
    return this.view ? this.view.scale : 1;
  }

  // World point -> canvas point.
  toPx(p) {
    const v = this.view;
    return {
      x: this.width / 2 + (p.x - v.cx) * v.scale,
      y: this.height / 2 - (p.y - v.cy) * v.scale,
    };
  }
}

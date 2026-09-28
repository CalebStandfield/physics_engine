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

  // Grow the fit box to hold every point in this frame. `guides` defaults to
  // all of them; a renderer that hides some passes the ones it actually draws,
  // so the view does not leave room for lines nobody sees.
  observe(frame, guides = frame.guides) {
    const pts = [frame.body];
    for (const guide of guides) pts.push(...guide.points);
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
  //
  // `inset` is the region the panels are covering, so the scene fits in what
  // is actually visible rather than centering under a panel.
  fit(width, height, inset = { left: 0, right: 0, top: 0, bottom: 0 }) {
    if (!this.box) return;

    const availW = Math.max(80, width - inset.left - inset.right);
    const availH = Math.max(80, height - inset.top - inset.bottom);

    const w = Math.max(this.box.maxX - this.box.minX, 1e-6);
    const h = Math.max(this.box.maxY - this.box.minY, 1e-6);
    const target = {
      cx: (this.box.minX + this.box.maxX) / 2,
      cy: (this.box.minY + this.box.maxY) / 2,
      scale: Math.min(
        availW / (w * (1 + 2 * PAD)),
        availH / (h * (1 + 2 * PAD)),
      ),
    };

    if (!this.view) {
      this.view = { ...target };
    } else {
      this.view.cx += (target.cx - this.view.cx) * LERP;
      this.view.cy += (target.cy - this.view.cy) * LERP;
      this.view.scale += (target.scale - this.view.scale) * LERP;
    }

    // Center of the uncovered region, in canvas pixels.
    this.originX = inset.left + availW / 2;
    this.originY = inset.top + availH / 2;
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
      x: this.originX + (p.x - v.cx) * v.scale,
      y: this.originY - (p.y - v.cy) * v.scale,
    };
  }
}

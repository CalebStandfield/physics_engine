// World meters to canvas pixels.
//
// The engine works in world coordinates with +y up; canvases have +y down, so
// this is the single place the flip happens. The fit box only ever grows
// between resets, otherwise an oscillating body would make the view breathe.

import { bodyPoints } from "./render/shape.js";

const PAD = 0.14; // fraction of the box added on every side
const LERP = 0.12; // how fast the view chases a changed box

// How far the user can zoom away from the fitted view, either way.
const MIN_ZOOM = 0.2;
const MAX_ZOOM = 20;

// Two layers: the fitted view, which follows the scenario's geometry on its
// own, and the user's view on top of it, a zoom about a point plus a pan in
// pixels, from pinches, drags and the zoom buttons. `reset()` clears the first
// and leaves the second, so a sim reset or a panel rebuild does not throw away
// where the user was looking. `resetView()` clears the second.
export class Camera {
  constructor() {
    this.reset();
    this.resetView();
    this.originX = 0;
    this.originY = 0;
  }

  reset() {
    this.box = null; // accumulated world bounds
    this.view = null; // what we are actually drawing with
  }

  // Back to the fitted view.
  resetView() {
    this.zoom = 1;
    this.panX = 0; // px
    this.panY = 0; // px
  }

  // Scale the user zoom by `factor`, keeping the point under `at` (canvas px)
  // where it is. Default is the middle of the uncovered region.
  zoomBy(factor, at = { x: this.originX, y: this.originY }) {
    const next = Math.min(MAX_ZOOM, Math.max(MIN_ZOOM, this.zoom * factor));
    const k = next / this.zoom;
    // Screen position = origin + pan + (world offset) * zoom, so holding the
    // point at `at` fixed means scaling its distance from origin + pan by k.
    this.panX = at.x - this.originX - (at.x - this.originX - this.panX) * k;
    this.panY = at.y - this.originY - (at.y - this.originY - this.panY) * k;
    this.zoom = next;
  }

  panBy(dx, dy) {
    this.panX += dx;
    this.panY += dy;
  }

  // Grow the fit box to hold every point in this frame: the body (both ends,
  // for a rod) and the guides. `guides` defaults to all of them; a renderer
  // that hides some passes the ones it actually draws, so the view does not
  // leave room for lines nobody sees.
  observe(frame, guides = frame.guides) {
    const pts = bodyPoints(frame);
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

  // Pixels per meter, user zoom included.
  get scale() {
    return (this.view ? this.view.scale : 1) * this.zoom;
  }

  // World point -> canvas point.
  toPx(p) {
    const v = this.view;
    const s = this.scale;
    return {
      x: this.originX + this.panX + (p.x - v.cx) * s,
      y: this.originY + this.panY - (p.y - v.cy) * s,
    };
  }
}

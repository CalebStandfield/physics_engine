// The animated scene.
//
// Draws whatever the engine put in `frame`, keyed only on `kind`. It has no
// idea which scenario is running, so a new scenario needs no change here.

import { COLOR, guideStyle, forceStyle } from "../theme.js";
import { polyline, hatch, arrow, turnArrow, block, label, fitCanvas } from "./draw.js";
import { coilPoints } from "./spring.js";
import { borderWidth } from "./mass.js";
import { isRod, rod, pivot } from "./shape.js";

// Painting order by guide kind. Lower draws first.
const DEPTH = { reference: 0, surface: 1, anchor: 1, spring: 2, axis: 3, pivot: 4 };

// Guides painted over the body rather than under it: a nail through a hole in
// the stick has to show on top of the stick.
const ABOVE_BODY = new Set(["pivot"]);

// Guides that annotate the setup rather than being part of it. Everything else
// is geometry the body actually touches.
const ANNOTATION = new Set(["reference", "axis"]);

const MAX_FORCE_PX = 110; // longest force arrow in the scene

// Room left around the geometry for force arrows and their labels.
const ARROW_ROOM = MAX_FORCE_PX + 40;

// On a small canvas (a phone) both shrink, so the margins never eat the scene:
// at most this share of the canvas's smaller side.
const ROOM_SHARE = 0.22;

const COUPLE_RADIUS = 16; // px, the curved arrow for a pure torque

function pad(inset = {}, room) {
  return {
    left: (inset.left ?? 0) + room,
    right: (inset.right ?? 0) + room,
    top: (inset.top ?? 0) + room,
    bottom: (inset.bottom ?? 0) + room,
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
  // `forces`, `annotations` and `labels` turn off the arrows, the reference
  // and axis guides, and the guide text. `room` is the margin left around the
  // geometry. The defaults are the full stage; a landing-page card wants the
  // shape of the scenario and nothing else, so it drops all three and shrinks
  // the margin.
  constructor(
    canvas,
    camera,
    {
      forces = true,
      annotations = true,
      labels = true,
      room = ARROW_ROOM,
      bodyFraction = 0.07,
    } = {},
  ) {
    this.canvas = canvas;
    this.ctx = canvas.getContext("2d");
    this.camera = camera;
    this.forces = forces;
    this.annotations = annotations;
    this.labels = labels;
    this.room = room;
    this.bodyFraction = bodyFraction;
  }

  // `body` is `{ mass, range }`: the current mass and the slider range it came
  // from, which together set how thick the block's border is drawn. Neither
  // changes any physics.
  // `inset` is the area the overlay panels cover, in CSS pixels.
  draw(frame, body, inset) {
    const { width, height } = fitCanvas(this.canvas, this.ctx);
    const guides = this.visible(frame.guides);
    this.camera.observe(frame, guides);
    // Arrows and their margin shrink together on a small canvas.
    const short = Math.min(width, height);
    const room = Math.min(this.room, short * ROOM_SHARE);
    this.maxForcePx = MAX_FORCE_PX * (room / Math.max(this.room, 1e-9));
    this.camera.fit(width, height, pad(inset, room));
    if (!this.camera.ready) return;

    const px = (p) => this.camera.toPx(p);
    guides.sort((a, b) => (DEPTH[a.kind] ?? 0) - (DEPTH[b.kind] ?? 0));

    const center = px(frame.body);
    const under = guides.filter((g) => !ABOVE_BODY.has(g.kind));
    const over = guides.filter((g) => ABOVE_BODY.has(g.kind));
    for (const guide of under) this.drawGuide(guide, px, center);

    if (isRod(frame)) {
      rod(this.ctx, center, frame, this.camera.scale, {
        fill: COLOR.bodyFill,
        stroke: COLOR.orange,
        tick: COLOR.orangeDim,
        dot: COLOR.orange,
      });
      for (const guide of over) this.drawGuide(guide, px, center);
      if (this.forces) {
        this.drawForces(frame.forces, center, 0, px);
        this.drawCouples(frame.couples ?? [], px);
      }
      return;
    }

    const side = this.bodySize(width, height);
    const pose = frame.pose ?? { angle: 0, support: { x: 0, y: 0 } };

    // The engine puts `body` on the surface the block rests on, so lift the
    // drawn box off that line by half its height. Canvas y points down.
    const seated = {
      x: center.x + pose.support.x * (side / 2),
      y: center.y - pose.support.y * (side / 2),
    };

    block(this.ctx, seated.x, seated.y, side, {
      fill: COLOR.bodyFill,
      stroke: COLOR.orange,
      border: borderWidth(side, body.mass, body.range),
      angle: -pose.angle,
    });

    for (const guide of over) this.drawGuide(guide, px, center);
    if (this.forces) this.drawForces(frame.forces, seated, side / 2, px);
  }

  // The guides this renderer draws, in painting order.
  visible(guides) {
    return guides.filter((g) => this.annotations || !ANNOTATION.has(g.kind));
  }

  // Block side in px, a fraction of the smaller canvas dimension. A card is
  // small enough that the stage fraction would bottom out at the clamp, so the
  // preview asks for a bigger one.
  bodySize(width, height) {
    return Math.max(18, Math.min(64, Math.min(width, height) * this.bodyFraction));
  }

  drawGuide(guide, px, bodyPx) {
    const style = guideStyle(guide.kind);
    const pts = guide.points.map(px);

    if (guide.kind === "pivot") {
      for (const p of pts) pivot(this.ctx, p, { color: style.color, fill: COLOR.bodyFill });
      return;
    }

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

    if (guide.kind === "reference" && this.labels) {
      const at = labelAnchor(pts, bodyPx);
      label(this.ctx, guide.label, at.x, at.y, {
        color: COLOR.muted,
        align: at.align,
        baseline: at.baseline,
      });
    }
  }

  // Pure torques as curved arrows around where they act. A couple has no
  // length to scale, so only its sense is drawn; the diagram gives the size.
  drawCouples(couples, px) {
    for (const c of couples) {
      if (Math.abs(c.torque) < 1e-9) continue;
      const at = px(c.at);
      const style = forceStyle(c.kind);
      turnArrow(this.ctx, at, COUPLE_RADIUS, c.torque > 0, { color: style.color });
      if (this.labels) {
        label(this.ctx, c.label, at.x + COUPLE_RADIUS + 6, at.y - COUPLE_RADIUS, {
          color: style.color,
          baseline: "bottom",
        });
      }
    }
  }

  // A force with an `at` point starts there; one without starts at the body,
  // pushed `offset` px out so it clears the drawn block.
  drawForces(forces, center, offset, px) {
    const max = forces.reduce(
      (m, f) => Math.max(m, Math.hypot(f.vector.x, f.vector.y)),
      0,
    );
    if (max < 1e-9) return;

    const perNewton = this.maxForcePx / max;

    for (const f of forces) {
      const mag = Math.hypot(f.vector.x, f.vector.y);
      if (mag < 1e-9) continue;

      // World y is up, canvas y is down.
      const ux = f.vector.x / mag;
      const uy = -f.vector.y / mag;
      const from = f.at
        ? px(f.at)
        : { x: center.x + ux * offset, y: center.y + uy * offset };
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

// Pinch, drag, scroll and the zoom buttons, all feeding the camera's user view.
//
// One pointer drags the view. Two pointers pinch: the zoom follows the change
// in distance between them and the view follows their midpoint, so a pinch can
// zoom and pan at once. On a trackpad a pinch arrives as a wheel event with
// `ctrlKey` set, which zooms; a plain two-finger scroll pans. Nothing here
// touches the engine.

const STEP = 1.25; // zoom per button press
const WHEEL_ZOOM = 0.01; // zoom per pixel of pinch-wheel delta

export function attachViewport(canvas, camera, { zoomIn, zoomOut, fit }) {
  const pointers = new Map();
  let last = null; // { mid, dist } of the previous move

  // Canvas px from a pointer or wheel event.
  const at = (e) => {
    const r = canvas.getBoundingClientRect();
    return { x: e.clientX - r.left, y: e.clientY - r.top };
  };

  // Where the gesture is: the one pointer, or the midpoint and spread of two.
  const gesture = () => {
    const pts = [...pointers.values()];
    if (pts.length === 1) return { mid: pts[0], dist: 0 };
    const [a, b] = pts;
    return {
      mid: { x: (a.x + b.x) / 2, y: (a.y + b.y) / 2 },
      dist: Math.hypot(a.x - b.x, a.y - b.y),
    };
  };

  canvas.addEventListener("pointerdown", (e) => {
    pointers.set(e.pointerId, at(e));
    // Keep getting moves when the finger leaves the canvas. Capture throws for
    // a pointer that is already gone, which only costs us the capture.
    try {
      canvas.setPointerCapture(e.pointerId);
    } catch {
      /* gesture still works inside the canvas */
    }
    last = gesture();
    canvas.classList.add("dragging");
  });

  canvas.addEventListener("pointermove", (e) => {
    if (!pointers.has(e.pointerId)) return;
    pointers.set(e.pointerId, at(e));
    const now = gesture();
    camera.panBy(now.mid.x - last.mid.x, now.mid.y - last.mid.y);
    if (now.dist > 0 && last.dist > 0) camera.zoomBy(now.dist / last.dist, now.mid);
    last = now;
  });

  const lift = (e) => {
    pointers.delete(e.pointerId);
    // Going from two fingers to one restarts the gesture from the one left,
    // or the view would jump to it.
    last = pointers.size ? gesture() : null;
    if (!pointers.size) canvas.classList.remove("dragging");
  };
  canvas.addEventListener("pointerup", lift);
  canvas.addEventListener("pointercancel", lift);

  canvas.addEventListener(
    "wheel",
    (e) => {
      e.preventDefault();
      if (e.ctrlKey) camera.zoomBy(Math.exp(-e.deltaY * WHEEL_ZOOM), at(e));
      else camera.panBy(-e.deltaX, -e.deltaY);
    },
    { passive: false },
  );

  // Buttons zoom about the middle of what is visible.
  zoomIn.addEventListener("click", () => camera.zoomBy(STEP));
  zoomOut.addEventListener("click", () => camera.zoomBy(1 / STEP));
  fit.addEventListener("click", () => camera.resetView());
}

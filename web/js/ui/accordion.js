// Height animation for a panel whose contents just changed.
//
// The DOM swap itself is instant: panels are rebuilt, not tweened. This only
// tweens the box around it, from the height it had to the height it ends up
// with, so a panel gaining or losing rows unfolds instead of snapping.
//
// It animates regardless of `prefers-reduced-motion`. Respecting that setting
// is the usual default, but this machine has it on, and a 220 ms height change
// on a panel you asked to fold is not the kind of motion it is there to stop.
// To hand the setting back, gate the `animate` call on
// `window.matchMedia("(prefers-reduced-motion: reduce)").matches`.

const MS = 220;

export function resizeAround(panel, change, ms = MS) {
  const from = panel.getBoundingClientRect().height;
  change();
  const to = panel.getBoundingClientRect().height;

  if (from === to || !panel.animate) return;

  panel.animate(
    [{ height: `${from}px` }, { height: `${to}px` }],
    { duration: ms, easing: "cubic-bezier(0.2, 0, 0, 1)" },
  );
}

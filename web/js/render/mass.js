// Mass shows up as how thick the block's orange border is, not as how big the
// block is. A heavy block used to grow until it swallowed the scene.
//
// The scale is logarithmic because the mass slider spans four decades
// (0.01 to 100 kg): linear would leave everything under ~20 kg hairline thin.
// At the bottom of the range the border is `MIN_BORDER`, at the top it is half
// the block, which `draw.block` renders as a solid orange square.

const MIN_BORDER = 1; // px at the lightest mass

export function borderWidth(side, mass, range) {
  return MIN_BORDER + (side / 2 - MIN_BORDER) * fraction(mass, range);
}

// Where this mass falls in its slider's range, 0 to 1.
function fraction(mass, range) {
  const min = range?.min;
  const max = range?.max;
  if (!Number.isFinite(mass) || !Number.isFinite(min) || !Number.isFinite(max)) return 0;
  if (!(max > min) || min <= 0) return 0;

  const t = Math.log(Math.max(mass, min) / min) / Math.log(max / min);
  return Math.max(0, Math.min(1, t));
}

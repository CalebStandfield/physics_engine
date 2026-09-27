// Mass shows up as how thick the block's orange border is, not as how big the
// block is. A heavy block used to grow until it swallowed the scene.
//
// The curve is the square root of where the mass falls in its own slider range:
// at the bottom the border is `MIN_BORDER`, at the top it is half the block,
// which `draw.block` renders as a solid orange square. Square root rather than
// linear because the range runs to 100 kg and everyday masses sit near the
// bottom of it; linear would leave all of them hairline thin.

const MIN_BORDER = 1; // px at the lightest mass

export function borderWidth(side, mass, range) {
  return MIN_BORDER + (side / 2 - MIN_BORDER) * Math.sqrt(fraction(mass, range));
}

// Where this mass falls in its slider's range, 0 to 1.
function fraction(mass, range) {
  const min = range?.min;
  const max = range?.max;
  if (!Number.isFinite(mass) || !Number.isFinite(min) || !Number.isFinite(max)) return 0;
  if (!(max > min)) return 0;

  return Math.max(0, Math.min(1, (mass - min) / (max - min)));
}

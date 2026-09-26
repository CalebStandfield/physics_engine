// Number formatting for the panels. Keeps columns from jittering as values
// swing through zero.

export function num(value, step) {
  if (value === undefined || value === null || !Number.isFinite(value)) return "--";

  const abs = Math.abs(value);
  if (abs !== 0 && (abs < 1e-3 || abs >= 1e6)) return value.toExponential(2);

  let places = 3;
  if (step !== undefined && step > 0) {
    places = Math.max(0, Math.ceil(-Math.log10(step)));
  } else if (abs >= 100) {
    places = 1;
  } else if (abs >= 10) {
    places = 2;
  }
  return value.toFixed(Math.min(places, 4));
}

export function unitSuffix(unit) {
  return unit && unit !== "(none)" ? ` ${unit}` : "";
}

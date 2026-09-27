// The simple | advanced switch.
//
// One mode drives every panel: `simple` shows the controls and readout rows the
// engine marked basic, `advanced` shows all of them plus the solver. The tier
// itself is the engine's call, so nothing here names a parameter.

export const SIMPLE = "simple";
export const ADVANCED = "advanced";

const MODES = [SIMPLE, ADVANCED];

// Whether a row or control of this tier belongs on screen in this mode.
export function shows(mode, tier) {
  return mode === ADVANCED || tier !== ADVANCED;
}

export function buildModeToggle(root, mode, onPick) {
  root.replaceChildren();

  for (const value of MODES) {
    const btn = document.createElement("button");
    btn.className = "mode-btn" + (value === mode ? " active" : "");
    btn.dataset.mode = value;
    btn.textContent = value;
    btn.addEventListener("click", () => onPick(value));
    root.append(btn);
  }
}

export function setActiveMode(root, mode) {
  for (const btn of root.children) {
    btn.classList.toggle("active", btn.dataset.mode === mode);
  }
}

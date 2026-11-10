// One on/off control: a checkbox and its label.
//
// `checked()` reads the state and `setChecked(bool)` writes it, so the caller
// decides what on and off mean as numbers. Nothing here names a parameter.

export function toggleRow(spec, { checked, setChecked, onChange = () => {} }) {
  const wrap = document.createElement("label");
  wrap.className = "ctl ctl-toggle";

  const box = document.createElement("input");
  box.type = "checkbox";
  box.checked = checked();
  box.addEventListener("change", () => {
    setChecked(box.checked);
    box.checked = checked();
    onChange();
  });

  const label = document.createElement("span");
  label.className = "ctl-label";
  label.textContent = spec.label;

  wrap.append(box, label);
  return { el: wrap, label };
}

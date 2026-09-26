// Scenario tabs, built from the engine's own catalog.

export function buildTabs(root, catalog, activeId, onPick) {
  root.replaceChildren();

  for (const entry of catalog) {
    const btn = document.createElement("button");
    btn.className = "tab" + (entry.id === activeId ? " active" : "");
    btn.textContent = entry.name;
    btn.title = entry.description;
    btn.addEventListener("click", () => onPick(entry.id));
    root.append(btn);
  }
}

export function setActiveTab(root, catalog, activeId) {
  [...root.children].forEach((btn, i) => {
    btn.classList.toggle("active", catalog[i].id === activeId);
  });
}

export function setCaption(el, entry) {
  el.replaceChildren();
  if (!entry) return;

  el.append(document.createTextNode(entry.description + " "));
  const coord = document.createElement("span");
  coord.className = "coord";
  coord.textContent = entry.coordinate_label;
  el.append(coord);
}

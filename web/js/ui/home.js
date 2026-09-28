// Landing page: the title block lives in the markup, the scenario cards here.
//
// Cards are built from the engine's own catalog, so adding a scenario in Rust
// adds a card here with no change to this file. The grid is auto-fit, so two
// scenarios sit side by side and four fall into a 2x2.

import { icon } from "./icons.js";

// Builds the card grid. Returns one `{ entry, canvas }` per scenario so the
// caller can draw a preview into each one.
export function buildCards(root, catalog, onPick) {
  root.replaceChildren();

  return catalog.map((entry) => {
    const card = document.createElement("button");
    card.className = "card";
    card.addEventListener("click", () => onPick(entry.id));

    const canvas = document.createElement("canvas");
    canvas.className = "card-view";

    const body = document.createElement("div");
    body.className = "card-body";

    const name = document.createElement("h2");
    name.textContent = entry.name;

    const about = document.createElement("p");
    about.textContent = entry.description;

    // Footer: the scenario's coordinate convention, and an arrow that reads as
    // "this card opens something".
    const foot = document.createElement("div");
    foot.className = "card-foot";

    const coord = document.createElement("span");
    coord.className = "card-coord";
    coord.textContent = entry.coordinate_label;

    foot.append(coord, icon("arrow-right"));

    body.append(name, about, foot);
    card.append(canvas, body);
    root.append(card);

    return { entry, canvas };
  });
}

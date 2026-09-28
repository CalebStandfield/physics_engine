// Phosphor Icons, vendored as path data.
//
// The page has no network at runtime, so nothing can be loaded from a CDN. Each
// entry is the `d` of one 256x256 Phosphor glyph, copied from
// @phosphor-icons/core (MIT). The whole set is the `fill` weight, matching the
// portfolio. Add an icon by pasting its path here, then name it in the markup
// with `data-icon`.

export const ICONS = {
  "house-simple":
    "M224,120v96a8,8,0,0,1-8,8H40a8,8,0,0,1-8-8V120a15.87,15.87,0,0,1,4.69-11.32l80-80a16,16,0,0,1,22.62,0l80,80A15.87,15.87,0,0,1,224,120Z",
  briefcase:
    "M152,112a8,8,0,0,1-8,8H112a8,8,0,0,1,0-16h32A8,8,0,0,1,152,112Zm80-40V200a16,16,0,0,1-16,16H40a16,16,0,0,1-16-16V72A16,16,0,0,1,40,56H80V48a24,24,0,0,1,24-24h48a24,24,0,0,1,24,24v8h40A16,16,0,0,1,232,72ZM96,56h64V48a8,8,0,0,0-8-8H104a8,8,0,0,0-8,8Zm120,57.61V72H40v41.61A184,184,0,0,0,128,136,184,184,0,0,0,216,113.61Z",
  "github-logo":
    "M216,104v8a56.06,56.06,0,0,1-48.44,55.47A39.8,39.8,0,0,1,176,192v40a8,8,0,0,1-8,8H104a8,8,0,0,1-8-8V216H72a40,40,0,0,1-40-40A24,24,0,0,0,8,152a8,8,0,0,1,0-16,40,40,0,0,1,40,40,24,24,0,0,0,24,24H96v-8a39.8,39.8,0,0,1,8.44-24.53A56.06,56.06,0,0,1,56,112v-8a58.14,58.14,0,0,1,7.69-28.32A59.78,59.78,0,0,1,69.07,28,8,8,0,0,1,76,24a59.75,59.75,0,0,1,48,24h24a59.75,59.75,0,0,1,48-24,8,8,0,0,1,6.93,4,59.74,59.74,0,0,1,5.37,47.68A58,58,0,0,1,216,104Z",
  "linkedin-logo":
    "M216,24H40A16,16,0,0,0,24,40V216a16,16,0,0,0,16,16H216a16,16,0,0,0,16-16V40A16,16,0,0,0,216,24ZM96,176a8,8,0,0,1-16,0V112a8,8,0,0,1,16,0ZM88,96a12,12,0,1,1,12-12A12,12,0,0,1,88,96Zm96,80a8,8,0,0,1-16,0V140a20,20,0,0,0-40,0v36a8,8,0,0,1-16,0V112a8,8,0,0,1,15.79-1.78A36,36,0,0,1,184,140Z",
  "arrow-right":
    "M221.66,133.66l-72,72A8,8,0,0,1,136,200V136H40a8,8,0,0,1,0-16h96V56a8,8,0,0,1,13.66-5.66l72,72A8,8,0,0,1,221.66,133.66Z",
  play: "M240,128a15.74,15.74,0,0,1-7.6,13.51L88.32,229.65a16,16,0,0,1-16.2.3A15.86,15.86,0,0,1,64,216.13V39.87a15.86,15.86,0,0,1,8.12-13.82,16,16,0,0,1,16.2.3L232.4,114.49A15.74,15.74,0,0,1,240,128Z",
  stop: "M216,56V200a16,16,0,0,1-16,16H56a16,16,0,0,1-16-16V56A16,16,0,0,1,56,40H200A16,16,0,0,1,216,56Z",
  "arrow-counter-clockwise":
    "M224,128a96,96,0,0,1-94.71,96H128A95.38,95.38,0,0,1,62.1,197.8a8,8,0,0,1,11-11.63A80,80,0,1,0,71.43,71.39a3.07,3.07,0,0,1-.26.25L60.63,81.29l17,17A8,8,0,0,1,72,112H24a8,8,0,0,1-8-8V56A8,8,0,0,1,29.66,50.3L49.31,70,60.25,60A96,96,0,0,1,224,128Z",
};

const NS = "http://www.w3.org/2000/svg";

// One glyph as an inline <svg>. Sized and colored by CSS (`fill: currentColor`),
// so the caller only picks a name.
export function icon(name) {
  const svg = document.createElementNS(NS, "svg");
  svg.setAttribute("class", "ph");
  svg.setAttribute("viewBox", "0 0 256 256");
  svg.setAttribute("aria-hidden", "true");

  const path = document.createElementNS(NS, "path");
  path.setAttribute("d", ICONS[name] ?? "");
  svg.append(path);

  return svg;
}

// Swaps every `<i data-icon="name">` under `root` for its glyph, keeping any
// class the slot carried. Lets the static markup name an icon without repeating
// path data.
export function hydrateIcons(root) {
  for (const slot of root.querySelectorAll("[data-icon]")) {
    const svg = icon(slot.dataset.icon);
    if (slot.className) svg.setAttribute("class", `ph ${slot.className}`);
    slot.replaceWith(svg);
  }
}

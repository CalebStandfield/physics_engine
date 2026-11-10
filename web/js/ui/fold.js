// Folding panels on a narrow screen.
//
// Each panel starts with a `.fold` button naming it. Tapping it toggles
// `.folded` on the panel, which hides everything but the button. The CSS only
// honors `.folded` under the narrow-screen media query, so on a wide screen the
// buttons are hidden and every panel is always open.

import { resizeAround } from "./accordion.js";

export function wireFolds(root) {
  for (const button of root.querySelectorAll(".fold")) {
    const panel = button.closest(".panel");
    const sync = () =>
      button.setAttribute("aria-expanded", String(!panel.classList.contains("folded")));
    sync();
    button.addEventListener("click", () => {
      resizeAround(panel, () => {
        panel.classList.toggle("folded");
        sync();
      });
    });
  }
}

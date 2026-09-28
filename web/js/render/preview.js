// Still renders for the landing page cards.
//
// Same scene renderer the stage uses, with the arrows, the reference lines and
// the labels off: at card size those are noise, the shape of the scenario is
// the point. Nothing is stepped, so a card is one draw per layout change.

import { Camera } from "../camera.js";
import { SceneRenderer } from "./scene.js";

const ROOM = 16; // px of margin around the geometry

const NO_INSET = { left: 0, right: 0, top: 0, bottom: 0 };

export class Preview {
  constructor(canvas) {
    this.camera = new Camera();
    this.scene = new SceneRenderer(canvas, this.camera, {
      forces: false,
      annotations: false,
      labels: false,
      room: ROOM,
      bodyFraction: 0.15,
    });
  }

  // `shot` is a `stillShot` from the engine. The camera is reset first so it
  // lands on its target in one draw instead of easing toward it.
  draw(shot) {
    this.camera.reset();
    this.scene.draw(shot.frame, shot.body, NO_INSET);
  }
}

// Wiring and the render loop. Reads state from the engine, hands it to the
// renderers and the panels. Computes nothing physical.

import { Sim } from "./engine.js";
import { Camera } from "./camera.js";
import { SceneRenderer } from "./render/scene.js";
import { FbdRenderer } from "./render/fbd.js";
import { buildTabs, setActiveTab, setCaption } from "./ui/topbar.js";
import { buildControls, buildSolver } from "./ui/controls.js";
import { buildLegend } from "./ui/legend.js";
import { Readout } from "./ui/readout.js";
import { overlayInset } from "./ui/inset.js";

const el = (id) => document.getElementById(id);

async function main() {
  let sim;
  try {
    sim = await Sim.boot();
  } catch (err) {
    el("boot").textContent = `engine failed to load: ${err}`;
    throw err;
  }

  const camera = new Camera();
  const scene = new SceneRenderer(el("scene"), camera);
  const fbd = new FbdRenderer(el("fbd"));
  const readout = new Readout(el("readout"));

  const rebuild = () => {
    camera.reset();
    buildControls(el("controls"), sim, () => {});
    buildSolver(el("solver"), sim);
    setCaption(el("caption"), sim.entry);
  };

  buildTabs(el("tabs"), sim.catalog, sim.id, (id) => {
    if (id === sim.id) return;
    sim.loadScenario(id);
    setActiveTab(el("tabs"), sim.catalog, id);
    rebuild();
  });
  rebuild();

  const transport = (running) => {
    sim.running = running;
    el("btn-start").disabled = running;
    el("btn-stop").disabled = !running;
  };

  el("btn-start").addEventListener("click", () => transport(true));
  el("btn-stop").addEventListener("click", () => transport(false));
  el("btn-reset").addEventListener("click", () => {
    sim.reset();
    camera.reset();
  });
  transport(false);

  el("boot").classList.add("hidden");

  const overlays = [...document.querySelectorAll(".panel, .transport, .caption")];

  let last = performance.now();
  const frame = (now) => {
    const elapsed = (now - last) / 1000;
    last = now;

    sim.tick(elapsed);

    // Paused frames still redraw, so slider changes show up immediately.
    const snap = sim.snap;
    const mass = sim.getParam("mass");
    scene.draw(snap.frame, mass, overlayInset(el("scene"), overlays));
    fbd.draw(snap.frame.forces);
    buildLegend(el("legend"), snap.frame);
    readout.update(sim);

    requestAnimationFrame(frame);
  };
  requestAnimationFrame(frame);
}

main();

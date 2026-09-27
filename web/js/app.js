// Wiring and the render loop. Reads state from the engine, hands it to the
// renderers and the panels. Computes nothing physical.

import { Sim } from "./engine.js";
import { Camera } from "./camera.js";
import { SceneRenderer } from "./render/scene.js";
import { FbdRenderer } from "./render/fbd.js";
import { buildTabs, setActiveTab, setCaption } from "./ui/topbar.js";
import { buildControls, buildSolver } from "./ui/controls.js";
import { SIMPLE, buildModeToggle, setActiveMode } from "./ui/mode.js";
import { resizeAround } from "./ui/accordion.js";
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

  // How much of each panel is on screen. One mode per panel: trimming the
  // controls should not also trim the readout. Page state, so both survive a
  // scenario switch.
  const mode = { controls: SIMPLE, readout: SIMPLE };

  // The mass slider's range, which sets how thick the block's border gets drawn.
  let massRange = sim.spec("mass");

  const rebuild = () => {
    camera.reset();
    massRange = sim.spec("mass");
    buildControls(el("controls"), sim, mode.controls, () => {});
    buildSolver(el("solver"), sim);
    el("solver-block").classList.toggle("hidden", mode.controls === SIMPLE);
    setCaption(el("caption"), sim.entry);
  };

  // One switch, the panel it resizes, and what to redraw when it moves.
  const wireMode = (toggle, panel, key, apply) => {
    buildModeToggle(toggle, mode[key], (next) => {
      if (next === mode[key]) return;
      mode[key] = next;
      setActiveMode(toggle, next);
      resizeAround(panel, apply);
    });
  };

  wireMode(el("mode-controls"), el("panel-controls"), "controls", rebuild);
  wireMode(el("mode-fbd"), el("panel-fbd"), "readout", () =>
    readout.update(sim, mode.readout),
  );

  const transport = (running) => {
    sim.running = running;
    el("btn-start").disabled = running;
    el("btn-stop").disabled = !running;
  };

  buildTabs(el("tabs"), sim.catalog, sim.id, (id) => {
    if (id === sim.id) return;
    sim.loadScenario(id);
    setActiveTab(el("tabs"), sim.catalog, id);
    // The engine pauses on a swap; keep the buttons saying so.
    transport(false);
    rebuild();
  });
  rebuild();

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
    const body = { mass: sim.getParam("mass"), range: massRange };
    scene.draw(snap.frame, body, overlayInset(el("scene"), overlays));
    fbd.draw(snap.frame.forces, body);
    buildLegend(el("legend"), snap.frame);
    readout.update(sim, mode.readout);

    requestAnimationFrame(frame);
  };
  requestAnimationFrame(frame);
}

main();

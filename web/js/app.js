// Wiring and the render loop. Reads state from the engine, hands it to the
// renderers and the panels. Computes nothing physical.

import { Sim, stillShot } from "./engine.js";
import { Camera } from "./camera.js";
import { SceneRenderer } from "./render/scene.js";
import { Preview } from "./render/preview.js";
import { FbdRenderer } from "./render/fbd.js";
import { buildCards } from "./ui/home.js";
import { buildTabs, setActiveTab, setCaption } from "./ui/topbar.js";
import { buildControls, buildSolver } from "./ui/controls.js";
import { SIMPLE, buildModeToggle, setActiveMode } from "./ui/mode.js";
import { resizeAround } from "./ui/accordion.js";
import { buildLegend } from "./ui/legend.js";
import { Readout } from "./ui/readout.js";
import { buildTimeScale } from "./ui/timescale.js";
import { overlayInset } from "./ui/inset.js";
import { hydrateIcons } from "./ui/icons.js";
import { readRoute, setRoute, onRoute } from "./router.js";

const el = (id) => document.getElementById(id);

async function main() {
  // Static markup names its icons with `data-icon`; swap them for real glyphs
  // before anything else so nothing pops in later.
  hydrateIcons(document);

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
  // The readout folds rows open, which changes the panel's height; hand it the
  // same tween the mode switch uses.
  const readout = new Readout(el("readout"), (change) =>
    resizeAround(el("panel-fbd"), change),
  );

  // How much of each panel is on screen. One mode per panel: trimming the
  // controls should not also trim the readout. Page state, so both survive a
  // scenario switch.
  const mode = { controls: SIMPLE, readout: SIMPLE };

  // The mass slider's range, which sets how thick the block's border gets drawn.
  let massRange = sim.spec("mass");

  // The control panel, redrawn from whatever the engine currently holds. Called
  // on its own when the values changed under the panel (a reset) rather than the
  // scenario changing.
  const refreshControls = () => {
    buildControls(el("controls"), sim, mode.controls, () => {});
    el("solver-block").classList.toggle("hidden", mode.controls === SIMPLE);
  };

  const rebuild = () => {
    camera.reset();
    massRange = sim.spec("mass");
    refreshControls();
    buildSolver(el("solver"), sim);
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

  // Every control back to its declared default. The run keeps going: this is the
  // panel resetting, not the clock.
  el("btn-reset-controls").addEventListener("click", () => {
    sim.resetParams();
    refreshControls();
  });

  // The right-hand panel shows one of two things above the readout: the free
  // body diagram, or the speed control. The clock is the switch.
  let showingTime = false;

  el("btn-clock").addEventListener("click", () => {
    showingTime = !showingTime;
    el("btn-clock").classList.toggle("active", showingTime);
    el("btn-clock").title = showingTime
      ? "Swap the speed control for the diagram"
      : "Swap the diagram for the speed control";
    resizeAround(el("panel-fbd"), () => {
      el("view-fbd").classList.toggle("hidden", showingTime);
      el("view-time").classList.toggle("hidden", !showingTime);
      if (showingTime) buildTimeScale(el("timescale"), sim);
    });
  });

  const transport = (running) => {
    sim.running = running;
    el("btn-start").disabled = running;
    el("btn-stop").disabled = !running;
  };

  // The scenario list, or one scenario. Which one is in the URL, so a scenario
  // can be linked to and the back button works.
  const ids = sim.catalog.map((e) => e.id);
  let onStage = false;

  // `id` is the scenario to show, or null for the list. Leaving a scenario
  // pauses it, same rule as switching between two.
  const show = (id) => {
    onStage = Boolean(id);

    if (id) {
      if (id !== sim.id) sim.loadScenario(id);
      setActiveTab(el("tabs"), sim.catalog, id);
      rebuild();
    }

    // The engine pauses on a swap; keep the buttons saying so.
    transport(false);
    el("stage").classList.toggle("hidden", !onStage);
    el("home").classList.toggle("hidden", onStage);
    el("tabs").classList.toggle("hidden", !onStage);
    if (!onStage) drawPreviews();
  };

  // Same, plus a history entry. Everything the user clicks goes through here;
  // `show` on its own is for the URL already having changed.
  const open = (id) => {
    if (id === (onStage ? sim.id : null)) return;
    setRoute(id);
    show(id);
  };

  onRoute(ids, show);

  buildTabs(el("tabs"), sim.catalog, sim.id, open);

  // One still shot and one renderer per card, both taken once. Cards only
  // redraw when the layout changes.
  const shots = new Map(sim.catalog.map((e) => [e.id, stillShot(e.id)]));
  const cards = buildCards(el("cards"), sim.catalog, open).map((card) => ({
    ...card,
    preview: new Preview(card.canvas),
  }));

  function drawPreviews() {
    for (const card of cards) card.preview.draw(shots.get(card.entry.id));
  }

  // The brand is the way back: no separate overview button, the landing page is
  // the only other view.
  el("btn-home").addEventListener("click", () => open(null));

  window.addEventListener("resize", () => {
    if (!onStage) drawPreviews();
  });

  rebuild();

  // Whatever the URL asks for. An unknown path is not an error page, it is the
  // list, and it rewrites itself so the bad path does not sit in history.
  const landing = readRoute(ids);
  setRoute(landing, { replace: true });
  show(landing);

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

    if (!onStage) {
      requestAnimationFrame(frame);
      return;
    }

    sim.tick(elapsed);

    // Paused frames still redraw, so slider changes show up immediately.
    const snap = sim.snap;
    const body = { mass: sim.getParam("mass"), range: massRange };
    scene.draw(snap.frame, body, overlayInset(el("scene"), overlays));
    if (!showingTime) fbd.draw(snap.frame.forces, body);
    buildLegend(el("legend"), snap.frame);
    readout.update(sim, mode.readout);

    requestAnimationFrame(frame);
  };
  requestAnimationFrame(frame);
}

main();

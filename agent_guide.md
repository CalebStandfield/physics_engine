# physics_engine agent guide

Real implementation guide for the phys2010 course project. `AGENTS.md` and `claude.md` in
this directory both just point here.

## What this is

A Rust physics engine compiled to WebAssembly, driving a small HTML/Canvas/JS frontend that
runs entirely in the browser. No backend server.

The course itself is algebra-based (no calculus) — that constraint governs the write-up/
report and anything user-facing (UI labels, report text, comments meant to explain physics),
not the engine's internals. Numerical integration (Euler or semi-implicit Euler stepping) is
fine to implement: it's an algebraic recurrence relation applied every timestep, not calculus
notation. Just never surface derivative/integral notation (dx/dt, integral signs, etc.)
anywhere the user or a grader will read it.

## Scenarios

Two real-world scenarios, both from the current unit (springs, force vectors):

1. **Block hanging from a spring** — vertical mass-spring system. Forces: spring force
   (Hooke's law, F = -kx) and gravity. Produces oscillation; good for showing force vectors
   and equilibrium.
2. **Block on an incline** — block on a hill at some angle. Forces: gravity decomposed into
   components along and normal to the incline, normal force, friction (static/kinetic).
   Good for showing vector decomposition.

"Real-world" means real parameters: an actual mass in kg, a spring constant in N/m, an
incline angle in degrees, a friction coefficient — not arbitrary unitless numbers. Any
numeric parameter or result quoted in the proposal or report must come from an actual
computation done in-session (python3, or the Rust engine itself), never recalled from
memory. This is the same numbers rule the rest of the repo follows (see root
`AGENTS.md`/`claude.md`).

## Architecture

Modular pipeline — each stage should be swappable without touching the others:

- **Core simulation crate**: state (position, velocity, forces), the integrator, and vector
  math. Scenario-agnostic — knows nothing about springs or inclines specifically, just how to
  step a body forward given a force.
- **Scenario modules**: one for the spring, one for the incline. Each implements a shared
  trait/interface (e.g. "compute forces given current state and scenario parameters") that
  plugs into the core integrator. Adding a third scenario later should mean writing one new
  module, not touching the core.
- **WASM bindings layer**: thin `wasm-bindgen` layer exposing step/reset/configure calls to
  JS. No physics logic here — just marshaling.
- **Frontend**: HTML/Canvas + JS. Only renders state and reads/writes configuration (mass,
  spring constant, angle, etc.). No physics logic in JS — if the frontend needs a physics
  answer, it asks the WASM engine, it doesn't compute one itself.

## Directory readmes

Every feature directory has its own short readme. Start there instead of grepping blind:

- `crates/physics-core/README.md` - state, integrators, the `Scenario` trait, params,
  recorder, analysis, the `Simulation` driver.
- `crates/physics-scenarios/README.md` - the spring and incline force laws, their
  parameters and guides, the registry, and how to add a third scenario.
- `crates/physics-wasm/README.md` - the full JS-facing API surface of the `Engine` class.
- `web/README.md` - frontend layout, the landing page and the stage, element ids, the
  `Sim` wrapper, the camera.
- `web/js/render/README.md` - canvas primitives, the scene, the free body diagram.
- `web/js/ui/README.md` - the DOM panels and what drives each one.
- `scripts/README.md` - build and serve, and `run.sh` usage.

Keep them current. If a change moves a file, renames a type, changes a force law, adds or
removes a parameter, changes an invariant, or changes how one stage talks to the next, update
that directory's readme in the same change. A stale readme is worse than no readme, because
the next agent trusts it. Skip the update only for work that leaves the feature and the
architecture exactly as described (a bug fix inside one function, a comment, formatting).
New directory means a new readme, and a new readme means a line in this list.

## Review

Any physics equations represented by code should go through the `physics-reviewer`
subagent (`.claude/agents/physics-reviewer.md`) before being passed off.

## Git

Git actions should always look human and be frequent.
All pushes to main will be done by the user.
 - No co-author
 - Commits should be short and concise
PRs if ever made should follow these same rules

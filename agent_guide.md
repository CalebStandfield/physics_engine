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

## Review

Any physics equations represented by code should go through the `physics-reviewer`
subagent (`.claude/agents/physics-reviewer.md`) before being passed off.

## Git

Git actions should always look human and be frequent.
All pushes to main will be done by the user.
 - No co-author
 - Commits should be short and concise
PRs if ever made should follow these same rules

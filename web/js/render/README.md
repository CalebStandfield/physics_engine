# web/js/render

Canvas drawing. Everything here takes pixel coordinates. Converting from meters is
`camera.js`. Colors come from `theme.js`.

| File | What is in it |
| --- | --- |
| `draw.js` | Primitives: `polyline`, `hatch` (ground ticks), `arrow`, `roundedRect` (optionally rotated), `block` (the body, shared by the scene and the diagram), `label`, `fitCanvas` (device-pixel resize, returns the CSS size everything else works in). |
| `scene.js` | `SceneRenderer`. The animated scene: guides, the block, force arrows with labels. Options trim it down for a small canvas. |
| `preview.js` | `Preview`. One still render per landing page card, using the scene renderer with everything but the geometry turned off. |
| `fbd.js` | `FbdRenderer`. The block alone with every force to scale, magnitudes in the labels, plus the net arrow. |
| `spring.js` | `coilPoints(a, b)`. The zigzag for a guide of kind `spring`. |
| `mass.js` | `borderWidth(side, mass, range)`. How heavy the block's border is drawn. |

## Scene

- Guides are painted in `DEPTH` order by kind: reference, then surface/anchor, then spring, then axis.
- `spring` guides go through `coilPoints`, `axis` guides draw as an arrow, everything else is a
  polyline plus hatching if the style asks for it. `reference` guides get a label.
- Reference labels anchor past whichever end of the line is farthest from the block
  (`labelAnchor`), otherwise a line starting at the body buries its own label.
- Force arrows scale so the largest force is `MAX_FORCE_PX = 110`. `ARROW_ROOM` pads the camera
  fit by that plus 40 px so arrows and labels stay on screen.
- The block is a fixed size, `min(width, height) * bodyFraction` (0.07 on the stage) clamped
  to 18-64 px. Mass shows up as
  border thickness instead (`mass.js`): a heavier block used to grow until it swallowed the
  scene. The border is painted inward, so the footprint never changes, and at the top of the
  mass range it fills the block. Looks only, no physics.
- The block is turned and seated by `Frame.pose`: rotated by `pose.angle`, and lifted half its
  height along `pose.support` so it sits on the surface instead of straddling it. The engine
  keeps `frame.body` on the surface line, which is where the physics puts it.

## Preview

The card renderer is the same `SceneRenderer` with four options: `forces: false`,
`annotations: false` (drops the `reference` and `axis` guides), `labels: false`, a 16 px margin
instead of `ARROW_ROOM`, and a bigger `bodyFraction` since a card is small enough that the
stage fraction would bottom out at the clamp. Arrows and dashed reference lines are noise at
that size; the shape of the scenario is the point. The camera is reset before each draw so it
lands on its target in one frame instead of easing toward it, and nothing is stepped, so a card
is one draw per layout change.

## Free body diagram

- Arrows scale to the canvas, floor of `MIN_ARROW = 18 px` so a small but real force still reads.
- The net arrow is the vector sum of the arrows already drawn. That is addition of numbers the
  engine handed over, not a physics decision made here.
- `MARGIN = 46 px` is kept clear for labels.
- The block carries the same mass border the scene draws, but stays axis-aligned: the diagram is
  the body on its own, cut out of whatever it was resting on.

## Spring coil

`COILS = 14` is fixed so the spring visibly stretches instead of adding loops. The zigzag
amplitude shrinks as it compresses so it never folds over itself. The engine only reports the
two endpoints; the coil is decoration and lives entirely on this side.

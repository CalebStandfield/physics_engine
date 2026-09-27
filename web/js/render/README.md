# web/js/render

Canvas drawing. Everything here takes pixel coordinates. Converting from meters is
`camera.js`. Colors come from `theme.js`.

| File | What is in it |
| --- | --- |
| `draw.js` | Primitives: `polyline`, `hatch` (ground ticks), `arrow`, `roundedRect`, `label`, `fitCanvas` (device-pixel resize, returns the CSS size everything else works in). |
| `scene.js` | `SceneRenderer`. The animated scene: guides, the block, force arrows with labels. |
| `fbd.js` | `FbdRenderer`. The block alone with every force to scale, magnitudes in the labels, plus the net arrow. |
| `spring.js` | `coilPoints(a, b)`. The zigzag for a guide of kind `spring`. |

## Scene

- Guides are painted in `DEPTH` order by kind: reference, then surface/anchor, then spring, then axis.
- `spring` guides go through `coilPoints`, `axis` guides draw as an arrow, everything else is a
  polyline plus hatching if the style asks for it. `reference` guides get a label.
- Reference labels anchor past whichever end of the line is farthest from the block
  (`labelAnchor`), otherwise a line starting at the body buries its own label.
- Force arrows scale so the largest force is `MAX_FORCE_PX = 110`. `ARROW_ROOM` pads the camera
  fit by that plus 40 px so arrows and labels stay on screen.
- Block size tracks the cube root of mass, clamped to 18-64 px. Looks only, no physics.

## Free body diagram

- Arrows scale to the canvas, floor of `MIN_ARROW = 18 px` so a small but real force still reads.
- The net arrow is the vector sum of the arrows already drawn. That is addition of numbers the
  engine handed over, not a physics decision made here.
- `MARGIN = 46 px` is kept clear for labels.

## Spring coil

`COILS = 14` is fixed so the spring visibly stretches instead of adding loops. The zigzag
amplitude shrinks as it compresses so it never folds over itself. The engine only reports the
two endpoints; the coil is decoration and lives entirely on this side.

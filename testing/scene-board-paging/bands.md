# The pixels one spare layer of headroom moves, 2026-09-26

Round two found 356 physical pixels at 256 by 256 where the board drawn with
one spare layer of headroom differs from the board drawn with none. Mark asked
for the cause before the branch merges. The probe is `headroom_bands` in
`crates/isometry-views/src/scene/band_tests.rs`, run by hand; its raw output
is out of tree, hashed at the foot of this page. Every finding below is the
same in the debug and release builds.

## Where the bands are

- **On screen:** two columns of the 890 by 752 scene texture, **445 and 871**,
  every third row from where the terrain starts in each column (row 153 and
  row 364) to the bottom of the frame. Through the headed composite (the pane
  at physical x 418.2, 2.002 physical pixels a texel) those are physical x
  1310 and 2163, the capture's two 3-pixel bands at 1309 to 1311 and 2161 to
  2163.
- **Under them:** the flat ground's top voxel layer (y = 1), along the world
  lines x - z = 20.26 and x - z = 162.41, from the map's far edge row
  (z = -640) toward the camera: column 445 over x -619 to -420, column 871
  over x -478 to -349. On the CPU 330 texels move: with headroom 1 the ray
  lands one voxel over in x (column 445) or in z (column 871). In 290 of them
  that voxel is in the same tile, with the same material and face. In 39 it
  is in the neighbouring tile, a tile being 5 voxels across, so the material
  changes (grass, stone, water); in 1 the ray slides off the map's far edge
  onto the background. No flip changes the face. The GPU shows **40**
  differing texels, all among the CPU's 330, and per column they are exactly
  the material changes and the miss: 24 in column 445, 15 + 1 in column 871.
  The visible bands are these.
- **Not the pointer volume's edges.** At headroom 0 the volume is x and z -640
  to -128, y 0 to 8; at headroom 1, x and z -640 to -112, y 0 to 16. Its side
  faces are off screen or on the map's far edge, and the bands run through
  its middle.

## What moves them

A replica of the traversal, `BrickMap::trace_ray` step for step in f32 with
the pointer box an argument, first reproduces both maps exactly (0 texels
differ from either), then moves the box one part at a time. Texels unlike
headroom 0, of 669,280:

| box | moved |
| --- | --- |
| headroom 1 (top up a layer, and wider) | 330 |
| top up a layer only | 330 |
| wider only (x and z to -112) | 0 |
| grown downward a layer | 0 |
| far faces out a brick | 0 |
| shifted out a brick, no framed brick lost | 0 |
| top up a quarter voxel | 0 |
| top up one voxel | 1 |
| top up two layers | 660 |

Only the top face matters, and the count grows with how far above the ground
it stands. Nothing about the volume's width, depth or position does.

## The cause: crossings accumulated in f32

The shared traversal (`brick_dda` in mere's `modulus`, mirrored on the CPU by
`isometer-lens`'s `trace_ray`) starts each ray where it enters the pointer
box, and advances it by adding `1 / |direction|` to each axis's next crossing
once per step. The board's rays begin on the slab's front wall, about 1,380
world units back (the board sizes its slab to reach the whole map), so by the
time they reach the ground the crossings are near 1,600 to 1,700, where an f32
step is 1.2e-4. The board's forward is (-0.612, -0.5, -0.612): 1 / 0.5 = 2 adds
exactly, but 1 / 0.612 = 1.633 rounds on every addition, so the x and z
crossings drift, about 6e-5 each time they advance, and the y crossing
does not.

The rays of these two columns pass a y boundary and an x (or z) boundary
within 5e-4 to 8e-4 world units of each other. The pixel grid makes that
recur down the whole column: a voxel is exactly 3 texels tall on the
headed pixel grid, so a column's rays meet the voxel grid in the same three
phases all the way down. Where the f32 x crossing has drifted by more than
the gap, the traversal steps x before y, and lands one voxel over.

The first moved texel of each column, where the f32 and exact walks part:

| texel | box | starts at | steps | exact crossings (x, y) | f32 crossings (x, y) | x drift |
| --- | --- | --- | --- | --- | --- | --- |
| (445, 153) | headroom 0 | y 7 | 21, never parts | | | -4.0e-4 at the ground |
| (445, 153) | headroom 1 | y 15 | parts at step 47 | 1709.98234, 1709.98149: y first | 1709.98132, 1709.98145: x first | -1.01e-3 |

Column 871's texel (871, 364) is the same with z for x: exact z 1595.12822
after y 1595.12773; f32 z 1595.12720 before y 1595.12769; z drift -1.02e-3.

One spare layer is about 28 more steps before the ground, and that is enough
drift to flip gaps of 5e-4 to 8e-4. A quarter voxel adds too few steps to flip any.

## Which picture is right

An f64 traversal of the same camera ray, started above every voxel and not
from any box, with every crossing recomputed from the ray's origin, agrees
with **headroom 0 at all 330 moved texels and with headroom 1 at none**. No
moved ray lies within 1e-9 of an edge, so the f64 answer is not itself in
doubt. Over the **whole frame**, though, headroom 0 already disagrees with
f64 at **201** texels, and headroom 1 at **531**, which is those 201 plus the
330. Headroom 0 is not the correct picture; it is a shorter traversal with
less drift. The fault is in the traversal, and headroom only lengthens it.

## Where a fix belongs

- **Not the headroom's sizing.** The headroom must sit above the terrain,
  and any rise of the box's top lengthens every ray's traversal. Everything
  the sizing can move without raising the top changes no pixel, as the table
  shows; raising it is the headroom. No sizing restores byte-identical
  pictures short of no headroom, so nothing was changed on this branch.
- **mere's `modulus` traversal**, for the cause. `brick_dda` could take each
  crossing as `t0 + n * delta` or `(boundary - eye) / direction` rather than
  by repeated addition, so the error stops growing with the steps taken and
  the picture stops depending on where the traversal starts; or it could
  measure `t` from the box entry rather than from the eye, so the numbers it
  rounds are tens rather than thousands. Either changes pictures everywhere,
  and brings them nearer the f64 one.
- **isometer's tracer**, for a mitigation within this repository. The tracer
  hands `brick_dda` an eye on the slab's front wall. Sliding it along the ray
  to just before the box, in `tracer.wgsl` and in the CPU mirror
  `bricks/ray.rs` together so picks keep agreeing with pixels, would shrink
  every `t` about a hundredfold and the drift with it, without touching mere.
  The board's own slab depth, sized to reach the whole map, is what puts the
  wall so far back; a slab sized to the frame would help too, and is the
  board's camera, not the sizing.

## Raw receipts kept out of tree

Under ruling 255 the probe's logs are in
`Code/testing/isometry/receipts/2026-09-26/scene-board-paging/bands/`. Each
opens with its commit, command and compiler; the two builds' findings are
identical.

| File | Bytes | SHA-256 |
| --- | --- | --- |
| `probe-release.log` | 92,620 | `81d275af82446362f61885cbeec4dc9373fdc4748ecaa84262d23632e3d4302d` |
| `probe-debug.log` | 92,466 | `ee5cdc43fd3942b6548b41a9ac657e2dc1801165a29167849e501c5b8a95277b` |

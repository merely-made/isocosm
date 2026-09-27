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

## At the pin bump, 2026-09-27

Re-run at 4663de8, on mere 0418391f with the atlas sized to the card, whose
modulus has the retarget fix but not the traversal fix, the probe's 357
output lines are the same as above to the character: 330 CPU texels, 40 on
the GPU. The headed captures agree. With one spare layer the 256 board
still differs from headroom 0 in the same 356 pixels, the two bands at
physical x 1309 to 1311 and 2161 to 2163, and its board region is pixel for
pixel the one 3312d08 drew on the old pins; every pixel that moved across
the bump is in the side panel. One capture of three at headroom 1 differs
from the other two in a single side-panel pixel, by one step of green, so
the pictures table reads 357 for it.

## Raw receipts kept out of tree

### Traversal repin gate, 2026-09-27

The bounded gate passed on `lane-e-paging` at
`e688a5e3f14549fce9237eff1a4fc96b6fecbbab`, after main's verified repin
`b4db31d380bb207a822eb87d04f8c439d8dbc48b` merged without conflicts. The
tested source uses mere `7bb5bfdab273cb5e8236ef519fb9f0eb8f37bee7` and
isometer's adapter over the shared CPU walk. The compiler was
`rustc 1.98.1 (48a229cea 2026-09-01)`, host `x86_64-pc-windows-msvc`,
LLVM 22.1.8. The working tree was clean when the run started.

```text
CARGO_TARGET_DIR=C:\t\cargo-targets\isometry
CARGO_BUILD_JOBS=4
cargo test --release -p isometry-views --lib headroom_bands --offline --locked -- --ignored --nocapture --test-threads=1
```

The one requested test passed; 120 tests were filtered out. It checked the
same 890 by 752 frame, 669,280 texels:

| Same-run check | Result |
| --- | --- |
| Retained old walker, headroom 0 against 1 | 330 moved texels, asserted |
| Fixed CPU walk, headroom 0 against 1 | 0 moved hits, asserted |
| Fixed GPU, headroom 0 against 1 | 0 moved pixels, asserted |
| GPU headroom 0 against the empty-terrain frame | 669,280 differing pixels |
| GPU headroom 1 against the empty-terrain frame | 669,280 differing pixels |
| Each of the three GPU captures | 2,677,120 bytes, asserted |

The empty-terrain control uses the same camera and grade with `terrain=None`.
Both terrain frames must differ from it; a missing or failed GPU capture
fails the gate. CPU traversal errors fail rather than becoming misses, and
the CPU frame must contain at least one terrain hit. The old walker remains
deliberately faulty, exposing the accumulated-crossing error in this same
run. Its box-variant diagnostics now compare that old walk to the fixed
map; those values are not failures of the fixed traversal.

The independent f64-camera diagnostic still reports 68 differing texels
for each fixed map. This gate establishes independence from headroom;
it does not claim agreement with that f64 camera at every texel. Broader
cost and headed-picture receipts were not rerun for this bounded gate.

Raw files are retained under
`Code/testing/isometry/receipts/2026-09-27/lane-e-traversal-band-control/`.
`provenance.json` records the command, compiler, branch, clean source commit,
integrated main, target, job count and SHA-256 values for the lock and test
sources. `result.json` records exit code 0. The SHA-256 of `SHA256SUMS` is
`d6a8d27e5d730c4ec2445487defd2d6b6f4788a78534176db0802c57fe28fa6b`.

| File | SHA-256 |
| --- | --- |
| `provenance.json` | `6e52389e0c330e9824f12c0329e8520445051e960fbb4461d9739d242bf8ec60` |
| `stdout.log` | `5a61dddf5f9aaf5118c4c73d14661deac3ee829b4c10a80d009d7af14bd957c3` |
| `stderr.log` | `158b82f8137279d60b3bfe3c9c00849c84d080a6dbf0b90602320973546666e2` |
| `result.json` | `f67f95af1e97d7593c7d24fdf6c841593a24999b84a29a40b0896a43cc9790bf` |

The preceding attempt at `1bbdf4f76202101b35466cf812351aea356412e4`
also passed the 330/0/0 assertions, but lacked the empty-terrain GPU control.
It remains separately archived in `.../lane-e-traversal-band-gate/` and is
not the final acceptance receipt.

Two source-review concerns remain follow-ups, not reproduced failures or
new rulings. Neither was changed by this traversal gate:

- Live settings: `BoardSource::set_residency` requests a residency rebuild,
  but `BoardSource::inputs` does not include the settings or a settings
  revision, and `SceneProducer::source_mut` does not invalidate the produced
  frame. With `needs_frame=false` and an unchanged signature, production can
  return early. The host currently sets this only during construction. If
  live settings are selected, verify draw, change settings, then draw with
  an unchanged camera and `needs_frame=false`.
- Overflow reporting: `Residency::update` copies `framed.overflow` into
  local stats, then returns `Current` before storing them when retained keys
  and dirty slots are unchanged. A different dropped outer set can therefore
  leave the reported overflow count unchanged. `ResidencyStats` currently
  describes the last change. If counted overflow becomes a standing policy,
  settle whether the report describes the current frame, then test the same
  retained keys with different dropped counts.

The three paging policy choices remain open. Passing this gate does not
authorize those choices or merge paging into main.

### Earlier raw receipts

Under ruling 255 the probe's logs are in
`Code/testing/isometry/receipts/2026-09-26/scene-board-paging/bands/`, and
the pin bump's in `.../scene-board-paging/4663de8/bands/`. Each opens with
its commit, command and compiler. The first round's two builds found the
same; the pin bump's was run in release.

| File | Bytes | SHA-256 |
| --- | --- | --- |
| `probe-release.log` | 92,620 | `81d275af82446362f61885cbeec4dc9373fdc4748ecaa84262d23632e3d4302d` |
| `probe-debug.log` | 92,466 | `ee5cdc43fd3942b6548b41a9ac657e2dc1801165a29167849e501c5b8a95277b` |
| `4663de8/bands/probe-release.log` | 92,389 | `799602bc73b2f405ac0dc76e23eaa7be9d5b6cbcae17327152f47812e1e25ae2` |

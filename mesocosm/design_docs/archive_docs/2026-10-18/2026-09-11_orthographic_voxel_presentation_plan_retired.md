# The orthographic voxel presentation plan: what moved out on 2026-10-18

**Why this moved.** On 2026-10-18 Mark made the old numbered rulings cases, kept as precedent, with new rules numbered by him alone, and asked every active doc to lose its stacked status banners, old progress entries, stale counters and pasted copies. This file holds, verbatim, what left the live [orthographic voxel presentation plan](../../2026-09-11_orthographic_voxel_presentation_plan.md): the four status paragraphs and the 2026-09-16 status addendum at its head, the Current state and Earlier current state sections that sat before the CSS standards list, and the oldest Progress entry. The live status folds in what those sections said about the tenant and the open done-conditions.

## Status banners

**Status, 2026-10-10, Q7:** Mesocosm's tenant migration is implemented,
with all four workspace compile gates and both CPU controls passed.
GPU runtime/headed certification remains after-pass work.
LiveBody remains for the VTT board,
Eponym's embedded producer, and the shared posed-face queries; Mesocosm
no longer uses its unlit draw.

**Status, 2026-10-10:** L2, L7 and L9 have landed, L7 on mere's kiss3d
tenant rather than the DOM path this plan first drew (734 to 736, 749).
Ruling 796 makes the tenant the wing's one body rasteriser, so Mesocosm
moves onto it and isometer-render's `LiveBody` retires. L10 is briefed and
unbuilt, with the views founding plan's minimap folded into its second
camera (801). See [the current state](#current-state-2026-10-10).
*(Earlier status lines follow as written.)*

**Status, 2026-10-01:** lane [L10, the detail ladder](#l10-the-detail-ladder-2026-10-01),
is briefed from the wing design record's rulings 423 to 426, 432 to 434, 437
and 438: five rungs in one camera, chosen by size within what the sim holds,
terrain in nested rings, seamless or dithered transitions, far marks drawing
the effects' glyphs as pixel icons through the player's filter, and minimaps
and overmaps as second cameras. Nothing is built. Whether examining fires as
the camera approaches waits on a measurement of the lift's latency.

**Status, 2026-09-13:** path C is the main spatial renderer under rulings
13–15: mutually occluding ground and bodies share one producer-owned depth
viewport, with body parts as instances. L0a/L0b and L0c are recorded below;
the 2026-09-13 follow-up completes continuous parent yaw and the content-keyed
sprite/shape-count probes. On identical poses at 1,000 bodies, A takes
135–148 ms median and C 2.14–2.43 ms. At 1,000 distinct torso shapes C takes
about 8 ms; repeated sprites now share 12 images. Residual pixels and timing
tails remain qualified in the receipt. Bench A now supplies posed bounds and
visible-part queries to the existing Section, with headless pixel checks.
Bench B now embeds that scene in Genet with CSS appearance and native part
input; its qualified native receipt is below. The standalone probes did not
require viewport embedding, style mapping or picking. The bench reuses
Mesocosm's creator and shared-depth section; visible disposable trials follow
in Bench C. General CSS 3D, the large-DOM first-frame gate and retained
planar fragment repair were independent work, not prerequisites for that
bench; all three have since landed (see the 2026-09-16 addendum below).
See [the assessed slices](#specimen-bench-prerequisites-2026-09-13).

**Status addendum, 2026-09-16:** genet's four lanes and their receipts are
recorded in `genet/design_docs/2026-09-12_css_3d_transforms_and_first_frame_plan.md`, which owns them; this plan cites, not copies.
T1 (genet `d61978378c1`): CSS Transforms Level 2 parses, computes and lowers
to the shared paint matrix; a three-part rigid body rendered through Ortet
agrees with `isometer-mesh`'s bake silhouette on 98.73% of the union with no
pixel more than one pixel out; an individual `rotate` animates one part's yaw
with the parent matrix unchanged. T2 (genet `b3250d3d91b`): the first-frame
quadratic is gone, exponent 2.16 to 1.17, 50,000 positioned elements 1,172 s
to 7.68 s; the 1 s bound is not reached and its residual is attributed to the
cascade. T3 (genet `cd54b1ef086`, `0b49031f1d7`): the host mutation
instrument reports work apart from presentation wait, and a hit-test
regression the seam had introduced on unsized form controls is repaired;
genet's seam acceptance is complete. T4 (netrender `06f3a12f4`): a fragment
placed inside a clip, opacity or filter layer retains, and the clip-wrapped
L0a rerun takes the 200,000-rectangle cell from 40.5-52.0 ms to 8.8-9.6 ms.
T4 is not yet reachable from Cambium: a 2026-09-16 headless host receipt in
mere found Rootstock registers custom leaves as retained fragments but its
emitter never places them, so no Cambium document exercises T4 until that
wiring is completed (`Code/testing/mere/cambium_t4_receipt_20260916/`).
The bench's remaining dependencies are unchanged: Mere's producer lifecycle,
picking/accessibility and update-cost receipts.

## Current state, 2026-10-10

Mesocosm's `section/tenant.rs` consumes Isometer's prepared, culled geometry
on the caller's shared device and encoder. The scene traces terrain first
into a sampleable `Depth32Float`; the tenant reads it as its depth pre-pass,
draws a transparent sRGB body target, and Netrender layers terrain, bodies
and chrome. Captures use the same final master; the chromeless path blends
the same two source targets. Shadows remain off (749). The `--body-light`
setting controls presentation-only ambient intensity and is recorded in the
played receipt, never by the simulation.

The pinned tenant has no world-cut entry, so its adapter clips the same
posed faces and slab/bounds used by picking. It keeps per-body palette,
tint, part selection and double-sided cutaway faces. Tissue marks retain
their local-voxel density grammar, subdividing only marked greedy faces.
These are source-level adapter mappings; headed lighting, palette and
selection preservation are still uncertified. The adapter rebuilds CPU
palette meshes for comparison each frame; `mesh_builds` counts that work.
Unchanged meshes avoid repeated GPU mesh uploads. Moving, turning or cutting
a body can cause a new world-space mesh upload; those costs and the mark
subdivision cost belong to after-pass measurement.

The body receipt carries the tenant's reported internal submissions and
refuses a nonzero count. Mesh upload bytes are estimates; the pinned API
does not expose its camera/light upload counters. The opaque Netrender
receipt labels the terrain import, with a separate external-layer pass
producing the final master. Its boundary counts are not whole-frame counts.
The extra terrain import's cost remains an after-pass item.

LiveBody retirement has additional real consumers beyond Mesocosm:
`isometry-views/src/scene/board.rs` and Eponym's `producer/source.rs` still
build the shared scene's unlit pass. Its pose/query types also support the
new tenant's geometry. Those callers keep the default shared scene path;
retirement waits until they have moved. No Mere change or repin is needed.
Ruling 796's full retirement remains open.

## Earlier current state, 2026-10-10

Added under ruling 793; the dated text above keeps its words, including
its places where renderling or the DOM is the body's path.

- **The body path is mere's tenant.** L7 landed on the kiss3d tenant
  (`crates/conatus/tenant` in mere, 745), not on the appearance crate plus
  DOM that L7's text describes: `eponym-client` draws bodies through it onto
  a transparent target that netrender layers over the traced colour (749),
  and renderling, crabslab, 729's carriage and the `ambience-lease` probe
  are gone. Palette colours reach kiss3d as a texture by UV (736). Torch
  brightness and shadows remain tuning findings in after-pass A4 (749).
- **One body rasteriser (796).** Mesocosm still draws bodies through
  isometer's scene with `isometer-render`'s unlit `LiveBody`
  (`shared/isometer/crates/isometer-render/src/live_body.rs`); it moves
  onto the tenant, and `LiveBody` retires once it has. L5's
  done-conditions carry onto the tenant, and L10's R0 row draws through it
  from then.
- **L7's open done-conditions** are the after-pass's (its A3): the S0
  replay hash unchanged, rg3c and `d1_depth`'s witness rewritten against the
  tenant's report, and L3 and L5 re-proved without the specimen bench lanes
  the switch deleted (Benches A to D ran on legacy Mesocosm; their receipts
  stand as history, and W4 lifts the bench).
- **Carried from the isometer family plan (archived 2026-10-10):** the
  terrain's per-axis scale, y and z as well as x (ruling 23), is unbuilt;
  isometer's only scale is one isotropic float per body placement.
- **L6 and L8** are open as written. **L10** is unbuilt; its done-condition
  8 now also carries the folded views founding plan's minimap receipts.

## Progress entries


- **2026-10-10, Q1:** current state/status and native source paths verified
  under 793 to 802; dated receipts preserved. This documentation pass adds
  no compile, test, draw or headed certification.

## Repaired references

Lines kept in the live plan whose pointer was repaired because the text they pointed at moved out. The line below each note is the original.

- Line in L1, repaired because it pointed at the status addendum that moved here.

  > genet T1 landed (`d61978378c1`, 2026-09-15; see the status addendum above).

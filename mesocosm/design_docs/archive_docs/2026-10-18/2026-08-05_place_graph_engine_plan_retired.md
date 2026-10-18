# The place-graph engine plan: what moved out on 2026-10-18

**Why this moved.** On 2026-10-18 Mark made the old numbered rulings cases, kept as precedent, with new rules numbered by him alone, and asked every active doc to lose its stacked status banners, old progress entries, stale counters and pasted copies. This file holds, verbatim, what left the live [place-graph engine plan](../../2026-08-05_place_graph_engine_plan.md): its eleven stacked status paragraphs and the Progress entries older than the five that describe the open work.

## Status banners

**Status, 2026-10-10:** SP1 to SP5 are built, SP4 and SP5 in
`isometer-space` on `Ground` (733, 737 to 744), and merged with 732's
push; their re-certification, the lift's bytes having moved with drawn
cliffs (744), is the after-pass's. SP6 waits on mere's T2. SP7's joins are
now the overlays': Mesocosm's full M4 in-site, Eponym's E3, and the VTT's
battlemap as a lifted site with the sim on (799). SP8 is open. §A.13 says
who owns what now, superseding §A.3 and §A.4's rows where they differ.
*(Earlier status lines follow as written.)*

**Status, 2026-10-09, later:** rulings 737 to 741 built on
`lane-places-2`, with SP5's border clause (cross-site passages and span
classes). SP5's done-condition holds within and across sites. The open
questions are listed under SP5's second annotation. *(Earlier status
lines follow as written.)*

**Status, 2026-10-09:** under ruling 733, SP4 and SP5 are built in
isometer on `Ground` (`shared/isometer/crates/isometer-space`, branch
`lane-places`), with the lift moved there (701), routing over SP5 (702)
and soil split out of the legacy places module (703). Open forks are
listed under SP5's annotation below. *(The earlier status lines follow
as written.)*

**Status, 2026-09-30:** SP4 and SP5 designed (412 to 422); SP4 next. *(Brought current 2026-10-06 under ruling 618; the
earlier status line follows as written.)*

*Earlier:* **Status, 2026-09-28:** rewritten to the wing design record as the spatial
spine's plan (rulings 389 to 392). SP0 is open, with four decisions in §A.6
for Mark. No code has moved. The first slice is new Isocosm capability and
touches neither `mesocosm-core`'s places family (ruling 195) nor mere
(ruling 363).

**Status, later on 2026-09-28:** SP0 is done: the four decisions were ruled
(393 to 396), terrain models in Isocosm, the skeleton as condition keys,
square sites first and a chunked lift. SP1 is next.

**Status, 2026-09-28, SP1 landed** at `6f25a89`: 121 tests pass, three
controls fail as they must, and 256 maps drawn from an unselected master
seed pass every check
([receipt](../testing/bench/receipts/2026-09-28/isocosm/SP1_MAP_DRAWS.md)).
SP2, the lift, is next.

**Status, 2026-09-28, SP2 landed** at `38ea90f`: 133 tests pass, four
controls fail as they must, and 64 worlds drawn from an unselected master
seed meet exactly at every border point, give their elevations back as exact
means against a brute-force sum of 195,970,560 columns, and lift the same
bytes twice
([receipt](../testing/bench/receipts/2026-09-28/isocosm/SP2_LIFT_DRAWS.md)).
SP3, the bench adapter into isometer, is next.

**Status, 2026-09-28, SP3 opened:** Mark approved the first integration
batch with "Ok. Let's proceed. Orchestrate away." SP3 implements the
settled handoff in §A.10; body-to-scene mapping and Mere ownership-plan
reconciliation proceed as documentation lanes alongside it. Neither lane
opens body absorption, generic physics bindings, or T2 implementation.
The source baseline is `e32890e`.

**Status, 2026-09-29, SP3 verified:** the retained bench's World terrain
view draws a base-grain border and a coarse two-site overview through the
existing isometer scene. Five focused tests, the locked package all-target
check and both native scenarios pass. The perturbed source visibly breaks
the border; restoring it restores the viewport exactly. The trial boundary
stays suspended while terrain is open and completes after returning.
[Receipt and captures](../testing/bench/receipts/2026-09-29/spine/SP3_TERRAIN.md).
SP4's asserted edits are next on the spatial chain; body absorption and
shared-store implementation retain their separate gates.

**Status, 2026-09-30:** SP4 and SP5 designed with Mark, rulings 412 to 422
(§A.11 and §A.12): earth is matter, edits are shape operations stored once
and read across borders, lifts carry caves as exceptions, spoil heaps beside
the cut, and places are walkable patches at the base grain with clearance on
their edges. SP4 is next to build.

## Progress entries

- **2026-10-10, Q1:** current state/status and native source paths verified
  under 793 to 802; dated receipts preserved. This documentation pass adds
  no compile, test, draw or headed certification.

- 2026-09-28: SP0 done, §A.6's four decisions ruled (393 to 396); §A.2's
  skeleton and §A.3's model owner updated to match.
- 2026-09-28: rewritten as the spatial spine's plan (rulings 389 to 392):
  §A added; §0 annotated with a reading of what stands; §1 marked superseded
  where §A.2 disagrees; the G0 to G4 history, the burrow run, the Findings
  and the Progress kept. SP0 opened with four decisions.
- 2026-09-01: reconciled the substrate record with the scale and playable
  ecology plans. The current Near/Far flag selects execution detail over an
  individual roster; future individual/cohort materialization is now stated
  as a gated target. Macro trophic state remains derived, and simulation
  fidelity is explicitly independent of per-view render residency.
- 2026-08-05: plan founded from the engine rumination. Fundamentals
  rulings recorded in §0 and amended into the founding record, `CLAUDE.md`,
  and the landscape doc the same session. V1 and V2 landed 2026-08-04
  (landscape §8.6), so G2 rides a proven browser harness and real bodies.
- 2026-08-06, later: renderling tenant proven (device unity, leak fix,
  95/95, wing-shaped scene; see Findings). **G0 constructor landed**:
  `Places::grown` with full distinctness, congruence, and connectivity
  receipts; genesis adoption deferred to its owner. Worlds now differ.
- 2026-08-06, later still: **G1 container landed** (`Ground`): brick
  truth raised from the relief, roofed burrows under nests, carve with
  revision + dirty discipline, occupancy and sight queries, and the
  mesh-projection receipt through `mesh_volume`. World wiring (carve
  intent, snapshot hash, genesis) rides the same deferred swap as G0.
- 2026-08-06, end of day: **G2 probe receipt** (tracer: ~1900/~1420 fps
  Vulkan/GL, pixel-identical, lifecycle in pixels) and **G3 core landed**
  (near-tier movement, tiers with hysteresis, the Hunter, the chase
  receipt on real ground, 300 hunters at ~62µs/tick). All four gates now
  have landed cores or probe receipts; G4's burrow run is composition
  work plus the deferred world adoption.
- 2026-08-06, **G2 tracer probe receipt** (scratchpad `rlprobe`, bin
  `tracer`). Fragment-only brick-map DDA over the real seed-4242
  `Ground`: pointers in an R32Uint 3D texture (16×4×16), materials in an
  R8Uint 3D atlas (128²×64, 383 bricks), one vec4-packed uniform, no
  storage buffers, no compute, device requested at
  `downlevel_webgl2_defaults`. Numbers, reported as found: **~1900 fps on
  Vulkan, ~1420 fps on wgpu-GL at 960×540** with a 420-step budget, and
  the two backends render **pixel-identical** captures (1 px in 518,400
  beyond tolerance 2). The edit lifecycle ran in pixels: a 72-voxel bore
  carved in ~18µs, 8 dirty bricks re-uploaded (4,096 bytes) in ~10ms
  (first-call overhead included), interior then rendered with a rock
  ceiling and correct occlusion out the opening. Findings along the way:
  hit-distance-in-alpha washes PNG receipts (force opaque on capture);
  nest chambers are too small to photograph from inside, so the interior
  receipt is a carved bore, which is the better lifecycle demo anyway.
  Remaining for G2 proper: port into the lens's retained pattern
  (`MapRevision`-style uploads), grade/souls pass, SDF body compositing,
  netrender frame entry, browser receipt.
- 2026-08-07: bonsai reading adopted as three directions (relief lab,
  presentation-amplification tier, sim/render LOD unification); donor row
  and caveat in landscape §8.3.
- **2026-08-08: G0/G1 world adoption landed.** Genesis grows
  (`Places::grown(seed ^ PLACE_SALT, ...)` reuses the exact site-draw
  sequence the old scatter consumed, so the partition is bit-identical
  and only links, relief, and ground are new); `World` owns a serialized
  `Ground` inside the replay hash; `Intent::Carve` is an ordered intent
  with anatomy reach legality, an `Outcome::Carved { at, removed }`, and
  an `Event::Carved` in history (carving air is not an event). Receipts:
  carve replays to identical hashes on a twin, survives snapshot,
  refuses beyond reach; the full core suite is green and strict clippy
  clean. Finding: grown links are better-connected than the lattice
  (the shipped 3x3 enclosure's diameter is exactly 2, the demote
  threshold), so the tier receipt now finds a maximally distant pair
  rather than assuming corners; a larger PLACE_SIDE would widen the far
  tier and is a world-size question, flagged, not silently retuned.
- 2026-08-14: G3 world ingress and autonomous embodiment landed, followed by
  the grounded 300-founder scale receipt. Remaining on the chain: G2
  integration and G4 composition.
- 2026-08-14: **G2 complete.** The retained Ground-to-DDA core now composes
  nearer SDF bodies, enters netrender's external-texture seam, renders headed
  through Browser WebGPU, passes wgpu-GL under the WebGL2-class limit profile,
  and carries synchronized 1080p Vulkan and GL tracer measurements. G4 is the
  remaining composed game-pressure gate.

# The place graph and its world store

**Status:** 2026-10-30: cut on 2026-10-28 from `2026-08-05_place_graph_engine_plan.md` and `2026-08-14_resident_views_composition_plan.md`, which are archived in
`archive_docs/2026-10-28/`. SP1 to SP5 are built: the site map and skeleton in `shared/cosmograph/src/map/`, the lift, edits and places over `Ground` in `shared/isometer/crates/isometer-space/`. Open: their re-certification (the lift's bytes moved when cliffs were drawn), SP6 (nisus under `Ground`, waiting on mere's T2), SP8 (the region), the resident-surface obligations, passage survival at coarse levels, the VRAM residency policy, the standing field-admission gate and PG7 (Places from the column rule and its exceptions, the cost `sim-execution` D30 names).

**For:** the spatial spine from world map to voxels and the resident planes the products draw from it: the lift at power-of-two cell sizes, edits as asserted facts, places and passages derived from the volume, `Ground` and its move to nisus, the region, and what a field must declare before it becomes a resident plane. Phases keep their SP labels; new ones are PG.

Not here: SP7, the absorption join of Mesocosm's in-site play, Eponym's scope crossing sites and the VTT's battlemap as a lifted site, is `mesocosm-overlay M4` (with `eponym-overlay E3` and `vtt-overlay V2` as the other two products' share). The tier line (`TierLine`) and the crowd are `sim-execution E4` and `sim-execution E5`; every speed or frame bar is `sim-execution`. Sensor-part positions, oriented collision, facing and gait (the interaction-expansion follow-ons) are `bodies`. The L10 detail ladder and its ring set are `orthographic-voxel-presentation L10`; this plan owns the lift they call.

## Decisions

- **D1, many projections of one store (2026-10-28), applies 18:** Mark: "Sounds fine to me. If we can do that, sounds fine to me. Makes a lot of sense to me. And they don't have to be like projections in like a destructive way, you know, or projections in like a, you only get one projection per thing." And: "I think that different creatures would see the world differently, right? And so it makes sense to be able to analyze the world according to these different dimensions." Top-down, side, 2:1 dimetric, 3D and slices along any axis are reads of the one quantized store; a read never writes back, and a thing may be read through as many projections at once as anything needs, a creature's senses included.

  Links: area World generation · draws on 18, presentation.1 · summary 2D, 2.5D and 3D views are non-destructive projections of one store, as many per thing as needed, a creature's senses among them.

## Constraints

- Done-conditions are seeded draws from a declared space, never fixtures (ruling 15), and each check carries a control that fails when the property it guards is deliberately broken (old plan §A.5).
- No second world representation inside any consumer; the sim and isometer share one store. No CPU whole-plane reads in the per-tick path (durability reads are deltas, hashes, checkpoints, reductions). Candidate planes never feed collision, persistence or peers, only committed revisions. A view is promoted to a shared contract only after multiple real consumers. A field is admitted by a named simulation or presentation consumer: "the existence of a resident plane type is not sufficient demand." A lower-dimensional or network phenomenon keeps its honest domain. Render mips, clipmaps and residency windows remain views. (Resident-views stop rules.)
- An edit is stored once; a neighbour reads it through the border's frame relation. Edge profiles are derived and never stored, and an edit on a border is an ordinary edit (414; 391's reading). Every edit conserves matter exactly (412).
- Edits are carves or fills over a sphere, box or route in base units in the frame of the site they were made in, asserted facts in the sim's state (413, 696). The sim derives places at a grain the world declares, never at a view's cell size (reading, not ruled).
- Isometer depends on no sim crate and the sim on no render crate; the adapter from the lift to `Ground` lives in the host that holds both (reading, not ruled). Soil stays the sim's, keyed by isometer's columns (703).

## Phases

### SP4/SP5. Re-certification
The lift's bytes moved with drawn cliffs (744) after the merge, so SP4 (edits) and SP5 (places over the bricks) are re-certified against the lift as it stands. This is the old plan's "Next".
- Done when: SP4: "seeded edit sequences replay to identical bytes, a site re-lifted with its edits equals the edited site, an edit on a border is seen from both sides, and stored bytes grow with edits and not with sites." SP5: "local re-derivation equals full re-derivation after every edit of seeded sequences, a route whose profile stops at a cliff or water reads impassable in the volume, and one that passes reads passable; the control, a stale dirty region, disagrees and is caught."
- Needs: -
- Rulings: 15, 412 to 416, 417 to 422, 733, 744
- From: `archive_docs/2026-10-28/2026-08-05_place_graph_engine_plan.md` status line 11 and §A.5, lines 159-163 and 197-229.

### SP6. The store moves to nisus
After mere's T2 (363), baseline chunks land in nisus's world store and edits pass through its revision log (330, 331); `Ground` thins or retires as 330 allows. T2's lane (335) was not written when last read, and production Burn is still a pre-release. Mark, 2026-10-22: "Owe nisus in a plan." This phase is that obligation; the Resident-views plan's "SP6's nisus join" is the same item.
- Done when: "T2's own done-conditions hold and SP2 to SP5's draws pass again on nisus, with saves keeping their hashes or converting by a recorded event, and each navigation result carries the world revision it read (333; doctrine, Stack 5 and 6)."
- Needs: mere's T2 (ruling 363, migration pre.4); SP4/SP5
- Rulings: 330, 331, 333, 335, 363, 697, 700
- From: `archive_docs/2026-10-28/2026-08-05_place_graph_engine_plan.md` §A.5 SP6, lines 279-285 and 152-153; `archive_docs/2026-10-28/2026-08-14_resident_views_composition_plan.md` lines 7 and 16.

### SP8. The region
Ruling 124's edge: a drawn region of hundreds of sites lifted where attention is.
- Done when: "a drawn region of hundreds of sites lifted where attention is, with memory bounded by the loaded window and not by the world, measured and receipted."
- Needs: SP4/SP5 (chunked lift and edit over many sites); `sim-execution E5` shares the rule that near is attention roots plus one hop
- Rulings: 124
- From: `archive_docs/2026-10-28/2026-08-05_place_graph_engine_plan.md` §A.5 SP8, lines 302-304.

### PG1. Eponym's resident-surface obligations
Eponym's one-budget, fixed atlas, host-limit and terrain-occlusion work remains live after the highest-solid query (`Ground::surface`). The far page asks for more full-height bricks than the atlas holds (a working-set selection issue in the product).
- Done when: not stated; owed.
- Needs: `eponym-overlay E3` (the product's real-time zoom and far-page budget)
- From: `archive_docs/2026-10-28/2026-08-05_place_graph_engine_plan.md` status line 11 and Progress 2026-10-10, lines 1311-1328.

### PG2. A capture of a border that crosses water
Ruling 408's point, where water lies and whether it meets across a border, is shown only by a unit test: the captured seed's two sites lie above their water level.
- Done when: not stated; owed (a native capture of a drawn border that crosses water, with water meeting across it).
- Needs: the SP3 bench
- Rulings: 408
- From: `archive_docs/2026-10-28/2026-08-05_place_graph_engine_plan.md` lines 451-454.

### PG3. The voxel material `world:earth`
Rename the voxel material seeded as `world:soil` to `world:earth`, so `world:soil` stays the edible pool of the ecology (412, withdrawing 403's naming).
- Done when: not stated; owed.
- Needs: -
- Rulings: 403, 412
- From: `archive_docs/2026-10-28/2026-08-05_place_graph_engine_plan.md` lines 399-404 and 466-468.

### PG4. Passage survival at coarse levels
A coarse lift point-samples each coarse cell's origin: `chunk()` in `lift/chunk.rs` reads each coarse column's surface at the cell origin only, and the edit replay in `edit/replay.rs` keeps only cells whose coordinates are multiples of the step. A carve, door, burrow or water channel narrower than the cell can vanish at a coarse ring. No test in `lift/tests.rs` or `edit/tests.rs` covers passages, and none of L10's done-conditions checks that a passage survives. The direction stated when this was raised: a coarse cell should stay open if any fine cell in it is carved, and carry the fine clearance. Mark doubts it ("sounds like full voxel territory"); the recommendation in Questions keeps coarse cells to a derived "opening here" bit.
- Done when: not stated; owed. The form the harvest suggested: a seeded-edit control where a one-base-cell passage is present at base grain, its survival or absence at each ring level is stated and checked, and a control fails when origin-sampling drops it.
- Needs: `orthographic-voxel-presentation L10` (its rings); SP2's `lift(site, level, at)`; the places derived at ring levels
- Rulings: 425, 296
- From: `archive_docs/2026-10-28/2026-09-11_orthographic_voxel_presentation_plan.md` lines 2394-2399 and 2436-2467; `shared/isometer/crates/isometer-space/src/lift/chunk.rs:91-113`, `edit/replay.rs:99-129`.

### PG5. VRAM residency policy
"The remaining genuinely new system": which chunks live resident, paging, the budget. CubeCL's allocator is the pool; the policy above it is ours. It overlaps L10's ring set and its brick and byte budget.
- Done when: not stated; owed.
- Needs: `orthographic-voxel-presentation L10`
- Rulings: 291, 296
- From: `archive_docs/2026-10-28/2026-08-14_resident_views_composition_plan.md` lines 710-714.

### PG6. Field admission, a standing gate
Before any product field enters `ResidentChunk`, its card names authority class, consumer, domain, resolution, cadence, interpolation, sources and sinks, boundary conditions, numeric range, conservation rule, scale reduction and acceptance evidence. Every extra channel gets a measured resident-byte and update-cost receipt. Resident mips, clipmaps and windows are views; an authoritative field-resolution change needs its own restriction, refinement, conservation, zero-tick round-trip and comparison-envelope contract. A 2D or network consumer does not gain a dense 3D plane merely because the carrier can express one. Open under it: field-plane visualization; the Lane A condition "no CPU whole-plane read exists in the path" is untested, so add a counter (counted upload bytes, asserted zero, as Lane F does) or strike the condition; and the sun field's tail (landed): the retained-receipt matrix with replay and conservation checks was not rerun, terrain tops shift hue from grey to blue at low light under the grade's ladder and the lane flagged this for a look, and the credit's limiting amount ("bring the limiting amount back as a tuning fork (888) before the credit changes") was measured over a few ticks only, so the per-day draw is unmeasured. Whether the tuning ruling is recorded was not checked.
- Done when: standing gate, none stated for the gate itself. For the audit gap: "Add that counter or strike the condition." For the sun field: "the four captures are inspected, the CPU control and the compile gates pass, the A1 gate holds and the record carries the tuning ruling."
- Needs: the wing design record (tuning ruling, 888)
- Rulings: 879 (energy is not matter), 971 to 974, 970, 979, 982 to 984, 888, 472
- From: `archive_docs/2026-10-28/2026-08-14_resident_views_composition_plan.md` lines 30-39, 140-176, 178-236, 326-330, 523-526, 877-901.

### PG7. Places from the column rule and its exceptions
`sim-execution` D30 ("Places cache, then far off-path") found Places derivation among the head's largest costs, and D32 placed it: first visits and LRU eviction across a one-site world's windows (`mesocosm-runtime/src/spatial/geometry_cache.rs`), not terrain change. Every derivation realizes a window's `Volume` and seeds the flood from every column, floor to top (`Places::derive`, `Cells::everywhere` in `isometer-space/src/places/`). The lift already says most of it without voxels: under the column rule air lies only above a column's surface, so a room needs an exception (an air exception below the surface, or rule air that a solid exception walls in); outdoor patches and water read each column's surface from the rule; a sight ray over open ground is blocked only where it falls below a column's top. Mark, 2026-10-30, of where the fix lives: "I'll take your recommendation. That's probably fine." The recommendation was here, since Places and their derivation are this plan's (SP5), with `sim-execution` measuring the result. The direction (from the 2026-10-28 discussion, not ruled): derive Places from the column rule and the exceptions, seed rooms only at exceptions, and realize voxels only where exceptions or a body's contact need them, so a window without edits costs a pass over its columns.
- Done when: on seeded edit sequences, Places derived from the rule and exceptions equal `Places::derive` over the realized volume (rooms, water, patches, passages), with a control that drops the seeds beside solid exceptions and misses a walled-in room; a window with no exceptions derives without realizing a `Volume`; the head's share of a played tick spent deriving Places is measured before and after by `sim-execution`.
- Needs: SP4/SP5 (re-certification)
- Rulings: 412, 415, 417 to 422; `sim-execution` D30, D32
- From: `sim-execution` D30, D32 and Findings 2026-10-28; Mark, 2026-10-30.

## Deferred and parked
- **A view for slicing the world by its projections.** Mark, 2026-10-28, of the scratch terrain-ladder widget: "in terms of like slicing up the map in a lot of different ways. I think that was cool", and "I'd be okay with that being like a Metacosmos, you know, GUI. I mean, obviously we have more work to do before we can make decisions like that." Not a phase and not a name; it waits on D1's projections existing. From: Mark, 2026-10-28.
- **Overflow (416)**: once capacity exists, what a carver cannot hold is filled back as a heap beside the cut, an automatic fill edit placed from the edit's seed; until then the carver holds all of it. Waits on a world capacity, which no plan defines. From: `archive_docs/2026-10-28/2026-08-05_place_graph_engine_plan.md` lines 213-216 and 482-484; rulings 412, 416.
- **Skeleton founding in isoscape**: the founding pipeline starts in Cosmograph's generator and moves to isoscape when isoscape is founded (392); Cosmograph's default base unit of 1,000 µm is a placeholder no world has been founded against. Waits on isoscape's founding. From: `archive_docs/2026-10-28/2026-08-05_place_graph_engine_plan.md` lines 343-345 and 157; ruling 392.
- **Relief lab and presentation-amplification tier**: two directions adopted 2026-08-07 as instruments, "not a gate": live seed and parameter twiddling over worldgen re-rendered through the tracer; GPU micro-detail that never enters the replay hash (enhanced profile only). Cues only; relief generation has since been replaced by the lift. From: `archive_docs/2026-10-28/2026-08-05_place_graph_engine_plan.md` lines 1205-1232.
- **Multi-plane patches**: `commit_plane_patch` refuses multi-plane bundles; a future form must validate and publish one batch (a bundle-wide stamp advanced after changing one plane would falsely mark untouched planes current). Waits on the first multi-plane consumer. From: `archive_docs/2026-10-28/2026-08-14_resident_views_composition_plan.md` lines 268-272.
- **Shared lease type extraction** (`ResidentChunk` / `SpatialLease` / `LeasedAtlas`): deliberately not done; "the trigger for it is a second shipped consumer". Cross-product promotion of voxel authority, identity, resident lease or frame contract is not claimed. From: `archive_docs/2026-10-28/2026-08-14_resident_views_composition_plan.md` lines 454-463, 798-800, 866-867.
- **GPU radix sort / LBVH harvest into conatus**: "a separate decision after receipts" (traversal and force accumulation are unmeasured, the harvest is hand-written WGSL); the Lane C verdict was to consolidate the explicit lane on CubeCL-JIT, and whether that was carried out is not visible. Waits on mere's conatus engine plan. From: `archive_docs/2026-10-28/2026-08-14_resident_views_composition_plan.md` lines 334-361 and 527-558.
- **Nexus rigid-body kernels**: adoption "at the bodies gate", superseded in shape 2026-08-28 (a Conatus backend behind `BodyWorld`, not a stack-level dependency); adoption-time gates are khal's wgpu row membership and nexus's feature requirements against the greedy tenant. `hecs` is approved only as a gameplay runtime index, with a concrete consumer, never a second world representation. Waits on a bodies gate that is not scheduled. From: `archive_docs/2026-10-28/2026-08-14_resident_views_composition_plan.md` lines 622-657, 775-780.
- **Browser target for the resident-ground receipt**: Mesocosm would need to restate Mere's temporary `cubecl-runtime` manifest patch (Cargo patches are workspace-root scoped); the gate is native today. From: `archive_docs/2026-10-28/2026-08-14_resident_views_composition_plan.md` lines 491-495.

## Questions for Mark
- **Which air is "cut off from the sky" (417)**: no air path to the sky, or solid anywhere above. Both are built and a caller must name one. Related reading, not ruled: the sim never sees a member move within its site, so its game updates `patch` through `Command::Patch` as it moves it. From: `archive_docs/2026-10-28/2026-08-05_place_graph_engine_plan.md` lines 238-244 and 276-278; rulings 417, 737, 740.
- **Passage rule at coarse levels**: Mark, 2026-10-28: "I don't know. Maybe not. I don't know. It kind of sounds like full voxel territory to me." Recommendation (not ruled): nothing physical reads a coarse lift (already §A.2.6: a coarse lift for a far view is presentation and never feeds a derivation), so fit, passage and sight for the sim always read base grain and its sparse exceptions, which are cheap to scan directly. A coarse cell keeps no fine geometry and no clearance; that would be voxel detail climbing the levels. It may carry one derived bit, "an opening is here", set when any exception falls inside it, so a far view or a coarsely sensing creature can show or notice that something is there without pretending to draw it. PG4 then checks only that the bit survives. From: `archive_docs/2026-10-28/2026-09-11_orthographic_voxel_presentation_plan.md` lines 2436-2467; Mark, 2026-10-28.

## Findings
- 2026-10-28: the border clause of SP5 is built: `places::join` builds passages across a border through the frame relation, with the cliff spans drawn (743, 744). The old SP5 annotation that it was "still open" is history.
- 2026-10-28: SP4 and SP5 live in `isometer-space` on `Ground` and do not wait for T2; only SP6 does. The old A.5 annotation of 2026-10-08 ("SP4 waits for mere's T2") is stale (733).
- 2026-10-28: `default_materials()` in `shared/cosmograph/src/map/mod.rs` still seeds `["world:air","world:water","world:soil","world:rock"]` and its comment cites 403, so ruling 412's rename has not landed (PG3 open).
- 2026-10-28: `Ground::surface` exists in `isometer-core/src/ground.rs` and Eponym's `eponym-sortie`, `eponym-motion` and `eponym-play` call it, so the old "Q5 integration remains pending" is done; the budget, atlas, host-limit and occlusion obligations remain (PG1).
- 2026-10-28: the lift files are `check, chunk, cliff, corners, lattice, mod, profile, tests` in `isometer-space/src/lift/`, edits are `mod, replay, shape, tests`; no test names passages or narrower-than-cell survival (PG4).
- 2026-10-28: A.3 and A.4 rows naming `mesocosm-core/src/places.rs` and `eponym-world/src/sites.rs` describe deleted code; §A.13 of the old plan holds the current owners. Eponym's site meanings are in `eponym-play/src/sites.rs`. The old plan's renderling findings are void (renderling is retired).
- 2026-10-28: the `TierLine` lives in `shared/cosmograph/src/directing/tier.rs`; its wiring is `sim-execution E5`.
- 2026-10-28: the 2026-10-19 narrow Mesocosm connection (generated whole-body stands, walking and biological child placement consuming `Ground` and Places through the native spatial clock) is a bounded SP7 slice; full in-site play is `mesocosm-overlay M4`.

## Progress
- 2026-10-30: PG7 added, the Places cost `sim-execution` D30 named.
- 2026-10-28: cut from `2026-08-05_place_graph_engine_plan.md` and `2026-08-14_resident_views_composition_plan.md`.

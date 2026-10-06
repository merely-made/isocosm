# Place-Graph Engine Plan (2026-08-05): the spatial spine

**Status, 2026-09-28:** rewritten to the wing design record as the spatial
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

**Rewritten to the record, 2026-09-28 (ruling 390).** W1 (2026-09-18) gave
this plan a rewrite verdict: the sim's spatial half, on which W2 could not
be founded while it stood undecided. The one-game assessment of 2026-09-28
([session notes §9](archive_docs/2026-10-06/2026-09-22_sim_design_session_notes.md#9-the-one-game-hypothesis-session-2026-09-28))
ranked connecting the sim's site graph to the voxel world first among the
gaps, and Mark chose to start there. §A is the spine: what connects the
world map's sites to the voxels under them, who owns each piece, what
exists, the phases and the decisions still open. Everything from §0 onward
is the plan as it stood on 2026-09-01, kept as history: its rulings of
2026-08-05 carry a dated reading of what stands, §1's model gives way to
§A.2 where they disagree, and the G0 to G4 build history, the burrow run,
the Findings and the Progress stand as the receipts they were.

*As of 2026-09-01:* **Status: active substrate record, clarified 2026-09-01.** Founded from the
2026-08-04/05 engine rumination. The G labels below record the order in which
the substrate was assembled. They are historical indexing, not current
acceptance gates. Sibling to the
[render lane landscape](archive_docs/2026-09-18/2026-07-30_engine_and_render_lane_landscape.md),
which owns renderer research and the V-gates, and subordinate to the
[execution waves plan](archive_docs/2026-09-18/2026-07-31_execution_waves_plan.md) for ordering.
This plan owns the world substrate: the place graph, volumetric truth, the
two-scale execution substrate, and the composed slice that proves them
together.

*W1, 2026-09-18:* **rewrite.** Tier: sim, §3.7. The sim's spatial half; W2
cannot be founded on an undecided answer here. Rewrite is a lane under the
record's W2 or W3; until it lands this plan's done-conditions are not
authoritative. Evaluated against the wing design record; see
[2026-09-18_wing_plan_evaluations.md](2026-09-18_wing_plan_evaluations.md)
§2.

## A. The spatial spine (2026-09-28)

### A.1 What it rests on

Rulings of the [wing design record](2026-09-18_wing_design_plan.md), quoted
there in full:

| Ruling | What it fixes for the spine |
| --- | --- |
| §1, the derivation rule | a world is a seed, its rules and its asserted facts; a site's volume is derived and only its edits are stored |
| 12 | the near rung is a volume with an interior, destructible and constructible, for every game |
| 13 | cells are power-of-two multiples of one base unit, chosen per chunk; bricks stay 8³ |
| 14, §3.7 | places above the bricks are derived from them, re-derived locally from dirty regions, and never disagree with them |
| 18 | a game's grid is a projection over the cubic volume |
| 23 | the terrain gets a per-axis scale, owed under W3 |
| 69, 70 | what is of note persists; the ambient regenerates from the same basic facts |
| 72, 73 | the world map is a graph of sites in the world's one shape, held as adjacency |
| 74, 88, D15 | nesting is one step with a down, an up, an across and a ratio, allocated as locations are generated |
| 92 | scopes are a game's, never the sim's |
| 124 | the first scale target is a region |
| 147, 148 | a site and a paged chunk are independent; a location's kind decides what it follows when its ground moves |
| 195 | `mesocosm-core`'s places family moves fourth into Isocosm |
| 330 to 333, 363 | nisus becomes the voxel store with a revision log; T2 builds it after the pre.4 migration |
| 389 to 392 | this rewrite: designed now and sliced around 195 and 363, this plan as its home, edge profiles, Isocosm lifts |

### A.2 The model

Far to near. Each layer is derived from the one above it unless it is
marked asserted.

1. **The world map** (asserted at founding). Sites and their adjacency in
   the world's one shape: Isocosm's `Site` and `Route`
   (`shared/isocosm/src/schema.rs:98-112`). Each adjacency also records how
   the two sites' frames meet: which side of one touches which side of the
   other, and in what orientation.
2. **The skeleton** (asserted at founding, top-down, the 2026-08-05 ruling
   7). Each site's coarse terrain facts, from which its volume is derived:
   at least an elevation, a relief amplitude and a water level, in base
   units, held as keyed entries in `Site.conditions` that the world's rules
   name (ruling 394). Biome stays a reading over
   conditions (ruling 72). The site's `terrain_seed`, drawn today and read
   nowhere (`shared/isocosm/src/generate.rs:275`), becomes the seed of its
   interior detail.
3. **Edge profiles** (derived, never stored, ruling 391). For each
   adjacency, a profile of the shared border: surface heights sampled along
   the edge, and whether each span passes, climbs or stops at a cliff or
   water, drawn from both sites' skeletons and a seed keyed by the unordered
   pair. Both sides compute the same bytes, the far side reading them
   mirrored into its own frame.
4. **The lift** (derived, the nesting step's down). Given a site, a chunk
   and a cell size, the chunk's baseline voxels: a surface interpolated from
   the site's skeleton towards each edge's profile; detail from the site's
   seed that fades to nothing at the edges, so every border equals its
   profile exactly; materials by depth and conditions; water under its
   level; cavities as data. A pure function: the same inputs give the same
   bytes on any machine, and an unvisited site allocates nothing.
5. **Edits** (asserted). Carving, filling and building are facts in the
   sim's record, keyed by site and cell. A site's volume is its lift with its
   edits replayed, so stored bytes are proportional to edits. Until T2
   lands, Isocosm keeps the edits and re-lifts; after it, nisus's world store
   and revision log hold them (rulings 330 and 331).
6. **Places over the bricks** (derived, ruling 14). Connected components of
   air at the world's declared grain, passages as edges and travel cost as
   weights, re-derived locally where edits dirtied the volume. Cross-site
   passages join through edge profiles, so the graph over the bricks spans
   sites. The sim reads the volume only at its declared grain: a coarse lift
   for a far view is presentation and never feeds a derivation (§5's third
   stop rule).
7. **Up** (restriction). A site's conditions summarise its volume where
   edits or processes have moved it from the baseline. An unedited site's
   conditions are its skeleton's, so restricting a just-lifted site returns
   what it came from by construction: a consistency check, never evidence
   about evolution (ruling 113).
8. **Across.** Leaving a site by an edge arrives in the neighbour's frame by
   the edge's recorded frame relation. The shared profile makes the ground
   meet; the frame change is arithmetic and never a scene transition (§5's
   first stop rule).
9. **Nested maps** (rulings 74 and 88). A location opening into a finer map,
   a dungeon's floors or a ship's decks, uses the same step: its own frame, a
   profile at its opening and its own lift, allocated when the location is
   generated.
10. **Consumers.** isometer draws a lifted site through `Ground` today, fed
    by an adapter, and through nisus's store after T2; conatus's colliders
    follow the store through T2; the sim's processes run over the places.
    The VTT's authored boards are asserted volumes displacing a site's lift
    (ruling 89); Mesocosm's enclosure is one site; Eponym's continuous scope
    crosses sites.

### A.3 Who owns what

| Piece | Owner | Ruling |
| --- | --- | --- |
| World map, skeleton, edge profiles, lift, edits as facts, places, up and across | Isocosm, `shared/isocosm` | 392 |
| The terrain models the lift calls: relief functions, detail noise, cavity shapes, all verb-free | Isocosm, as verb-free modules; isometer-core keeps the `Terrain` trait and `Ground` | 392, 393, amending the isoscape plan's ruling 11 |
| The skeleton's founding pipeline and presets | isoscape, once founded | 392 |
| The voxel store and revision log | nisus after T2; Isocosm's edit list and re-lift before | 330, 331, 363 |
| Rendering | isometer, through an adapter from the lift's description to `Ground` | 392 |
| Colliders and spatial queries | conatus, through T2 | 330 to 334 |

### A.4 What exists, checked 2026-09-28

At isometry `d80163b` and mere `origin/main` `a31b9a14`:

| Piece | Where | Holds | Gap against §A.2 |
| --- | --- | --- | --- |
| World map | `shared/isocosm/src/generate.rs:260-290` | up to 256 sites in a ring plus one random extra route each; routes with travel time and transmission | no shape, no geometry, no frame relation; `terrain_seed` read nowhere. *2026-09-28:* SP1 adds grid layouts with borders and skeletons (`src/map/`) and corners and edge profiles (`src/terrain/`) |
| Sites and locations | `shared/isocosm/src/schema.rs:98-120` | `Site { terrain_seed, conditions, accounts, routes }` and `Location { sites, claims, parent, ratio }` | no skeleton, no edge profile, no volume |
| Brick container and seam | `shared/isometer/crates/isometer-core/src/ground.rs` | one bounded extent grown column by column from `Terrain`'s surface plus cavities; 8³ bricks; materials to 63; one revision; a dirty queue one reader drains | not chunked by site, no cell sizes, no additive write; the crate depends only on `wing-formats`, `serde` and `postcard` |
| Voxel mechanics | mere `crates/conatus/nisus/src/lib.rs` | 64-bit chunk addressing, a product-owned chunk extent, revision-gated edits, dirty regions | no world store, chunk map or revision log: T2's, after pre.4 (ruling 363) |
| T2's lane | mere's conatus engine plan §2 | §2's general done-condition, one chunk/revision path | the lane ruling 335 placed there is not written; production Burn is still `0.22.0-pre.2`, S13 open |
| Relief and places | `mesocosm-core/src/places.rs` and `places/`, 1,925 lines | a 65×65 diamond-square `Relief` from its own seed, `Grown` implementing `Terrain`, places over a fixed three-by-three partition | one field per enclosure, which cannot tile across sites; moves fourth under 195 |
| Authored terrain | `crates/isometry-views/src/scene/terrain.rs` | `MapTerrain` implementing `Terrain` over the VTT's authored maps | the VTT's boards as asserted volumes over a site (SP7) |
| Eponym's places | `eponym/crates/eponym-world/src/sites.rs` | stable surface and underground slots; a closed five-kind site enum | its words crossed against ruling 72 (the record's §3.7.1) |
| Base unit | `shared/isocosm/src/generate.rs:35` | a default of 1,000 µm | a placeholder no world has been founded against |

### A.5 Phases and done-conditions

Done-conditions are seeded draws from a declared space, never fixtures
(ruling 15), and each check carries a control that fails when the property it
guards is deliberately broken.

- **SP0, the spine ruled.** Done when §A.6's decisions are taken. Opened by
  rulings 389 to 392 on 2026-09-28. **Done 2026-09-28** (rulings 393 to
  396).
- **SP1, the world map with geometry, the skeleton and edge profiles.** A
  generator family draws a world map in a shape with every adjacency's frame
  relation, a skeleton per site, and edge profiles derived from site pairs.
  Done when every new type byte-round-trips, a profile computed from either
  side is the same bytes mirrored, two runs of one seed give identical maps,
  and the draws cover every shape decision 3 names; the control, a profile
  keyed by the ordered pair, fails the symmetry check. **Done 2026-09-28** at
  `6f25a89`: `shared/isocosm/src/map/` lays the grids and validates borders,
  `src/terrain/` walks corners and draws profiles, and the receipt's 256
  unselected draws cover planes, rings and tori.
- **SP2, the lift.** A site's baseline volume as a plain description in its
  frame at a stated cell size, conditioned on its skeleton and its edges'
  profiles. Done when on seeded draws a site lifts to identical bytes twice
  and in a second process, every pair of neighbours agrees exactly on every
  shared border sample, restricting a just-lifted site returns its skeleton,
  and an unvisited site allocates nothing; the control, detail that does not
  fade at the edges, fails the border check. **Done 2026-09-28** at
  `38ea90f`: `src/terrain/lattice.rs` holds the exact surface and
  `src/terrain/lift.rs` the chunks; a second process repeats a lift's
  digest, and lifting leaves the world's digest unchanged.
- **SP3, the bench.** On the specimen bench, Isocosm's first host, an
  adapter implements `Terrain` over the lift, and two neighbouring sites of a
  drawn world render through isometer across their shared border. Done when
  a native capture shows the border with no seam in surface or material and
  the receipt carries the seed, the parameters and the border digests; the
  control, one side lifted from a perturbed skeleton, shows the seam and
  fails the digest. **Done 2026-09-29:** native border/overview and
  perturb/restore pass, with exact source/material checks and a retained
  trial-boundary regression scenario; see the SP3 receipt above.
- **SP4, edits.** Carve and fill as asserted facts over the lift. Done when
  seeded edit sequences replay to identical bytes, a site re-lifted with its
  edits equals the edited site, an edit on a border is seen from both sides,
  and stored bytes grow with edits and not with sites.
- **SP5, places over the bricks.** Connected air, passages and travel cost at
  the declared grain. Done when local re-derivation equals full
  re-derivation after every edit of seeded sequences, a route whose profile
  stops at a cliff or water reads impassable in the volume, and one that
  passes reads passable; the control, a stale dirty region, disagrees and is
  caught.
- **SP6, the store moves to nisus.** After T2 (ruling 363): baseline chunks
  land in nisus's world store and edits pass through its revision log
  (rulings 330 and 331), and `Ground` thins or retires as ruling 330 allows.
  Done when T2's own done-conditions hold and SP2 to SP5's draws pass again
  on nisus, with saves keeping their hashes or converting by a recorded
  event.
- **SP7, the absorption join.** Ruling 195's fourth family:
  `mesocosm-core`'s places move onto the spine, Mesocosm's enclosure becoming
  one site of a drawn world, the VTT's authored boards asserted volumes
  displacing a lift (ruling 89), and Eponym's scope crossing sites. Done
  under the Mesocosm overlay plan's M2 for places.
- **SP8, the region.** Ruling 124's edge: a drawn region of hundreds of
  sites lifted where attention is, with memory bounded by the loaded window
  and not by the world, measured and receipted.

SP1 to SP5 need neither mere nor `mesocosm-core`'s places; SP6 waits on T2
and SP7 on ruling 195's order.

### A.6 Decisions for Mark

1. **Where the terrain models live.** Ruling 392's clauses meet: models
   "beside the Terrain seam" sit in isometer-core, which the no-render-crate
   clause keeps out of the sim's graph, and isometer-core renders nothing.
   Either the models live in Isocosm, amending the isoscape plan's ruling 11,
   or Isocosm may depend on isometer-core, amending ruling 392's clause.
2. **How the skeleton is held.** Keyed entries in `Site.conditions` named by
   the world's rules, typed fields on `Site`, or a wider typed set with
   climate and watersheds now.
3. **The first slice's shapes.** Square sites with four edges on planes,
   rings and tori first, with polygon footprints designed into the profile
   and built second; polygons and a geodesic sphere now; or one site against
   generated borders.
4. **The lift's grain.** Chunks at power-of-two cell sizes from the start, so
   a site of any size costs what is lifted; or whole small sites at one cell
   size first, with chunking arriving with the store at T2.

*All four ruled 2026-09-28:* 1 by ruling 393, the models in Isocosm; 2 by
394, condition keys; 3 by 395, square sites on planes, rings and tori
first; 4 by 396, chunks at power-of-two cell sizes from the start.

### A.7 Readings, not ruled

- The 2026-08-05 ruling 6 holds within a site's frame and gives way between
  sites, where crossing an edge changes frame (ruling 391's reading); §5's
  "no portals" stands as a rule about what a crossing reads as.
- Edge profiles are derived and never stored, and an edit on a border is an
  ordinary edit (ruling 391's reading).
- The adapter from the lift to `Ground` lives in the host that holds both,
  the specimen bench, since isometer depends on no sim crate and the sim on
  no render crate.
- The sim derives places at a grain the world declares, never at a view's
  cell size (§5's third stop rule).
- The skeleton's founding pipeline starts in Isocosm's generator and moves to
  isoscape when isoscape is founded (ruling 392's "isoscape keeps founding
  presets").

### A.8 SP1's brief (2026-09-28, rulings 397 to 399)

- **Types, all optional and skipped when absent, so every existing world,
  receipt and digest is unchanged:** a `Border` on `Route` naming the side it
  leaves by, the side it enters and whether the two meet flipped (397); a
  `Footprint` on the world's traits, its side count and side length in base
  units; the rules naming the skeleton's condition keys (394); and the map
  domain as `Founding.map` (398).
- **Validation:** borders only on a world with a footprint; sides within the
  footprint's count; each side of a site used by at most one route; every
  border's reverse present and matching; skeleton keys declared in the rules
  and present on every site.
- **The generator:** square sites on planes, rings and tori (395), the
  skeleton laid top-down from integer value noise over the grid, and each
  site's `terrain_seed` kept for its interior detail. Its width times its
  height is the site count; a founding whose `sites` disagrees is refused
  (398's reading).
- **The terrain models, in Isocosm (393):** integer value noise from
  `isocosm::draw`; corner classes found by walking borders, each corner's
  height drawn from the sites around it and keyed by the class's least
  member; each edge profile running between its two corner heights, with
  detail that vanishes at both ends, keyed by the unordered pair of border
  records and read reversed from the far side.
- **Receipts:** tests over derived seeds as regression pins, and
  `isocosm-bench --map-draws N` drawing from an unselected master seed and
  saving its receipt (ruling 15), over grids from 2×2 to 16×16 and site sides
  from 256 to 2,048 base units (399).
- **Controls:** a profile keyed by the ordered pair fails the symmetry check;
  a corner height keyed per site fails corner agreement.
- *Readings, not ruled:* corners must be shared for borders to agree exactly
  at their ends; terrain math is integer-only, the sim's encoding rule
  (`shared/isocosm/src/lib.rs`); a span's passability is a reading of its
  slope against the water level and is never stored.

### A.9 SP2's brief (2026-09-28, rulings 400 to 403)

- **The surface** (rulings 400 and 401). Each site's surface is the exact
  bilinear interpolation of a 17 by 17 lattice laid over its footprint at
  spacing `side / 16`. Its boundary rows are the four edge profiles' control
  heights, so every border is its profile exactly and both sides agree.
  Interior points are the Coons patch of that boundary, plus detail from the
  site's seed windowed to vanish at the edges and bounded by its relief,
  plus a correction shaped by the same window that makes the mean surface
  over the site's base-grain columns equal its elevation exactly.
- **The lift** (ruling 402). A chunk is a window of columns, 32 by 32 at
  its cell size of `2^level` base units, keyed by site, level and chunk
  position and clipped at the site's far edges. Each column carries its
  surface top in cells, rounded down from the exact surface; the chunk
  carries the water top and the material-by-depth rule: soil within the soil
  depth of the surface, rock below it, water from the surface up to the
  water level where the surface lies below it, air above.
- **Materials** (ruling 403). A table saved with the world's traits names
  each id's material and the lineage it is nis of; the map seeds it with
  `world:air`, `world:water`, `world:soil` and `world:rock`, all nis of
  `world:ground`. `world:soil` is the same nis the ledger's soil account
  names. *Withdrawn 2026-09-30 by ruling 412:* Mesocosm's TD6 keeps terrain
  and edible soil separate, so the voxel material is earth, to be renamed
  `world:earth`, and `world:soil` stays the edible pool.
- **Checks, over seeded draws, each beside a control:** a site lifts to
  identical bytes twice and in a second process; every pair of neighbours
  gives exactly the same surface along their shared border, and detail that
  does not fade at the edges fails that; a brute-force sum of every
  base-grain column's exact surface equals the elevation times the column
  count, and dropping the correction fails that; detail stays within
  relief; lifting allocates nothing in the world.
- *Readings, not ruled:* the surface is held in units of `1 / (4 s²)` base
  units, `s` the lattice spacing, so the mean's closed form is an integer
  sum and the correction spreads an exact integer over the interior lattice;
  a lift checks that the side is a multiple of 16 and at most 2^16 base
  units, which keeps every sum within 128-bit integers; columns are sampled
  at their origins, a presentation choice the sim never derives from (§5's
  third stop rule); soil depth is an eighth of the site's relief, at least
  one base unit.

### A.10 SP3's handoff (2026-09-28, rulings 408 and 409)

Ruled; implemented and native-verified 2026-09-29 (receipt above).
The accepted handoff, with its implemented adapter:

- **Host.** The specimen bench in `mesocosm-genet`, Isocosm's only
  consumer, which compiles against SP1 and SP2 (checked 2026-09-28). Its
  terrain today is a Mesocosm `World`'s `Ground`: the producer
  (`src/app/bench/producer.rs`) hands `world.ground()` to the section
  (`src/section.rs`), which wraps it as isometer's `GroundTerrain`.
  The bench uses its retained mesquite scenario/capture lane
  (`src/app/bench.rs` and `src/app/bench/probe.rs`). The main host's
  `Host::capture_to` in `src/app/drive.rs` is a separate path; SP3 uses the
  bench lane.
- **The adapter** (reading, not ruled: it lives in the bench, which holds
  both crates). It implements isometer-core's `Terrain` over lifted chunks
  and grows a `Ground` for the scene with `Ground::grow_with`, mapping
  world-local material ids to palette indices, water as an opaque palette
  colour (408). `Ground` grows every column from height zero, so the
  adapter subtracts a local datum, the window's lowest top less the soil
  depth, or tall sites realize thousands of voxels a column.
- **Two captures** (409): a window straddling the shared border of two
  neighbouring sites at the base grain, and both whole sites at a coarse
  level. Reading, not ruled: in the wing's default view,
  `SlabCamera::dimetric_2_1` (rulings 382 and 387). Each receipt carries
  the seed, the parameters and the border digests.
- **Control:** one side lifted from a perturbed skeleton shows the seam and
  fails the digest.
- **Done when** SP3's done-condition in §A.5 holds: a native capture shows
  the border with no seam in surface or material.
- *2026-09-30:* no capture shows water: seed 7's two sites lie above their
  water level, so ruling 408's point, where water lies and whether it meets
  across the border, is shown only by a unit test. A capture of a drawn
  border that crosses water is owed.

### A.11 SP4's brief (2026-09-30, rulings 412 to 416)

- **Matter** (412). The volume is the world's own body. Each voxel material
  carries a density, the ledger amount one base-unit cell of it holds, and
  names the matter account its mass moves through; both are world rules.
  Carving moves each emptied cell's mass into the carver's account for that
  material and filling draws from the filler's, so for every edit the
  volume's loss is the ledger's gain exactly. Air weighs nothing. An edit by
  a DM, the world editor or creative mode takes and returns matter through
  the dev source outside the conserved total and labels the run assisted
  (ruling 271). The voxel material seeded as `world:soil` becomes
  `world:earth`; `world:soil` stays the edible pool of TD6 and Isocosm's
  ecology.
- **Edits** (413). An edit is an operation, carve to air or fill with a
  named material, over a shape: a sphere (centre and radius), a box (two
  corners) or a route (points and a radius), in base units in the frame of
  the site it was made in. It carries its tick and a global sequence, is an
  asserted fact in the sim's state and history, and replays in sequence
  onto every chunk its extent reaches.
- **Borders** (414). An edit is stored once; a neighbour reads it through
  the border's frame relation, so both sides see the same fact. The list of
  foreign edits reaching a site is derived from the edits, never stored.
- **Caves** (415). A chunk keeps its columns and gains a sparse list of
  exceptions at the base grain, cells whose material differs from what the
  column rule gives; a coarse lift point-samples the base cell at each
  coarse cell's origin, as columns do.
- **Overflow** (416). Once capacity exists, what a carver cannot hold is
  filled back as a heap beside the cut, an automatic fill edit placed from
  the edit's seed; until then the carver holds all of it.
- **Checks, over seeded draws, each beside a control:** seeded edit
  sequences replay to identical bytes; a site re-lifted with its edits
  equals the edited site; an edit across a border is seen from both sides,
  and a neighbour that ignores foreign edits fails that; every edit
  conserves matter exactly, and a carve that credits nobody fails that;
  stored bytes grow with edits and not with sites.
- *Readings, not ruled:* an edit carries its own coordinates, supplied by
  whoever asserts it, the foreground game's handoff, a DM or the dev source,
  since the sim knows no positions inside a site (§3.8); an edit's extent
  stays under one site side, reaching only immediate neighbours; overlapping
  edits from different sites apply in global sequence; a carve's mass is
  counted column by column over the shape's vertical interval, never cell by
  cell; densities are world rules each founder may set.

### A.12 SP5's brief (2026-09-30, rulings 417 to 422)

- **Places** (417). Rooms, caves and tunnels are air cut off from the sky.
  The outdoors floods over walkable surface, an air cell over a solid one
  with headroom, and splits where a step exceeds the world's climb or water
  begins; a patch larger than the world's cap is cut on a grid of that cap.
- **Grain and clearance** (418). Places are derived once at the base grain.
  Each passage between two places records its clearance: the widest and
  tallest body that fits, and the step it climbs. A route for a body keeps
  only the passages it fits.
- **Climb** (419). A world rule, default one base unit of rise per unit of
  run; a steeper step splits patches and is recorded on the edges that
  cross it.
- **Cap** (420 and 421). Presets, sides in base units: extra small 16, small
  32, medium 64, extra medium 128, large 256, the default, and extra large
  512; or a founder's own x by y, laid in each site's own frame.
- **Location** (422). An entity names its site; in a lifted site where its
  game has placed it, it names its patch or room, and restriction returns it
  to its site.
- **Checks, over seeded draws, each beside a control:** local re-derivation
  equals full re-derivation after every edit of seeded sequences, and a
  stale dirty region disagrees and is caught; a route whose border profile
  stops at a cliff or water reads impassable in the volume, and one that
  passes reads passable; a body wider than a passage is refused by the
  clearance filter, and a broken filter lets it through.
- *Readings, not ruled:* an unlifted site is one place, and lifting opens it
  into its places, the nesting step's down, with its conditions read back up
  (ruling 74); water bodies are places of their own, joined to the land by
  edges a body wades or swims; a place's identity is its site and its least
  base cell, so re-derivation keeps the ids of places that still hold that
  cell; an edit re-derives the places its extent touches and their
  neighbours.

## 0. Rulings this plan rests on (2026-08-05)

*Reading, not ruled, 2026-09-28, on what stands:* 1, 3, 5, 7 and 8 stand,
7 carried into ruling 391; 2 stands with its "no shared schedule" retired by
the founding record's 2026-09-18 amendment; 4's camera line and 9's
reference targets are history; 6 holds within a site's frame and gives way
between sites (§A.7); 10's determinism line stands, the record citing it for
ruling 28, while its tool choices are history beside conatus's private
Rapier; 11 is superseded by the record's §4.2, renderling retired.

Recorded here and amended into the founding record, `CLAUDE.md`, and the
landscape doc in the same session:

1. **One authority per capability, stack-wide.** The anti-Spore rule's
   kernel: one simulation authority; projections plural and cheap; refuse a
   second authority, and refuse duplicating functionality the stack already
   owns. (Narrows "refuse a second simulation or a second renderer".)
2. **Shared engine organs are encouraged; sovereignty lives in the verbs.**
   Vessels share nouns (space, bodies, fields, time, provenance); each owns
   its verb and person (metabolize, address, adjudicate). An organ is
   shareable while it stays verb-neutral; when a component encodes what you
   do, it belongs to a game. (Narrows the founding's "no engine" clause,
   which guarded against coupling-as-obligation and genre convergence; that
   guard stands as: no shared genre, no shared schedule, no shared verbs.)
3. **Volumetric world truth is permitted.** The lens-only stop rule opens.
   The anti-Minecraft insight stays: no machinery adopted because prior art
   ships it; acceleration structures are admitted by trace.
4. **Verticality is a simulation affordance.** The third axis must be
   mechanically legible (wings escape ground threats, canopy holds
   resources, burrows hide). Any projection that makes those facts readable
   qualifies; 3D rendering is not the only way.
   *Camera amended 2026-08-06:* Mesocosm's camera is **pulled back, Rain
   World-style**, superseding the 08-05 Barony-march ruling. First person
   survives in agency (person is agency, not camera). The Barony/Delver
   first-person march reference moves to Paredros, which is the brick
   tracer's long-term primary consumer.
5. **World graphs must differ.** No two worlds share topology unless they
   are copies. Distinctness is receipted with graph metrics, not vibes.
6. **Nesting is elective and continuous.** Containers index one global
   coordinate space; an edge is never a portal; a place with no internal
   topology stays a leaf.
7. **Worldgen is hybrid.** Top-down skeleton (relief, watersheds), then
   bottom-up detail within regions.
8. **The unit of proof is a slice.** Engines fail at the joints. No gate is
   proven for the wing until the composed run exercises it beside the
   others.
9. **Design reference targets (2026-08-06).** Mesocosm: Rain World,
   Voxatron, Caves of Qud. Paredros: Barony, Delver, Gotcha Force,
   RimWorld. Isometry: Foundry, the Larian/Owlcat adaptations, Wildermyth,
   tactical RPGs virtual and tabletop. Reading that matters: none of the
   nine demands a heavyweight renderer; all are simulation-deep and
   presentation-modest. Investment order follows: agency, procedural
   bodies, place graph, storyteller over renderer horsepower.
10. **Physics is three tiers (2026-08-06).**
    - **Authority: integer, ours.** Occupancy and movement legality over
      brick truth, inside the replay hash. Never a physics engine.
    - **Advisor: parry queries + bespoke kinematics.** Raycast, shapecast,
      TOI, contacts from parry (no dynamics world, no stepping, no
      persisted handles); move-and-slide and verlet chain gait are owned
      code that quantizes into integer outcomes. The nalgebra seam is one
      adapter around a query library.
    - **Ambience: GPU, outcome-free.** Debris, foliage, cloth, ragdoll
      (nexus-shaped, eventually). GPU float ordering varies by hardware,
      so this tier is constitutionally barred from outcome-bearing facts.
    - Rapier-the-dynamics-engine is **in reserve**: it returns only for a
      proven constraint-dynamics need, and its documented cross-platform
      determinism mode is why it alone may then sit near the fact plane.
      Avian rejected 2026-08-06 (Bevy-coupled by design, parry underneath,
      no determinism story).
11. **Renderer tenancy follows the cohesion contract** (landscape §8.9):
    one device, one frame, declared capability profiles, glam at the
    presentation boundary, receipts. Renderling is the lead mesh-tenant
    *candidate* pending its fork probe; kiss3d is a donor (AOV
    segmentation bakes, 2D GI); nexus is ambience-tier pending its own
    device audit; rust-gpu is welcome, carried by the fork family.

## 1. The model

*2026-09-28:* §A.2 supersedes this section where they disagree. One
continuous coordinate space holds within a site's frame; places over the
bricks are derived (ruling 14), while the world map's sites are asserted at
founding (ruling 72).

One continuous integer coordinate space (the core's `[i32; 3]`), with
layers of meaning over it:

- **Place graph (macro).** Places own regions of the space: a generation
  recipe, a revision, dirty state, a simulation tier. Adjacency is derived
  from the landscape (link where travel is possible), never asserted.
  Nested containers are elective: a burrow system is a subgraph on a place;
  a meadow stays a leaf. The chartulary resemblance is noted and not
  depended on; a portable profile follows two real consumers, per the
  standing rule.
- **Brick truth (micro).** Per-region brick maps: coarse grids of pointers
  to dense material bricks; an absent pointer is air or unloaded. Dense
  bricks because they hash, diff, and serialize flat; SVO/DAG stay
  benchmark specimens (landscape §8.3 stands). Interiors are carved bricks
  in the same space.
- **Two-scale execution target.** Places near the played body run embodied
  individuals; distant places may run deterministic cohorts once the
  conversion and comparison receipts in the
  [scale plan](2026-08-29_scale_plan.md)
  land. The incumbent tick is not there yet: `organism/ecology.rs` still
  advances every organism record and derives `Cohort` only as a conserved
  summary. Promotion and demotion already happen at a hops-distance boundary
  with hysteresis, but that line currently selects locomotion and perception
  detail rather than changing canonical representation.

**Scale names are not authorities (clarified 2026-09-01).** “Macro” and
“micro” above describe the spatial index and voxel detail. They do not name
separate simulations. The future individual and cohort forms are two
canonical representations of one world; the trophic web is a derived reading
of their stocks and flows. The standard transition verbs are **materialize**
and **aggregate** because hydration already means water in this ecology. A
zero-tick round trip must preserve the aggregate record exactly, while
evolution over time is compared against an all-individual reference under a
declared error envelope. Camera distance, render LOD, and cache residency
cannot choose simulation detail. Recorded focus and other world facts may.
The [playable ecology plan's downstream gates](2026-08-31_playable_ecology_plan.md#7-downstream-architecture-gates)
route that scale-owned contract into PE6.

Presentation stays a family of projections of one truth (landscape §8):
a brick-map DDA raymarch grows out of the landed march for the
first-person lens; `mesocosm-mesh` pointed at world bricks is the raster
projection; the grade is unchanged; bodies ride the landed
`BodyLensProjection` (V2). Fields (numen) attach to places and bricks. Tactile
advice derives from the same snapshot through the landed T1 adapter
(`mesocosm_runtime::tactile` over Conatus, Rapier private inside it;
§8.4's advisor tier unchanged).

## 2. The slice: the burrow run

One composed scenario used to expose joins between systems.

Worldgen lays a place graph over one voxel space; one region generates
with a burrow. A catalogue critter hunts the player by real sight-lines
through real bricks. The player flees or carves, and hides inside
geometry. The hunter follows across the threshold without a stutter.

The original integration claims, now exercised in one run, were:

- the run replay-hashes identically from its recorded intents;
- the brick tracer renders it headed, native and browser, both souls, with
  fps and frame spans recorded beside netrender's (V1 harness reused);
- a carve lands: intent, dirty brick, region re-upload, collision refresh,
  with the latency recorded;
- the burrow needs no special casing in tracer, collision, or perception;
- per-tick sight-line cost is recorded at the target critter population.

The run is an integration specimen, not a game-completion test. Whether one
unfinished encounter feels tense neither validates nor blocks the engine. The
useful next move is to connect another independently owned fact to the same
scene and see which behavior appears without an authored branch.

The engine document (profile, capability matrix, external references) is
written after this run exists, from its numbers, not before.

## 3. Build history

### G0. A graph worth the name — **COMPLETE 2026-08-08** (constructor 2026-08-06; genesis adoption 2026-08-08)

Landed additively as `Places::grown(seed, side, extent) -> Grown` in
`places/relief.rs` (integer diamond-square, 65², own seed, percentile sea)
and `places/grown.rs` (traversability-derived links: climb + ford limits
over sampled crossings, Chebyshev-2 candidates, union-find reconnection
over least-bad blocked passes; `Nest` interiors where ruggedness earns
them). `scatter` and its serialized shape are untouched, because `Places`
rides inside snapshots and world genesis draws from a shared stream after
it; adoption is a one-line swap in `world/genesis.rs` deferred while that
file is in flight. Receipts (all green, strict clippy): same-seed
bit-equality; per-seed topology uniqueness over an 8-seed corpus; the
lattice-regression test (non-uniform interior degree, every seed); edge,
diameter, nest, and **bridge** counts all varying across the corpus with
at least one world growing a chokepoint; whole-world connectivity;
congruence (every unlinked grid-adjacent pair fails the crossing test,
which required making `crossing` direction-canonical after integer stride
rounding made A→B and B→A disagree); serde round-trip. Threshold
calibration kept as an `#[ignore]` spectrum test.

Replace `Places::scatter`'s lattice. Today: stratified jittered sites with
hardcoded 4-connected links, so every world is topologically identical
(uniform interior degree, fixed diameter, zero bridges, hubs, or dead
ends), and `links` disagrees with `at()`'s nearest-site partition, which
can make diagonal Voronoi neighbours the graph denies. Build the hybrid
generator: relief and watersheds top-down, local detail bottom-up,
adjacency derived from traversability, elective nesting where interiors
have topology.

**Done when:** the same seed reproduces the same graph bit-exactly; across
a seed corpus, degree distribution, bridge count, cycle structure,
diameter, and nesting depth all vary, and a lattice-regression test
rejects uniform degree; derived links agree with the partition (no
Voronoi-adjacent pair denied without a landscape reason); the reckoning
consumers (`hops`, `spread`, `scale`) keep their existing receipts.

### G1. Brick truth with a lifecycle — **COMPLETE 2026-08-08** (container 2026-08-06; world adoption 2026-08-08)

Landed additively as `places/bricks.rs`: `Ground::grow(&Grown, extent)`
raises dense 8³ bricks (ordered map, serde-flat) from the relief; nests
realize as **roofed burrows anchored at the highest column near their
host** (a low host digs into its hillside rather than cratering; rooms
scale to afforded depth; every chamber keeps a ceiling). Each also has a
direct one-voxel descending entry to a roofed room: a former vertical shaft
looked like an interior but could not be climbed by the owned near-stepper.
`carve` bumps one
revision and marks dirty bricks; `drain_dirty` is the projection's upload
discipline. Occupancy (`solid`, `stands`) and integer line-of-sight
(`sees`) land here too, seeding G3's perception. Receipts, all green with
strict clippy: same-world bit-equality + serde round-trip; identical
carves replay to identical bytes; a radius-1 carve dirties ≤8 bricks and
carving air is not an edit; burrows are roofed voids near every nest; every
generated entry step is legal and ends under a roof;
hills block sight and a bored tunnel grants it (the first scan version
walked into a burrow corridor, which is its own kind of receipt); and in
`mesocosm-mesh/tests/ground_projection.rs`, every matter-bearing brick
meshes through the same `mesh_volume` path bodies use, with clean bricks'
meshes untouched by a neighbour's carve. The original remaining G1 pieces —
carve as an ordered `Intent`, `Ground` inside the world snapshot/replay hash,
and genesis wiring — all landed with the `grown` swap on 2026-08-08.

Place-keyed brick regions over the one space; carve as an ordered intent;
dirty regions revisioned (the lens's `MapRevision`/`MapChange` discipline
generalized); the snapshot/replay hash covers bricks; `mesocosm-mesh`
consumes a world brick as a `VolumeSource` for the raster projection;
occupancy derives per brick for collision and perception queries. The dirty
queue is projection-only: it is omitted from snapshots and equality, so a
host's upload cadence cannot alter replay state.

**Done when:** a carved world replays to an identical hash; an unchanged
world uploads zero brick bytes across frames; a carve uploads only its
region; occupancy answers stand, burrow, and see for the near tier.

### G2. The tracer, riding the landed V1 harness — **COMPLETE 2026-08-14**

Fragment-only brick-map DDA in the lens's retained pattern: pointers and
brick atlas in 3D textures, no storage buffers, no compute; both souls;
SDF bodies composited as now. *Camera note (2026-08-06):* the DDA is
camera-neutral; with Mesocosm pulled back (ruling 4), the tracer's
long-term primary consumer is Paredros's first-person lens, and
Mesocosm's shipped projection is chosen at playfeel within the
pulled-back framing (mesh raster, side-view, or a pulled-back trace).
G2's proof value (volumetric presentation + downlevel receipts) is
unchanged.

**Done when:** the tracer renders a carved region with interiors correctly
occluded; downlevel and wgpu-GL receipts pass (the glprobe pattern,
landscape §8.8); a browser receipt lands through the same composition seam
and recording discipline V1 proved; 1080p fps is recorded on the 4060 and
on GL, and the number is reported whatever it is.

**2026-08-14, P0 Ground tracer.** `BrickMap` is a lens-owned, rebuildable
projection of `Ground`: `R32Uint` 3D pointers address a dense `R8Uint` 3D
material atlas, with slot zero as air. `BrickTracer` does fragment-only DDA
against those textures, encodes into a caller-owned target, and owns neither
world state nor submission. A real seed-4242 hill and bored horizontal
tunnel receipt confirms the visual lifecycle: the initial 265,612-byte upload
is followed by 43 removed voxels in two changed brick slots, a 1,032-byte
upload, and changed capture pixels. Steady frames upload zero brick bytes.
This proves Ground-to-texture provenance and narrow carve propagation. It does
not by itself prove composition, host, browser, downlevel, or 1080p evidence.

**2026-08-14, P1 composed cross-host receipt.** `BrickFrameInput` now carries
an optional presentation-only `CritterPose`; the DDA shader sphere-traces its
capsules and lets the nearer of its SDF hit and Ground hit own each pixel.
`g2_frame` is the headed harness: generated seed-4242 Ground is reconstructed
into `BrickMap`, one SDF body is composed with it, and the caller-owned trace
texture enters netrender at external scene boundary zero. The native release
run presented the composed 1920×1080 frame on the RTX 4060/Vulkan with scene
digest `fnv1a64:411f3c4f92446b5a`; the steady second frame made no brick or
uniform upload and recorded netrender's 991µs span ledger. The browser ran
the same 232,803-byte scene digest, trace and netrender path headed through
Browser WebGPU at 960×540, again with a steady no-upload frame and no browser
validation errors. The rendered canvas, not merely the receipt text, was
inspected: voxel ground, nearer SDF body, and chrome were all visible.

`g2_glprobe` requested `Limits::downlevel_webgl2_defaults()` on wgpu's real
GL backend (maximum 2D texture 2048): both clay and retro body-plus-Ground
frames rendered at 960×540, the grade change changed pixels, and its second
frame uploaded zero Ground bytes. `g2_bench` performs synchronized, readback-
free steady DDA frames at 1920×1080, including Ground and the SDF body:
**482.4 fps median** (2,073µs, 1,825–3,219µs) on Vulkan and **520.8 fps
median** (1,920µs, 1,697–2,641µs) on GL. These are tracer spans, deliberately
reported beside rather than mistaken for the headed netrender frame budget.

### G3. The near tier — **core LANDED 2026-08-06; ecology drive/tier slice LANDED 2026-08-07; player ingress, autonomous occupancy/sight wiring, and 300-body scale receipt LANDED 2026-08-14**

Landed additively as `places/near.rs` (kinematic `step` with slide, climb,
gravity settle, and doorway drops; `spot` sight; `Tier`/`TierLine` with
promote/demote hysteresis over hops). The former `places/hunt.rs` probe
supplied the historical chase receipt, which passes on the real seed-4242
ground: a
half-speed hunter acquires its quarry, loses it behind a hill, projects
along its heading, is forced off a cliff edge, re-acquires at a bored
den's mouth, and follows it inside, with per-tick continuity asserted
throughout (no teleports). Determinism receipt: identical runs
bit-identical. The former standalone Hunter probe measured **300 hunters in
~62µs** (release), but it did not exercise world ecology, Ground, or the
far-cohort projection and is retained as probe evidence only. Tier receipt:
the line does not flap crossing the band, and a far hunter wakes on
promotion. Two movement laws were earned by failure, both the same
lesson: **preferences order, they never refuse** — climb-when-descending
and the cliff-edge comfort drop each deadlocked an agent until the
preference became an ordering with a forced fallback. The Hunter wiring was
deliberately not adopted. E0-E4 now wires the tier line, far-tier receipts,
and anatomy-driven drives into the world's organisms; the old chase receipt
remains probe evidence rather than an authoritative FSM.

**Superseded in destination, 2026-08-06.** The `Hunter` FSM is **a probe,
not the design**. Search/Stalk/Memory is authored behaviour with a tuned
patience constant, and it duplicates a feeding model the ecology already
owns. Its lasting contribution is proving the *queries* work: sight
through terrain, pursuit across a burrow threshold, one-voxel continuity,
and the movement law that preferences order rather than refuse. The
[general model plan](archive_docs/2026-09-26/2026-08-06_general_model_plan.md) gate **E4**
replaces it with drive-and-affordance selection, where pursuit is what a
fast, large-mouthed, starving body does about a reachable meal. Do not
build further behaviour on the FSM.

Perception as sight-lines (brick DDA or parry raycast), advisor-tier
locomotion per ruling 10 (parry queries + owned move-and-slide + verlet
chain gait, quantized into integer outcomes, no persisted handles), hunt
and flee for catalogue critters; tier promotion and demotion at a hops
boundary with hysteresis.

**Done when:** a hunter acquires, pursues, loses, and re-acquires the
player across a place boundary and a burrow threshold without stutter or
teleportation; per-tick agent cost is recorded at the target population;
the individual far-body path stays within its existing receipts and its
derived cohort projection conserves the recorded scalars.

**2026-08-14, V0 player closure.** Genesis now puts every founder on real
brick footing. `Intent::Move` resolves one `near::step` toward its requested
offset, charges only horizontal distance actually travelled, and cannot use a
large offset to cross a wall or slope. The focused receipt finds a real
climb-blocking face, records a carve that turns it into a doorway, crosses it,
and replays to an identical hash. This is deliberately not G3 completion:
`ecology::disperse` still uses its abstract integer step and has not yet read
`Ground` occupancy or `spot` sight.

**2026-08-14, V1 autonomous closure.** Replayed worlds now call
`ecology::step_with_ground`: near-tier food and carrion acquisition uses
`spot`, pursuit takes one or more bounded `near::step`s under the existing
locomotion budget, and exhausted near bodies wander by a legal grounded step.
Far bodies retain their place-graph movement and perception, while graph
traversal and near-tier birth keep embodied bodies on valid surface footing.
`Cohort` remains a derived conservation projection here, not the executing
far-tier record. The receipt
finds a generated occluding wall, proves an autonomous predator does not steer
through it, records a carve, then proves it enters the opened doorway and
replays identically. Population-cost and full burrow-run receipts remain open.

**2026-08-14, V2 grounded scale receipt.**
`grounded_ecology_receipt` starts from the actual seed-4242 world with 300
living Near founders, warms a disposable clone, then measures five
independent, identical 64-tick idle windows. Refreshed after the G4 perception
work, this Windows host's release build reports **897.05µs/tick** median
(887.95–928.89µs; 903.81µs mean), and all samples finish at the same state
hash. The receipt also rejects an
unfooted living Near body and rejects any scalar disagreement between raw Far
bodies and their cohort projection. It replaces the old Hunter number as the
G3 capacity evidence; it is a host-specific measurement, not a frame-budget
guarantee. Full player-and-burrow composition remains G4.

### G4. The burrow run — **COMPOSED 2026-08-21; interaction expansion continues**

G0 through G3 now meet in §2's scenario. P0-P8 below are the assembly record,
not a reason to stop extending the causal joins.

**2026-08-14, P0 one-world doorway run.** `burrow_run_receipt` and the
Lens `burrow_run` example now begin from one real `World`, rather than an
adjacent demo Ground. A played producer begins occluded from a near consumer
by generated brick terrain; an ordered `Idle`, then legal `Carve`, opens the
doorway; Ground sight changes; the consumer enters by its ordinary grounded
ecology step; and a twin produces the same outcomes and final replay hash.
The lens refreshes just one dirty atlas slot (516 bytes) and the DDA capture
changes after the eight-voxel carve. This is the composed authority and
projection join, but not the full G4 verdict: it is a generated occluding
doorway, not yet a selected roofed burrow encounter; the live run is not yet
headed through the G2 netrender/browser harness; population sight cost and
the human tension judgment remain open.

**2026-08-14, P1 generated-entry correction.** A nest's old vertical hollow
was a visual and raycastable burrow, not one an embodied body could enter:
`near::step` cannot climb a shaft. Generation now produces a direct,
one-voxel descending route into a roofed room and relocates the generated
room cluster at that entry. The Ground receipt walks every route edge through
the actual stepper. The world receipt then places a producer under that roof
and a Near consumer at the mouth: on `Idle` it sees the producer, descends on
the generated route, remains grounded, and twin-replays identically. This
settles actual continuous burrow ingress, not the full G4 scene. The existing
hide/reacquire-and-carve proof is still a selected generated doorway; combining
it with a turning interior needs a local path-search policy, not another
geometric special case.

**2026-08-14, P2 local lost-sight pursuit.** `LastSeen` is now serialized
organism state, rather than a host perception cache: a Near consumer refreshes
it from direct `spot` acquisition; it lasts for eight failed perception ticks
and clears on expiry, target death, or a tier change. `route_step` is a small,
deterministic breadth-first search over the actual `near::step` transition
relation, limited to an eight-voxel horizon and 256 examined stances. It adds
no collision map and makes no global-path claim. The generated-entry receipt
proves direct observation writes the replayed state. A separate one-world
receipt carves an L-shaped bore with the ordinary Ground primitive, puts a
target beyond an occluded turn, and proves a consumer's remembered target
takes the legal first detour and twin-replays. The expiry and Near/Far boundary
are unit-receipted too. Reacquiring a *moving* target through that turning
interior, the headless G2 scene composition, performance at population, and
the human tension judgment remain G4 work.

**2026-08-14, P3 external-frame closure.** The Lens `burrow_run` receipt no
longer stops at a tracer capture. Its same-device `G4Frame` encodes the
World-derived DDA view into an external texture and passes that texture through
netrender, which owns the final master frame. The before/after frames differ
after the recorded eight-voxel carve; the after frame carries one dirty atlas
slot and 516 brick-upload bytes into the composed master. The native receipt
also records netrender span count and writes that actual master as PNG. This
proves the headless frame seam for this run, not browser parity for this
particular scenario: the existing browser/downlevel G2 host remains a generic
Ground projection proof. A browser-directed burrow-run harness, moving-target
reacquisition through the turn, population cost under sight/routing, and the
human tension judgment remain G4 work.

**2026-08-14, P4 moving-target reacquisition.** The World receipt now uses
the played producer, rather than a fixed quarry: it is seen by a Near
consumer, moves through an ordinary player-carved L turn out of sight, leaves
the consumer following its serialized last-seen position, and is reacquired
within the eight-tick local window. Every step stays grounded and the whole
intent trace twin-replays. This composes acquisition, pursuit, loss, and
reacquisition through a real turning obstruction. It is intentionally a local
carved-turn proof, not the final §2 claim across both a generated burrow
threshold and a place boundary. Those two conditions, the browser-directed
burrow run, population sight/routing cost, and the human tension judgment still
keep G4 open.

**2026-08-14, P5 embodied scene subject.** The G4 frame no longer supplies a
handmade capsule as its player. It projects the controlled organism's
authoritative `BodyDocument` through `BodyLensProjection`, checks every living
part became an SDF capsule, and records that body revision with the composed
frame receipt. Thus the DDA terrain, played body, and netrender master all
come from the same World run without introducing a presentation body format.
The selected player currently has one living root part, so this is a real
ownership join rather than a multi-part visual stress case. The remaining G4
conditions are unchanged.

**2026-08-16, P6 sight cost at population.** §2 asks for per-tick
sight-line cost at the target critter population, and the existing
`grounded_ecology_receipt` did not answer it: it measures 64 ticks over
which the founders fall from 300 Near to 145, so its figure averages a
shrinking population. For a capacity question that is the whole
question. `sight_cost_receipt` measures a short window with the
population intact and sweeps it, so level and shape are both visible.
This Windows host, release: 75 bodies 133us/tick, 150 -> 398,
300 -> 1234, 600 -> 3747. **At the target, a tick with 300 Near bodies
seeing and routing costs about 1.23ms, roughly 7.8% of a 16.7ms
frame.** Per-body cost grows 2.31x across a 4x population: super-linear,
so the pairwise sight term is real, but well short of the 4x that fully
pairwise would give, and the tier line holds the target with room. The
span is a whole `Idle` tick with sight and routing active rather than
sight isolated, since there is no sight-off control to subtract; the
shape rather than the level is what implicates sight. Its assert
catches a blow-up rather than freezing a constant.

**2026-08-21, P7 browser-directed doorway run (implemented locally).**
The V1/G2 headed host now accepts a scenario module rather than owning one
hardcoded G2 document. `g4_frame` uses that same surface, device, tracer,
external-texture, netrender, and receipt path for P0's independently generated
seed-4242 Worlds and ordered `Idle`/`Carve` trace. Native Vulkan presented at
1920×1080; Browser WebGPU presented at 960×540. Both reported replay hash
`10803197323918604988`, scene digest `fnv1a64:ef6d3107b0e8b508`, eight removed
voxels, one dirty slot, a 516-byte brick upload, zero readback, and the
netrender span ledger. The browser canvas was inspected headed and emitted no
warnings or errors.

This receipt also corrected two presentation lies found during the run.
P0's hunter-side capture changed pixels but was mostly a wall, so the browser
receipt uses the judgment harness's actual subject position: the hidden
player's eye watches the authoritative hunter body enter the opening. That
view initially looked horizontally above the lower doorway; the browser and
judgment harness now derive the downward pitch from eye to threshold. The page
also puts the appended canvas before the long JSON receipt visually. These are
camera and host corrections over the same World and intents, not new scenario
facts. The two-frame timings are receipts, not an fps benchmark; G2's
synchronized 1080p spans remain the performance evidence.

**2026-08-21, P8 generated crossing.** The final machine claim now has one
run rather than an inference across fixtures. Seed 0 grows a five-stance nest
entry. The threshold is its first step; its third step crosses Place 3 into
Place 0. The played producer descends three ordinary `Intent::Move` steps and
the autonomous consumer selects `Pursue` on every ecology tick, following
exactly one generated stance behind. Both bodies cross the place edge, the
hunter crosses the burrow threshold, every position is grounded and unique,
and an independent twin returns the same seven history events and state hash
`7286350926008272852`. `Grown::nest_entries` is the narrow read model that
lets this receipt and a projection name the generator's exact route; terrain
generation still owns the construction rule and Ground remains the collision
authority.

The shared headed host presents that same three-intent run. Native Vulkan at
1920×1080 and Browser WebGPU at 960×540 reported scene digest
`fnv1a64:e90af60c63d5c6ec`, the same replay hash, revision zero before and
after, zero dirty slots, zero brick upload on the final movement frame, zero
readback, and the netrender span ledger. Native's final frame reported 163 µs
of tracer preparation and 4591 µs in netrender; the browser reported 0 µs and
1300 µs respectively. These four-frame timings are receipts rather than a
benchmark. The browser canvas was inspected headed and its diagnostics log was
empty. A too-close first camera cut through the wide hunter, so the final
projection keeps the actual player eye, aims at the body centre, and scales the
body to fit the one-voxel separation. World state and the ordered trace did not
change.

**2026-08-21, P9 body-terrain composition.** The universal two-high movement
column is now only the compatibility wrapper. `WalkerShape` derives a turning
cross-section from each live `BodyDocument::aabb`: one primitive mass segment
maps to one Ground voxel, height remains exact at that scale, and the shorter
horizontal axis lets an elongated body align with a passage until facing
becomes authoritative. Player movement, near pursuit, remembered routing,
wander, graph-to-Ground realization, founders, and births all use the derived
shape.

The resulting behavior needs no hunter variant. At the same generated seed-0
threshold, compact and broad consumers both select `Pursue`; the compact body
descends while the broad body is diverted onto the roof. Both branches replay
identically. Live anatomy owns the answer: adding a broad part widens the
cross-section, severing it narrows the body again, and incorporation that would
expand a body through its current tunnel rolls the whole meal transaction back
as `NoRoom`. `body_clearance_receipt` keeps the two pursuit branches visible.

The deliberate approximation is now explicit: this is a turning
cross-section, not oriented collision. Facing, gait, and deformability can
refine it later without adding another body-size field or changing Ground
authority.

**2026-08-21, P10 body-perception composition.** `spot_for` now derives the
terrain ray's endpoints from both live `WalkerShape`s. A baseline body keeps
the established one-voxel head offset; anatomy taller than the old walker
lifts its top-centre sight point while the stance-to-stance horizon remains
bounded. The ecology tick's temporary living and carrion target views carry
those derived shapes, so feeding, fauna policy, lost-sight acquisition, and
scavenging all ask Ground about the bodies that are actually present.

At generated seed 0, the same observer stance `[-15, 15, -16]`, prey stance
`[-10, 16, -16]`, terrain, range, and default fauna policy split on one live
part. The compact hunter's sight point `[-15, 16, -16]` is occluded and it
records no target decision. Adding a narrow vertical part keeps passage width
unchanged, raises the sight point to `[-15, 17, -16]`, clears the ridge, and
selects `Pursue`. Independent copies replay to the same hashes;
`body_sight_receipt` exposes the pair.

This is body-height sight, not a claim that the top of every critter is an
eye. `Process::Sense` already exists, but Ground has no derived location for a
living sensor part yet. A later composition can replace the top-centre fallback
with sensor-part positions without changing sight authority or storing another
perception profile.

## 4. Wing checks

During G1, before the brick container's shape hardens: read it against
Paredros's settlement-edit pressure (landscape V3 wording: one revisioned
snapshot, only affected products update, stale jobs rejected) and against
Isometry's bake reads. The shared organ serves three vessels or it is
Mesocosm-local and says so. Extraction to a permissive crate follows the
standing rule: after two real consumers, never declared in advance.

## 5. Stop rules

- No portals. If crossing a boundary ever reads as a scene transition, the
  design is wrong, not the tuning.
- No second authority. Caches, meshes, SDFs, occupancy planes, and DAGs
  never become world truth.
- No camera, render LOD, clipmap, or cache-residency decision may select the
  authoritative simulation representation.
- No cohort execution claim lands on projection-only evidence. It needs an
  exact zero-tick round trip and a measured evolution envelope against the
  all-individual reference.
- No chunk machinery by availability; admit acceleration structures by
  trace against real mutation workloads (V4 discipline).
- No graph ships without its distinctness receipts.
- Compile-only wasm evidence is not browser support.
- A component proven alone is weak evidence; joins are the unit of progress.
- Prefer another causal join over declaring the current encounter finished.

## Findings

- 2026-09-29, **SP3 must admit the selected window against the actual
  scene capacity.** The first native control produced 3,664 bricks and
  `GroundTerrain::brick_map` refused its default 2,047-brick map. The
  pixel-change check correctly failed because the prior image remained.
  The retained adapter now counts occupied bricks before `Ground` allocation
  against `AtlasLimits::DEFAULT`, in addition to its voxel-work and memory
  estimates. The base-grain border window is 32 by 32 columns, and the
  deliberately changed site's elevation increases by 64 base units.
  Overviews still include both whole sites at a coarse grain. This verifies
  a bounded scene connection, not paging or a laptop frame-rate target.
- 2026-09-29, **SP3's presentation units and materials.** Lifted heights,
  water and soil are already expressed in cells at the requested level;
  an overview must not scale their vertical values again. The adapter
  uses a common local datum across both sites and resolves material IDs
  through the world's key table. Soil thickness may legitimately differ
  with site relief. Surface-border agreement and faithful material lowering
  are separate checks; matching surface digests alone cannot prove both.
- 2026-09-29, **retaining another bench view requires lifecycle checks.**
  Source review found that pausing the trial's ordinary play flag leaves
  its separate epoch-boundary advancement active. The terrain view must
  suspend both paths and report its own presented camera, rather than the
  prior specimen's camera and body counters. A height-only bound also
  permits too much work: guard total voxel construction before growing
  `Ground`, and report the selected window's workload with its receipt.
- 2026-09-28, **SP2: an exact mean over base-grain columns needs a
  closed form, and the lattice gives one.** Were each column to round on
  its own, the sum of a site's columns would have no closed form, and ruling
  400's exact mean would cost a sum over every column before any lift, about
  4.2 million for a 2,048-unit site. Holding the surface as the exact
  interpolation of the 17 by 17 lattice, in units of `1 / (4 s²)`, makes the
  sum a weighted sum of lattice heights and the correction an exact integer
  spread over the interior (§A.9). The receipt checks it against brute force
  over 195,970,560 columns.
- 2026-09-28, **SP2: two instruments were incomplete on first writing and
  were fixed before the commit.** The relief check had no control, and a
  bound true by construction proves nothing without one: detail drawn past
  its window now fails it. The water test could have passed without any
  column under water: it now asserts that some drawn columns are. The border
  check's own control, a far side read in the near side's direction, is in
  the receipt.
- 2026-09-28, **SP1: the corner control needed a positive check beside
  it.** A corner height keyed per site fails corner agreement, but a corner
  walk returning single slots would fail that control too, through the check
  that borders end at their corners, so the control could not prove the walk.
  `corner_walks_find_every_corner_of_the_grid_once` counts the corners
  instead: (W+1)(H+1) on a plane, W(H+1) on a ring and W·H on a torus, each
  slot in exactly one (`shared/isocosm/tests/spine.rs`). The direction rule,
  a far side reading its border backwards, has its control in the receipt: a
  forced forward read fails at "site 1 side 3 misses its corners".
- 2026-09-28, **what the spine starts from** (§A.4 in full). Isocosm's
  sites carry a `terrain_seed` drawn at `generate.rs:275` and read nowhere.
  `Ground` is one bounded extent with one revision and no additive write, in
  a crate that renders nothing (`wing-formats`, `serde`, `postcard`). nisus
  has no world store, and the T2 lane ruling 335 placed in mere's conatus
  engine plan §2 is not written there; production Burn is still
  `0.22.0-pre.2`. Mesocosm's relief is one 65×65 field that cannot tile.
  Isocosm's default base unit is 1,000 µm (`generate.rs:35`), a placeholder
  no world has been founded against.
- 2026-09-01, **the shipped cohort is a projection, not an execution tier.**
  `step_inner` advances the individual organism roster, then calls
  `cohort::from_organisms` to populate conservation tallies. `Cohort` carries
  count, biomass, energy, and summed age, but not enough state to execute
  lifecycle, reserves, process, relationship, or evidence semantics. S3 must
  therefore promote a second canonical representation deliberately rather
  than treating the current summary as one.
- 2026-08-07, **three directions adopted from the bonsai reading**
  (landscape §8.3 carries the donor row and the unverified-claim caveat):
  1. **The relief lab.** Worldgen tuning becomes an instrument: live seed
     and parameter twiddling over `grown` + `Ground`, re-rendered through
     the tracer per keystroke. Bonsai buys its "voxel shadertoy" loop by
     moving generation to the GPU; we get the same loop keeping authority
     on the CPU, because authoritative gen is deliberately coarse (65²
     relief, ~400 bricks, milliseconds). G0's distinctness receipts
     become feelable, not just assertable. Instrument for G0/G2, not a
     gate.
  2. **The presentation-amplification tier.** The terrain twin of the
     ambience tier, same constitutional line: GPU-side micro-detail
     (micro-relief within faces, scatter, texture variation) derived
     from authoritative bricks, **never entering the replay hash**,
     enhanced capability profile only. The grade already amplifies
     colour; this extends the same contract to apparent geometry.
     Bonsai's "second-stage decoration reading terrain derivatives" is
     the worked example.
  3. **Sim/render LOD: shared facts, separate events.** *Corrected by
     audit, 2026-08-07: the first form of this direction ("one line
     serving both, demotion as the same event") was wrong.* Simulation
     tier depends on the authoritative recorded focus and its
     transitions; render LOD may depend on a local camera, and under
     one-state-N-windows there are many cameras and one authority. The
     two consumers share distance and region *facts* (hops, place
     membership) and nothing else. Far places may still collapse to
     hulls or mips for rendering, per viewer, without touching tier
     state.


- 2026-08-06, **renderling wgpu-29 port receipt** (fork at
  `Code/crates/renderling` + `Code/crates/crabslab`): the 26→29 bump is
  mechanical. ~277 initial errors were mostly a two-wgpu split (craballoc
  0.3.1 pins wgpu 26; forked at the `v0.6.6` tag + three edits: wgpu pin,
  `PollType::Wait` fields, `pub fn len`). Renderling itself took a scripted
  pass (descriptor field renames, `Option`-wrapped depth state,
  `bind_group_layouts` Option-wrapping, `MipmapFilterMode`,
  `CurrentSurfaceTexture`, `experimental_features`) plus hand fixes. Lib and
  renderling-ui check clean; test modules pass in isolation including image
  goldens. Open: a cross-test device-teardown leak
  (VUID-vkDestroyDevice-05137, OOM accumulation over 95 sequential
  device creations) to chase before the suite is a receipt; upstream's
  suite also assumes parallel per-test devices, which our no-concurrent-
  test rule already forbids. Next gate: `Context::new` on netrender's
  `WgpuHandles` with a composed external texture and a headless golden.
- 2026-08-06, **device-unity probe PASSED**. One instance/adapter/device/
  queue (netrender `WgpuHandles`, feature union `REQUIRED_FEATURES` +
  renderling's four, intersected with the adapter). Renderling
  `Context::new(RenderTarget::from(texture), ...)` rendered a three-
  triangle ortho stage into an `Rgba8UnormSrgb` texture on the shared
  device; netrender composed it at scene-op boundary 0 under vello chrome
  via `ExternalTextureComposite` and presented a master. Headless readback:
  72 distinct colors, full coverage, spans recorded (vello_render 5.1ms,
  master_compose 48µs, tenant render outside netrender's spans). Zero
  copies, zero validation errors. Cohesion contract clauses 1 and 2 hold
  for renderling as a tenant. Probe: scratchpad `rlprobe`; receipt PNG
  captured. Renderling's mesh-tenant candidacy is now **proven at the
  seam**; remaining before the seat is confirmed: the device-teardown leak
  chase, a wing-shaped scene (voxel mesh body via `mesocosm-mesh`), and a
  browser receipt per D0 discipline.
- 2026-08-06, **leak found, fixed, suite green: 95/95.** Bisection
  (context-only survives 120 cycles; context+stage OOMs) plus wgpu's
  alive-resource report (exactly 8 buffers leaked per Stage create/drop,
  zero textures) attributed the teardown leak to a **self-referential Arc
  cycle in craballoc's `SlabBuffer`**: the allocator stores a `SlabBuffer`
  inside the `Arc<RwLock<Option<SlabBuffer>>>` slot that the stored copy's
  own `source_slab_buffer` field points back at. Fork fix: the back-pointer
  is now `Weak` (allocator owns the strong slot; handles upgrade while it
  lives; a dead allocator means no newer buffer can exist). Receipts: stage
  churn now leaks zero buffers over 24 cycles; the long-lived frame loop
  was already clean (flat counts over 600 frames); renderling's full suite
  passes 95/95 single-threaded on wgpu 29. Upstream-worthy fix; the fork
  carries it for now.
- 2026-08-06, **wing-shaped scene through renderling**. A real body grown
  by play (`World::new(2024, 80)`, the fixture's eat loop, 10 living
  parts), greedy-meshed by `mesocosm-mesh`, triangulated by
  `mesocosm-render::build_vertices`, rendered by renderling (perspective
  camera, per-face normals) on netrender's device, composed under vello
  chrome. 120 triangles, every vertex tracing to a part with provenance;
  receipt PNG captured. The pipeline holds end to end with no hand-placed
  geometry. The image also re-demonstrates the known appearance gap: solid
  same-tag volumes read as one blocky mass, which is the axial-plan and
  V2-capsule work's territory, not the tenant's. Remaining for the seat:
  the browser receipt (D0 discipline).

## Progress
- 2026-09-28: **SP2 landed** at `38ea90f`. The world-local material table on
  the world's traits, seeded by the map; the lattice with its Coons interior,
  faded detail and exact correction in `src/terrain/lattice.rs`; column
  chunks in `src/terrain/lift.rs`; the border, mean and relief checks;
  `isocosm-bench --lift-draws` and `--lift-digest`. Twelve tests in
  `tests/lift.rs`, 133 in all, with Clippy and rustfmt clean. Receipt in
  `mesocosm/testing/bench/receipts/2026-09-28/isocosm/`.
- 2026-09-28: SP2's forks ruled (400 to 403) and its brief written as §A.9,
  the exact mean kept by a lattice with closed-form sums rather than a sum
  over every column.
- 2026-09-28: **SP1 landed** at `6f25a89`. Optional `Border`, `Footprint`,
  `Skeleton` and `Founding.map`, skipped when absent so old worlds hash as
  before; the grid layout and validation in `src/map/`; corners, profiles
  and the two checks in `src/terrain/`; `isocosm-bench --map-draws`. Ten
  tests in `tests/spine.rs`, 121 in all, with Clippy and rustfmt clean.
  Receipt in `mesocosm/testing/bench/receipts/2026-09-28/isocosm/`.
- 2026-09-28: SP1's brief written as §A.8 and its three forks ruled (397 to
  399); the Isocosm baseline passed 111 tests before any SP1 change.
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

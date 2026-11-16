# What a body is geometrically: research brief (2026-11-15)

**Status, 2026-11-15:** research brief; nothing ruled; no plan. It changes no
code. It answers the research owed by the lineage rounds plan's D41
(`mesocosm/design_docs/2026-11-09_lineage_rounds_plan.md`, line 267). Every
choice with more than one defensible answer is in section 5 as a question for
Mark. Index row: add to `DOC_README.md` when Mark takes the brief (this file
was written without touching any other file).

**Read-only research.** Nothing was built or run. Every code claim carries a
path; where a path was named from an import and not opened, the text says so.
The sibling-repo survey (section 4) was done by reading and searching only.

## 0. The question, and the lead idea

Mark, 2026-11-15, asked how radial symmetry should work given boxes on a
lattice with six facings, and answered:

> "I suppose 1… but wait, wait wait wait is that what bodies should be or is
> that just what bodies ended up as? cause if there's a wiser way to do this,
> you know, a more visually interesting way to do this, a more efficient way to
> do this, then I'm certainly willing to check them out. You know alternate
> conceptions of what a body could be that would make more sense, taking a
> holistic perspective of the wing and stack and the options for developing
> both to replace existing less efficient/cool attempts? again, we're dealing
> with a prototype."

Added the same day, and treated here as the lead alternative (e):

> "I'm pretty interested in, like, you know, microvoxels. I've been talking
> about them a lot. ... I like the idea of being able to subdivide a voxel into
> smaller parts, and sum a voxel up into larger parts, and for that to be like
> a nestable, you know, recursive self-relation. That to me, that feels
> intuitively logical and interesting, but I don't know how to do that
> efficiently. And it'd be cool if bodies and environments, all that, you
> know, worked with that. I don't know, but maybe that's asking too much."

D41 ruled that symmetry becomes a grammar rule (bilateral, radial with an arm
count, none), "today's lattice shows what it can of the count (four)", and
that "body shape gets its own brief".

### What this brief finds, short

1. The simulator's truth about a body is already two layers, and only one of
   them depends on boxes. The **logical layer** (a part tree, cells counted
   per function, matter ledgers per part) never reads a coordinate; the
   **geometric layer** (half-extents, offsets, quarter-turn yaw) is read by
   about 105 source files (302 mentions of `half_extent`) and by the whole
   physical stack, 9,158 lines under `shared/cosmograph/src/spatial/`.
2. Roughly half of what reads a box reads it only to get a *role*
   (`classify`, `boxed`), a *size* (cells, mass) or a *sign* of an offset
   (under the head, above the parent). Those are cheap to move to another
   truth. The expensive readers are the physical ones: non-overlap, walking,
   balance, contact, sensing eye points, and the saved and hashed documents.
3. A part already carries a miniature cell lattice, at most 4 by 4 by 4 = 64
   cells, addressed by index with no stored coordinate
   (`shared/cosmograph/src/mosaic/mod.rs:8` to `:12`, `:36` to `:38`). That is
   one level of a nested voxel tree that was never given a coordinate or a
   second level.
4. Isometer already has fixed-point (1,024 to the unit) capsule geometry and
   fixed-point rotation by any angle for walking
   (`shared/isometer/crates/isometer-core/src/legs.rs:14`,
   `legs/rotation.rs`), a float SDF presentation of bodies as rounded boxes
   and capsules (`shared/isometer/crates/isometer-render/src/implicit.rs`),
   content-addressed volumes (`VolumeRef`, `body.rs:28` to `:39`), and 8-cubed
   bricks with a "uniform brick kept as one material" collapse for terrain
   (`isometer-core/src/ground.rs:5` to `:12`). Each is a piece of an
   alternative; none is connected to the others.
5. Exact summation of matter up a tree is compatible with how matter is kept
   (integer `u64` ledgers, `schema.rs:15`), provided every split has a fixed
   remainder rule. The sim already has one (largest remainder in part order,
   `anatomy.rs` header). Dynamics are not exact under coarsening; the doctrine
   already says so for crowds (Processes 6).
6. Terrain is the harder half of "one structure". It is not sim state: it is
   derived from site conditions and edits and discarded
   (`shared/cosmograph/src/spatial/site_geometry.rs:11` to `:25`), holds a
   palette index and no amount (`ground.rs:32` to `:38`), and its store is
   owed to nisus (Stack 5; place-graph plan SP6).

## 1. What depends on boxes today

The truth being replaced is: **a `Part` has an integer half-extent `[i32; 3]`,
an integer offset from its parent's pivot, and a quarter-turn yaw about the
vertical** (`isometer-core/src/body.rs:42` to `:102`, `:104` to `:140`);
attachment is flush along one of six facings
(`shared/cosmograph/src/rules/recipe.rs:16` to `:62`,
`development.rs:113` to `:120`). A body is a `BodyDocument` of parts plus a
`BodyPlan` (`body.rs:190`); the sim's `Part` physiology is keyed by part id
(`schema.rs:44` to `:82`).

"Reads geometry" below means it uses extents, offsets or yaw as numbers.
"Reads structure" means only the tree, ids, counts or ledgers. Counts are from
a search of `.rs` files outside `target/`, at the time of writing; they are
indications, not an audit.

### 1.1 Table of dependencies

| Area | What it does | Where | Reads |
|---|---|---|---|
| Building a body | Each child's offset is `flush(parent_half, child_half, facing)`: `±(h_parent + h_child)` on one axis. Sockets mirror left and right when the policy is bilateral; chains use six `ChainFacing`s | `shared/cosmograph/src/development.rs:113`-`:120`, `:123`-`:227` | geometry (six facings, box sizes) |
| The recipe's vocabulary | `Facing` (6), `Anchor`, `ChainFacing` (6), `Template.half_extent`, `Policy{bilateral, preferences by box name, tolerance}` | `rules/recipe.rs:16`-`:233` | geometry, as data |
| The grammar | Unfolds into tagmata; `Branch` takes a `Facing`; `ALONG` is the three positive axes | `rules/grammar.rs:19`-`:50` | geometry via `Facing`; the rest is structure |
| Placement and non-overlap | `free`: axis-aligned separating test, touching allowed; `seat`: try preferred offset then each facing's flush box; `resolve`: for each facing, each living part, first flush box that overlaps nothing | `growth.rs:30`-`:76` | geometry |
| Regrowth, buds, incorporation | The same `free`/`seat`: healing (`harm/healing.rs:32`), buds (`meaning/births.rs:160`), a taken-in part (`growth.rs:64`) | those files | geometry |
| Grafts | Carried part placement: rotate half-extents by yaw, require no overlap with the recipient, "touching is allowed as in growth::free" | `graft/placement.rs:37`-`:75` | geometry (a second copy of the same box test) |
| Part cell lattice | Cells per axis = `|h|/2 + 1`, clamped to 4; at most 64 per part; adjacency by index; no stored coordinate | `mosaic/mod.rs:8`-`:38`, `anatomy.rs:47`-`:51` | extent only; identity is by index |
| Mass and capacity | A part's voxels `(2h+1)` per axis priced at a reference mass; cell mass is `ceiling / capacity` floored | `anatomy.rs:28`-`:75` | geometry (extent) |
| Function scaling | `measure()` sorts the axes: length, area, cross-section, volume; `share_of` = measure x cells / capacity | `anatomy.rs:77`-`:92` | geometry (extent) |
| Part names and roles | `boxed()` reads a box as lump, rod, sheet or point; `name()` adds branch and joint from the tree; `classify` (isometer's) gives Mass, Limb, Plate, Sensor | `anatomy.rs:102`-`:131`; `isometer-core/src/plan.rs:107`-`:129`; `process/mod.rs:361`-`:373` | geometry for the shape; the tree for branch and joint |
| Reach | Furthest living contracting part's pivot distance plus its extent; bulk is root extent plus a constant | `process/mod.rs:380`-`:414` | geometry |
| Mouths and ports | Founding ports seeded from "a part attached to the root with `offset[1] < 0`", Mass or Limb by `classify` | `mosaic/ports.rs:61`-`:105`; `trophic.rs:55`-`:73` | geometry (offset sign, extent) |
| Canopy and producers | Plates hung above their parent with no lateral overhang | `trophic.rs:26`-`:55` | geometry |
| Speed | Legged: leg link lengths; legless: `aabb().extent()` in x and z times a rate | `speed.rs:38`-`:56`; `legs.rs:80`-`:140` | geometry |
| Legs | A chain is found by a convention: terminal foot, upright shin, lateral thigh, all `Role::Limb`, offsets with fixed signs, `half_extent <= 1024` | `isometer-core/src/legs.rs:84`-`:140` | geometry |
| Embodiment (physical placement) | Each part becomes an axis-aligned box in world units by placing its 8 corners through the document and the pose's quarter yaw; work is bounded at 1,048,576 cells a body | `spatial/embodiment.rs:82`-`:130` | geometry |
| Walking, strides, turns, balance, contact, support, swing, footholds, food stands, narrow passages | The rest of the 9,158-line `spatial/` tree, all over part boxes and quarter yaw. Turns sample fixed-point angles (`Rotation::at(yaw, turn)`) but "authored yaw stays exact" | `spatial/embodiment/*`, `isometer-core/src/legs/rotation.rs:1`-`:5` | geometry |
| Balance and centre of mass | Mass proportional to `8 * product(half_extent)`, centre by `centre_of_mass` in `i128` | `spatial/embodiment/support.rs:17`-`:24`; `isometer-core/src/body.rs:355`-`:378` | geometry |
| Sensing | An eye is the centre of a sensing part's placed box, or its placed half-extent if unboxed; then a clear line over terrain | `spatial/foraging/senses.rs:80`-`:110` | geometry |
| Births | Birth clearance reads the body's `aabb()` | `spatial/birth_geometry.rs:48` | geometry |
| Wounds and severing | Tombstoned parts, cascade down the tree, cells lost by index | `isometer-core/src/anatomy.rs:1`-`:40`; `harm.rs:150`-`:190` | structure (cells by index) |
| Matter in a body | Ledgers per part, spread "by what each holds", reserve in store cells | `schema.rs:58`-`:61`; `anatomy.rs` header | structure; extent only for caps |
| Systems (nerves, circulation) | Reach over the part tree; room by capacity | `systems/carry.rs:103`, `systems/nerves.rs:28` | structure plus capacity |
| Hashing | The state preimage carries the body document as is (`body: Option<&BodyDocument>`) | `simulation/preimage.rs:68`, `:114`, `:146` | everything in the document, including geometry |
| Golden digests | Eight authored bodies, plus every generated lineage for seeds 1 to 5, at the recipe's counts and drawn somas 1 to 3; one digest each and one over all, "must not change" | `shared/cosmograph/tests/golden_bodies.rs:1`-`:8`, `:15` | every geometry field |
| Saves and wire | Old formats carry `half_extent`, `offset`, `pivot`, `yaw`; additions use serde `default` plus `skip_serializing_if` so old data "serializes and hashes as before" | `eponym/crates/eponym-play/src/save_wire/body.rs:10`-`:33`; `shared/cosmograph/src/history/read/wire.rs:20`-`:40`; `rules/recipe.rs` throughout | geometry; versioning is the existing pattern |
| Meshing | Per-part voxel volumes generated from role and envelope, content-addressed by `VolumeRef`, greedy-meshed, placed by quarter yaw. The presentation extent is `2h` while mass counts `2h+1` | `isometer-mesh/src/content.rs:1`-`:30`, `greedy.rs`, `live.rs:1`-`:30`; `isometer/src/volumes.rs:1`-`:12` | geometry |
| Views | Scene bodies by `aabb` (Isocosm board, Eponym sources), capsule comparison mode (Mesocosm), implicit SDF renderer | `crates/isocosm-views/src/scene/board.rs`; `eponym-client/src/producer/source.rs`; `mesocosm-genet/src/section/capsules.rs`; `isometer-render/src/implicit.rs` | geometry (a view may swap) |
| Eponym's own contact probe | A float box world with "the integer `Aabb` remains the durable document" | `eponym/crates/eponym-motion/src/contact.rs:1`-`:10` | geometry |

### 1.2 Counts

- `half_extent` appears in 105 `.rs` files (302 lines), 35 of them test files.
  `Yaw` appears in 87 files. These are searches, not call graphs.
- Of the non-test readers, the cosmograph readers that need *geometry* fall in
  five groups: build (development, recipe, grammar), placement (growth, graft,
  healing, births), physical (spatial: 9,158 lines), measures (anatomy, speed,
  reach, trophic, ports, legs), and the hashed documents (preimage, saves,
  golden tests).
- Readers that need only *structure*: the tree walks in `isometer-core`
  `anatomy.rs`, system routing, matter and cell ledgers, tracts by `CellId`
  (`mosaic/mod.rs:10`: "nothing stores a coordinate"), situs `[u8; 3]`
  (segment, index, slot; `development.rs:160`, `growth.rs:111`-`:117`).

### 1.3 What this says

1. **Physical truth is boxes and quarter turns, deliberately.** The embodiment
   header says "Whole native part boxes over the existing terrain ... No
   display volumes participate in physical checks"
   (`spatial/embodiment.rs:3`-`:5`). So the look can already differ from the
   truth, and the SDF renderer and the capsule view do exactly that.
2. **The grammar is not the obstacle; `Facing` is.** The grammar produces
   tagmata, and tagmata name a `Facing` from six values. Radial and spiral
   bodies need a turn that is not one of six; the growth rules would be
   unchanged.
3. **Box sizes carry a lot of meaning.** Role, cells, mass, function strength,
   reach, mouth and canopy are all read off the box. Any alternative that
   drops boxes must say where each of these now comes from. The cheap route is
   to keep a box (or a size triple) as a *measure* and drop it as the *shape*.
4. **A mismatch is already in the tree.** Mass prices `2h+1` voxels an axis
   (`anatomy.rs:28`-`:30`), embodiment and presentation use `2h`
   (`embodiment.rs:95`-`:99`, `isometer/src/volumes.rs:6`). Not a finding
   about this brief's question, but any change that touches extents meets it.
5. **Compatibility has a pattern.** Additive fields with `default` and
   `skip_serializing_if` leave old hashes alone (`recipe.rs:71`-`:76`,
   `:136`-`:158`). The golden digests move only if existing bodies change, so
   an alternative that adds an angle field defaulting to "none" keeps them.

## 2. Alternative conceptions of a body

Each is judged by: what the sim's truth would be, what it allows, what from
section 1 it breaks, determinism, and cost. The sim is integer-only and
hashed (`isometer-core/src/lib.rs:11`-`:20`; `body.rs:12`-`:16`), and
floats stay in presentation
(`isometer-render/src/implicit.rs:3`-`:4`: "none of these floating point
values enters a body save").

### (a) Keep the lattice boxes

- **Truth:** as today. Radial is four-fold at most (Left, Right, Front, Back
  around the vertical); the lattice "shows what it can of the count (four)"
  (D41). An arm count of 3, 5, 6 or 8 is stored in the grammar and realized
  as 4 or as the nearest facing set.
- **Allows:** everything built. Bilateral and four-fold radial are exact.
  Visual richness comes from the look layer (microvoxels per cell, case 1149;
  the SDF renderer).
- **Breaks:** nothing. Spirals and phyllotaxis cannot be placed; a five-arm
  star is a four-arm one.
- **Determinism:** unchanged; the golden digests stand.
- **Performance:** unchanged. Box tests are the cheapest geometry there is.

### (b) Logical structure with turns by angle, realized into the lattice

The body stays a part tree. A child's attachment gains a **turn**, an integer
angle in fixed-point turns, beside or instead of the `Facing`. Radial with n
arms places arm k at `k * 4096 / n` (4,096 units a full turn, the 1,024 per
quarter the legs code already uses, `legs.rs:14`, `rotation.rs:15`).
Phyllotaxis uses the golden angle, 137.5 degrees = 1,564 units of 4,096
(rounded; the rounding is part of the rule). Prior art: L-system turtle
graphics, Fibonacci phyllotaxis, Sims 1994 creatures (the plan.rs header
names them, `isometer-core/src/plan.rs:19`-`:22`).

Two ways to realize it, and they differ in what stays true:

- **(b1) Snap for physics, keep the angle for the look.** The simulator's
  boxes still sit on the lattice at the nearest quarter turn; the document
  also stores the angle, and the look draws the part at the angle. The sim
  keeps every section 1 row working unchanged; a 5-arm star is physically a
  4-arm-ish box set while it draws as five.
- **(b2) Truth at the angle.** Boxes become oriented boxes. Every row marked
  "geometry" in 1.1 changes: `free` and `graft::placement::check` become
  oriented-box tests, `visit_boxes` (`embodiment.rs:82`-`:130`) stops being
  corner-of-an-axis-aligned-box, and walking, balance, contact and
  sensing follow. The fixed-point rotation exists
  (`legs/rotation.rs:19`-`:66`, with `bounds()` returning the axis-aligned
  hull) and is the model: it is exact, uses `i128`, and rounds once.

- **Allows:** any n-fold radial, spirals, curved plates and tails if a chain
  turns a little each link (a tail of ten links at 100 units each curls).
- **Breaks:** (b1) nothing in the sim, but the look and the physical body can
  disagree, which the SDF and capsule views already tolerate. (b2) the
  placement, graft, embodiment and balance rows, about 9,000 lines read, and
  the 35 test files; mouth and canopy tests by "offset sign" need an angle
  test.
- **Determinism:** exact and platform-independent if only integer angle and
  the existing `Rotation` are used. Rounding rules must be written down once
  (rotate then round half away from zero is what `Rotation::rotate` does,
  `rotation.rs:42`-`:50`). The golden digests hold when the angle defaults to
  none.
- **Performance:** (b1) none. (b2) oriented-box tests cost a few multiplies
  more than axis tests; the walking checks are already memoised
  (`embodiment/memo.rs`, `walking/checks.rs`); the bigger cost is cache
  misses from losing axis alignment in clearance counts.

### (c) Skeleton with an implicit surface, rasterized to micro-voxels for presentation

The simulator keeps an integer skeleton (the part tree; each part a segment
with a size) as in (a) or (b1). A presentation step turns the skeleton into a
smooth surface (capsules, rounded boxes, metaballs, a signed distance field)
and rasterizes it into micro-voxels, "squashed" or not. Prior art: Spore's
creatures (a spine of metaballs with limbs as attached parts, float
skinning), Sims, Blender-style metaball skins, convolution surfaces over a
skeleton.

What exists: `isometer-render/src/implicit.rs` defines `Primitive` (rounded
box, capsule) with a CPU distance function shared with the GPU (`:30`-`:75`),
an anatomy-to-primitives path (`implicit/anatomy.rs`), IK chains, a gait
(`implicit/gait.rs`, 236 lines) and a GPU renderer with budgets of 128 bodies
and 8,192 parts (`implicit/gpu.rs:9`-`:10`). `mesocosm-genet/src/section/
capsules.rs` (69 lines) is a capsule comparison mode. Case 1149 already rules
microvoxels in the look (a "number of microvoxels per cell, shaped by its
form"), with "finer body cells than terrain's" to come "where play needs
them" (`design_docs/2026-09-18_wing_design_plan.md:12854`-`:12865`;
`2026-11-02_denizen_plan.md:222`-`:226`).

- **Truth:** unchanged; presentation only. The skeleton may have more
  freedom than boxes (angles) only if combined with (b).
- **Allows:** smooth, organic silhouettes at any resolution; limbs that taper
  and blend; the mordial "squashed micro-voxel" look and a `.vox` export
  (raster the field to a grid, write MagicaVoxel's palette-indexed grid).
- **Breaks:** nothing in the sim. Pick and hit tests must agree with the drawn
  surface: the implicit renderer already shares its distance field between
  draw and pick (`implicit.rs:1`-`:3`).
- **Determinism:** presentation floats are fine for display. An *exported*
  `.vox` or a lineage record that cites a raster is a stored artifact; if two
  machines must produce the same bytes, rasterize in fixed-point (the legs
  code does the same arithmetic in integers: `legs.rs:60`-`:75`, `length` by
  integer square root) or store the skeleton and treat the raster as
  derivable.
- **Performance:** GPU raymarching per frame is built; CPU rasterization to a
  volume is a one-time cost per distinct body (cache by content address, as
  `LiveBodyProjector` does by `VolumeRef`, `live.rs:21`-`:34`).

### (d) Cell-level bodies, grown cell by cell (D32's direction)

The body is a set of cells at integer coordinates, each cell with a function
and matter; a part is a named run of cells, or disappears and the body is a
cell adjacency graph. Prior art: ALIEN (chrxh/alien; a body of connected
"cells" and "particles", with growth by constructor cells) as D32 cites it
(`lineage_rounds_plan.md:231`); cellular automata and Voxel-grown creatures
generally. I know ALIEN's physics is a floating-point GPU particle simulation,
so its determinism model is not this wing's; its *body construction* is the
relevant part.

What exists toward it: each part already has up to 64 cells with identity,
function tracts, lost cells and paid regrowth (`schema.rs:70`-`:81`,
`mosaic/mod.rs`, `harm.rs`). What does not exist: coordinates for cells.

- **Truth:** a map from integer coordinate to cell (function, matter).
  Shape is whatever cells occupy; a part is a label on a connected set.
  Radial, spiral, any shape come free: lay cells where the rule says.
- **Allows:** the D32 behaviours directly: each cell paid for as it is laid,
  wounds at cell grain, a body that is its own record of its growth, and the
  mordial cell, colony, body arc as the same structure at different counts.
- **Breaks:** `Template.half_extent` as the unit of a part; `boxed()` shapes
  (a part has no box; lump, rod, sheet and point would be read from the cell
  set, Bodies 4 allows "read from the box and the tree"); mass by extent
  (becomes mass by cell count, which is simpler: `ceiling` and `cell_mass`
  collapse into one multiply); reach, mouths and canopy (read the cell set);
  the physical stack (collision becomes cell-set versus terrain, which is
  *more* exact than boxes but many more tests); saves and the golden digests
  (a different document).
- **Determinism:** exact if coordinates and sets are integers kept in sorted
  order (a `BTreeMap` as elsewhere in the schema). Every hash and digest
  changes at the cut-over.
- **Performance:** body size grows from tens of parts to hundreds or
  thousands of cells. Today a 100-part body has at most 6,400 cells by the
  lattice bound; hashing, diffing and cloning scale with that, and so do the
  per-tick physical tests unless a coarse hull is kept. This is why (e) adds
  hierarchy to it.

### (e) Nested voxels: a hierarchical tree, subdivide and sum up

This is Mark's lead idea. The structure is a tree in which a voxel
subdivides into N^3 children and a parent summarizes its children, applied to
bodies and to environments alike.

#### e.1 Known forms, and which suits an integer, hashed simulator

These are from general knowledge, not from anything in the repository.

| Form | Idea | Strength | Weakness for this sim |
|---|---|---|---|
| Sparse voxel octree (SVO) | 2x2x2 children, depth d gives 2^d per axis; each node points to its children | simplest; any zoom; natural for "subdivide a voxel" | deep trees, a pointer per level, poor cache use; the same subtree stored many times |
| Sparse voxel DAG (SVDAG) | An octree in which identical subtrees are stored once (hash-consing); the root's id summarizes the content | large savings on repetitive content; **a root id is a content hash**; fits "hashed and diffable" | edit means rebuilding the path to the root and interning new nodes; needs a canonical form |
| VDB-style wide tree | A shallow tree with a large fixed branching (for example 32^3 at the top, 16^3, 8^3 at the leaf), tiles for uniform regions, leaf bricks | shallow (3 or 4 levels), mutable, fast neighbour access, uniform tiles | wide nodes cost memory unless sparse; branching is fixed in the format |
| Brickmap (pointer volume plus brick pool) | One coarse pointer grid; non-empty cells point to dense bricks in a pool | good for GPU traversal | one level of sparsity only |
| "64-tree" (4x4x4 children with a 64-bit occupancy mask, children packed and counted by popcount) | A wide octree variant: a node is one `u64` mask plus a child list | compact, shallow (half the depth of an octree), integer only, and 4^3 = 64 | newer and less documented; not a library to download here |

What the repository already has, by shape:

- `Ground`: a map of **8^3 bricks** keyed `[i16; 3]`, with "a one-material
  brick kept as that material", hashable and diffable
  (`isometer-core/src/ground.rs:5`-`:12`, `:54`-`:63`). That is a two-level
  VDB-like tree with one tile kind.
- `modulus`: a pointer volume plus an 8^3 brick atlas for the GPU
  (`~/.cargo/git/checkouts/seiche-cff5584be5f35834/0c1076b/crates/conatus/
  modulus/README.md`, read; it is Seiche's crate, not under `repos/`).
- `VolumeRef`: a 32-byte content address for a part's voxels, so identical
  parts share one mesh (`isometer-core/src/body.rs:28`-`:39`; `isometer-mesh/
  src/live.rs:1`-`:12`). That is DAG-style sharing at part grain.
- The part lattice: 4x4x4 = 64 cells per part with identity and adjacency
  (`mosaic/mod.rs`). **4x4x4 is exactly the 64-tree node.** A part is
  already one 64-way node; what is missing is children below it and a way to
  address upward.

Which suits this simulator: the integer, hashed, replayed state wants a
**canonical form** (one encoding per content, the rule the uniform-bricks plan
already imposed: "A brick that becomes uniform through edits is stored
uniform, so a ground edited back to its grown state encodes, and hashes, as
the fresh grow does. The replay witness rests on this",
`archive_docs/2026-11-06/2026-11-05_uniform_bricks_plan.md`, Constraints).
A hash-consed tree with the collapse rule "a node whose children are all equal
becomes a tile" gives that for free and makes the state hash the root id.
Branching 4 (64 children) keeps depth low and matches the part lattice;
branching 2 matches the "subdivide a voxel" intuition and doubles the depth;
branching 8 matches `Ground` bricks (8^3 = 512).

**Grammar-grown, self-similar bodies are the favourable case for sharing.**
`Branch` makes side runs "like" their bearer, shorter by `taper`
(`rules/grammar.rs:35`-`:40`), a radial arm repeats n times, bilateral limbs
are mirror images, and the golden digests show that siblings and lineages
share kinds. In a DAG each repeated arm is one subtree referenced n times
(mirror images are a second subtree or a flag on the reference). The 5-arm
star costs one arm, not five. The per-part mesh cache by `VolumeRef` already
exploits the same thing.

#### e.2 What "sum up" means for the simulator: compatible with exact matter

How matter and ledgers are kept (checked):

- `Ledger = BTreeMap<Key, u64>` (`schema.rs:15`): exact integers.
- `Part.matter` is a ledger per part; `Entity.accounts` one per entity;
  `Site.accounts` one per site (`schema.rs:58`-`:61`, `:116`, `:335`).
  Places are not stored; they are derived over the bricks
  (`isometer-space/src/places/mod.rs:1`-`:12`).
- Spreading uses "units left over going to the largest remainders in part
  order" (`anatomy.rs:1`-`:11`). A cell's mass is `ceiling / capacity`
  floored (`anatomy.rs:73`-`:75`), so `ceiling` is not exactly `capacity *
  cell_mass`; the remainder today is dropped, not carried.
- The doctrine's coarse form already conserves a sum: "Lifting samples by
  seed ... restriction conserves the accounts back" (Processes 6,
  `design_docs/doctrine.md:344`-`:346`); matter is "exact stocks over four
  material channels" (`matter/mod.rs:3`-`:5`).
- A territory is already "its matter in each covered place's ledger under its
  key" (Bodies 1, `doctrine.md:779`-`:786`). That is a sum field over places,
  the same shape as a parent summarizing its children.

So a parent's ledger as the exact sum of its children's is compatible with the
sim as it keeps matter, under three rules a plan would have to state:

1. **Sum up is `u64` addition**, exact and order-independent. Cache the sum
   in each node; an edit adds a delta along the path to the root.
2. **Split down needs a fixed remainder rule**: a parent value is shared among
   its N^3 children by a stated weighting, leftovers to the largest
   remainders in Morton order (the same as `anatomy.rs`'s part order). Then
   split-then-sum returns the parent, and sum-then-split is stable.
3. **Mass at a leaf is a measure, not a remainder**: define one cell's mass
   once and compute ceilings as sums, retiring the floored division that
   drops units (`anatomy.rs:73`-`:75`).

What is **not** exact, and not claimed: nonlinear rules (thresholds, rates)
run on a coarse node do not equal the sum of the fine run. The doctrine
already accepts statistical agreement for the crowd (Processes 6: "exact
agreement is a world setting, statistical the default", bound 0.2 per
reading). The same bound would govern running a subtree coarse "where nobody
looks and fine where they do". Conservation of matter is exact; behaviour is
statistical. I did not find a measurement of how the coarse tier fares on
spatial structure (the coarse-step brief is about populations); that is an
open measurement, listed in 6.

#### e.3 One structure for bodies and terrain

What terrain is today:

- `Ground` holds solid voxels as 8^3 bricks of **one-byte material ids**
  (palette indices, at most 63, `ground.rs:32`-`:38`), has no matter amount,
  and is **derived**: `SiteGeometry` keys a cache (capacity 64) on the seed,
  world rules, site conditions and edits and recomputes the volume and places
  (`spatial/site_geometry.rs:11`-`:25`, `:5`). The sim keeps "soil and its
  diffusion" itself, keyed by isometer's columns (Stack 5,
  `doctrine.md:1779`-`:1789`).
- A site is a 16-span lift into columns (`isometer-space/src/lift/profile.rs:
  12`), voxels at 5 per tile across (`VOXELS_PER_TILE` = 5, ruled
  2026-09-15, `design_docs/archive_docs/2026-10-18/
  2026-09-15_board_on_isometer_plan_retired.md:165`-`:190`; I did not find the
  constant by name in the current source with a direct search, so the
  retired plan is the cited source).
- Its store is owed to nisus, "the voxel authority" (Stack 5; place-graph plan
  SP6, `mesocosm/design_docs/2026-10-28_place_graph_engine_plan.md:33`-`:34`),
  which in the checkout I could read is dense chunks plus revision-gated edits
  (`mere/design_docs/archive_docs/2026-10-28/2026-08-22_conatus_engine_plan.md:
  530`-`:539`). The newest nisus is outside `repos/`; I read only older
  checkouts (section 4).

What bodies are: a tree of parts with a content-addressed volume each
(`VolumeRef`) moved by a pose; **bodies move and terrain does not**, so a body
tree is a rigid sub-volume with its own frame and terrain is the one world
frame.

What unifying them would touch:

- A node would need *both* a material (what terrain uses) and an *amount of
  matter by key* (what bodies and sites use). Putting `Ledger` on every voxel
  is far too heavy; the workable form is a ledger on a node only above leaf
  level, and a leaf holding a material and a quantised amount. This is a
  design question for Mark, not a decision here (fork 5.4).
- `Ground` API callers: the uniform-bricks plan counted 25 call sites that
  read `brick_materials` then `brick.raw()` and kept them compiling
  (`archive_docs/2026-11-06/2026-11-05_uniform_bricks_plan.md`, Constraints);
  those are the terrain readers a new structure must serve or replace
  (isometer-lens, isometer-space, eponym-client, mesocosm runtime).
- `isometer-space` (lift, edit, volume, places: about 2,164 lines in
  `volume.rs`, `volume/`, `places/` alone) would read through a node tree
  instead of `Ground`; places derive over columns, so a column query on a
  tree is the new primitive.
- `modulus`'s GPU ABI is fixed at 8^3 bricks and one pointer level; a deeper
  tree means either flattening to bricks for the GPU (cheap: the GPU form can
  stay what it is, derived from the tree) or a new ABI.
- The reverse direction costs less: **keep terrain as is, share only the
  *node convention*** (canonical form, hash-consing, ledger sums), and let
  bodies use it. Terrain and bodies would then agree on the idea and not the
  storage.

#### e.4 How it meets rulings already made

- **D32** (`lineage_rounds_plan.md:231`): growth cell by cell, each cell paid
  as it is laid down, after the eight operators. In a tree a laid cell is a
  leaf insert plus a ledger delta up the path. (e) is (d) with hierarchy; the
  grammar's eight operators then act at any level (a `Branch` at a coarse
  level is a growth of a whole subtree).
- **World 8** (`doctrine.md:1348`-`:1357`) and Bodies 1: "myco and micro
  become territories and surfaces", a territory being matter in each covered
  place's ledger. A territory is a sum field over places; with a tree, a
  place subdivides and a territory's resolution is a choice of level. The
  doctrine already states "micro life is a surface: its matter on places and
  in hosts under its lineage's key".
- **Case 1149** (microvoxels in the look now, finer body cells later,
  `wing_design_plan.md:12854`-`:12865`): (e) is the "finer body cells"
  clause with one added idea, that the fine grain is *nested in* the coarse
  one rather than a separate base unit. Under (e) the sim cell is a node at
  a chosen level and a microvoxel is a node a few levels down; the look draws
  at the level its projected size warrants, which is Presentation 2's detail
  ladder (`doctrine.md:1655`-`:1672`) applied inside a body.
- **Mordial's epoch shift** (`2026-11-15_mordial_brief.md:81`-`:83`, `:95`-
  `:96`): "territory becomes your body", the map zooms out and a lineage's
  territory "collapses into one critter at the next scale". In a tree this is
  **one level up**: the board is the finest level; zooming out wraps the world
  in a new root, so the old board becomes the new finest cell grain and a
  lineage's territory, which was a set of squares, is a set of cells of the new
  body. Its contents (the matter under its key, already summed up the tree)
  decide the parts. The structure is the same at every epoch (cell, colony,
  body); only the root moves. A caution: it requires coordinates that are
  relative to a node (path addresses), not the absolute `[i16; 3]` brick keys
  or `i32` extents used now, or the integers overflow after a few epochs. No
  mordial code exists, so this is a reading of the brief and not a check.
- **Presentation 1** (dimetric, quarter turns): unaffected; the tree is
  cubic and the view is a projection.
- **Bodies 2** (geometry is isometer's body document, physiology the sim's,
  "geometry measures what a mechanic needs"): fits if the tree lives in
  isometer's document and the sim keeps ledgers keyed by node or part id.

#### e.5 What isometer already has toward this

- Volumes and addresses: `Volume` (dense, `size` plus `Vec<u8>`,
  `isometer-mesh/src/volume.rs:17`-`:25`), `Voxels` (palette index per cell,
  `voxel.rs:1`-`:30`), `VolumeRef` (`isometer-core/src/body.rs:28`), generated
  content packs with declared grammar version and exact bytes
  (`isometer-mesh/src/content.rs:1`-`:30`), meshes cached by `VolumeRef`
  (`live.rs:21`-`:34`), `DeclaredExtentVolumes` ("one immutable form per
  distinct declared extent, shared across documents", `isometer/src/
  volumes.rs:14`-`:16`).
- Hierarchy and compaction: 8^3 bricks and uniform collapse (`ground.rs`),
  the brick ABI (modulus), and "marked-face subdivision" of tissue marks in
  the body renderer, a local-voxel density grammar subdividing only marked
  greedy faces
  (`mesocosm/design_docs/archive_docs/2026-10-18/
  2026-09-11_orthographic_voxel_presentation_plan_retired.md:84`-`:86`):
  presentation-only level of detail that exists.
- Not present: a node tree with a summary at each level; relative addressing;
  any "sum" over voxels (checked by search for octree, SVO, VDB, brickmap in
  `shared/` and the sibling repos: no tree implementation found in the
  repositories read).

What Mark has said or ruled about microvoxels (searched the design docs):
case 1149 above; the denizen plan's microvoxel paragraph
(`2026-11-02_denizen_plan.md:222`-`:226`); the 2026-09-15 subdivision ruling
for terrain (one tile = 5 voxels, a step = 2); the mordial brief's keepsake
("a microvoxel presentation, kind of like squashed", `:177`-`:178`). No ruling
yet makes a nested hierarchy the structure.

#### e.6 Costs and risks, concretely

- **Everything physical and hashed changes** if the tree becomes sim truth:
  the embodiment stack (9,158 lines), placement and graft tests, the golden
  digests, saves and the preimage (section 1). If the tree is *derived* from
  the logical structure, none of that changes (fork 5.2).
- **Edits must stay canonical.** A hash-consing table is state that must not
  leak into the hash (the way `Ground`'s dirty set is "not world authority",
  `ground.rs:64`-`:69`). A DAG needs reference counting or generation to free
  nodes; deterministic freeing order matters if any id is hashed.
- **Coordinates.** Relative addressing and `i64` or path keys; the current
  `i32` extents and `i16` brick keys are bounded.
- **Neighbour access is the classic tree cost** (finding the cell next to this
  one across node borders); wide trees and Morton ordering make it cheap, a
  naive octree does not. Physical checks need neighbour access.
- **Mixed resolution in one sim** needs a rule for what a part at a coarse
  node can do (can it be wounded at cell grain?) and a statistical bound for
  coarse dynamics (Processes 6's 0.2).
- **Tooling lag.** The creator's voxel editor, the pack formats and `.vox`
  export work on dense volumes today (`DeclaredExtentVolumes`, `Voxels`); a
  tree needs a flatten to dense at a chosen level for each.
- **Unproven speed.** I found no measurement of tree traversal against the
  current dense and brick forms in this tree. Memory evidence exists for
  bricks (uniform bricks: growth about 30 times faster, memory a third to a
  quarter, `archive_docs/2026-11-06/2026-11-05_uniform_bricks_plan.md`
  Status), none for a deeper tree.

### (f) Capsule skeleton as the physical truth, in fixed-point

Not in the brief you gave me; found while reading. Walking already treats
legs as fixed-point **capsules** (`legs.rs`, `legs/collision.rs`, `Capsule`),
with a stance in integer units of 1/1,024 and an integer square root for
lengths. Make that the general physical truth: each part is a segment with a
radius (or a rounded box), not an axis-aligned box.

- **Truth:** a fixed-point segment and radius per part, placed by a chain of
  integer rotations. Boxes become one special case (a capsule of radius zero
  is a segment; a box a thick capsule's hull).
- **Allows:** the smooth look of (c) *with* matching physics; any angle as in
  (b2); legs, arms and tails without special cases.
- **Breaks:** the box tests of section 1, as in (b2), plus the part
  cell lattice (cells of a capsule need a definition) and `boxed()` shapes.
- **Determinism:** exact if the existing integer routines are reused; the
  precedent is already tested for walking.
- **Performance:** capsule-capsule and capsule-voxel tests cost more than
  box tests but fewer primitives; the walking code already does them.

### Summary of the options

| | Truth | Radial and spiral | Sim rows changed | Hashes | Visual gain | Cost |
|---|---|---|---|---|---|---|
| a | boxes, six facings | four-fold only | none | stand | by look only | none |
| b1 | boxes plus a stored angle | any n, spirals, in the look | placement only for the angle; physics snapped | stand (default none) | high, cheap | low |
| b2 | oriented boxes | exact | placement, grafts, embodiment, balance, sensing | new for angled parts | high | high |
| c | boxes or skeleton; surface derived | with b | none | stand | highest for the look; `.vox` | low to medium; mostly built |
| d | cells at coordinates | any | nearly all geometric rows | new document | high | high; larger state |
| e | tree of nested voxels | any, at any scale | all, if truth; none, if derived | new; root-id hash | highest; scale-free | highest; unmeasured |
| f | fixed-point capsules | any | as b2 plus cells | new | high, matches physics | high |

The options combine. (c) over (b1) or (e-derived) leaves every sim row alone
and delivers radial, spirals and the microvoxel look; it is the cheapest way
to answer Mark's "more visually interesting". (e) as sim truth is the largest
change and the only one that makes bodies, territories and the epoch shift one
structure.

## 3. A note on determinism and floats

- The sim is integer; `isometer-core` states that "Nothing here is a float, so
  a body's derived quantities are bit-identical on every platform"
  (`body.rs:12`-`:16`).
- Floats exist in presentation (`implicit.rs`, GPU) and in Eponym's contact
  probe (`eponym-motion/src/contact.rs:7`-`:10`, `FIXED_DT_SECONDS: f32`),
  which says the integer `Aabb` stays the durable document.
- So any alternative can use floats for drawing; the question is whether a
  *stored* thing (a save, a lineage record, a `.vox`, a digest) depends on a
  float. The rule that has worked: derive the look from integer truth and key
  caches by content address.

## 4. What the stack already offers

Read by path and by search; no builds. "Hit" means something an alternative
could reuse.

| Where | Offers | Used by | Alternative |
|---|---|---|---|
| `shared/isometer/crates/isometer-core/src/legs.rs`, `legs/rotation.rs`, `legs/collision.rs` | Fixed-point (1,024 a unit) capsule geometry and rotation by any angle; `i128` and integer square root; exact authored yaw | walking, balance | b, f |
| `shared/isometer/crates/isometer-core/src/ground.rs` (and `ground/`) | 8^3 bricks, uniform collapse, canonical encoding, revision and dirty set | terrain, places, renderers | e |
| `shared/isometer/crates/isometer-core/src/body.rs`, `plan.rs`, `anatomy.rs` | The part tree, severing, `Symmetry::{Bilateral, Radial, None}` (an enum in isometer's plan already, `plan.rs:142`-`:150`, "Radial: repeated around the vertical axis"), `classify` | sim, products | a, b |
| `shared/isometer/crates/isometer-render/src/implicit.rs` and `implicit/` | Rounded-box and capsule primitives, CPU and GPU distance field, IK and gait presentation | Eponym, Mesocosm views | c, f |
| `shared/isometer/crates/isometer-mesh/src/` (`content.rs`, `greedy.rs`, `live.rs`, `volume.rs`, `voxel.rs`) | Generated part voxels, greedy meshing, cache by `VolumeRef`, dense volumes and palette voxels | body renderers | c, e (flatten) |
| `shared/isometer/crates/isometer-space/src/` (`volume.rs`, `places/`, `lift/`, `edit/`) | Terrain derivation, places, passages, edits over `Ground` | sim site geometry | e (terrain half) |
| `shared/cosmograph/src/mosaic/mod.rs` | A part's 4x4x4 cell lattice by index; tracts; connectivity | sim | d, e |
| Seiche checkout `crates/conatus/modulus` and `nisus` (outside `repos/`; read in `~/.cargo/git/checkouts/seiche-*/0c1076b/`) | Sparse 8^3 brick ABI with WGSL traversal and a CPU trace; revisioned voxel chunks. Older checkouts only; nisus as the owed voxel authority was not available to read | terrain renderers | e |
| `repos/mere` (`crates/domain`, `graph`, `system`) | Graph kernel, linked data, catchment, registry: no voxel or surface code found by search | n/a | none found |
| `repos/netrender` | A paint-list 2D renderer; the search found no voxel, SDF or tree code beyond a clip-rectangle shader | UI | none found |
| `repos/genet` | A browser engine; nothing relevant found | n/a | none found |
| `repos/wgpu-cue`, `wgpu-graft`, `wgpu-scry`, `wgpu-weld` | Webview and texture embedding for wgpu hosts | hosts | none (not geometry) |

Searched, in `seiche`, `mere`, `netrender` and `genet` source and docs, for
octree, SVO, sparse voxel, brickmap, VDB, microvoxel, marching, SDF, metaball,
surface nets, dual contour: the hits were unrelated (a clip rectangle, web
test documents). Absence is evidence only as far as that search; I did not run
a positive control beyond finding `modulus` and `Ground` by their known terms.

**Reading for Mark's rule** ("no app-local copies of stack capabilities;
growing the stack with a new component is fine"): isometer is the right home
for a body tree, fixed-point rotation and capsules (it owns geometry, Bodies
2); nisus (Seiche) is the right home for a voxel tree store, since SP6 already
owes it the terrain. A tree used by bodies and terrain belongs to a shared
component, probably a new crate beside `nisus`, with isometer and the sim
depending on it. That is a stack decision, and so a fork (5.6).

## 5. Forks for Mark

Each has two to four options, with what each commits to. Recommendation
first. Evidence in a line.

**5.1 What is the simulator's truth about a body's shape?**
Evidence: physics reads 9,158 lines over boxes (1.1); the look can already
differ from it (embodiment header; implicit SDF; capsule view); 105 files
mention `half_extent`.
- **A. Keep integer boxes and quarter turns as truth; add an angle as
  data for the look and the grammar (b1) over a skeleton-derived surface (c).**
  Commits to: radial n-fold and spirals are real in the document and the
  drawing, and physically snapped to the lattice; no hash moves.
- B. Oriented boxes or fixed-point capsules as truth (b2 or f). Commits to:
  rewriting placement, grafts, embodiment, balance and sensing; new hashes
  for angled bodies.
- C. A nested-voxel tree as truth (e). Commits to: the largest change, with
  the largest unification.
- D. Stay at (a): four-fold radial only.

**5.2 If a nested tree is wanted, is the tree the sim's truth, or derived from
the logical structure?**
Evidence: the logical layer (part tree, cell counts, ledgers) is already
separable and does not read coordinates (1.2); terrain is already derived and
discarded (`site_geometry.rs`), and the golden digests pin today's bodies.
- **A. Derived first: the tree is a deterministic function of the part tree
  and the grammar (like terrain is of its seed), keyed by content address,
  never hashed into state; promote it to truth later if play needs
  wounds or collision at cell grain.** Commits to: no migration now; the
  tree can be thrown away; sum-up exists as a reading.
- B. Truth now: the tree holds the matter and the cells; the part tree
  becomes a view. Commits to: new state, hashes and saves, and the physical
  rewrite.
- C. Truth for microbes and territories only (mordial, World 8's surfaces),
  derived for animals. Commits to: two representations that must meet at the
  epoch shift.

**5.3 Branching factor, if a tree is built.**
Evidence: a part's lattice is 4x4x4 (`mosaic/mod.rs:36`-`:38`); `Ground`
bricks are 8^3 (`ground.rs:21`); `modulus` atlas bricks are 8^3; a
2^3 octree is twice as deep as 4^3 for the same reach.
- **A. 4x4x4 (64 children, a `u64` occupancy mask).** Commits to: reuse of the
  part lattice as one node. Terrain's 8^3 bricks are not a power of 4, so
  they would meet this tree as a 2^3 level over 4^3 leaves, or stay a
  separate store (fork 5.5).
- B. 2x2x2 (octree). Commits to: the simplest "subdivide a voxel" and the
  deepest trees.
- C. 8x8x8. Commits to: matching `Ground` and `modulus` bricks exactly,
  leaves of 512.
- D. Mixed by level (wide at the top, narrow at the leaf, as VDB does).
  Commits to: more parameters to fix and to hash.

**5.4 How many levels, and what a leaf is.**
Evidence: terrain voxels are a material id of at most 63 values and no amount;
body and site matter is a `u64` ledger by key; a base unit of 5 voxels to a
tile is ruled (2026-09-15).
- **A. A leaf is a cell: a material or function plus a quantised amount; only
  nodes above leaf keep a full ledger sum. Levels are a world setting, with a
  sim floor at today's cell grain and look levels below it.** Commits to:
  "sim at cell grain, look finer" (case 1149) with the tree explaining how
  they relate.
- B. A leaf is a microvoxel in the sim too (the sim floor is the finest
  level). Commits to: far larger state and per-tick cost.
- C. Fixed depth, say three levels, in the format. Commits to: simplicity and
  a hard bound on cost.

**5.5 Do terrain and bodies share one structure?**
Evidence: terrain is derived, bodies are state; terrain has a material and no
amount; `Ground` has 25 reader call sites (uniform-bricks plan Constraints),
and its store is owed to nisus (SP6).
- **A. Share the node convention and the crate, not the storage: bodies and
  territories first; terrain adopts it when nisus takes the store (SP6).**
  Commits to: one idea, two stores, a convergence point already owed.
- B. One structure now: terrain and bodies in the same tree. Commits to:
  rewriting `Ground` readers and `isometer-space`, and a node that carries
  material and amount.
- C. Separate: bodies get a tree; terrain stays as is. Commits to: no
  unification; the epoch shift and territory-to-body stay a special case.

**5.6 Where does the component live?**
Evidence: isometer owns body geometry (Bodies 2); nisus is the owed voxel
authority (Stack 5, SP6); the engine split kept `nisus` and `modulus` as their
own crates "because each has more to become" (mere `decisions.toml:830`-
`:833`).
- **A. A new small crate in the Seiche family beside nisus (or inside nisus),
  with isometer and Cosmograph depending on it.** Commits to: no app-local
  copy; a stack change in a repo outside this tree.
- B. Inside isometer-core now, moved to nisus at SP6. Commits to: a move
  later; speed now.
- C. Inside Cosmograph. Commits to: an app-local copy of a stack capability,
  which the rule discourages.

**5.7 Radial symmetry now, while this brief waits.**
Evidence: D41 builds the operators "now on the branching structure"; radial
repeats a run round the vertical and stores an arm count; today's lattice shows
four.
- **A. Store the arm count and the angle in the grammar and the document now
  (default none); realize four-fold on the lattice and the rest in the look
  (b1).** Commits to: no hash change; arm counts other than four draw right
  and place approximately.
- B. Four-fold only until a body-shape ruling. Commits to: arm counts above
  four unavailable.
- C. Wait for the body-shape ruling before building radial. Commits to:
  radial late.

**5.8 What does the microvoxel look come from?**
Evidence: case 1149 rules microvoxels in the look; the implicit renderer and
the capsule view exist; the mordial brief wants a `.vox` export and "squashed"
voxels.
- **A. Rasterize a skeleton-derived surface (c) to micro-voxels at the level
  the view warrants, in fixed-point where the bytes are exported.** Commits
  to: one pipeline for the screen and the `.vox`, determinism only where it
  is stored.
- B. Generate voxels per part by grammar as today (`content.rs`) at a finer
  grain. Commits to: part-level forms, no smooth joins.
- C. Take the voxels from the tree (e). Commits to: the tree first.

**5.9 Measure before ruling?**
Evidence: no tree traversal or memory measurement exists in this tree; the
uniform-bricks plan set the precedent of testing at "10x, 100x, and 1000x"
before building (its D1).
- **A. Yes: a small throwaway measurement of the tree against `Ground` and a
  dense part volume, at those scales, before fork 5.2 is ruled.** Commits to:
  a measurement lane, no wiring.
- B. Rule from the reading alone. Commits to: deciding without the numbers.

## 6. Unverified, and what to check next

- **Not run or built.** No test, build or probe was executed, so nothing here
  is a runtime verification; Mark's rule (prefer runtime over static tracing)
  is unmet for the cost estimates in 1.2 and e.6.
- **Counts are greps.** The 105 files and 302 lines are `half_extent` text
  matches; call-graph counts need `sem_impact` or rust-analyzer.
- **Not opened** (named from imports or listings only): `isometer/src/
  volumes.rs` beyond its header, `isometer-mesh/src/greedy.rs`,
  `isometer-render/src/implicit/gait.rs` and `chains.rs`, most of
  `spatial/embodiment/*` beyond `embodiment.rs`, `support.rs` and the grep
  lines, `eponym-client` producer sources, `isocosm-views` scene board.
- **`VOXELS_PER_TILE`** was not found by a search of the current source; its
  only citation here is the retired board plan.
- **Newest nisus** was not available locally (`repos/` holds no Seiche); the
  nisus description is from an archived Mere plan and an older checkout.
- **Coarse dynamics on spatial structure** are unmeasured; the coarse-step
  brief's bounds are for populations and cohorts.
- **Mordial** has no code; the epoch-shift reading in e.4 is of its brief.
- **No external prior art was consulted.** SVO, SVDAG, VDB, 64-tree, ALIEN,
  Spore, L-systems and phyllotaxis are described from memory; any figure or
  claim about them should be checked before it is cited in a ruling.

## Answers (Mark, 2026-11-15)

Recorded in the words given; not yet numbered cases. The forks' numbers are
section 5's.

- **5.2 and 5.9, truth or derived.** "Derived first, measured, then decide
  (Recommended)": the tree is computed from the part tree and grammar and
  never saved or hashed; a small throwaway measurement at 10x, 100x and
  1000x sizes comes before any promotion to truth.
- **5.3, branching.** "4 x 4 x 4 (Recommended)".
- **5.5, terrain and bodies.** "1 makes sense (distinguishes world or world
  class entities from normal entities), but what're the implications of 2?"
  Option 1 was the same node format and code with separate stores; option 2
  one store for everything. Taken as option 1, with the implications of one
  store answered in conversation and summarized here: terrain would have to
  become saved state holding amounts; a body that moves would rewrite the
  shared tree every step unless it is held as its own subtree by reference,
  which is separate stores again; the gain is that eating ground, dying into
  it and depositing on it become one operation, which a shared node format
  also allows by moving a subtree between stores.
- **5.6, home.** "A new small crate beside nisus (Recommended)". Seiche's
  current state has not been read; that comes first.

Still open: 5.1 (the sim's truth about shape), 5.4 (levels and what a leaf
is), 5.7 (radial while this waits; lineage rounds D41 stores an arm count),
5.8 (where the micro-voxel look comes from).

### 2026-11-16

- **5.1, the sim's truth about shape.** "Decide after the nested-voxel
  measurement".
- **5.4, levels and the leaf.** "A leaf is a cell; levels are a world setting
  (Recommended)": the smallest voxel is today's cell, everything above sums
  its children exactly, and the simulator never goes finer than a cell.
- **5.5, confirmed.** "Yes, separate stores (Recommended)".
- **5.8, the look.** Shown an illustration of one made-up body drawn as a
  soft surface cut into micro-voxels and as one cube per cell
  (`Code/testing/isocosm/look_compare_2026-11-16.png`): "3 appeals to me if
  we get 2 for free either way. Are there other [looks] that could be
  derived, cause if so, we could consider the microvoxel look just
  essentially a lens?" Option 3 was both, as a view setting. So a look is a
  lens over one body, and none is the simulator's truth; `isometer-lens`
  already holds that boundary ("Simulation authority stays outside this
  crate"). Asked which lenses come first, he chose all four offered: cells
  (the derived tree drawn directly), soft micro-voxels (rounded along whole
  runs, since rounding each box pinches stacked segments), function and
  matter maps, and the flat sprite. Named and not chosen yet: a smooth
  uncut surface, a silhouette or glyph, a skeleton of runs and joints, and
  sound.
- **Owed work.** "One Sonnet lane for both, now (Recommended)": a read of
  Seiche's current state, then the throwaway measurement at 10x, 100x and
  1000x in scratch space outside every repository.

### 2026-11-16, after the first measurement

A lane measured a 4x4x4 tree against dense per-part arrays, a tree with
shared subtrees and 8x8x8 bricks, on synthetic bodies shaped like the tips
preset from 864 to 864,000 cells (`Code/scratch/nested-voxel-measure`,
`results_run1.md` and `results_run2.md`; one configuration rerun and
matching). Memory was counted from node sizes, not measured. At 864,000
cells in 144,000 small parts: dense 9.9 MiB, tree 8.0 MiB, shared tree
9.7 KiB, bricks 38.6 MiB; a stored root sum reads in 0.4 ns against 1.2 ms
summed from dense cells; a region sum is 9.6 us against 164 us; an edit with
sums kept is 66 ns against 5 ns with none. With 144 large parts dense was
about 4x faster on region sums and slightly smaller. Sharing fell from about
1000x on identical arms to about 10x with one cell in 32 changed and to
nothing on irregular content, and made an edit about 30x slower. A tree
rebuild is 50 to 580 us at 8,600 to 108,000 cells and 4.5 to 10 ms at
864,000.

Seiche, from the one local checkout (`Code/worktrees/seiche-history`,
`a6b8641`, 2026-11-07, possibly behind its remote): `nisus` is dense
revisioned voxel chunks with no tree, sums or amounts; `modulus` is 8x8x8
bricks of one-byte material and is the GPU contract; a new crate would sit
under `crates/conatus/`.

- **Truth or derived.** "I am alright with 1, but wonder if a plant doesn't
  capture maybe some of the costs that could be mitigated by saving and
  hashing the nested tree structure. but I also wonder if there's prior art
  on this that might lend some insight into this decision." Option 1 was
  stay derived. Unmeasured, and where a saved, hashed tree would pay: a
  population of near-identical bodies, hashing per tick, sharing between
  saves and ticks, and comparing two worlds by hash. Prior art given from
  memory, unchecked: Hashlife, OpenVDB, sparse voxel DAGs and HashDAG,
  Dreams, Teardown, Minecraft, Merkle trees; a tree is truth where content
  changes slowly or regularly and geometry is derived where it changes
  constantly.
- **A second measurement.** "Yes, with real bodies and a population
  (Recommended)": the real preset and a founded lineage's members, about 150
  bodies changing a little each tick, measuring per-tick hash cost, save
  size across ticks and diff cost against today's whole-document hashing.
- **5.1, the sim's truth about shape.** "Wait for the second measurement".
- **Sums.** "A sum in every node (Recommended)".
- **Sharing.** "1, probably world-class entities (rarely change)": the plain
  tree first, with shared subtrees as an optional layer, switched on for
  world-class entities.
- **Bricks.** "Why not change modulus to better suit different, recursively
  derived sizes?" Then "Yes as a direction; assess first (Recommended)":
  modulus is to draw nested 4x4x4 nodes in place of bricks; a read of its
  shader and real consumers comes first and becomes a plan in Seiche; until
  then the tree is flattened to bricks.

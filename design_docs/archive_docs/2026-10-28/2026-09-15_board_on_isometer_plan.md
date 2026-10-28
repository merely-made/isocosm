# The board on isometer

**Date:** 2026-09-15

**Status, 2026-10-18:** The authored board and its paging are implemented: the
lanes landed 2026-09-16 and paging was integrated 2026-09-27. The sim-off path
is `MapTerrain`; case 799 adds a lifted-site sim-on path under V2, which the
VTT overlay owns, while this plan owns its scene lowering, selection and
presentation gates. Open are that V2 path, the DOM board's fate and the switch
(section 5, decision 4, held open) and Genet text acceptance. Mere's tenant is
the selected body rasteriser (case 796), with product adoption recorded by the
presentation plan.

**W1, 2026-09-18:** rewrite. Tier: mixed, stack rendering and game overlay.
The lanes landed and the code is good; the done-conditions were parity with
the DOM board and a constant, and §3.6 rewrites the geometry under them.
Rewrite is a lane under the record's W2 or W3; until it lands this plan's
done-conditions are not authoritative. Evaluated against the wing design
record; see
[mesocosm/design_docs/2026-09-18_wing_plan_evaluations.md](../mesocosm/design_docs/2026-09-18_wing_plan_evaluations.md)
§1.

**Owns:** drawing Isometry's board through the wing's shared scene, `isometer`,
instead of one DOM element per tile, prop and token, while keeping the locked
2:1 isometric lens, the tileset-as-stylesheet contract, and every tile and
token selectable as today. It is the third consumer the presentation plan's
lane L4 and the isometer family plan both name, and the one that tests whether
isometer is neutral rather than Mesocosm-shaped.

**Does not own:** camera freedom (the locked lens stays the shipped 2D lens;
2.5D and 3D lenses need their own plan, per CLAUDE.md), the session model,
system plugins, campaign packs, or isomere (the wing GUI layer, designed
separately).

**Consumes:** `mesocosm/design_docs/2026-09-11_orthographic_voxel_presentation_plan.md`
(rulings 1, 5, 6, 12 to 15; lanes L4, L5, L6), the isometer extraction and
family plans of 2026-09-14, and the watchtower plan's board receipts.

---

## 1. The board as it draws today

One absolutely positioned element per ground tile, exposed cliff face, prop,
marker and token inside `.board`; the container's inline offset is the
camera; depth is `isometry_core::depth_key` written as `z-index`
(`crates/isometry-views/src/board.rs:1-7`). Tiles are diamonds by `clip-path`,
faces are trapezoids by inline polygon, tokens are 24 by 36 boxes whose
`background-image` is a sprite baked by `isometer_mesh::bake` from a voxel
recipe and palette-swapped into `.token-<sprite>` rules
(`crates/isometry-views/src/theme/tokens.rs`). Identity is closure-captured
per element (`tile_el` calls `ui.click_tile(at)`, `token_el` calls
`ui.click_token(id)`); nothing on the DOM names a tile, and two geometric
fallbacks already exist for the pane (`UiState::tile_at_pane`,
`token_drag_candidate`). The class vocabulary `tile-<kind>`,
`token-<sprite>`, `prop-<kind>`, `cond-*`, `beat-*` is the modding contract
(PROJECT_DESCRIPTION pillar 3).

Receipts: 632 board elements per steady frame in the 2026-09-06 headed
session; M4 layout median 36.8 ms; atlas interaction pointer 1.9 ms median;
the clip receipt `board_tile_clips_its_hit_area_to_the_visible_diamond`
defines what selectable means; 363 root tests with all features.

## 2. What isometer offers, and where it is still Mesocosm-shaped

Neutral today: `SlabCamera` (free forward, world up fixed), `Cutaway`,
`SceneBody` with `Pose` and continuous yaw, `SceneVolumes::{Voxels,
DeclaredSolid}`, the glyph batch on the shared depth, `pick_ndc`,
`presentation_bounds`, `part_bounds`, `capture`, `SceneSource` and
`SceneProducer` with the sRGB straight-alpha contract, `SubjectKey(u64)`.

Assumptions the board runs into, each with the smallest addition that
removes it:

| Assumption in isometer | Where | Addition |
| --- | --- | --- |
| Terrain must be a `BrickMap`; `HostTerrain` takes a whole map and `GroundTerrain` a `Ground` | `scene/terrain.rs`, `lens/bricks.rs` | None for structure: implement isometer-core's `Terrain` trait over the height field and grow a `Ground`, so the tracer, depth join and cutaway work unchanged (the peer's route). Extent becomes a square bound; the host offsets. |
| Terrain materials are soil, rock and unknown, hard-coded in the tracer shader | `isometer-lens/src/tracer.wgsl:87-97`, `TerrainAppearance` | **I1, material palette:** the trace takes a bounded material-to-colour table (u8 index, up to 64 entries) instead of three fixed slots, and `Ground::grow` takes a material sampler so a tile kind becomes a brick material. Mesocosm's two-material appearance is entry 2 and 3 of the same table; its receipts stay byte-identical by construction. |
| Picks address bodies only; a terrain hit is an occluder with no identity | `query.rs`, `scene.rs::terrain_ray` | **I2, terrain hit:** `pick_ndc` answers `Pick::{Body(BodyPick), Terrain(TerrainHit { position, brick, normal, distance })}` and the host maps position to column, row and elevation with its own iso math. `depth_key` is retired as an ordering rule. |
| The scene renders at the leaf's physical size; no internal resolution or nearest upscale | `producer.rs` | **I3, integer render scale:** the producer takes a render scale so the scene draws at a low internal size and the leaf presents it nearest-neighbour; the host owns the pixel grid it wants. |
| No sprite path; tokens are meshed bodies or nothing | `bodies.rs`, `glyphs.rs` | **I4, tokens as live bodies:** `isometer_mesh` gains `Voxels` to `Volume` (the bake lane's palette-index cells become a mesh-lane volume with a material per palette entry) and a one-part `BodyDocument` builder, so a voxel recipe is drawn live under the same camera as the terrain. Ruling 6 wants live faces first; the bake stays the far and many-token tier for a later hybrid. |
| Cutaway reaches bodies only; terrain is cut by a filtered map rebuild | `camera.rs::clip`, `bricks.rs::from_ground_filtered` | None: a focus elevation is a keep predicate over the grown Ground, rebuilt on focus change, as Mesocosm's terrarium does. Cost is one full upload per focus change; measured in B5. |
| `SceneFrame` carries Mesocosm's capsule fallback and `dirty` brick keys | `scene.rs` | None for this lane; the board leaves capsules `None` and reports dirty bricks from `Ground::drain_dirty` after an edit. Noted under §6. |
| The 2:1 dimetric angle is a forward vector, but nothing pins it | `camera.rs` | **I5, camera preset:** `SlabCamera::dimetric_2_1(centre, half_height, aspect, depth)` with a test that unit tile diagonals project 2:1 and an elevation step projects to the board's `elev_step` ratio. |

Every addition is tested inside isometer against its existing receipts:
Mesocosm's bench acceptance and the spatial-coverage and l9-terrain-parity
sets must stay byte-identical, since none of them uses the new inputs.

## 3. The board through the scene

`crates/isometry-views` keeps the map, state, editor and session code and
gains a scene adapter; `isometry-core` stays pure and keeps iso math and
`depth_key` for any DOM overlay that still wants a paint order.

- **Terrain adapter.** `MapDocument` (elevation grid, ground and prop layers,
  tile kinds) implements isometer-core's `Terrain`: surface is the elevation
  in voxel units (one voxel per `elev_step`), cavities are none, the material
  sampler maps a cell's tile kind to a palette index and the kind name to a
  colour through the tileset stylesheet's existing colour rules, so the
  stylesheet still decides looks. Props that are voxel recipes become bodies;
  props that are flat sprites stay DOM for now and are counted.
- **Token adapter.** Each token becomes a `SceneBody` with a one-part
  document from its recipe (I4), pose at the tile's world position with yaw
  from its four-way facing, `SubjectKey` from `TokenId`, tint from its
  owner. Conditions and beats stay CSS on the DOM control that ruling 14
  keeps optional, positioned by `pixel_of` over the body's bounds.
- **Camera.** The I5 preset, centre from the pan, `half_height` from the
  viewport in tiles, `Cutaway::Bounds` from the map box, a focus elevation
  as the filtered-map predicate; render scale from the board's pixel grid.
- **Viewport.** One `custom_leaf` per board pane with a `SceneProducer` over
  the adapter, registered exactly as Mesocosm's bench and Paredros's session
  do; the pane's wheel and pointer handlers stay.
- **Selection.** A pointer press resolves through the leaf to `pick_ndc`:
  a body pick is a token, a terrain hit maps to a tile through
  `screen_to_tile`'s inverse on the hit position, and the two existing
  fallbacks are retired once the pick agrees with them under test. The
  editor's nine modes and the session's local and remote branches are
  unchanged; only how a click finds its tile changes.
- **Overlays.** Reach, path, template, fog shroud, selection and doors are
  the presentation plan's glyph and material channels: reach and path as
  material tints on the terrain palette (I1 gives the slots), selection and
  templates as glyph strokes or a tinted material until isomere owns
  overlays. Markers and the context menu stay DOM.
- **Switch.** The DOM board and the scene board coexist behind one flag
  until parity, so every existing receipt can run against both.

## 4. Lanes and done-conditions

Isometer additions first, each a Terra lane touching `shared/isometer` only,
with the family's receipts unchanged; then the board.

- **I1 material palette.** Done when the tracer draws a 16-material fixture
  with each material at its table colour, Mesocosm's habitat and classic
  frames are byte-identical, and `Ground::grow` with a sampler yields the
  same bricks as today for the soil and rock sampler.
- **I2 terrain hit.** Done when a pick over open ground returns the brick
  and world point under the pixel, a pick over a body still returns the
  body, a pick through a ridge returns the ridge, and the existing pick
  receipts pass.
- **I3 integer render scale.** Done when a scene requested at scale 4
  produces a texture a quarter the leaf's size that the leaf presents
  nearest-neighbour with no interpolation, and scale 1 is byte-identical to
  today.
- **I4 tokens as live bodies.** Done when a bake-lane `Voxels` becomes a
  mesh-lane `Volume` with one material per palette entry, a one-part body
  from it draws under the scene camera, and its baked sprite and its live
  render agree on silhouette coverage within the bake's existing tolerance.
- **I5 camera preset.** Done when unit tile diagonals project 2:1 and an
  elevation step projects to `elev_step` over `tile_h` within one pixel at
  the board's scales.
- **I6 body material palette** (found by I4, 2026-09-15). The live
  renderer colours a quad by a hashed material colour times the body's
  tint and takes no colour table, so a token from a recipe draws with the
  right silhouette in the wrong colours. Done when a `SceneBody` can carry
  a per-material colour table (the palette shifted by one, as
  `material_colours` gives it), the renderer samples it per material with
  the body tint as a multiplier, a body without a table draws exactly as
  today, and a rendered token's colours match its baked sprite's at every
  covered pixel.
- **B1 terrain adapter (Isometry).** Done when the watchtower map grows a
  Ground whose surface matches the elevation grid at every cell and whose
  materials match the tile kinds, in a headless test.
- **B2 scene board behind the flag.** Done when the scene board renders the
  watchtower map and its tokens, the DOM board still renders, and the two
  agree on which tile is under every one of a grid of probe pixels (the
  parity gate: same tiles selectable, not same bytes, since the renderers
  differ).
- **B3 selection and interaction.** Done when every host_routing, host_zoom
  and watchtower receipt passes on the scene board, including the clip
  receipt, and the geometric fallbacks are retired.
- **B4 overlays and edits.** Done when reach, path, selection, fog, doors
  and encounter sites are visible on the scene board, an elevation edit
  reaches the next frame through `drain_dirty`, and a focus elevation hides
  the layers above it.
- **B5 receipts and the switch.** Done when the scene board's frame profile
  is recorded beside the M4 and atlas numbers on the same machine, the
  headed session shows zero board elements emitted, and the DOM board is
  archived with rationale or kept behind the flag by Mark's ruling.

## 5. Decisions for Mark

1. **Tokens live or billboarded.** The plan takes ruling 6 at its word:
   live bodies from recipes (I4), sprites kept for a later far tier. The
   alternative is a camera-facing image batch beside the glyph batch, cheaper
   and closer to today's look, and it is what a hybrid would add later.
2. **Pixel grid ownership.** Internal resolution and integer scaling move
   from the DOM's `board_scale` to the producer's render scale (I3). The
   GBA crispness pillar then holds inside the scene rather than in CSS.
3. **What overlays wait for isomere.** Reach, path and selection tints can
   land through materials now; richer overlays (range templates, facing
   arcs, labels) are isomere's and would otherwise be built twice.
4. **The DOM board's fate** after parity: archive, or keep as a downlevel
   path.

## 6. Improvements noticed in isometer, recorded for its owner

- `Cutaway` cuts bodies only; terrain needs a whole-map rebuild per focus
  change. A slab plane in the tracer would make the cutaway one concept.
- `TerrainAppearance` and the tracer's material switch are a two-material
  palette; I1 generalises it, and the `unknown` colour becomes table entry 0.
- `HostTerrain` has no incremental path; a host that authors bricks needs
  slot refresh, which `GroundTerrain` gets for free.
- `SceneFrame::capsules` and `SceneHost` are Mesocosm's, marked as seams of
  the extraction; a third consumer implements them as no-ops.
- `always_visible` is named for Mesocosm's controlled critter; a focus
  policy (presentation plan open decision 1) would replace it.
- The heightfield march and chain critter sit unconsumed in isometer-lens.
- No producer-side pixel scaling (I3) and no camera presets (I5).
- The bake's four facings versus the scene's continuous yaw: the recipe's
  `Clip` vocabulary is reserved but empty.
- Found by B5, 2026-09-16, with the cap measured: a ground is grown before
  its brick map is built, so a board past `MAX_BRICKS` pays the whole regrow
  (695 ms debug, 79 ms release at 70 by 70) and *then* is refused. Nothing asks
  "will this fit" ahead of the growth, and nothing reports a brick budget, so
  a host cannot warn an author before the board is unusable. Nor does the
  family publish the bytes one brick uploads: a host pricing a board has to
  divide a tracer receipt by its own brick count to find 1,016. And §8's
  measurement of the grow — 4 to 9 ms release, 36 ms debug, for the demo's
  121 by 121 columns — is the price of `Ground` having no material write, paid
  in full on every one-tile tint.
- Found by B1, 2026-09-15: `Ground` has no public material-at-voxel
  accessor, so a consumer re-derives the brick and local index by hand;
  a `material(&self, at)` beside `solid` would retire that copy. The
  void is implicit: `Terrain::surface` says "nothing here" by returning
  a y below zero so the growth range is empty; an `Option` or a stated
  line on the trait would make it part of the seam. `MAX_MATERIAL` (core,
  63) and `MAX_TERRAIN_MATERIALS` (lens, 64) are one bound counted twice
  with no compile-time link.

## 7. Findings

- The family-plan risk that the Isometry root pins an older mere is closed
  as of 2026-09-15: root, isometer and both products share mere 876320fd and
  genet 5ae30cad.
  *(2026-09-27: all moved together to mere 0418391f, genet 0cf4f30b and netrender c8c09f16, with the toolchain
  at 1.98.1 (wing design record rulings 292, 297 to 300, 321, 327).)*
- No Isometry-side document cited the L4 done condition before this plan.
- `isometry-runtime`, which holds the earlier fixed-isometric GPU tenant,
  is excluded from the root workspace and untouched by this plan.
- **2026-09-26, a paging assessment (Lane E, ruling 282):** paging already
  exists at the pinned mere and is wired nowhere here. `modulus`'s
  `BrickMap::with_capacity`, `retarget` and `refresh` page a fixed-capacity
  atlas, `isometer-lens` wraps them (`bricks.rs:95,105`), and the tracer
  uploads only newly loaded slots (`tracer/residency.rs:97-161`, receipt at
  `tracer_tests.rs:665`), so the board can page inside this repository.
  Three facts bound it. The resident cap stays 2,047 bricks: in the headed
  pane (972 by 820) a flat 256-tile board needs 1,345 visible bricks and
  fits, one with 0 to 7 relief needs 2,063 and does not. A defect in
  `modulus` at 876320fd, unchanged on mere's main *(fixed there on
  2026-09-26 at 668854c9, ruling 288, and pinned here from 2026-09-27)*:
  after a retarget that
  shrinks the selection, a kept brick can hold a slot past the resident
  count, so its refresh panics and picks pass through visible ground
  (`modulus/src/lib.rs:424-433`); Eponym's V1b carries it latently, never
  editing or picking. And the CPU regrow: every overlay change regrows the
  whole map, 0.4 to 2.8 s a click in release at 256 (single samples), which
  paging the atlas does not touch. Mark ruled on 2026-09-26 (wing design
  record, rulings 288 to 292): fix mere first, size the atlas to the card,
  page the CPU side too, select residency by the view's footprint in an
  isometer helper, and move isometry's pins to mere's current main. The
  assessment's raw files are out of tree in
  `Code/testing/isometry/paging-assessment-2026-09-26/`. §5's decision 4
  stays open; it now turns on those rulings landing.

## 8. The scene board's cost model (B5, 2026-09-16)

One place for what the scene board costs, pulled together from B4, the
subdivision and B5's own measurements, each re-taken rather than carried.

**The instrument, stated once.** Windows laptop; the repo's unoptimised `dev`
and `test` profiles (the six packages at `opt-level = 2` in the root manifest;
`isometry-views` and the isometer family are not among them). Headed figures
come from `ISOMETRY_PROFILE=1` on the demo map at device scale 2, interface
zoom 0.917, producer render scale 2, with `ISOMETRY_OVERLAY_SELFTEST=1` armed
so the app keeps redrawing a board on which nothing changes; the frames counted
are the ones before the self-test fires, less the first three and the capture
frame. Headless figures are one `BoardSource` frame into a 640 by 480 pane at
render scale 1. **Slot and byte counts are deterministic and do not move with
the build; every timing does** — which is why the build is named here and was
not named in the entries this section corrects. Where a timing is worth having
in both, the release figure is given beside the debug one:
`tests/scaled_world.rs` already says why ("release matters: debug timings are
dominated by unoptimized BTreeMap walks"), and the regrow is exactly such a
walk.

### A steady frame, both arms, same machine and map

Medians over the steady window, in milliseconds
(`Code/testing/isometry/scene-board-profile/`):

| phase | DOM board, n = 66 | scene board, n = 183 |
| --- | --- | --- |
| **total** | **32.16** (p25 31.73, p75 32.82, p95 33.36, 31.02 to 35.18) | **12.50** (p25 11.82, p75 13.91, p95 16.77, 10.75 to 20.76) |
| emit | 10.50 | 2.26 |
| a11y | 13.03 | 2.48 |
| raster | 6.71 | 5.53 |
| relayout | 0.33 | 0.09 |
| present | 0.19 | 0.24 |
| producer | no producer | 0.05 |
| staging / stages | 0 / 0 | 0 / **0** |

The scene board's steady frame is 2.6 times cheaper, and the saving is not in
the renderer: `raster` is within 1.2 ms. It is `emit` and `a11y` — building and
projecting 632 boxes — which is what a board element costs when nothing about
it has changed. `stages = 0` is the producer skipping: with the signature
unmoved the scene is never re-traced, so a still board's whole scene cost is
the 0.05 ms signature check.

### What a change costs

| change | ground | GPU upload | source |
| --- | --- | --- | --- |
| nothing moves | none | 0 bytes | measured, B4's receipt re-run |
| a pan | none | 0 bytes; one re-traced frame, 2.57 ms headless | **measured, B5** |
| a token step | none | 0 bytes; one re-traced frame, 2.64 ms headless | **measured, B5** |
| one elevation edit | regrow + diff 0.54 ms | 4 slots, 2,064 bytes of the map's 264,192 | measured, matches B4 exactly |
| one tile tint | regrow + diff 0.54 ms | 4 slots, 2,064 bytes | measured, B5 |
| the whole overlay set | regrow + diff 0.55 ms | 50 slots of 260 bricks | measured, matches B4 |
| a focus elevation on | regrow + filtered rebuild 5.90 ms debug | one full map, 263,168 bytes | upload matches B4 to the byte; its 0.25 ms is not a debug figure |
| a focus elevation off | regrow + rebuild 2.53 ms | one full map | measured, B5 |
| the regrow itself, demo map | **36 ms debug, 4 to 9 ms release** | — | **measured, B5** |

A pan and a token step are the two cheapest things a session does and they cost
the ground nothing: the camera and a body pose are signature fields, so the
price is one re-traced frame and no upload at all. Everything else is the
regrow, and the regrow is the model's whole weight — `Ground` exposes no
material write (§6), so a one-tile tint raises 33,000 voxels again.

Two cautions about that number, both found by measuring it repeatedly. It is
build-dominated: 35.98 ms debug against 3.99 ms release for the same bare
grow. And a single sample is not a number at all — the first grow in a fresh
release process read 45.27 ms where the *larger* 30 by 30 board read 7.55 ms a
moment later. Quote the regrow with a build and a warm process, or do not
quote it.

One dead path found while measuring: `BoardGround::sync` has a branch for "the
focus moved but the revision did not", which cannot fire, because `Overlays`
carries the focus and so a focus change always moves the view's revision. It
costs nothing and it is this crate's, not isometer's.

### The ceiling, and the worst case the repo can produce

`modulus::MAX_BRICKS` is 2,047. Walked rather than quoted, on `synth_map`:

| board | bricks | brick bytes | regrow, debug | regrow, release |
| --- | --- | --- | --- | --- |
| demo 24 by 24 (the app's own map, and `maps/demo_skirmish.json`) | 260 | 258 KiB | 36 ms | 4 to 9 ms |
| synth 30 by 30 (`ISOMETRY_SYNTH`'s default stress board) | 400 | 396 KiB | 98 ms | 8 ms |
| 64 by 64 | 1,600 | 1,587 KiB | 568 ms | 65 ms |
| **70 by 70 — the largest square board with a scene arm** | 1,936 | 1,920 KiB | 695 ms | 79 ms |
| 72 by 72 | refused at 2,116 bricks | — | — | — |

§6's "roughly 70 tiles square" is confirmed to the tile: 70 fits, 72 refuses.
The worst case that matters is not any of these, though — it is
`isometry_campaign::MAX_GENERATED_MAP_EDGE`, which is **256**. The substrate's
own map generator will author a board three and a half times past the edge
where the scene board stops having a brick map at all, and nothing warns the
author. That is the fact §5 decision 4 is held open on.

### Figures carried in this plan that needed a build to mean anything

Every slot and byte figure re-measured to the byte. Every timing needed its
build named before it could be compared at all:

- the subdivision entry's "the grow 0.04 to **1.40 ms**" and "**17 ms** at 64
  by 64", and B4's "the grow itself is unchanged at **1.1 ms**" — 36 ms and
  568 ms in the debug build, 4 to 9 ms and 65 ms in release. Release accounts
  for a factor of nine of it and warm-up for much of the rest, so these were
  release figures; they are not reachable in the build the plan's frame and
  element receipts are taken in. B4's *own* headed captures under
  `Code/testing/isometry/scene-board-overlays/` read 35 to 40 ms beside the
  entry that said 1.1, which is the same gap seen from the other side.
- B4's "a focus elevation is **0.25 ms** and one full upload" — the upload is
  exact to the byte; the filtered rebuild is 5.90 ms debug, and was not
  re-taken in release.

The Progress entries are left as written, because a progress entry records what
a lane saw. This section is the current one, and it names its build.

## Progress

- **2026-09-16, Mark ruled the DOM board's fate: decide later.** §5
  decision 4 is deliberately held open until the family's brick cap is
  resolved. The two facts that moved it: a board past roughly 70 tiles
  square refuses its brick map at `modulus::MAX_BRICKS` of 2,047, where
  the DOM board has no such limit, and the DOM board is the only arm
  that works on a machine with no usable adapter. So B5 records the
  frame profile and the element count and stops there; both boards stay,
  behind the flag, without that standing as a commitment to keep them.
  The switch is re-decided when isometer's owner rules on paging or a
  larger cap, and the cap is recorded in §6 for exactly that.
- **2026-09-16, B5 landed.** The scene board's frame profile stands beside the
  DOM board's on one machine, one map, one window and one set of conditions:
  **12.50 ms median against 32.16 ms**, over 183 and 66 steady frames, and the
  2.6x is not the renderer. `raster` differs by 1.2 ms; what moves is `emit`,
  10.50 to 2.26, and the accessibility projection, 13.03 to 2.48 over 111 nodes
  against 744. That is what 632 boxes cost on a frame none of them changed on.
  On such a frame the producer stages nothing, so the scene's whole cost is a
  0.05 ms signature check. §8 holds the table and the rest of the cost model.
  The control has **not** drifted: the DOM arm's census reads 631 elements
  beside the one marker the old metric counted with them, which is §1's 632 to
  the element, and the atlas receipt re-run here reads pointer 1.961 ms median
  against §1's 1.9. §1's "M4 layout median 36.8 ms" has no twin on this machine
  and never did — it was a Mac on a preserved older closure that re-resolved
  style and re-laid text on every drag step; the same test here resolves no
  style, lays out no text, and reads 0.172 ms.
  The element receipt is three tests rather than a number: a walk of the
  `.board` containers by class over the retained tree the shipping host laid
  out, naming what it counts — `tile`, `tile-face`, `fog-shroud`, `prop`,
  `token` and the `beat` wrapper each sprite rides — and what it allows, B4's
  ground markers and the one leaf. The DOM arm is the positive control in the
  same process at 635 unfogged and 226 shrouded; the scene arm is 0 and 0. It
  needs no adapter, so unlike B3's set it runs everywhere, and the fog arm is
  there because a shroud is the class most likely to come back by accident.
  The headed line now says the same thing per frame and names its container:
  `board census in .board: ground 0, props 0, tokens 0 = 0 board elements;
  allowed beside them: markers 1, scene leaves 1`, unchanged after the overlay
  self-test puts every overlay on the board.
  What the captures show beside the numbers: the arms agree on the water's and
  the stone path's bounding boxes to a pixel and on the board's span to one, so
  B4's drawing fix holds. Three things the scene arm still does not draw, all
  previously recorded and all confirmed: props, the `alt` checkerboard shade,
  and the DOM's `:hover` tile highlight, which B3 left half restored and which
  a board with no per-tile element cannot have at all. One thing not previously
  said: the tracer shades a top face, so an overlay tint on the scene board is
  about 9% darker than the same hex on the DOM board (`#ffd766` arrives as
  231, 196, 92). And B4's one-pixel column is in **both** arms again, at
  physical x 1878 where B4 read logical 1024, so it tracks the window rather
  than a coordinate — which confirms rather than weakens B4's correction.
  Every slot and byte figure the plan carries reproduced exactly; not one
  timing did, and §8 now gives each with its build. The subdivision's grow of
  1.40 ms and 17 ms at 64 by 64, and B4's "unchanged at 1.1 ms", are release
  figures: debug reads 36 ms and 568 ms, release 4 to 9 ms and 65 ms, and
  `tests/scaled_world.rs` already warned that a debug timing over these
  BTreeMap walks misreports the ratio. B4's own headed captures read 35 to
  40 ms beside the entry that said 1.1, which is the same gap from the other
  side. B4's 0.25 ms focus change is 5.90 ms here, its upload exact to the
  byte. A single grow sample turns out not to be a number at all — a fresh
  release process read 45.27 ms for the demo and 7.55 ms for a *larger* board
  moments later. Two costs the plan had never priced: a pan and a token step
  each cost the ground nothing and upload nothing, and are one re-traced frame.
  The cap is walked rather than quoted — 70 by 70 fits at 1,936 bricks, 72 by
  72 refuses at 2,116 — and the number beside it is
  `isometry_campaign::MAX_GENERATED_MAP_EDGE`, **256**: the substrate's own
  generator will author a board three and a half times past the edge where the
  scene board stops having a brick map, with nothing to warn the author. Per
  the ruling above, B5 stops at the measurement: nothing is archived, the
  flag's default is untouched, the private flat-ground arm stands, and both
  boards stay. One seam moved: `edit_tests`'s fixture is now `scene/harness.rs`
  so the cost receipts drive the same board rather than a second copy of it.
  109 views tests, 405 across the workspace.

- **2026-09-27, paging integrated.** Lane E `983aa43` passes final gates
  and independent read-only review. Under 368/369/372/374, the atlas
  allocates the chosen feasible budget upfront (8 MiB default), reports
  current omissions and rebuilds once after a live budget change. The
  requested budget persists locally outside campaign data. Root verified
  112 raw receipt hashes and 50 final-source hashes, with 411 root and
  293 shared tests passing and all three consumer checks passing.
  Six native sessions prove the 1 MiB / 247 omitted to 2 MiB / zero
  transition, restart persistence, zero headroom pixel drift and the
  32,424-pixel overlay-absent control. Raw failed attempts are retained.
  [Integration receipt](../testing/scene-board-paging/integration.md)
  qualifies each gate's source, Eponym's exact lock and the final
  formatting-only change. Cost-table `headed` rows are headless harness
  measurements at headed-pane dimensions, and their `frame` covers
  `Board::draw` plus completion wait, not host input-to-present latency.
  Pins, tracked locks and sim checkpoint 5 are unchanged. The switch and
  Genet text acceptance remain governed by their existing open rulings.


---

*Archived 2026-10-28.* Cut, with the other active plans, into a new set: what was
still open, deferred or a follow-on here now lives in `2026-10-28_board_on_isometer_plan.md` (in the product's
`design_docs/`), which cites this file by line. This copy is history: its
status lines, receipts and claims are as they stood, not current.

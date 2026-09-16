# The board on isometer

**Date:** 2026-09-15

**Status:** assessment, for Mark's sign-off. No code moved, no commit.

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
- No Isometry-side document cited the L4 done condition before this plan.
- `isometry-runtime`, which holds the earlier fixed-isometric GPU tenant,
  is excluded from the root workspace and untouched by this plan.

## Progress

- **2026-09-15:** founded from three read-only assessments (the board today,
  isometer for a third consumer, the wing GUI inventory) and the isometer
  owner's API notes. Approved the same day: tokens as live bodies, the
  producer owns the pixel grid.
- **2026-09-15, I5 landed (845b6b0).** `SlabCamera::dimetric_2_1`: 45
  degrees azimuth down the x=z diagonal at a 30-degree pitch, which is what
  a 2:1 tile ratio requires once the projection is divided by the camera's
  own up vector (the edge slope's 26.6 degrees is the wrong number); a 9 by
  9 by 3 grid lands where `tile_to_screen` puts it within a pixel, the two
  board axes project exactly 2:1, an elevation step projects straight up.
  The camera test file sits at the 600-line ceiling and splits next.
- **2026-09-15, I4 landed (61b5056).** `Volume::from_voxels` (palette index
  i is material i+1, absent cells 0), `TokenBody` (a one-part body plus its
  volume map from voxels, layers or an `Appearance`, content-addressed),
  `material_colours`, and a CPU silhouette over the bake's own projection
  proving the baked sprite and the live mesh agree on every pixel for three
  subjects at four facings. Found I6: the live renderer takes no colour
  table, so a token draws in hashed colours until it does.
- **2026-09-15, I3 landed (b71ced7).** `render_scale` on the request and
  the signature; the scene draws at the leaf size over the scale and a
  nearest fullscreen pass presents each pixel as one block at exactly the
  leaf's size; scale one takes no pass and is byte-identical. Netrender
  samples nearest at every stage of the external-image path. Two field
  initialisers landed in Paredros, whose acceptance hash is unchanged. Two
  notes: `Scene::capture` still reads the internal image, so a headed
  capture at scale four is the small master; and nothing calls
  `set_render_scale` until the board does.
- **2026-09-15, I2 landed (db1d81e).** `Scene::pick` and `pick_at_pixel`
  answer the nearer of a body pick and a `TerrainHit` (world point, entered
  voxel, brick, normal, distance, material) from the presented frame's own
  camera, ray and map; ties go to the ground as the depth test does;
  `pick_ndc` keeps its bodies-only behaviour. The lens needed nothing: the
  brick ray already carried every field. Five headless receipts. The hit
  carries no frame stamp, since it names geometry rather than a minted
  identity; a host that wants one reads the query generation.
- **2026-09-15, I6 landed (1ea3572).** The live renderer takes an optional
  256-entry colour table per body, bound as one shared uniform block with a
  dynamic offset and batched by palette slot; absent or empty tables keep
  the exact hashed colour and face shade arithmetic. The facade registers a
  table per subject on the body layer rather than adding a field to
  `SceneBody`, so no existing literal moved. A facade receipt draws the demo
  hero through the scene and finds only its baked palette entries under the
  six face shades; pixel-for-pixel agreement with the sprite waits on the
  camera alignment B2 owns.
- **2026-09-15, I1 landed.** `TerrainPalette` (64 colours, entry 0
  unknown) rides in the trace params by extending the terrain array, so
  the bind group layout is unchanged; the shader falls back to the exact
  classic and habitat arithmetic when no table is bound, with an explicit
  no-palette receipt. `Ground::grow_with` takes a material sampler over
  column and depth below the surface and `grow` is the soil-over-rock
  case. One deviation from §2: the palette is host state on `Scene`
  (`set_terrain_palette`), like the terrain map and the render scale,
  rather than a field on the frame or the appearance, which would have
  broken six exhaustive literals in the products; B1 binds it from the
  tile-kind mapping. 35 core, 58 lens, 46 facade tests. All six isometer
  additions are now on main; the board lanes B1 to B5 follow.
- **2026-09-15, B1 landed.** `isometry_views::MapTerrain` reads a
  `MapDocument` as isometer-core `Terrain`: map `(col, row)` is terrain
  `(x, z)` centred on the origin with no axis flip, one voxel per height
  unit, sea level at -1 so every painted cell is dry, and an off-map or
  empty column at -2 so `grow` lays nothing there. Material is the tile
  kind plus one; `terrain_palette` reads the kind colours from the one
  table (`theme/kinds.rs`) that now generates the `.tile-<kind>` rules at
  their old cascade positions, and a test parses the hex back out of
  `board_css` so a hand-edited rule fails. Props are counted, not grown.
  isometer-core and isometer-lens are root workspace path dependencies;
  the lens brings wgpu unconditionally, which B2 needs anyway. Noticed
  and left alone: `overmap/atlas_terrain.rs` carries a second kind-to-
  colour table for the overmap swatch, a drift risk for a later pass.
  80 views tests, 368 workspace tests with all features.
- **2026-09-15, B2 landed.** `ISOMETRY_SCENE_BOARD=1` draws the board as
  one `custom_leaf` over an `isometer::SceneProducer`: B1's `MapTerrain`
  grown to a `Ground` and bound as a `BrickMap` with `terrain_palette`,
  each token a one-part `TokenBody` from the same recipe table the
  sprite bake reads (now `theme::token_recipes`, read by both), coloured
  through `material_colours` on the body layer and tinted by owner,
  under `SlabCamera::dimetric_2_1`. The camera is the whole of the
  alignment: one tile is one world unit, the centre is taken on the top
  plane of flat ground and offset half a voxel, because a terrain column
  occupies a half-open range while a tile is a diamond about a point.
  Miss either and the board sits half a tile up the screen. The parity
  gate is 256 probe pixels over the demo map: of the 100 that land on a
  flat top face the two paths name the same tile 100 times, 18 land on
  side faces the DOM has no element for, and 8 on raised tops where the
  DOM's flat inverse names the tile behind it, as it documents. The
  producer owns the pixel grid. Without the flag nothing changes.
  Deviations: `dirty` is empty so a terrain edit regrows the whole
  ground (B4 owns the incremental path); props are counted, not drawn,
  and since the tile layer is absent under the flag they vanish rather
  than staying DOM; `BoardPick::Tile` gained the hit's elevation and
  face, which §3 did not name. Seen in the capture and still open:
  no props, no selection diamond, no markers, the tracer's sky fills
  the pane instead of its dark ground, a one-pixel darker column runs
  the pane's full height, and material edges are about three physical
  pixels soft rather than hard blocks, so the crispness the render
  scale exists for is not yet visible. `main.rs` is now at exactly 600
  lines: the next lane to touch it splits it first. 84 views tests,
  372 across the workspace.
- **Found by B2, 2026-09-15, for §6.** `TokenBody::from_volume` is
  private, so a host holding a `Volume` round-trips through `Voxels`.
  `SceneVolumes::Voxels` takes one `VolumeMap` while each `TokenBody`
  owns its own, so merging N tokens clones every volume; a
  `VolumeMap::extend` would retire that. The I6 colour table is
  registered per subject with no way to clear the set, so a host that
  reuses keys across maps leaks stale palettes. `FrameRequest` carries
  no layout scale, so a source cannot see the pixel grid it draws on and
  the host sets the render scale out of band. Nothing helps a host say
  "one world unit is k pixels": every consumer re-derives the half
  height. And the soft edges above want I3's owner before B5 records the
  pixel grid as landed, since at scale 2 with a fractional fit the
  nearest upscale looks resampled once more by the compositor.
- **2026-09-15, Mark ruled the cliff height: subdivide the voxel grid.**
  B2 found the scene's cliffs standing 2.45 times the DOM board's,
  because a voxel is a cube: one elevation step is one world unit and
  projects 19.6 px under the 2:1 dimetric, where the DOM's step is 8 px
  against a 16 px tile. The wanted height per step is 8/19.6, or 0.408
  world units, which a cubic ground at one voxel per tile cannot
  express. The ruling is a finer grid rather than a non-cubic ground or
  a rebased look: **a tile is 5 voxels across and an elevation step is 2
  voxels tall**, a ratio of 0.400 that projects 7.84 px, within a fifth
  of a pixel of the DOM's step and within two thirds of a pixel over the
  demo map's full height range. The cost is 25 times the columns, which
  a 24 by 24 map absorbs at 120 by 120, and the gain is sub-tile terrain
  detail the voxel-sourced appearance can use later. The alternatives
  and why they lost: a world scale in y on the ground would keep the
  column count but makes voxels non-cubic, which the tracer's stepping
  assumes, so it is a change to the family's floor and its owner's to
  make; and accepting the taller cliffs would have moved the shipped
  look off the GBA-era shallow step that CLAUDE.md names as the target.
  This revises B1's one-voxel-per-height-unit convention and B2's camera
  and probe grid, and it lands as its own lane before B5 records the
  pixel grid.
- **2026-09-15, the voxel subdivision landed.** `VOXELS_PER_TILE` = 5
  and `VOXELS_PER_STEP` = 2 carry the cliff-height ruling, with the
  derivation in their doc so the next reader sees why 5 and 2: under the
  2:1 dimetric a world unit of height rises by `cos 30` where a tile is
  `sqrt 2` wide, so with `t` voxels to a tile the step that matches the
  DOM's 8 px against its 16 px tile is `t * 0.4082`; at `t` = 1 that
  wants 0.408 of a cube and cannot have it, and at `t` = 5 it wants
  2.041, so 2 gives 7.84 px. A cell's 5 by 5 footprint takes its
  material, the surface of elevation `e` is `2e+1`, sea level and the
  void rescale, and a void column still lays nothing. The world mapping
  follows: 4.53 px per voxel, 7.84 px per step, and B2's half-open
  column against a diamond about a point is the same correction at the
  new size. A token's scale gained the tile's width in voxels, so its
  on-screen size is pixel-identical to B2's capture. The parity gate
  passes at the new scale: of 256 probes, 104 land on flat top faces and
  the two paths name the same tile 104 times, with side faces down from
  18 to 7 because the rim is a step rather than a whole tile. A new
  receipt pins the ratio in pixels, 7.838 against the DOM's 8.0 with
  0.647 px of drift over the demo map's four steps, so 5 and 2 cannot
  change silently. Measured in the capture, the cliff face is 16
  physical px against a 32 px tile where the DOM's is 14.7 against 29.3,
  both exactly half a tile, where B2's stood at 1.22 tiles. Cost,
  measured: the demo's columns go 625 to 14,641, solid voxels 660 to
  33,000, the grow 0.04 to 1.40 ms, bricks 16 to 260, the upload 128 to
  258 KiB, since the atlas allocates in 128 KiB rows. B2's open visual
  list is unchanged. 87 views tests, 375 across the workspace.
- **Found by the subdivision, 2026-09-15, for §6 and for Mark.**
  `modulus::MAX_BRICKS` is 2,047, so a board past roughly 70 tiles
  square now refuses its brick map where before the subdivision the same
  cap sat near 360 tiles square; a 96 by 96 map already fails at 3,600
  bricks. The demo, the watchtower and the 30 by 30 stress board are far
  inside it, but a large authored map is not, and isometer has no paging
  or capacity path wired into its ground terrain. Separately,
  `Ground::surface` searches a fixed band of 24 voxels, which at 2
  voxels per step is a height limit of 11 elevation units expressed in
  the wrong unit; it is a helper the receipts use, not the render path.
- **2026-09-15, B3 landed.** A press, a click, a drag and a hover resolve
  through `UiState::board_at`, one resolver with two arms: the scene
  pick, a borrow of the host's producer asked at the moment the gesture
  runs rather than a snapshot that would be a frame stale by
  construction, and with the flag off the flat inverse. The scene arm
  dispatches the click the leaf cannot dispatch for itself, in the DOM
  elements' own two forms and in their order, so the drag still records
  what the press applied; the nine modes, the local and remote branches
  and every rule about what a click means are untouched. Both geometric
  fallbacks are retired: the token candidate's mode gate was policy and
  moved into the press, and the flat inverse's arithmetic is a private
  arm that B5 retires with the DOM board. What made that safe is a
  receipt over one real frame: 104 flat probes agree 104 of 104, and
  every raised probe the two differ on has the DOM naming the tile
  behind the one you see, asserted rather than excused. Host routing,
  zoom and watchtower pass in both arms at 10 each; under the flag eight
  receipts skip loudly because they measure per-tile DOM boxes or need a
  producer a windowless harness never drives, and the scene arm has its
  own five, the clip receipt included, each asserting the tile-element
  count at zero beside it. Hover is half restored: a captured drag and
  leaving the leaf reach it, free hover waits on the shared host, which
  routes no hover move and publishes no cursor position. 90 views tests,
  383 across the workspace.
- **Found by B3, 2026-09-15: the scene board draws too large.**
  `BoardWorld::camera` takes its centre from the pane but its half
  height from the texture, and the texture is the pane times device
  times zoom over the render scale. That ratio is 1 only when device
  times zoom is already whole, so on this laptop the scene board draws
  1.090 times the DOM board's size, measured at 1.10 across the demo
  map's 24-tile span. It is also B2's soft edges: the fit is fractional,
  so the nearest upscale is resampled once more. The pick is unaffected,
  since it reads the frame's own camera, so a click always lands where
  the user sees. A one-line change, and the next drawing lane's.
- **2026-09-16, B4 landed.** The board's overlays are terrain materials:
  the palette I1 gave the tracer now carries the tile kinds, the six
  state tints beside them and a shrouded twin of both, so reach, path,
  selection, templates, doors and encounter sites tint a tile's top
  voxel and remembered ground stays legible under the same translucent
  black the shroud rule composites. Unexplored ground reports the void
  and lays nothing, as the DOM emits no element. Markers came back as
  DOM in a container carrying the pan, which only works because the
  drawing defect went first: the camera no longer sees the texture at
  all, and the scene board's span is 1440 px against the DOM's 1441
  where it had been 1.0897 times too large and overflowed the pane. The
  pane's own near-black ground replaced the tracer's sky through the
  terrain appearance; transparency is not expressible, since the tracer
  writes alpha 1 on a no-hit pixel. An edit reaches the frame as slots
  through this crate's own terrain source, and the reason it needed one
  is a bug shown rather than argued: the shared `GroundTerrain` stamps
  `Ground::revision()`, which a grown ground never moves off zero, so
  the tracer skipped the upload and **B2's terrain edits never reached a
  pixel** unless the brick extents moved. The receipt drives B2's exact
  path with a positive control in the same run and reads back revision
  unchanged, zero bytes. One elevation edit is now 4 slots and 2,064
  bytes against the whole map's 264,192; the whole overlay change is 50
  bricks of 260; a still frame is nothing. The grow itself is unchanged
  at 1.1 ms and unavoidable, because `Ground` exposes no way to write
  one voxel's material. A focus elevation is a filtered rebuild, 0.25 ms
  and one full upload, and it cuts pieces by omission rather than by the
  cutaway, because a plane laid on the ground a focus keeps takes every
  token standing on it off at the ankles. Two corrections to earlier
  entries: the one-pixel column is **not the scene board's**, sitting at
  logical x 1024 in both arms and in every capture since B2, so it is a
  host paint seam below this crate; and B2's "three pixels soft" is one
  blended pixel in 595 of 600 rows, caused by the pane's fractional
  physical origin rather than by the render scale, with the DOM arm
  softer still at two. Props and the checkerboard shade are the visible
  gaps left: the alt shade is a second colour per kind that the material
  channel has no slot for. 105 views tests, 398 across the workspace.
- **Found by B4, 2026-09-16, for §6.** `Ground` has no public material
  write: `place` is private, `carve` writes only air, so a host that
  authors terrain regrows the whole ground for a one-tile tint. The
  filtered rebuild's closure can only zero a material, never recolour
  one, forcing the same regrow. `GroundTerrain` is unusable by any host
  that regrows rather than carves, per the revision bug above; it should
  take a host revision or its doc should say carve only. The tracer
  writes alpha 1 on a no-hit pixel, so a scene cannot leave its
  background transparent for the pane to show through. And the frame's
  dirty set has no companion for "the brick set itself moved", so a host
  must detect that and rebuild whole.

# The board on isometer: what moved out on 2026-10-18

On 2026-10-18 Mark made the old numbered rulings cases (precedent, numbered 1
to 984 in the design record), kept new rule numbers for himself, and asked
every active doc to lose its stacked status banners, old progress entries,
stale counters and pasted copies. This file holds, verbatim, what left the live
[board on isometer plan](../../2026-09-15_board_on_isometer_plan.md): its
stacked status paragraphs and the dated current-state section (2026-10-10,
2026-09-27 and the earlier assessment status), and every progress entry except
the three that describe the open switch decision and the integrated paging.
Relative links in the moved text are kept as written, so they resolve from the
live doc's folder, not from this one.

## Status banners

**Status, 2026-10-10:** the authored board and paging foundations remain
implemented. Its sim-off path is MapTerrain; ruling 799 adds a lifted-site
sim-on path under V2. This is not a new parity or headed receipt.

## Current state, 2026-10-10

`crates/isometry-views/src/scene/board.rs` draws the board through
isometer, with terrain lowering in `scene/terrain.rs` and resident paging
in `scene/ground.rs`. `MapTerrain` reads the authored map, tile materials
and elevation. That implemented path remains for sim-off play (799).

With the sim on, V2 must read a site through
`shared/isometer/crates/isometer-space/src/`, using the sim's accepted
edits and placement rather than keeping an independent map authority.
The VTT overlay owns that integration and the faction-turn retirement;
this plan owns its scene lowering, selection and presentation gates.
The old “one board geometry” assumption is superseded by 799's two paths,
not by deleting the authored map path.

Native places and the lift do not make token collision, movement or
system rules isometer-owned. Pointer picks answer the drawn scene;
accepted tactical play remains the VTT's. Mere's tenant is the selected
body rasteriser (796), with product adoption recorded by the presentation
plan rather than assumed from this board's old receipts.

*The dated board design and receipts below remain as written (793).*

**Status, 2026-09-27:** Lanes landed 2026-09-16; paging integrated 2026-09-27. *(Brought current 2026-10-06 under ruling 618; the
earlier status line follows as written.)*

*Earlier:* **Status:** assessment, for Mark's sign-off. No code moved, no commit.

## Progress entries

- **2026-10-10, Q1:** current ownership, gates and status checked against
  the lane tree under 793 to 802; earlier dated text retained. No new
  compile, test, draw or headed receipt claimed.

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
- **2026-09-18, the stale-terrain fix landed; the map-size machinery was
  dropped.** The map-size warning lane of 2026-09-16 found that a map
  whose brick map failed to build left the previous map's terrain drawn
  under the new map's camera with the error cleared, because the ground's
  sync recorded the revision before the build. The fix records the
  revision only after the build succeeds, and a refused map empties the
  ground and holds its error until a map that builds replaces it; a
  receipt drives it through the brick library's own natural refusal of a
  96 by 96 map, with a positive control in the same run, and was proved
  by reverting the fix and watching it fail. The rest of that lane, a
  brick budget derived from the cap, a generator warning, a self-test arm
  and a modulus dependency, about 780 lines, was dropped by Mark's ruling
  of 2026-09-18: the wing design record retires the cap as a limit, and
  this plan is under rewrite. 110 views tests, 408 across the workspace.
- **2026-09-26, paging opened and assessed.** Ruling 282 opened a lane to
  wire paging into the scene board; it stopped at forks after assessing, and
  Mark ruled them the same day (rulings 288 to 292, §7). A mere lane fixes
  the retarget defect and sizes the atlas to the card, isometry's pins then
  move to mere's current main, and the board lane builds the CPU-side paging
  and the view-driven residency helper meanwhile, keeping any shrinking
  retarget off paths a user can reach until the pin moves.
- **2026-09-26, paging built on `lane-e-paging`, merging after two
  additions.** A 256 by 256 board renders, flat and in relief, and a
  click at 256 fell from 496 to 1,258 ms to under a millisecond in
  release. Mark ruled the lane's three items (rulings 301 to 303): its
  interim overflow rule stands until the card-sized atlas lands, the
  pointer volume reserves headroom above the tallest point, and the
  board's tallest elevation is cached rather than scanned each frame.
  The lane adds the last two before the branch merges.
- **2026-09-26, paging's second round.** The three rulings are built on
  `lane-e-paging`: the tallest elevation is cached, the producer phase at
  256 falling from 2.8 to 0.03 ms and the headed frame from about 18 to
  15 ms; headroom is a setting, one spare layer by default,
  provisionally; and the overflow rule stands. One spare layer changes
  356 pixels of the 256 board where headroom 0 is byte-identical, so the
  cause is found before the branch merges (ruling 318), and the overlay
  self-test's tie is broken with it (325).
- **2026-09-26, the 356 pixels explained.** Lane E traced them to the
  shared ray traversal, `modulus`'s `brick_dda`, which adds each crossing
  step in 32-bit floats from an eye about 1,380 units behind the board:
  the error grows along each ray's walk, and headroom only lengthens the
  walk. Against a 64-bit traversal, headroom 0 is already off at 201
  texels and headroom 1 at 531, so neither picture is exact
  (`testing/scene-board-paging/bands.md` on the branch). Mark ruled the
  fix into mere (ruling 336) and paging's merge after it (337).
- **2026-09-27, the traversal fixed in mere, on a branch.** Lane K's
  `dda-precision` computes each crossing as the boundary less the eye
  over the direction: on this board's frame no fault and no headroom
  move remain, on the CPU and four GPU setups, for 0.02 to 0.08 ms a
  frame. isometer will call modulus's CPU walk rather than copy it
  (ruling 361), and the start clamps into the box (362). Paging merges
  once that reaches isometry's pins.
- **2026-09-27, traversal repin authorized with coordination (ruling 365).**
  Mere's remote main is now `7bb5bfda`, containing Lane K's fix. Isometry
  still pins `0418391f`. Mark authorized the separate traversal repin with
  "You can repin but communicate with the isocosm agent". Identifying and
  contacting that agent remain prerequisites; the repin, picture receipts
  and paging merge are still pending. Genet's text fix and burn pre.4 follow
  separately.
- **2026-09-27, pre-repin audit, partial.** At isometry `a9409e0` and mere
  `7bb5bfda`, all 30 files in
  `Code/testing/mere/receipts/2026-09-27/dda-precision/SHA256SUMS` matched
  their SHA-256 entries. The manifest itself hashes to
  `df10dc7594331a5ac3b9d694dc2c83915af3f1dcb767afc3c608428203be4d3f`.
  Source inspection confirmed the same-run accumulated-walk control in
  `modulus/src/traversal_tests.rs`: 330 and 660 moved pixels at one and
  two headroom layers, versus zero required from the fixed CPU walk.
  These counts were not recomputed in this audit: the fresh locked,
  offline modulus test invocation waited on another session's Cargo
  package-cache lock and was stopped before compiling. No new target was
  created. GPU receipt files were integrity-checked, not rerun.
  The repin interval also includes `a464dc2a`, which updates Knot's two
  dependency revisions; consumer checks must cover that resolution change.
  Lane A's `shared/isocosm/Cargo.toml` still pins hagiograph at `53648d3a`,
  whereas main pins `0418391f`. That manifest and lock need coordination
  before repinning to `7bb5bfda`; the pins remain unchanged.
- **2026-09-27, orchestration resumed (ruling 367).** The RPG session is
  inactive; Mark authorized this session to orchestrate the repin and
  remaining lanes. The coordination hold above is superseded. Traversal
  verification and the CPU wrapper change proceed, with independent audits
  of sim Part B, paging and the queued pre.4/text forks. Unresolved design
  choices remain with Mark.
- **2026-09-27, traversal repin verified and applied.** Seven manifests
  now pin mere `7bb5bfdab273cb5e8236ef519fb9f0eb8f37bee7`; the three
  tracked lockfiles change only that revision. Genet and netrender stay
  at `0cf4f30b` and `c8c09f16`. Isometer's `trace_ray` delegates traversal
  to modulus, retaining ray validation, normalization, world-space hit
  reporting and exhaustion errors (361). The added boundary-start test
  hits voxel 262143 from coordinate 262144 at distance 0.0001 (362).
  At the repin source based on main `0f3cc37`, root tests pass 397 with
  three intentional ignores; shared isometer passes 280 with one ignored
  doc test. Lens library clippy passes with warnings denied. Root,
  Mesocosm and Eponym all-features/all-targets checks pass, offline and
  locked; main's Isocosm Part A passes 69 tests and wing-integration four.
  Modulus passes 26 tests; its separately rerun receipt measures
  zero fixed-walk faults over 669,280 board rays at each of three headroom
  settings, against the same-run old-walk control's 2, 332 and 662 faults
  and 330/660 moved pixels. These are fresh CPU measurements; the four
  archived GPU setups above remain integrity-checked historical evidence.
  Fresh lens GPU tests execute with visible terrain and occlusion controls.
  Eponym's headed D1 passes 64 frames and probes 9/9, 9/9, 0/9, 0/9,
  independently recomputed from its PNG; the picture is byte-identical
  to the preserved August 26 picture. Mesocosm's DC4 example records all
  16 body cuts and a contact sheet, visually reviewed as capture evidence.
  Logs, metadata and pictures are under
  `Code/testing/isometry/receipts/2026-09-27/traversal-repin/`.
  The default source audit has zero duplicate families. All features
  retains 41 under ruling 298's optional Cleromancy exception: comparing
  Lane E's pre-repin closure gives the same 125 family identities after
  normalizing the mere revision; removing only Cleromancy gives zero.
  Paging is still unmerged: its diagnostic had no assertions, so a bounded
  gate patch is prepared on Lane E, requiring the old-walk control to move
  330 pixels, the fixed CPU/GPU pictures to move zero, and GPU acquisition
  to succeed. It must run against this repin. Overflow after card sizing,
  the host's budget setting and upfront versus growing allocation await
  Mark; Eponym's actual device limits are already required by ruling 295.
- **2026-09-27, paging's traversal gate passes after the repin.** Lane E
  merged main `b4db31d` without conflicts; tested source `e688a5e` adds
  assertions and a same-run empty-terrain GPU control. With Rust 1.98.1,
  the locked offline release gate passes: the old walker moves 330 pixels,
  the fixed CPU and GPU each move zero, and each of the two terrain frames
  differs from the empty-terrain frame at all 669,280 pixels. Capture
  lengths are checked at 2,677,120 bytes. Complete stdout, stderr and
  source/compiler provenance are in
  `Code/testing/isometry/receipts/2026-09-27/lane-e-traversal-band-control/`.
  The separate f64-camera diagnostic still differs at 68 texels for both
  fixed maps: zero headroom moves is not universal pixel exactness. The
  lane's receipt annotation is pushed at `fffee6a`; its tested code remains
  `e688a5e`.
  The earlier passing attempt without the empty-terrain control is retained
  separately as `lane-e-traversal-band-gate`; it is not the final gate.
  This verifies the declared traversal frame, not moving bodies/contact or
  a combined simulated-scene budget. Paging remains unmerged pending the
  three policy answers and remaining integration receipts.
  A separate read-only review, confirmed against source at `1bbdf4f`,
  identified two policy-linked follow-ups, not reproduced runtime failures:
  `BoardSource::set_residency` changes settings but its input signature does
  not, so the producer can skip the requested rebuild when `needs_frame`
  is false; and `Residency::update` returns `Current` before publishing a
  changed overflow count if the retained keys and dirty set stay unchanged.
  Live settings need a producer-level change/rebuild/skip test; a counted
  fallback needs a same-keys/different-overflow reporting test. The current
  host sets headroom at construction, and `ResidencyStats` describes the
  last change, so these findings do not establish a current host regression.
- **2026-09-27, overflow and budget setting ruled (368, 369).** The existing
  centre-first dropping rule becomes a standing fallback, amending 301;
  the host reports the current omitted-brick count, including changes that
  leave retained keys unchanged. The player gets a device-bounded atlas
  budget setting, 8 MiB by default, whose changes rebuild the atlas. Lane E
  implements these with focused reporting and producer-invalidation tests.
  Upfront allocation versus growth remains unanswered, as does sim flow
  retention. Eponym's actual-device-limit propagation proceeds separately
  under existing ruling 295, preserving its own budget.
- **2026-09-27, allocation premise reopened (370).** Mark asked whether a
  replacement texture can retain unchanged bricks while uploading only
  changes, and what it costs. Allocation remains open. Inspect and compare
  GPU copying with full CPU re-upload, including temporary coexistence of
  old/new textures and preservation of slot addresses. No growth policy is
  authorized by the question alone.
- **2026-09-27, budget persistence ruled (372).** Save the atlas budget
  locally per device in a small application preference store, outside
  campaign data. Restore it on restart and enforce current device bounds
  when applying it. Verify a restart roundtrip and missing/malformed data;
  allocation policy remains open while GPU-copy costs are measured.
- **2026-09-27, atlas replacement measured for 370.** The test-only probe
  on Lane E `3375a36` verifies unchanged slot coordinates while atlas rows
  grow, and complete destination readback matches after GPU copying plus
  uploads of new/edited bricks. Missing-copy and missing-patch controls
  both produce wrong bytes. On the RTX 4060 Laptop (Vulkan 610.88), ten
  measured runs after two warmups, with order rotated, give median
  allocation-through-completion CPU times of 0.198/0.589 ms for full
  uploads at 260→700 and 2,504→5,000 bricks; bulk old-atlas GPU copies plus
  changed uploads take 0.325/3.310 ms, and copying individual retained
  slots takes 1.301/10.945 ms. Bulk copying reduces CPU bytes from
  524,288→225,792 and 3,145,728→1,278,464. Peak old+new texture payload is
  786,432 and 4,456,448 bytes. The full path has one upload/submission;
  bulk has five/fourteen changed-box writes and two submissions. These
  include completion waits, are not GPU timestamps, omit a production
  growth integration, and do not control unrelated machine load. They
  establish feasibility and this implementation's cost, not an inherent
  speed ranking for every batched copy design or a frame-time guarantee.
  Raw samples and controls live in
  `Code/testing/isometry/receipts/2026-09-27/lane-e-atlas-replacement/`.
  Allocation has been put back to Mark: upfront budget (recommended),
  growth with full uploads, or a more tightly batched copy measurement
  before deciding. No allocation policy is inferred from the inquiry.

- **2026-09-27, tighter batching requested (373).** Measure one ordered
  submission for retained GPU copying and staged changed-data transfers
  before choosing allocation. Independent review confirmed the first
  experiment's feasibility and timings, with qualifications: byte savings
  are logical upload payload, not measured bus traffic; the full baseline
  includes empty capacity; identical unedited brick values do not detect
  wrong source coordinates; and ten samples do not establish stable tails.
  The next fixture must distinguish keys/voxels and detect wrong offsets.
  CPU map generation, pointer/bind-group recreation and rendering remain
  outside the bounded replacement timing unless separately measured.

- **2026-09-27, tighter batching measured for 373.** Lane E tested
  `e69e1df` and pushed receipt head `0ad455c`. One encoder orders the
  retained texture copy before packed patch-buffer copies, with one
  submission. Fresh texture/buffer allocation, box discovery, CPU packing,
  encoding and completion wait are included. Thirty measured samples per
  arm after three warmups rotate order on the same RTX 4060 Laptop.
  For 260→700 bricks, full upload / previous bulk / new batched medians
  are 0.191 / 0.303 / 0.348 ms; for 2,504→5,000 they are
  0.598 / 3.325 / 1.584 ms. Batched observed maxima are 1.374 / 1.890 ms,
  not latency guarantees. The larger copy improves, but full uploads
  remain fastest in these two measured cases.
  Exact padded staging is 475,136 / 2,588,672 bytes versus changed logical
  texels of 225,792 / 1,278,464 bytes. Older queue-write staging is opaque;
  this does not measure bus traffic. The full baseline includes empty
  capacity. CPU map creation, pointers, bind groups, rendering and readback
  stay outside every timer; unrelated CPU load was not controlled.
  All 198 whole-atlas readbacks match with asserted-distinct, nonzero
  per-key/per-voxel patterns. Same-run missing-copy, missing-patch and
  swapped-retained-brick controls detect corruption in both cases; the
  swaps differ at 1,022 texels. Root independently checked all nine raw
  manifest hashes, parsed samples and statistics. Raw evidence and
  `root-recomputed.json` are in
  `Code/testing/isometry/receipts/2026-09-27/lane-e-atlas-batched/`.
  Earlier experiments remain historical. Production growth is not
  implemented or authorized; allocation returns to Mark as an open choice.
  Independent read-only review also verifies the tested source/lock hashes,
  all 198 parsed rows, statistics, ordering, timer boundaries and controls,
  and finds no further measurement gate needed for 373. Fresh packing and
  staging are substantial costs; reusable staging/direct packing were not
  tested, so this is not an optimized-copy lower bound. The next question
  offers upfront allocation (recommended), row growth with full uploads,
  or further copy research while allocation stays open.

- **2026-09-27, upfront allocation ruled (374).** Mark accepted allocating
  the chosen budget upfront: "Agreed. The numbers have spoken." This closes
  the allocation inquiry and measurement gate in 370/373. Keep the 8 MiB
  default, device bounds, live rebuild on budget change and local per-device
  persistence under 369/372, plus current omission reporting under 368.
  Ordinary terrain population changes use the existing fixed capacity;
  production row growth is not part of this implementation. Lane E must
  finish the consumer, cost and headed receipts before integration.


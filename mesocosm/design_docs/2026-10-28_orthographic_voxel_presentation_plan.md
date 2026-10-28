# Orthographic Voxel Presentation

**Status:** 2026-10-28: cut from `2026-09-11_orthographic_voxel_presentation_plan.md`
(and open items of the Mesocosm overlay, environmental surfaces and sim host
plans), which are archived in `archive_docs/2026-10-28/`. `shared/isometer`
(core, lens, mesh, render, space) is the scene crate, and Mesocosm runs the
tenant path with smooth bodies and the 2:1 dimetric surface view as defaults.
Open: headed GPU certification, the lane re-proofs on the tenant path, GPU
bake, HTML interchange, and the whole L10 detail ladder, none of which is in
code.

**For:** the wing's one presentation lane: the shared-depth scene, bodies as
scene instances, the detail ladder (L10), HTML interchange, the GPU sprite
bake, and the tenant path's visual leftovers (water, terrain scale, sun,
`LiveBody` retirement). Not here: tick latency, cold route and startup search
cost (`sim-execution`, its Speed bars); body appearance, footing, senses and
binding (`bodies`); the effects system's presentation remainders
(`functional-generation`); passages lost at coarse lift levels and the VRAM
residency policy (`place-graph-engine`); the VTT's overmap readings
(`vtt-overlay V2`); the board's own stylesheet-versus-voxel choice
(`board-on-isometer`).

Receipts from this plan's runs go to `archive_docs/<date>/`, never into this
plan (DOC_POLICY §8). Evidence that matters is a test in the repository or a
line in Findings.

## Decisions

- **D2, rich bodies (2026-10-22), applies 511, 512, 531, 758:** Mark:
  "Reconnect the richer bodies, functional colours, and actual limb chains,
  reusing what's there. Then run an unchosen seed and capture them moving so
  we can see whether it reads." This explicitly lifts the native adapter's
  chain refusal. It connects existing paid developmental content and function
  appearance; it does not establish a new actuator force or muscle law.

  Links: area Bodies and anatomy · draws on 511, 512, 531, 758 · governs isocosm::development, isocosm::datasheet::native, mesocosm-genet::section · summary Existing paid body chains and function appearance connect to ordinary play.


- **D1, surface camera (2026-10-21), applies 382:** Mark: "can we... make the
  camera isometric? just seems odd to prioritize looking at the cutaway of the
  ground, when most the stuff is happening on the surface... probably needs
  options". This applies the wing's selectable isometric view to Mesocosm's
  default, reopening its older shallow-section preference. Section views stay
  selectable. Articulation is requested over the existing graph and physical
  machinery, without inventing appendages that anatomy does not declare.

  Links: area Presentation · draws on 382 · governs mesocosm-genet::section::camera · summary Mesocosm defaults to a selectable isometric surface view with section options.

- **D3, the camera decides neither the sim nor the rendering (2026-10-28), refines D1:** Mark: "It can be axonometric, could be oblique, could be isometric, could be diametric, could be trimetric, could be... You know, it really just, it does, that's not the most important thing. The most important thing is that the question of the camera is not determinative of the question of the sim or the question of the rendering, necessarily. Unless we decide to do quantized facing, which I don't think we're doing, necessarily." Which projection a view uses is a host setting; D1's "isometric" and doctrine Presentation 1's 2:1 dimetric need no reconciling.

  Links: area Presentation · draws on 382, presentation.1, orthographic-voxel-presentation D1 · refines orthographic-voxel-presentation D1 · summary The projection a view uses is a setting; the camera determines neither the sim nor the rendering.

## Constraints
- Ground and bodies share one depth attachment; a body's parts are scene
  instances (rulings 13 and 14). Mark, ruling 14: "One surface per body is
  refused: it would lose depth between bodies again." Source:
  `archive_docs/2026-10-28/2026-09-11_orthographic_voxel_presentation_plan.md:1049-1111`.
- The default view is the 2:1 dimetric at 30 degrees in quarter turns, in
  every product; other views stay selectable and Mesocosm's section stays
  (doctrine Presentation 1; D1 above). Mark, doctrine Presentation 1: "Follow
  the wing: 2:1 dimetric, quarter turns".
- Rung choice is a pure function of view and what the sim holds: no thing shows
  a rung finer than the sim holds for it; examining lifts detail (ruling 424,
  doctrine Presentation 2). Mark, on whether approaching examines: "Per mode".
- Collision envelopes stay rigid; native joint constraints are a separate
  condition, owned by `bodies` (doctrine Presentation 4).
- No upload-silence or performance claim stands until counters are measured
  (PR1). Bodies carry no appendage the anatomy does not declare (D1).

## Phases

### L1. Livery 3D transforms (genet scope)
Genet's T1 and T2 landed 2026-09-15; its T3 harness was revised after rulings
13 to 15. The done-condition has parts nobody has recorded closing.
- Done when: "genet's plan founds it with its WPT `css/css-transforms` counts; a wing fixture body of rigid parts renders through Ortet with the same silhouette the appearance crate's bake produces; individual transform properties animate a part's yaw without touching the parent matrix; T2's done condition holds on the L0b grid; T3's harness reports a live per-frame number for a body whose parts move every frame."
- Needs: genet `design_docs/2026-09-12_css_3d_transforms_and_first_frame_plan.md` (genet-owned)
- Rulings: presentation rulings 3 and 11
- From: `archive_docs/2026-10-28/2026-09-11_orthographic_voxel_presentation_plan.md:1252-1278`

### L3. Shared scene depth and document composition (re-proof on the tenant)
Bench B proved a native consumer; the 2026-10-18 banner asks for a re-proof
after the tenant migration.
- Done when: "the body/terrain scenes from L0c, including a long part over a ledge, render with equivalent poses and declared coverage/tie tolerances; the viewport composites between DOM siblings under clips, transforms and opacity; resize and producer retirement preserve resource identity; unchanged geometry has no static re-upload; and producer, staging and document work are timed separately from presentation wait. Interior-only rounded agreement does not establish exact pixels or picking."
- Needs: PR1
- Rulings: presentation rulings 13, 14
- From: `archive_docs/2026-10-28/2026-09-11_orthographic_voxel_presentation_plan.md:1305-1322, 1018`

### L4. Ground as tile layers, underground included
Rewritten once by ruling 13 (the resident chunk drawn with the slab); no code
found. The underground cutaway must stay possible. Whether the section keeps the
tracer by default or also moves to tile layers is a question below.
- Done when: "a carved burrow is visible through the cutaway with no tracer in the frame; a radius-zero carve regenerates only the ground units its brick touches, proved by fragment or instance identity; unit count and bytes per revision are reported for one map of each vessel; Isometry's board draws the same picture through the chosen path as from its tile elements today, with the same tiles selectable."
- Needs: the Mesocosm-section question below; `board-on-isometer` for the board's half
- Rulings: presentation rulings 5, 13
- From: `archive_docs/2026-10-28/2026-09-11_orthographic_voxel_presentation_plan.md:1324-1348`

### L5. Live body instances, appearance and interaction (re-proof on the tenant)
Bench A and B supplied first consumer proofs; the tenant needs its own.
- Done when: "a multi-part body turns continuously with nonzero attachment pivots and unchanged static geometry uploads; style changes affect the intended instances; pointer selection resolves the visible part while list selection may address an occluded part with a depth-correct highlight; both reject stale anatomy; and a part edit preserves unrelated cached geometry."
- Needs: PR1
- Rulings: presentation rulings 10, 14, 15
- From: `archive_docs/2026-10-28/2026-09-11_orthographic_voxel_presentation_plan.md:1350-1369`

### L6. GPU sprite bake on the resident lane
A CubeCL kernel writes the sprite atlas; downlevel keeps the CPU bake. It feeds
L10 rung R1. Not built.
- Done when: "a GPU-baked sheet matches the CPU bake within the coverage tolerance the render tests already use; one device, one allocation, no per-frame upload on an unchanged body."
- Needs: `place-graph-engine` VRAM residency policy and the resident chunk; the GPU-bake question below
- Rulings: presentation ruling 8 (angle 1)
- From: `archive_docs/2026-10-28/2026-09-11_orthographic_voxel_presentation_plan.md:1371-1381`

### L7. Renderling exits Paredros
Verify, then close or finish. Only the first condition has been checked (see Findings).
- Done when: "`eponym-client` builds with no renderling dependency; the S0 replay hash is unchanged; the room composes through the same layers as Mesocosm's section; the probes that still want renderling are archived, not patched."
- Needs: PR1
- Rulings: presentation ruling 2; 793; case 749
- From: `archive_docs/2026-10-28/2026-09-11_orthographic_voxel_presentation_plan.md:1383-1393, 1018`

### L8. HTML as interchange
A body, prop or memorial as a self-contained HTML fragment with a declarative
shadow root. Not built; genet's Shadow DOM has landed and nothing blocks it.
- Done when: "a body round-trips document to HTML to document with the same part placements and palette; Isometry loads a Mesocosm-authored body from that fragment over P2P; the hagiograph emits a memorial as one such fragment."
- Needs: `wing-organs` (the hagiograph's memorial)
- Rulings: presentation ruling 7, angle 3
- From: `archive_docs/2026-10-28/2026-09-11_orthographic_voxel_presentation_plan.md:1395-1407`

### L10. The detail ladder
Rungs R0 live parts, R1 baked sprites, R2 a mark per individual, R3 an
aggregate mark per group, R4 the world map. Terrain residency becomes rings,
outer rings coarsening first (a reading, not ruled), inside 16,383 bricks and
8 MiB (296). Transitions are seamless where rungs share geometry and an ordered
dither with hysteresis where they do not (426). Far marks carry glyph displays
rasterised once to an opaque pixel atlas (432, 434, 438); the minimap and
overmap retire into second cameras held at R4 (437). It also supplies what the
site overview (`O`, the coarse terrain view) lacks today: distant body
visibility and landscape landmarks. Owner tree: `shared/isometer`, beside the
residency helper (291).
- Done when:
  1. "Rung choice is a pure function in isometer, tested over seeded views and holdings: no thing shows a rung finer than the sim holds for it, and a zoom oscillating inside a hysteresis band never changes a rung. A choice with the bound removed, and one with a zero band, are the controls that must fail.
  2. Over seeded maps and views the ring set stays within 16,383 bricks, base grain covers the examined region, and the level rises by one per ring outward. A 2,047-brick control coarsens the outer rings instead of refusing, and the specimen bench runs on the host's budget.
  3. A receipt shows the GPU pass landing the bake's pixels and face shading at the R0 to R1 threshold for seeded bodies at all four facings, with a perturbed palette as the control. Until it holds, R0 and R1 dither.
  4. Every terrain slide frame has integer column tops between the parent's and the column's own, ending at the two lifts, and every plate-easing frame holds the site's exact mean, checked on the numerator; an ease that does not hold it is the control.
  5. Every pixel of every dithered frame comes from exactly one of the two rungs' pictures, the same for the same thing and frame; an alpha-blended control is caught.
  6. An individual's mark shows exactly the known glyphs its filter passes, and a group's shares equal its histogram's counts; an effect the viewer cannot know is the control and never shows. Rests on S7.
  7. The atlas holds each glyph's display once, as opaque pixels at mark size, and marks drawn from it occlude and are occluded by the scene at its depth. A canon revision moves which icon an effect shows without rebuilding the atlas: the receipt counts atlas builds, and a control that rebuilds on revision shows a second build.
  8. The VTT and Mesocosm each show their map through a second camera held at R4, matching the sites, routes and readings their retiring overmap and minimap drew, and the canvas and HUD renderers are removed."
- Needs: `sim S7` (conditions and modifiers, for condition 6); the lift (SP2, built; `place-graph-engine`) and its coarse-lift passage item for conditions 2 and 4; L6 (rung R1); `place-graph-engine` VRAM residency policy; `wing-organs` glyph displays; the overlay contract; `vtt-overlay V2` for overmap readings
- Rulings: 212, 291, 296, 382, 387, 423, 424, 425, 426, 432, 434, 437, 438, 427, 49, 207, 431, 801
- From: `archive_docs/2026-10-28/2026-09-11_orthographic_voxel_presentation_plan.md:2365-2491`

### L10b. Condition 8 for Mesocosm
Folded in 2026-10-10 from the views founding plan (ruling 801). Condition 8
also holds when "a region's dominant holder visibly shifts between two captures
of one long draw under a seed nobody chose; when its regions are the place
graph's own nodes once places reach Mesocosm; and when a region is a hit target
selecting its place." Fields evaluated over a place stay consumer-pull. The
adapter-first rule stands: the views crate discloses facts and never computes
layout; region meaning is per game, derived at projection time and never
stored; the cambium lane is ordinary chrome for text.
- Done when: as quoted above
- Needs: L10; `mesocosm-overlay M4.6` (SP7) and `place-graph-engine SP8`
- Rulings: 801, 783
- From: `archive_docs/2026-10-28/2026-09-11_orthographic_voxel_presentation_plan.md:2477-2491`

### PR1. Headed certification of the tenant
Mesocosm's lit bodies against the traced terrain have not been seen on a GPU.
Look at occlusion, palette and selection under each camera and after a resize in
a headed run, and at body voxel scale against the ground cells, which still
needs tuning. Compare the captured final layered master with the window,
including the chromeless route. Then measure the presentation-only
`--body-light` setting, per-frame CPU palette reconstruction and per-face pose
validation, mesh uploads on motion and camera cuts, marked-face subdivision and
the retained terrain receipt's extra import. The body receipt's mesh-byte
estimates omit the tenant's camera and light uploads.
- Done when: not stated; owed. Standing text: "no upload-silence claim stands until counters are measured"
- Needs: a headed GPU run
- Rulings: case 970 (tenant sun), case 749
- From: `archive_docs/2026-10-28/2026-09-11_orthographic_voxel_presentation_plan.md:1018, 2493-2505`; `archive_docs/2026-10-28/2026-09-25_mesocosm_overlay_plan.md:303-304`

### PR2. `LiveBody` retirement and test debt
Migrate the remaining `LiveBody` consumers to the tenant before retiring it
(796). Two `isometer-lens` tests were failing in the 2026-10-17 run, the roster
test ("the second body is drawn beside the first, not instead of it") and the
unlit test ("no unfogged ambient-lit rock face in either frame"); the mass-balance
attachment test in `isometer-mesh` had no later run. Fix each, or retire it
under case 672 naming where the invariant is now checked. The `Quad::corners`
winding was recorded as reversed for y faces against its own doc comment;
re-check it and note the result for the owner.
- Done when: not stated; owed
- Needs: -
- Rulings: 796; case 672
- From: `archive_docs/2026-10-28/2026-09-11_orthographic_voxel_presentation_plan.md:2506-2516, 2800-2803`

### PR3. Water and surface appearance
Palette correction gives water colours only: no transparency, underwater
visibility or physical water behaviour, and the tracer treats water as opaque.
Transparent compositing is also listed open under the benches. Rendering
environmental surfaces as voxel appearance over the traced ground is the
isometer family's (`vtt-overlay V3` owns the surfaces themselves; the board's
stylesheet-versus-voxel choice is `board-on-isometer`).
- Done when: not stated; owed
- Needs: `board-on-isometer` (surface appearance on the board)
- Rulings: -
- From: `archive_docs/2026-10-28/2026-09-11_orthographic_voxel_presentation_plan.md:1007-1009, 2083-2086`; `design_docs/archive_docs/2026-10-28/2026-07-08_environmental_surfaces_plan.md:68-69`

### PR4. Terrain per-axis scale
Case 23: terrain gets per-axis scale (x, y, z) under W3, and paging is fixed one
way or another in every consumer.
- Done when: not stated; owed (case 23 is the condition)
- Needs: -
- Rulings: case 23
- From: `archive_docs/2026-10-28/2026-09-11_orthographic_voxel_presentation_plan.md:1018`

### PR5. Tenant sun leftovers
The dither-like hatch on one cube's front face went away but was not diagnosed
(likely coplanar quads of different baked tint). The tonemap stays kiss3d
Neutral, unmeasured above 0.76. Terrain tops shift hue from grey to blue at low
light under the grade's ladder, which the lane flagged for a look. No
performance or upload claim stands.
- Done when: not stated; owed
- Needs: PR1
- Rulings: 970, 472
- From: `archive_docs/2026-10-28/2026-09-11_orthographic_voxel_presentation_plan.md:2879-2911`; `archive_docs/2026-10-28/2026-08-14_resident_views_composition_plan.md:178-236`

### PR6. Rich bodies read on the tenant path
Crowded silhouettes and the classic terrain's dense pattern still obscure
anatomy, and a second independent seed had no supported controlled pose, so no
picture was shown for it. D2 asks to run an unchosen seed and capture bodies
moving to see whether they read. The body gates for blending and strength are
separate work.
- Done when: not stated; owed
- Needs: `bodies BO1`, `bodies BO2`
- Rulings: D2; cases 511, 512, 531, 758
- From: `archive_docs/2026-10-28/2026-09-11_orthographic_voxel_presentation_plan.md:453-464, 486-491`

### PR7. Bench residuals
GPU ID picking and independent continuous joint animation follow their own
measured needs. Any future cheap bounds cache stays conservative (the core
AABB omits attachment yaw and noncentral pivots); the percentage-padding query
approximation is not promoted to a conformance claim; whole-game migration to
the Cambium native host stays separate. DOM elements intersecting objects inside
the depth surface, and accessible descriptions of scene objects, each need a
named integration owned by genet T3. Layoutit-inspired follow-ons, without
expanding Bench B's gates: a portable inspection view extending the saved
generation request with pose, camera and part selection; geometry editing and
export that derive visibility from the same shape data as the scene and
queries; mixed-shape boundaries as an explicit agreement fixture if such shapes
are admitted (GPL-2.0 source studied for interaction only; nothing
incorporated).
- Done when: not stated; owed
- Needs: genet T3
- Rulings: -
- From: `archive_docs/2026-10-28/2026-09-11_orthographic_voxel_presentation_plan.md:1189-1193, 1454-1462, 1529-1535, 1572-1575, 1609-1611`

### PR8. Genet ledger rows for earmarked CSS features
"Each row that opens a lane gets a ledger row in genet naming this doc as the
consumer. That edit is genet's, in the same session the lane opens." Tier 0
rows: 3D transforms, `image-rendering: pixelated`, custom properties and
`@property`, `color-mix()` and oklch. This doc is archived, so the ledger rows
should name this plan.
- Done when: not stated; owed
- Needs: genet `design_docs/2026-09-07_standards_to_features_ledger.md`
- Rulings: -
- From: `archive_docs/2026-10-28/2026-09-11_orthographic_voxel_presentation_plan.md:2518-2546`

## Deferred and parked
- **Perspective camera**: `SlabCamera` is orthographic; reserved for a mode that needs first person or over the shoulder. Mark, 2026-09-28, put without objection: "the camera reserves room for a perspective projection now and builds it when a mode needs one" (donor: Eponym `r1-proof`). The old exclusion list also bars perspective cameras in any vessel. Waits on a mode that needs one. Rulings 382, 387. From: `archive_docs/2026-10-28/2026-09-11_orthographic_voxel_presentation_plan.md:1168-1179, 2577-2578`.

## Questions for Mark
- **Mesocosm section by default (L4)**: whether the tracer keeps Mesocosm's section by default, or the section also moves to tile layers once L4 lands. From: `archive_docs/2026-10-28/2026-09-11_orthographic_voxel_presentation_plan.md:2587-2624`.
- **GPU bake profile (L6)**: whether L6's GPU bake targets the enhanced capability profile only with the CPU bake as the downlevel tier, or replaces the CPU bake outright. Same source.
- **Poses and clips**: whether a pose (a set of part transforms) and a clip (keyframes over poses) become body-document data played as CSS animations on the individual transform properties, with player-set facing slots as stored poses; and which wing plan owns which poses exist and when they fire (presentation here, behaviour elsewhere). Raised 2026-09-12. Same source.
- **Hybrid switch policy**: the old plan reads it as answered by the ladder (424), marked "Reading, not ruled"; genet's T2 landed, so it is now a quality choice rather than a scaling necessity. Confirm or rule. Same source.
- **Far-rung function tint**: whether the far rungs of the detail ladder (doctrine Presentation 2) need the function tint, or only the marks. From: `archive_docs/2026-10-28/2026-10-22_body_appearance_plan.md:302-303`.
- **L10 readings not ruled**: that outer rings coarsen first; that the glyph rasteriser is netrender's text path (parley, skrifa, vello) run once per glyph, cached and thresholded to opaque pixels. Bench readings still owed with numbers: thresholds, hysteresis bands, each mode's default glyph filter, dither pattern and frame count; and whether the GPU pass lands the bake's pixels and face shading (the silhouette test proves coverage only). Mark's "Per mode" already settles who decides whether examining fires on approach. From: `archive_docs/2026-10-28/2026-09-11_orthographic_voxel_presentation_plan.md:2395-2400, 2420-2422, 2469-2476, 2601-2605`.
## Findings
- 2026-10-28: `shared/isometer/crates` holds core, lens, mesh, render and space. A grep finds no L10 machinery (no rung or ring set, no rung hysteresis; `isometer-lens` has a tracer palette dither, which is not the rung dither), no `gpu_bake`, and no HTML serializer.
- 2026-10-28: no `renderling` in any manifest or lock of this repository; Eponym's README says no local fork is needed since L7. L7's first condition holds; the S0 replay hash and the room-composition condition are unchecked.
- 2026-10-28: the old claim that `LiveBody` serves the VTT board and Eponym's producer no longer matches the code. Outside `shared/isometer`, only Mesocosm's tenant geometry and a scene-cache test name it. PR2 should reread the consumers before retiring.
- 2026-10-28: the three tests named in PR2 still exist (`isometer-lens` `tracer_tests/roster.rs:83` and `unlit.rs:437`, `isometer-mesh/tests/attachment.rs:59`); their pass state is unverified. `Quad::corners` (`isometer-mesh/src/greedy.rs:49`) documents counter-clockwise from outside; the y-face reversal is not re-checked.
- 2026-10-28: the VTT overmap (`crates/isocosm-vtt/src/overmap.rs`) and Mesocosm's minimap and HUD (`mesocosm-genet/src/{maps,hud}.rs`) still exist, so L10 condition 8 has nothing done.
- 2026-10-28: the tenant now lights with a sun (`section/tenant/geometry.rs` shades by the world normal), so the sim host plan's finding that the body tenant lit with ambient only is closed. Its default-launch capture finding was a one-run observation and is dropped.
- 2026-10-28: doctrine Presentation 1 records Mark's ruling that Mesocosm opens in the 2:1 dimetric default, which closes the overlay plan's open question on the default view. Doctrine Presentation 2 records Mark's "Per mode" on whether examining fires on approach.
- 2026-10-28: pres rulings 2 and 5 (bodies and props as DOM, ground as tile layers) are overridden by rulings 10, 13 and 14 (path C); ruling 6 is answered by the ladder. The old plan's L0 and resource paths name crates that no longer exist (`mesocosm-mesh`, `mesocosm-render`); the appearance crate is `isometer-mesh` and the renderer `isometer-render`.
- 2026-10-28: the passage-survival gap at coarse levels is real (`isometer-space` `lift/chunk.rs` samples each coarse column at its cell origin; `edit/replay.rs` keeps only multiples of the step), and no L10 condition checks it; it is `place-graph-engine`'s item, and L10 conditions 2 and 4 wait on it.

## Progress
- 2026-10-28: cut from the orthographic voxel presentation plan and the open items of the overlay, environmental surfaces and sim host plans.

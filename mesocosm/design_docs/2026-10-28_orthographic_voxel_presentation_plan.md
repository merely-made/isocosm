# Orthographic Voxel Presentation

**Status:** 2026-10-28: cut from `2026-09-11_orthographic_voxel_presentation_plan.md`
(and open items of the Mesocosm overlay, environmental surfaces and sim host
plans), which are archived in `archive_docs/2026-10-28/`. `shared/isometer`
(core, lens, mesh, render, space) is the scene crate, and Mesocosm runs the
tenant path with smooth bodies and the 2:1 dimetric surface view as defaults.
Open: headed GPU certification, the lane re-proofs on the tenant path, GPU
bake, HTML interchange, and the whole L10 detail ladder, none of which is in
the product path. Bounded renderer probes below exercise only part of R0/R1.

2026-11-03: reviewed against the current foreground and denizen plans. The
product presentation proposal below is unruled; perspective camera work is
owned by `isocosm/design_docs/2026-11-02_denizen_plan.md` P1.
The near-live/far-cached hybrid with adaptive image refresh is accepted on
2026-11-03 (case 1161). Its production integration is planned below; the
broader ordinary-view proposals and numerical tuning remain unruled.

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

## Decisions

- **D2, rich bodies (2026-10-22), applies 511, 512, 531, 758:** Mark:
  "Reconnect the richer bodies, functional colours, and actual limb chains,
  reusing what's there. Then run an unchosen seed and capture them moving so
  we can see whether it reads." This explicitly lifts the native adapter's
  chain refusal. It connects existing paid developmental content and function
  appearance; it does not establish a new actuator force or muscle law.

  Links: area Bodies and anatomy · draws on 511, 512, 531, 758 · governs cosmograph::development, cosmograph::datasheet::native, mesocosm-genet::section · summary Existing paid body chains and function appearance connect to ordinary play.


- **D1, surface camera (2026-10-21), applies 382:** Mark: "can we... make the
  camera isometric? just seems odd to prioritize looking at the cutaway of the
  ground, when most the stuff is happening on the surface... probably needs
  options". This applies the wing's selectable isometric view to Mesocosm's
  default, reopening its older shallow-section preference. Section views stay
  selectable. Articulation is requested over the existing graph and physical
  machinery, without inventing appendages that anatomy does not declare.

  Links: area Presentation · draws on 382 · governs mesocosm-genet::section::camera · summary Mesocosm defaults to a selectable isometric surface view with section options.

- **D3, the camera decides neither the sim nor the rendering (2026-10-28), refines D1:** Mark: "It can be axonometric, could be oblique, could be isometric, could be diametric, could be trimetric, could be... You know, it really just, it does, that's not the most important thing. The most important thing is that the question of the camera is not determinative of the question of the sim or the question of the rendering, necessarily. Unless we decide to do quantized facing, which I don't think we're doing, necessarily." Which projection a view uses is a host setting; D1's "isometric" and doctrine Presentation 1's 2:1 dimetric need no reconciling.

  Links: area Presentation · draws on 382, presentation.1, orthographic-voxel-presentation D1 · amends orthographic-voxel-presentation D1 · summary The projection a view uses is a setting; the camera determines neither the sim nor the rendering.

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
- Done when: "a carved burrow is visible through the cutaway with no tracer in the frame; a radius-zero carve regenerates only the ground units its brick touches, proved by fragment or instance identity; unit count and bytes per revision are reported for one map of each vessel; Isocosm's board draws the same picture through the chosen path as from its tile elements today, with the same tiles selectable."
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
- Done when: "a body round-trips document to HTML to document with the same part placements and palette; Isocosm loads a Mesocosm-authored body from that fragment over P2P; the hagiograph emits a memorial as one such fragment."
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

#### R0/R1 hybrid and image refresh (accepted direction, case 1161)

Accepted on 2026-11-03: live creatures up close, cached sprites farther out,
with image refresh slowing as the view zooms out. Case 1161 refines L10's
existing projected-size choice and examining override with a separate
image-refresh policy inside R1. Orthographic depth is not projected size.
Image quality and E8/E10 decision/physical cadence remain independent;
examination remains bounded by the detail the sim holds.

The bounded `isometer-render` example replays the retained generated creature
and native foot poses. Settings remain probe configuration, not product
defaults: enter live at 56 projected pixels and leave at 44; sprite refresh
every 2, 4 or 12 displayed frames, with bands around the rate boundaries.
Position and depth follow the current root every frame; only local limb
pose is held. One examined subject stays live while its neighbours use R1.
Unproved R0/R1 parity keeps L10's ordered dither, including the temporary
cost of rendering both pictures. Existing L6 and the full L10 conditions
remain open.

Probe done-condition (met; archived receipt): captured zoom levels and limb
poses show the mixed scene; release timings include refresh, held frames and
a cold-entry/dual-render switch; oscillation within the rung/rate bands is stable, with zero-band
and removed-holding-bound controls; each switch pixel comes from one input;
root transforms remain current on held-pose frames; settings and limits are
retained in the existing receipt. Retained-output cap: 100 MB in a fresh
hybrid-comparison lane. Residency eviction, invalidation beyond pose and
camera zoom, budgeted refresh scheduling and product wiring stay open.

Owner split: isometer owns projected-bound selection, explicit previous
choice/bands, held-detail bounds, representation composition and image-cache
work. Games own quality presets and when a subject is examined (Presentation
2, case 1161). Per-product integration: Mesocosm keeps an examined specimen
live; Eponym can examine its controlled or closely inspected actors; Isocosm's existing BD6
already reserves sprites for the far/many-token tier. None changes game
outcome authority. Sprite image cadence is a presentation setting independent
of both native physical resolution and lineage decision periods.

Before product integration: key cached pictures by anatomy/appearance,
orientation and lighting revisions; keep current world transforms separate
from held local poses; bound atlas memory and refresh work under the existing
residency owner; allow staggered refresh; invalidate/prefetch on promotion;
measure cache-resolution choices at small projected sizes. A fixed 64-pixel
bake plus one examined live body leaves a noticeable floor even when the
neighbours occupy ten pixels. R2 marks remain the next far rung. The prototype
uses equal-size cloned specimens and an examining override under orthographic
projection, not a perspective mixed-distance scene or the final per-body
multi-rung selector.

Production done-conditions, still open:

1. The shared selector chooses per subject from projected bounds, explicit
   prior choice, examination and held detail, under both orthographic and
   perspective cameras. In-band oscillation changes neither rung nor refresh
   class; removed-band and removed-holding-bound controls fail. Promotion
   requests missing detail through the existing examination path.
2. The resident cache keys anatomy/appearance, facing and lighting revisions;
   current root transforms and picking identities remain independent of held
   limb images. Growth, injury, equipment and changed lighting invalidate the
   affected pictures. Entry and re-entry validate current source revisions
   and allowed pose age before reusing an image.
3. L6 and `place-graph-engine PG5` bound retained bytes and refresh work, with
   current counters for occupied slots, bytes, hits/misses, refreshes and
   transition peaks. Eviction, synchronized invalidation and rapid zoom reversals
   stay within the declared budgets. Compare staggered refresh and smaller
   cache resolutions before choosing defaults; R2 remains the next far tier.
4. Product adapters consume that shared mechanism and expose quality settings.
   Mesocosm verifies changing native anatomy, Eponym verifies close interaction
   and inspection, and Isocosm BD6 verifies selection, fog, depth and faithful
   edition outcomes. The same input and authoritative state produce the same
   outcomes across image settings; E8/E10 retain their own clocks. The full
   L10 acceptance list, including L6 parity and transition depth, still applies.

The recorded GPU probe informs the direction and budget questions. Production
acceptance is still open; pixel thresholds, refresh rates, cache resolution
and transition length remain configurable tuning gates.

Status, 2026-11-08 (Findings): conditions 1 and 2 hold in shared code.
Selection, the cache and the production bake, composite and pick path are
`5f95cd40` and `9e31bce4`. All three refresh boundaries have zero-band controls.
A product reports held detail through `SceneHost::hybrid_held` (case 1171). A
held sprite bound now keeps even an examined subject a sprite.

The shared body pass and the sprite bake light by the host's sun
(`SceneHost::body_light`, case 1170). Images are keyed by a stamp of that sun,
rounded to `HybridSettings::light_step`. Without a sun the pass keeps its fixed
face shade, so other products are unchanged.

Mesocosm's part of condition 4 has a wound receipt on the device and a headed
generated run (M4.7 in the overlay plan). Hybrid stays opt-in in Mesocosm
until the zoom dither no longer re-renders the whole scene (34–59 ms p95 per
frame in run 4) and the loss of the smooth mode's contact gait is settled
(case 1175, reopening 1172). The light step defaults to 1/32 (1177). The residency bound is a byte budget in `HybridSettings`
(`scene/residency/body_images.rs`), and condition 3's L6/PG5 budgets would
attach there. Condition 3 itself stays open.

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

### Product presentation choices (proposal, 2026-11-03)

Reading, not ruled: the shared scene can serve different ordinary views per
game. Camera projection (D3), displayed representation (L10), physical
foreground (`sim-execution` E10) and decision cadence (`sim-execution` E8)
are separate choices. Voxel source data can produce a mesh or a baked image;
displaying either does not replace the world's spatial authority.

| Product | Candidate ordinary view | Detail worth paying for |
| --- | --- | --- |
| Isocosm | An illustrated board with baked terrain and token images; live miniatures for inspection or editing. | Legal positions, facing, elevation, conditions, equipment and the edition's decisions. |
| Eponym | Articulated bodies in a bounded resident part of the continuing world, with overview and close cameras. | Reach, contact, stance, injury and nearby motion; distant travel at place grain. |
| Mesocosm | A living diorama with articulated nearby organisms, selectable sections and ecological readings. | Changing anatomy, feeding, growth and incorporation; distant ecology through the population model. |

These are candidates to compare, not adopted defaults. Isocosm's board plan
BD6 currently chooses live recipe bodies and reserves sprites for the far,
many-token tier. A sprite-based ordinary board would amend that choice and
Presentation 2's finest-available-rung rule. Perspective already belongs to
denizen P1 (cases 1131 and 1140); it needs no second renderer. A bounded
resident scene keeps the same world and its consequences; it does not imply
separate encounter worlds or change who the player controls.

A bounded first comparison is an existing token recipe as a live body
and as its CPU-baked image in the same board scene. That is not yet a general
posed-native-body sprite pipeline or the full L10 ladder. Mesocosm's changing
part trees may favour reusable part meshes over baking every anatomy, pose
and facing. Replacing voxel spatial authority with layered tiles, heightmaps
or polygon navigation is a separate architectural proposal; L10's coarse
picture must preserve the authoritative passages and clearance queries.

Before adopting a candidate, compare the same source identities and revisions
in each representation: known facts, legal moves and edition outcomes stay
the same for the same inputs and attention; picks address the same subjects;
depth, fog and occlusion remain correct. Image-quality settings do not select
simulation rules. Changes to physical resolution follow E10's admission and
handoff contract and the existing exact-invariant/distribution checks. Time
baking, geometry rebuilds/uploads, rendering, movement search and background
ticks separately; speed numbers belong only to `sim-execution`.

The token comparison is authorized on 2026-11-03 (Mark: "same token recipe
as a live miniature and a cached sprite. show me how they look and what each
costs"). Owner: the isometer-render example, using the existing token recipe,
mesher and live-body renderer. The first controlled cache captures four
64-pixel colour-and-depth views of the live draw so camera and lighting agree;
this is an experimental GPU bake, not L6 or a replacement for the CPU baker.
Done when: retained native-size, enlarged and overlapping-board captures are
inspected; native-size equality is checked; release measurements separate
initial bake/cache bytes, CPU preparation/submission and GPU draw timestamps,
for static and moving tokens; the receipt names its adapter and sampling
method and limits claims to this probe. Retained-output cap: 100 MB. Receipts
go in this plan's existing archived receipt file.

An articulated follow-up is authorized on 2026-11-03: a generated Mesocosm
creature following a native admitted stride, compared as a live miniature and
a posed colour-and-depth sprite. Include recurring cache refresh, at full and
reduced cadence, and distinguish a shared pose from per-creature cache slots.
Native generation exports JSON from Cosmograph; the isometer-render probe
remains product-neutral. Done when actual limb-motion captures are inspected,
release CPU/GPU measurements include refresh frames and held frames, and the
receipt states sampling, cache memory, omissions and the chosen anatomy.
Fresh retained-output cap: 100 MB in the creature-comparison lane.

Measured: a generated four-legged, 34-part organism follows one admitted
stride through 24 native foot poses. A full-refresh 64-pixel sprite matches
all 24 native captures; the aligned 16-body board has identical silhouettes
and 11 shading-pixel differences. With 64 separate slots, refreshing every
frame costs 2.67--2.70 ms GPU versus 2.93--2.94 ms live; refreshing every
fourth frame averages 0.68--0.75 ms, with held limb poses. One shared pose
costs 0.58 ms including refresh and draw. These are the existing smooth
articulated renderer's headless costs, not the earlier rigid mesh or an
application budget. The archived receipt records CPU ranges, refresh spikes,
cache/workspace bytes, generator selection, controls and remaining limits.

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
- **Perspective camera (moved to active work)**: cases 1131 and 1140 supersede the old deferral under rulings 382 and 387. One shared isometer camera gains orthographic and perspective projections, with product presets and live switching. Owner: `isocosm/design_docs/2026-11-02_denizen_plan.md` P1. That plan still records the shared camera as unbuilt; an app-local proof is not its completion. Earlier deferral: `archive_docs/2026-10-28/2026-09-11_orthographic_voxel_presentation_plan.md:1168-1179, 2577-2578`.

## Questions for Mark
- **Mesocosm section by default (L4)**: whether the tracer keeps Mesocosm's section by default, or the section also moves to tile layers once L4 lands. From: `archive_docs/2026-10-28/2026-09-11_orthographic_voxel_presentation_plan.md:2587-2624`.
- **GPU bake profile (L6)**: whether L6's GPU bake targets the enhanced capability profile only with the CPU bake as the downlevel tier, or replaces the CPU bake outright. Same source.
- **Poses and clips**: whether a pose (a set of part transforms) and a clip (keyframes over poses) become body-document data played as CSS animations on the individual transform properties, with player-set facing slots as stored poses; and which wing plan owns which poses exist and when they fire (presentation here, behaviour elsewhere). Raised 2026-09-12. Same source.
- **Hybrid switch policy**: the old plan reads it as answered by the ladder (424), marked "Reading, not ruled"; genet's T2 landed, so it is now a quality choice rather than a scaling necessity. Confirm or rule. Same source.
- **Far-rung function tint**: whether the far rungs of the detail ladder (doctrine Presentation 2) need the function tint, or only the marks. From: `archive_docs/2026-10-28/2026-10-22_body_appearance_plan.md:302-303`.
- **L10 readings not ruled**: that outer rings coarsen first; that the glyph rasteriser is netrender's text path (parley, skrifa, vello) run once per glyph, cached and thresholded to opaque pixels. Bench readings still owed with numbers: thresholds, hysteresis bands, each mode's default glyph filter, dither pattern and frame count; and whether the GPU pass lands the bake's pixels and face shading (the silhouette test proves coverage only). Mark's "Per mode" already settles who decides whether examining fires on approach. From: `archive_docs/2026-10-28/2026-09-11_orthographic_voxel_presentation_plan.md:2395-2400, 2420-2422, 2469-2476, 2601-2605`.
## Findings
- 2026-11-08 (case 1176): the voxel-mode and hybrid colour paths share palette lookup, tint, the focus brightening (1.08) and tissue marks (`tissue_colour`, the same formula as `live_body.wgsl`). They differ in four places:
  1. The tenant renders through kiss3d's HDR film and its default Khronos PBR Neutral tonemap, which Mesocosm never sets. The shared pass has no tonemap.
  2. kiss3d writes `pow(c, 1/2.2)` into the linear view of an sRGB target, and the composite reads that back as sRGB. The shared pass encodes exact sRGB.
  3. The tenant shades with GGX (roughness 0.5, F0 0.04, diffuse times `1 - F`). The shared pass uses plain Lambert.
  4. The tenant mixes the selection accent before lighting; the shared pass mixes it after.

  Tonemap and encoding together account for nearly all of the near-view difference. A post-hoc model of the run 4 capture drops it from 39,546 px >8 / 805 px >24 / mean 4.9 to 196 / 154 / 0.43. A GPU render with the shared pass's opt-in `BodyDisplay::NeutralGamma22` gives 196 / 183 / 0.40 (run 5).
  - Each one alone makes it worse or does too little: the 2.2 encoding alone gives 14,777 px >24, and the tonemap alone gives 24,215 px >8.
  - The residual sits on silhouette edges (52 px) and one small face patch, consistent with the GGX term and raster differences.
  - Which look the single definition carries is with Mark.
- 2026-11-08: cases 1170 and 1171 are built.
  - `live_body.wgsl` decodes each face's axis from its fixed shade, rotates it by the instance model, and applies `ambient + sun * max(n·toward, 0)` in place of the shade, but only when a `BodyLight` is set. `isometer-render`'s 54 tests pass unchanged.
  - A sun moved by less than the quantum keeps the image: 0 refreshes and 1 hit at a step of 1/16. Moved past the quantum, the sprite rebakes, and an exact sun rebakes on any move (`scene/body_pass/tests/light.rs`). A held sprite bound keeps an examined subject a sprite, and with live detail held the subject promotes.
  - In the M4.7 near view, hybrid against voxels: pixels differing by more than 24 codes fell from 13,448 to 805, and the mean difference fell from 8.2 to 4.9 codes. A small offset remains between the shared pass and the tenant's raster and colour path.
- 2026-11-08: conditions 1 and 2, checked against `5f95cd40` and `9e31bce4`. The selector chooses per body under both cameras. Its zero-band controls now fail at all three boundaries: Medium/Slow was added in `hybrid/tests.rs`. The held-detail control fails as it should. Two gaps remain:
  - The scene passes `BodyDetail::Live` for every body (`scene/body_pass/hybrid.rs`), so the held bound is never exercised in a product.
  - The lighting stamp is `0`. `live_body.wgsl` and the sprite bake read no light uniform: face shade is fixed per axis.

  Stamps: appearance hashes tint, materials and palette; facing hashes camera axes and yaw; anatomy is `BodyDependencyRevision`. That revision covers surviving parts, volumes, pivots and attachment tags. An on-device test shows a native `Wound` rebaking one sprite while the sim's hash matches a control.

  In headed Mesocosm, the hybrid mode's live bodies come from this unlit shared pass. Voxel mode draws them through the sun-lit tenant. The two differ by about 35,000 pixels at the same near view (`m4-7-hybrid/2026-11-08/run3/compare-near-moved.png` under `Code/testing/isocosm`).
- 2026-11-03: `isometer-render --example hybrid_compare` replays the same generated native creature, holds local limb images at size-dependent cadence and applies current root transforms every displayed frame. Its explicit choice function passes rung/rate hysteresis and held-detail controls; a zero-width band changes tiers 63 times. GPU ordered selection preserves input pixels across 442,368 checks, while an alpha control invents 1,259 pixels. This is a bounded R0/R1 example, not completion of L6 or L10. At 40 projected pixels and 64 visible clones, one examined live body plus 63 sprites averages 1.68--1.69 ms GPU versus 2.47 ms all live; the dual-render switch peaks at 6.30--6.51 ms. The ten-pixel case saves much less GPU time with the fixed-size cache. Full timings, limits and captures are in the archived receipt.
- 2026-11-03: the authorized rigid-token comparison now runs as `isometer-render --example token_compare`, with the existing `demo::hero` recipe, mesher and cached live-body adapter. A four-facing colour-and-depth sprite cache matches the live draw at native size and on overlapping-token boards; enlargement exposes its fixed image resolution. This is an experimental GPU capture cache, not CPU-bake parity, a production sprite tier or full L10 acceptance. Measurements, adapter, captures and scope limits are in this plan's archived receipt file; no whole-game speed claim follows.
- 2026-11-03: the per-game presentation proposal fits D3 and L10, but an ordinary sprite board changes BD6's live-body choice. Code already has the appearance recipe and CPU bake (`isometer-mesh/src/bake/recipe.rs`, `bake/body.rs`); the native board still submits `SceneBody` instances (`isocosm-views/src/scene/board/native/source.rs`). E10 now has checked foreground handbacks (`cosmograph/src/spatial/handback.rs`), separate round pacing (`cosmograph-host/src/drive/round.rs`) and place-grain travel outside all participants' foregrounds (`cosmograph/src/spatial/foraging/travel.rs`). These are inspected source facts, not a new runtime or performance qualification.
- 2026-10-28: `shared/isometer/crates` holds core, lens, mesh, render and space. A grep finds no L10 machinery (no rung or ring set, no rung hysteresis; `isometer-lens` has a tracer palette dither, which is not the rung dither), no `gpu_bake`, and no HTML serializer.
- 2026-10-28: no `renderling` in any manifest or lock of this repository; Eponym's README says no local fork is needed since L7. L7's first condition holds; the S0 replay hash and the room-composition condition are unchecked.
- 2026-10-28: the old claim that `LiveBody` serves the VTT board and Eponym's producer no longer matches the code. Outside `shared/isometer`, only Mesocosm's tenant geometry and a scene-cache test name it. PR2 should reread the consumers before retiring.
- 2026-10-28: the three tests named in PR2 still exist (`isometer-lens` `tracer_tests/roster.rs:83` and `unlit.rs:437`, `isometer-mesh/tests/attachment.rs:59`); their pass state is unverified. `Quad::corners` (`isometer-mesh/src/greedy.rs:49`) documents counter-clockwise from outside; the y-face reversal is not re-checked.
- 2026-10-28: the VTT overmap (`crates/isocosm-native/src/overmap.rs`) and Mesocosm's minimap and HUD (`mesocosm-genet/src/{maps,hud}.rs`) still exist, so L10 condition 8 has nothing done.
- 2026-10-28: the tenant now lights with a sun (`section/tenant/geometry.rs` shades by the world normal), so the sim host plan's finding that the body tenant lit with ambient only is closed. Its default-launch capture finding was a one-run observation and is dropped.
- 2026-10-28: doctrine Presentation 1 records Mark's ruling that Mesocosm opens in the 2:1 dimetric default, which closes the overlay plan's open question on the default view. Doctrine Presentation 2 records Mark's "Per mode" on whether examining fires on approach.
- 2026-10-28: pres rulings 2 and 5 (bodies and props as DOM, ground as tile layers) are overridden by rulings 10, 13 and 14 (path C); ruling 6 is answered by the ladder. The old plan's L0 and resource paths name crates that no longer exist (`mesocosm-mesh`, `mesocosm-render`); the appearance crate is `isometer-mesh` and the renderer `isometer-render`.
- 2026-10-28: the passage-survival gap at coarse levels is real (`isometer-space` `lift/chunk.rs` samples each coarse column at its cell origin; `edit/replay.rs` keeps only multiples of the step), and no L10 condition checks it; it is `place-graph-engine`'s item, and L10 conditions 2 and 4 wait on it.

## Progress
- 2026-11-08: case 1176 audit. The shared pass gains an opt-in display encoding (`BodyDisplay`), and Mesocosm exposes it as `--body-display srgb|tenant`. The default stays `srgb`, so no product changes look. With `tenant`, hybrid matches voxel mode to 196 px >8 in the near view (run 5).
- 2026-11-08: cases 1170 and 1171 built: a sun-lit shared body pass and bake with a quantized lighting stamp, and `SceneHost::hybrid_held`. The M4.7 run4 receipt measures rebakes at four light quanta. Hybrid went back to opt-in (1175, reopening 1172); the light step defaults to 1/32 (1177); a colour-path audit against the tenant is next (1176).
- 2026-11-08: Mesocosm consumes the 1161 mechanism. Examination is played, followed or dev focus (Mark), and a zero-band control was added for the slow refresh boundary. A headed generated run shows identical native hashes across voxels, hybrid and half-scale hybrid, with switch and refresh costs measured (receipt). Held detail, lighting and default presets wait on Mark.
- 2026-11-03: Mark accepted the hybrid direction as case 1161; Presentation 2 now includes adaptive R1 image refresh, L10 has explicit production gates, and the board, product overlays, residency and simulation plans carry their ownership seams. Trial numerical presets remain tuning values.
- 2026-11-03: captured and measured the authorized R0/R1 hybrid probe and reverse-order confirmation; added configurable projected-size/rate bands, current-root composition, ordered switching and a captured zoom/motion comparison. Product defaults, cache residency and runtime integration remain open.
- 2026-11-03: built and ran the token representation comparison and its capture/cost viewer; the archived receipt records visual checks and release CPU/GPU measurements. Ordinary product representations remain proposals.
- 2026-10-28: cut from the orthographic voxel presentation plan and the open items of the overlay, environmental surfaces and sim host plans.

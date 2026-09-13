# Orthographic voxel presentation plan

**Date:** 2026-09-11

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
planar fragment repair remain independent work, not prerequisites for that
bench. See [the assessed slices](#specimen-bench-prerequisites-2026-09-13).

**Owns:** how the three vessels present voxel worlds and bodies through the
web engine, and the unification of Isometry, Paredros, and Mesocosm on one
presentation lane of broader use to the stack.

**Does not own:** world truth (`Ground`, the body document, the core hash),
resident compute (the [resident views plan](2026-08-14_resident_views_composition_plan.md)),
Livery's CSS conformance program (genet's own plans), or netrender's
execution graph (`netrender/netrender-notes/2026-09-04_wgpu_execution_graph_plan.md`).

**Prior art:**

- [PolyCSS](https://github.com/layoutit/polycss), MIT: polygon meshes and
  `.vox` volumes as DOM elements under `matrix3d`.
- [Layoutit Voxels](https://github.com/layoutit/layoutit-voxels/tree/35ee4110a8c9c66eb210d0ad0d184e66641a01fd),
  GPL-2.0-only: source studied on 2026-09-13 as an interaction reference for
  the specimen bench and parts examiner. No implementation is incorporated.
- [Zdog](https://zzz.dog), MIT: flat-shaded pseudo-3D drawn as 2D vector
  paths under 3D transforms.
- [Bonsai](https://github.com/scallyw4g/bonsai), WTFPL: already adopted three
  ways on 2026-08-07 (landscape §8.3); its meshing and deferred shading no
  longer apply here.
- GBA-era tactics games (Tactics Ogre, FFTA): many animated bodies as baked
  sprites at a locked angle. Isometry's founding look.

## Rulings (Mark, 2026-09-11)

1. **Paredros is orthographic.** A true isometric game. This closes the camera
   question the [Paredros execution plan](../../paredros/design_docs/2026-08-07_paredros_execution_plan.md)
   left open until the spatial laws could judge it, and it retires the
   Barony/Delver close-perspective reference lane. Camera is not person:
   Paredros stays second person in agency.
2. **The wing presents through genet and netrender.** Bodies and props are
   DOM under CSS; ground is layered. Renderling exits Paredros once L7's
   receipt holds. The engine of broader use is genet plus netrender plus
   conatus, and the three games are DOM consumers of one appearance crate.
3. **Livery gains CSS 3D transforms.** Authorized as genet scope. Genet's own
   plan founds it; this doc names the consumer.
4. **One appearance crate.** `isometry-voxel` and `mesocosm-mesh` merge into a
   wing-neutral crate that emits a sprite sheet or a face list from the same
   volume.
5. **Ground is tile layers, provided underground stays possible.** One layer
   per height step, cutaway by hiding layers above the focus. Layering is also
   the surface-interaction seam.
6. **Live faces are the default to scope first.** Mark wants to feel
   live-all-the-time; the expected landing is a hybrid of live faces in focus
   and during rotation, baked sprites elsewhere. L0 decides which is honest.
7. **HTML is an interchange format.** A body or memorial is a document.
8. **Angles 1-4 are adopted** (§ Novel angles): GPU sprite bake on the resident
   lane, ground as generated tile layers, the hagiograph as a consumer, games
   as documents.
9. **The probe runs first.** No lane past L0 lands before its receipt.
10. **L5 is body-level (Mark, 2026-09-12, from the L0 verdict).** A body or
    part is one DOM element whose paint is a retained netrender fragment
    below it, spliced the way genet splices an iframe's paint list into a
    replaced box. Faces never become elements. This promotes L3's face
    batch from "not yet earned" to the mechanism L5 runs on.
11. **L1's genet plan carries two more consumers (Mark, 2026-09-12):** a
    first-frame style and layout scaling slice, linear and about two orders
    cheaper per element, and a host-side DOM mutation harness with real
    timing. Founded in genet as
    `genet/design_docs/2026-09-12_css_3d_transforms_and_first_frame_plan.md`.
12. **Ordering is decided by comparison, not by construction (Mark,
    2026-09-12, superseding the same day's strip ruling).** Depth strips
    sorted in the 3D rendering context are candidate path A of L0c, beside
    cached sprites (B) and resident geometry with a depth attachment (C).
    The one-cell rule is a condition of the strip experiment, never an
    application constraint. Mark's wider ruling: the vessels may have
    isometric or orthogonal cameras, fixed or free, and quantized heights
    are an application setting; the engine establishes its resources first
    and applications take constraints from the receipts. See § Specifying a
    presentation.
13. **Path C is the main spatial renderer (Mark, 2026-09-13, from the L0c
    receipt).** Ground and mutually occluding bodies share one depth
    attachment in a scene viewport whose producer owns depth; genet receives
    the image through its document composition path. That is what the L0c
    probe's path C implements. Genet's T3 embeds that viewport.
14. **L5 puts parts into the shared scene as instances (Mark, 2026-09-13).**
    One surface per body is refused: it would lose depth between bodies
    again. Body and part identities stay stable, DOM controls stay
    optional, and an explicit mapping from supported style properties to
    instance data is the bridge that still has to be built and measured.
    CSS classes, semantic identity and GPU drawing coexist across it.
15. **Continuous presentation rotation (Mark, 2026-09-13).** A body
    transform above the articulated part transforms. The adapter's
    quantizer turns each part about its own pivot and leaves attachments in
    place, so fractional part yaw alone would not turn a body. Discrete
    simulation orientation stays useful on its own.

## The picture

```text
Ground / body document / accepted activity  (application authority)
    -> geometry + appearance + presentation pose
       -> shared colour + depth viewport    (one scene producer)
          -> ordinary SceneImage            (netrender external image staging)
             -> DOM paint order             (genet clips, transforms, opacity)
                -> one wgpu device          (host-owned device and queue)

DOM controls / resolved style -> admitted appearance and view settings
viewport-local pointer -> camera ray -> visible body/part identity
```

Parallel cameras remain useful presets. They do not require painter order
or a body splitting rule. Production Mesocosm already joins bodies and
traced terrain through one depth attachment; the bench carries that image
through the normal document paint stream. Other DOM content composites with
the viewport as a whole. Its pixels do not acquire the producer's internal
depth relationships. Planar fragments and sprites remain useful representations
with their own measured limits. External image staging is a GPU conversion
and copy on the shared device, not a zero-copy claim.

What this buys the stack rather than the games alone: CSS Transforms Level 2
is a web standard genet lacks and has WPT coverage for, so the Livery work
pays in turnstone and Pelt on real pages and in mere's spatial graph views,
which already carry padded 3D positions. Renderling paid off in one Paredros
crate and two probes, at the cost of a local fork, rust-gpu pins, and a
crabslab fork.

## Authority boundaries

| Layer | Owns | Must not decide |
| --- | --- | --- |
| `Ground`, body document, core | voxel facts, revisions, part placements, the hash | how anything looks |
| appearance crate | volume to sprite or face list; palette; facing; greedy merge | world facts; DOM structure |
| scene producer and application projection | scene camera, part pose, depth, visible geometry query, admitted appearance mapping | simulation facts; CSS interpretation |
| DOM + CSS (genet) | element layout, resolved style, document paint/input geometry, semantic controls | application part identity; scene depth; device |
| netrender Scene / compositor | affine rasterization, retained fragments, external textures, boundaries between the DOM lane, chrome and embedded surfaces | scene meaning; tenant internals |
| conatus / CubeCL | resident planes, GPU bakes into atlas textures | appearance policy |
| tracer (`conatus-brick`) | the optional live-volume lens for any camera that needs it | ground presentation by default |

The execution graph plan is not amended here. Its RG3 second consumer
becomes the tracer as an optional lens rather than renderling as a mesh
tenant, and D1's depth join stops being a promotion gate. That amendment is
Mark's, in netrender's notes, after L0.

## Specifying a presentation

Ruled 2026-09-12 with ruling 12. An application specifies these five
independently; the engine chooses batching, splitting, caching and
representation behind them and preserves object identity across whatever it
chooses. Isometric 2:1, orthographic isometric, and the terrarium section are
presets over this, not separate systems.

| Choice | The application states | The engine must not require |
| --- | --- | --- |
| Camera | projection, orientation, scale, permitted movement | a fixed camera for correctness |
| Cutaway | a world-space plane, slab or volume | a cutaway aligned to a height step or a screen row |
| Appearance | geometry, materials, markings, pose | pre-split parts to avoid a rendering defect |
| Visibility | which objects participate, which surfaces occlude | painter order as the only occlusion |
| Quality | resolution, animation cadence, detail, supported effects | one effect route for every object |

The boundary that stays explicit: a depth-rendered surface embedded in a
document is correct within itself, and ordinary DOM around it follows
document composition. DOM elements intersecting objects inside that surface,
picking, and accessible descriptions of those objects each need a named
integration, priced by L0c and owned by genet's T3.

Resources already present, verified 2026-09-12:

| Resource | Gives | Limit today |
| --- | --- | --- |
| `mesocosm-render/src/live_body.rs` | wgpu adapter with a depth attachment, instanced placements, cached geometry keyed by volume bytes, a world-space clip slab; no renderling | a product adapter, not a genet component |
| Paredros D1 receipt | headed proof that traced ground and raster geometry occlude each other | a fixture, not a cost comparison |
| `netrender/src/external_texture.rs` | GPU-produced content enters composition without CPU readback | staging on the image route is not free |
| genet host-content slots (`paint.rs`) | host-painted content inside CSS paint order | layer retention (T4) and 3D (T1) open |
| conatus resident planes, modulus traversal | shared spatial data with revisions and read epochs | each consumer binds and updates on its own policy |
| sceno identity and projection contracts | stable source and instance identity back to application meaning | no 3D picking or accessibility inside a surface |

## Lanes

Each lane names its owner tree and a done condition. L0 blocks everything.

### L0. The ceiling probes

Two untracked probes under `mere/crates/probes/` (the gitignored probe
workspace), receipts copied into `Code/testing/wing/`.

- **L0a, netrender face throughput.** Bodies as retained fragments of
  rectangles under CPU-side orthographic projection, bypassing Livery.
  Measure CPU scene build, Vello encode, and GPU frame time across a grid of
  body count and rectangles per body, static and with per-part transforms
  changing every frame.
- **L0b, genet element cost.** The same counts as absolutely positioned,
  transformed elements in a document through the Livery route. Measure style,
  layout, and paint per frame, static and with transforms mutated per frame
  where the scripted profile allows it.
- **L0c, the three-path comparison (measured 2026-09-12; equivalent yaw and
  content-variety follow-up measured 2026-09-13, picking still open).** One scene, one camera, one
  cutaway, one set of materials and motions, presented three ways from Rust
  into netrender with no genet DOM in the loop: (A) retained planar
  fragments, strips and parts under the six-cell placement, re-lowered
  through `Renderer::update_fragment` when a part turns or a strip's brick
  is dirty; (B) cached sprites from the appearance bake, one image per
  facing; (C) resident geometry through Mesocosm's `live_body` adapter into
  a depth attachment, composited through netrender's external texture. The
  scene carries six cases: two bodies crossing, a long articulated part, a
  terrain edit at a chunk boundary, a clipped viewport, and a material
  change, plus continuous yaw. Each path reports correctness against a per-pixel depth oracle,
  frame time, upload bytes, memory, and what a change invalidates. Path A's
  yaw re-lower cost, per re-lowered rectangle, is one of its numbers, and one
  of its cells wraps every body in a content-box clip layer, which is genet
  T4's rerun condition. What this cannot price is CSS ownership of per-part
  classes, effects and hit testing, which only runs through genet once T3
  and T4 exist; the receipt says so.
**Done when:** both receipts exist with device, driver, and commit named; the
table states the largest static and live element count under a 16 ms frame;
and the doc's § Receipts records whether live faces, the hybrid, or bake-only
is the honest default. L0c adds: its receipt exists at
`Code/testing/wing/l0c_three_paths.{json,md}`; all six cases have sampled pixel
comparisons and the receipt names each path's residuals and quality policy;
and § Receipts states, per path, frame time,
upload bytes, memory and invalidation on the same host and commit, plus for
path A the facing count at which quantized swaps cost the same as re-lowering.

### L1. Livery 3D transforms (genet scope)

CSS Transforms Level 2: `matrix3d`, `translate3d`, `rotate3d`, `scale3d`,
`perspective` and `perspective-origin`, `transform-style`,
`backface-visibility`, 3D `transform-origin`, and the individual `translate`,
`rotate`, `scale` properties. Lowering projects each element to netrender's
existing column-major 4x4 `Transform` and z-sorts within a 3D rendering
context. Netrender's `Transform` is ready; nothing in Livery parses these
today.

Founded in genet on 2026-09-12 as
`genet/design_docs/2026-09-12_css_3d_transforms_and_first_frame_plan.md`
with three lanes: T1 the transforms above; T2 first-frame style and layout
scaling, linear and about two orders cheaper per element, plus phase timing
behind a flag; T3 a host-side DOM mutation harness with real timing and the
scene viewport used by L5 (T3 revised after rulings 13–15). Ruling 11 names
the three engine consumers; T1 and T2 remain independent of the first bench.
The L0b attribution of the superlinear first frame
lives at `Code/testing/wing/l0b_first_frame_attribution.md` and seeds T2.

**Done when:** genet's plan founds it with its WPT `css/css-transforms`
counts; a wing fixture body of rigid parts renders through Ortet with the
same silhouette the appearance crate's bake produces; individual transform
properties animate a part's yaw without touching the parent matrix; T2's
done condition holds on the L0b grid; T3's harness reports a live per-frame
number for a body whose parts move every frame.

### L2. One appearance crate

This broader wing consolidation does not block the specimen bench, which
can consume the existing mesocosm-mesh and mesocosm-render APIs directly.

Merge `isometry-voxel` (recipe, palette, `bake_facing`, `Sheet`) and
`mesocosm-mesh` (body document, rigid parts, greedy `Quad`, `PartMesh`) into
one wing-neutral crate. Two projections from one volume: a sprite sheet at a
locked angle and facing, or a face list with per-face normal and palette
index. The body document stays the shared organ; nothing already on a body is
remeshed when a part attaches.

**Done when:** Isometry's tileset bake and Mesocosm's live body both build
from the merged crate with pixel-identical sheets and quad-identical meshes
to today's; a face list carries enough for L3 to emit DOM without touching
voxels again; the crate has no engine or wgpu dependency.

### L3. Shared scene depth and document composition

Ground and mutually occluding bodies use the same camera and depth target.
The application owns that scene producer; netrender stages its colour view
as an external image in the ordinary paint stream. Genet supplies the
content box and document geometry, while Mere's Cambium host supplies the
producer lifecycle and rendering hook. Reuse the existing custom-leaf splice,
DrawExternalTexture translation and external-image staging APIs. T4's
retained-fragment fix remains useful for planar content independently.

**Done when:** the body/terrain scenes from L0c, including a long part over
a ledge, render with equivalent poses and declared coverage/tie tolerances;
the viewport composites between DOM siblings under clips, transforms and
opacity; resize and producer retirement preserve resource identity; unchanged
geometry has no static re-upload; and producer, staging and document work
are timed separately from presentation wait. Interior-only rounded agreement
does not establish exact pixels or picking. Bench B below owns the first
end-to-end consumer of this lane.

### L4. Ground as tile layers, underground included

Materialize `Ground` bricks into ground units by the path L0c names, with
the cutaway specified as a world-space plane or slab (§ Specifying a
presentation) and applied by class or by clip slab: units past the cut hide,
units at the cut draw with their surface, units behind draw dimmed. Under
path A a unit is a depth strip, cells sharing a depth key under the vessel's
parallel projection, about two thousand elements for a 64 by 64 by 16 room;
under path C a unit is a resident chunk drawn with the slab. Isometry's
board today is the per-tile shape with `isometry_core::depth_key` as
z-index, a paint-order convention and not a definition of camera depth. The
tracer remains the lens for any camera that needs a live volume.

Destructible and constructible ground rides the existing revision contract:
`Ground` bumps its revision and drains dirty bricks; only the strips a dirty
brick touches regenerate, through the same content-replace call a turning
part uses; retained fragments keep every other strip. A carve, a burrow, or a placed block is the same path as
generation.

**Done when:** a carved burrow is visible through the cutaway with no
tracer in the frame; a radius-zero carve regenerates only the ground units
its brick touches, proved by fragment or instance identity; unit count and
bytes per revision are reported for one map of each vessel; Isometry's board
draws the same picture through the chosen path as from its tile elements
today, with the same tiles selectable.

### L5. Live body instances, appearance and interaction

Rulings 13–15 supersede the fragment-backed body/part element shape. Bodies
share one scene viewport, with cached geometry and part instance data.
Continuous parent presentation pose rotates attachments with their body;
core orientation and simulation records retain their own discrete semantics.
Body and part controls address the same identities as geometry picking.

Resolved style may supply a declared set of decorative appearance values.
These are distinct from biological phenotype fractions. Pose and appearance
changes update instance fields; topology or voxel edits invalidate only their
actual geometry dependencies. A sprite representation remains an optional
measured quality choice, not a requirement imposed by T2's large-DOM gate.

**Done when:** a multi-part body turns continuously with nonzero attachment
pivots and unchanged static geometry uploads; style changes affect the
intended instances; pointer selection resolves the visible part while list
selection may address an occluded part with a depth-correct highlight;
both reject stale anatomy; and a part edit preserves unrelated cached geometry.
Bench A and Bench B supply the first consumer proofs.

### L6. GPU sprite bake on the resident lane

A CubeCL kernel projects a resident volume into a sprite atlas texture that
netrender samples as a registered image. Zero CPU pixels: a runtime-attached
part is rebaked on the GPU and enters the DOM as a class change. This unifies
the engine review's R2 bake path with Isometry's lane. Downlevel keeps the CPU
bake.

**Done when:** a GPU-baked sheet matches the CPU bake within the coverage
tolerance the render tests already use; one device, one allocation, no
per-frame upload on an unchanged body.

### L7. Renderling exits Paredros

Paredros's room, D1, crossing, lighting, and residency paths move to the
appearance crate plus DOM. The `d1_depth` receipt is retired with rationale,
since the join it proved is replaced by L3 interleave. The renderling fork,
spirv-std pins, and crabslab fork leave the Paredros manifest.

**Done when:** `paredros-client` builds with no renderling dependency; the S0
replay hash is unchanged; the room composes through the same layers as
Mesocosm's section; the probes that still want renderling are archived, not
patched.

### L8. HTML as interchange

A body, prop, or memorial serializes as a self-contained HTML fragment: a
declarative shadow root carrying its own style and its face or sprite markup,
with the body document's identity as data attributes. Genet's Shadow DOM lane
landed 2026-09-07/08, and the parser/script interleaving plan closed
declarative attachment through the custom-element registry on 2026-09-08,
so nothing in genet blocks this lane.

**Done when:** a body round-trips document to HTML to document with the
same part placements and palette; Isometry loads a Mesocosm-authored body
from that fragment over P2P; the hagiograph emits a memorial as one such
fragment.

## Specimen bench prerequisites (2026-09-13)

**Status:** assessment, both standalone probe prerequisites, Bench A and
Bench B's bounded native integration are complete. Three parallel source
audits checked rendering, host embedding, and world/activity inputs.
The existing creator is the starting point, not a second specimen model.
Its prior headed receipts establish earlier source revisions; they were not
rerun in this assessment. Current source anchors were Isometry 5e98d14,
Mere 1f30117a, Genet 6ebd3598 and netrender 3961aca9, with concurrent
Rootstock input/layout and Genet scripted/realm edits left intact.

### Existing resources

- [Creator and habitat comparison](2026-07-31_wing_phenotype_contract_plan.md#held-body-and-habitat-comparison-2026-09-08):
  versioned seed/criteria requests, held-body rerolls, coalesced worker and
  stale-result rejection. Prepared candidates own disposable founding worlds.
  Current code is in crates/mesocosm-core/src/world/generation.rs and
  crates/mesocosm-genet/src/app/creator.rs.
- The same plan's trial-evidence section records bounded observations over
  accepted world activity. Prepared::observe currently returns aggregates;
  a visible stepping trial is the missing host join.
- crates/mesocosm-genet/src/section.rs already draws bodies and traced terrain with
  one depth attachment. mesocosm-render supplies instance tint, phenotype
  materials and upload/cache counters. VB1 and VB3 receipts live under
  Code/testing/mesocosm; VB3 explicitly leaves pointer picking open.
- Ortet has accepted ordinary external-image composition. Genet's T3 owns
  the engine geometry work and references Mere's missing Cambium producer
  bridge. The external_texture element constructor alone is not a connected
  rendering path.

### Interaction reference: Layoutit Voxels (2026-09-13)

Mark authorized source study as a reference independently of code reuse.
The inspected revision is `35ee4110a8c9c66eb210d0ad0d184e66641a01fd`.
Its face-addressed tools (`src/components/layers.vue`) suggest a direct
selection-to-inspection flow: the drawn part and its ordinary DOM reading
refer to the same organism, part and geometry revision. Bench B uses the
existing creator and `mesocosm_views::part_of` to expose that correspondence.
Selection expires when its preview or geometry is replaced.

The cube presentation separates simple geometry from face colour/texture
choices. The bench's declared appearance controls therefore exercise instance
updates while keeping resident geometry cached. Report staging and geometry
uploads separately. Palette experiments need no new body representation.

Two follow-ons are useful without expanding Bench B's gates. A portable
inspection view can extend the existing saved generation request with pose,
camera and part selection, retaining enough versioned inputs to reproduce it.
Geometry editing/export must derive visibility from the same shape data as
the scene and queries: Layoutit's cube preview and export helper disagree
about occlusion by non-cube neighbours (`cube.vue`, `visibleFaces.mjs`).
Mixed-shape boundaries become an explicit agreement fixture if such shapes
are admitted. Snapshot undo and one DOM element per voxel are implementation
choices in that reference, not requirements of the bench.

### Immediate probes, before embedding

**Execution order corrected by Mark, 2026-09-13.** The bench's integrated
done conditions depend on missing viewport, style, picking and body-pose
work. They are not gates for pricing the renderer. The existing L0c probe
can settle the two immediate questions without Genet or any new picking API.

1. **Continuous body transform:** add finite parent yaw to LiveBody above
   part attachments; keep simulation Yaw and immutable mesh identity. Rerun
   A and C on identical continuous poses and the CPU depth oracle, then
   measure the same population/camera/motion. B's snapped facings remain a
   separately labelled approximation. Done when attachment coordinates and
   pose-only GPU reuse pass, sampled pixel differences are reported, and the
   equivalent-pose timing/upload receipt replaces the earlier unequal-yaw
   claim. Production callers default to zero presentation yaw.
2. **Variety in the existing probe:** share sprites by content, dimensions,
   projection and facing, with placement kept separate. Control shape count
   independently from body count and report actual unique geometry and cache
   capacity. Done when duplicate bodies reuse images, real content/facing
   changes invalidate them, deterministic shape workloads agree across paths,
   a small varied scene is checked against the oracle, and equivalent
   population runs report timing, uploads and memory. This precedes T3.

**Receipt:** the probe, raw JSON and source identities are retained in
`mere/crates/probes/wing-three-paths/` at cc4c079c; the full local capture is
`Code/testing/wing/l0c_2026-09-13/README.md`, with its source snapshot beside
it. LiveBody now accepts finite continuous parent yaw;
17 renderer tests, 10 probe tests and the Mesocosm workspace all-features/
all-targets check pass. Fresh A/C yaw runs share the same
workload digest and report 135–148 / 2.14–2.43 ms median at 1,000 bodies,
with zero measured geometry uploads on C. The corrected sprite cache uses
12 images for 1,000 duplicate bodies. At 1,000 distinct torso shapes, C has
1,005 live mesh keys and costs about 8 ms; B's crossing cost is 10.53 ms,
with 276 MiB of images. The notch sampler measures geometry reuse, not
general generated anatomy. Sampled coloured residuals were traced to
coincident surfaces; isolated terrain pixels and timing tails remain explicit.

The probe code owners are mesocosm-render and the existing
mere/crates/probes/wing-three-paths probe. Bench A below supplies the bounds
and part-query consumer; CSS appearance and host producer lifecycle remain
Bench B's integration work.

### Bench A. Presentation pose and visible-part queries

**Status, 2026-09-13:** implemented in the existing Section API, following the
adapter/probe baseline at Isometry bf05d2b and Mere cc4c079c. Fresh CPU and GPU
pixel checks pass. Native pointer routing and Genet viewport input remain
Bench B's integration work.

**Owner:** Mesocosm rendering and host projection. The immediate probe supplied
validated parent yaw above resolved part attachments, leaving core Yaw,
BodyDocument and VolumeRef unchanged. Apply that pose to presentation culling/framing bounds;
the current core AABB alone cannot bound a rotated body. Add a CPU query
against actual part quads using the same matrices,
slab and bounds as drawing; compare the result with the nearest visible
terrain hit. GPU ID picking and independent continuous joint animation can
follow their own measured needs.

**Done when:** a nonzero-pivot body completes a continuous turn with intact
attachments and no new static geometry upload; independent 45/90-degree
coordinates agree; overlapping bodies, nearer terrain and cutaway clipping
yield the visible part; a long body rotating at a viewport/slab edge is not
incorrectly culled; severing invalidates stale selection. Preserve the
existing quarter-turn and shared-depth tests.

**Implementation findings, 2026-09-13:** draw, bounds and body queries now
share the renderer's part matrices. Bounds visit actual transformed quads:
the core AABB omits attachment yaw and noncentral pivots, so it is not a safe
presentation rejection test, even before adding parent yaw. A future cheap
bounds cache must remain conservative for those admitted shapes. This host
culling work has not been included in the standalone L0c cost numbers.

The Section query joins addressed body faces to the filtered terrain map
that it encoded. It refuses incomplete frames and capsule fallbacks, and
separates a frame-specific hit from a geometry-revision-scoped semantic
selection. Equal-distance body identities carry an explicit tie flag; the
stable CPU tie choice does not promise the GPU's colour owner on coincident
surfaces. Exact raster-boundary coverage is likewise not a CPU ray guarantee.

Review also exposed a standing-wall camera error: the slab constructor needs
world vertical, not screen-up. Pitched-view acceptance must test front and
far cuts as well as projected XY. Frame identity must survive Section
replacement without aliasing, and changed habitat bounds/chamber must retire
both the filtered terrain cache and the previous query receipt. These are
part of the integrated acceptance checks, not new simulation rules.
Terrain uploads also remain pending while an isolated preview omits terrain
or a frame fails. The next successful terrain encode receives the full pending
map; ordinary uninterrupted terrain edits retain their incremental uploads.

**Verification, 2026-09-13:** 32 mesocosm-render library tests, six lens terrain
ray tests, two lens camera tests, 33 Section tests and six grafting/preview
tests pass, as do all 10 standalone probe tests. The Section suite includes
fresh whole-frame selection-colour readbacks compared with every pixel query,
independent leading/top/trailing/outside rectangles, pitched front/far wall
pixels, the noncentral attachment, nearer terrain, cutaway changes made while
isolated, Section recreation and severing. Existing shared-depth, expression
material and family-scene tests remain green. This is headless rendering
acceptance; it does not claim a headed or document-embedded specimen bench.
Both the Mesocosm workspace and Isometry's root workspace pass
`cargo check --workspace --all-features --all-targets --offline --locked`.

The retained implementation is in `mesocosm-render/src/live_body/{pose,query}.rs`,
`mesocosm-lens/src/bricks/ray.rs`, and
`mesocosm-genet/src/section/{bodies,inspection,view}.rs`; all are under
`mesocosm/crates/`. Section's `set_body_yaw`, `presentation_bounds`,
`pick_pixel`/`pick_ndc` and `validate_pick` form the next viewport's consumer
seam. The preview uses these presented bounds to fit the body beside its panel.

### Bench B. One interactive Genet viewport

**Status, 2026-09-13:** bounded native integration accepted with the evidence
and limits below. The `--bench` entry uses Cambium's native host and the existing
creator's Request/Prepared worlds. Whole-game host migration remains separate.

**Owners:** Mere/Cambium producer lifecycle, Genet content-box and common 2D
paint/input geometry, Mesocosm scene producer and appearance mapping. Reuse
custom_leaf, DrawExternalTexture and stage_external_image. Initialize from
the host device/queue, stage changed generations before normal rasterization,
and retire removed producers. Declare colour encoding and alpha explicitly.

Content-box origin/size must account for padding and borders. Pointer-local
coordinates use the inverse accumulated paint transform, then the camera ray;
an axis-aligned bounding rectangle is insufficient. Expose the existing
resolved-style queries through a read-only host seam for a declared set of
appearance properties. Ordinary DOM specimen/part controls provide labels,
focus and actions for the same identities.

**Done when:** two occluding bodies and terrain render between DOM siblings;
border/padding, scrolling, clip edges, transform origin, 2D scale/rotation,
opacity, zoom and output scale agree in pixels and picks; an overlaid control
wins input; resize/hide/remove/recreate preserve lifecycle; unchanged sources
skip staging and decorative style changes leave geometry cached. Use the
accepted Ortet image tests and Cambium native smoke host as starting fixtures.

**Implementation:** Cambium's `ProducerRegistry` binds a producer to the
existing custom-leaf slot after layout. It supplies the host device/queue,
content extent and declared resolved appearance. Changed view identities or
generations are staged before ordinary document paint. Resize invalidates the
old image; hiding suspends; removal retires the registration. A recreated node
registers again. Encoded sRGB and source alpha are explicit contracts.

Genet's `ElementGeometry` supplies content origin, inverse accumulated 2D
transform and ancestor clips to the host input path. Ordinary DOM hit routing
decides the recipient first, so the clear-selection overlay wins. The slot
does not introduce a clip around legacy retained fragments: its image bounds
and authored CSS overflow supply clipping, preserving the T4-independent path.
Pixel padding is covered; the existing percentage-padding query approximation
is not promoted into a conformance claim by this work.

Mesocosm's producer reuses `Section`, including its body/terrain depth and
visible-part queries. CSS `color` converts from encoded sRGB into a linear
multiplier on the organism's ordinary presentation tint. The instance override
does not edit world or material facts. Group opacity stays in document paint.
Unchanged scene inputs skip producer rendering as well as image staging.
The parts examiner uses the existing `part_of` projection; buttons and viewport
hits identify the same organism/part. Preview replacement expires selection
even when a new candidate reuses local IDs.

`testing/bench/acceptance.scenario` drives native pointer input and captures
the presented document. Its decoded-PNG checks isolate viewport content from
labels, buttons and the overlay: tint must change body pixels with background
fixed, while decorative CSS must preserve content pixels and upload counters.
The separate habitat and deliberate-failure scenarios exercise the composed
habitat and the receipt's failure path. These are correctness checks, not the
population and frame-cost comparison assigned to Bench D.

**Native receipt, 2026-09-13:** the final development build passes acceptance
in 167 frames with 15 captures, at actual output scale 2. The initial seed-1
candidate has 33 parts and uploads 8,064 static mesh bytes. Each tint comparison
changes 102,660 body pixels while all 1,128,060 sampled background pixels remain
fixed. Decorative CSS leaves all 1,230,720 sampled content pixels unchanged;
opacity changes composition and resets to the same pixels. Neither operation
rerenders the producer or adds mesh/instance uploads. Transformed centre clicks,
UI zoom 1.25, overlay priority, resizing, removal/recreation and candidate
replacement pass through real host input. The independent geometry fixture
calculates the authored 7-degree/0.9 transform rather than querying Genet's
inverse back as its own oracle.

The habitat scenario passes in 25 frames with ten detailed bodies and 343
parts beside the traced terrain, with no capsule fallback or world-hash change.
This frame establishes native assembly; exact occlusion is carried by Bench A's
body/terrain pixel tests, and literal clipping/transform edges by Genet's and
Cambium's focused fixtures. Body count alone is not an occlusion oracle. The
deliberately false assertion produces process exit 1, `ok: false` and two fresh
PNGs. A separate Section GPU test verifies tint changes, reset pixels, invalid
input, unchanged geometry and preserved material facts.

Compact receipts and source-blob identities are retained under
`testing/bench/receipts/2026-09-13/`; full PNGs are under
`Code/testing/specimen-bench/native-final/`. The source record identifies the
actual executable, avoiding the older binary under `mesocosm/target`. Those
captures use development path overrides, with source blobs recorded before
commit. A separate full Mesocosm workspace check passes on Rust 1.97.1 with
the committed Git dependencies and no sibling overrides:

```text
cargo check --release --offline --workspace --all-features --all-targets -j 3
```

That graph uses Mere `4f4de1d05ec99461f7fa3cdc4e514e904a999213`,
Genet `101d9e9ade8671564e723443d9f0498e899a33f1` and netrender
`3961aca919f707ab09a786379eb4ce8bb121258e`, with one source identity per
family. Genet's 61 focused checks, Cambium's 34 CPU and two GPU producer tests,
62 native-host regression tests, and the Section tint GPU test pass. The
separate Isometry root workspace also passes its required locked/offline
all-features/all-targets check on its unchanged dependency baseline; it is
not a second host acceptance of the new pins. Remaining warnings are the
existing unused TraceCamera helper and core-test mutable binding.

### Generation comparison (2026-09-13)

**Priority:** Mark paused world-trial work to explore generation with the bench.
The 48-specimen audit is at `Code/testing/generation-audit-2026-09-13/audit.md`:
the random draw uses little of the existing shape and appendage-chain vocabulary.
The first experiment changes proportions through admitted mass-part shapes.

**Status:** implemented and native-verified. The bench retains the
original beside four explicit single-stretch alternatives. Recipe topology,
individual segment counts, appendage assignments and mass stay fixed. Shape
selection uses only distinct envelopes actually admitted by the current palette.
The ordinary generation admission and founding path checks each alternative;
missing choices and refusals remain visible. More proportions advances through
the finite available edits; it is not a claim of unlimited unique generation.

Core ownership is `generation/proportions.rs`. It reuses the existing recipe
admission and founding operations, leaving ordinary seeded requests unchanged.
The UI owns disposable comparison worlds, shared-scale or individually fitted
cards, an inspected selection, and generation-change readings for affected parts.
Each card uses the existing Cambium producer/Section path. Meshes are currently
cached per viewport; this slice does not claim cross-viewport mesh sharing or
integrated population-cost acceptance.

Save comparison writes a fresh local file containing versioned selection inputs,
the palette, admitted voxel content and the expected world hash. Reopening with
`--bench --comparison FILE` validates content and reconstructs that hash before
showing the comparison. It does not rewrite a played world or reinterpret a
shape selector using whichever palette happens to be current.

**Done when:** native captures show the original and four alternatives; choosing
one updates the same parts examiner and expires old selection; shared scale and
fit-each behave distinctly; generating more alternatives leaves the original
hash unchanged; exported inputs reopen the selected world exactly; corrupt
records refuse; and the existing generation/bench checks remain green.

The broader controls inventory, arbitrary topology edits, chain variation,
general per-part generation traces and surface-pattern variation remain later
experiments. This first slice reports the changed stretch and affected part IDs,
not a complete provenance map of every development operation.

**Receipt, 2026-09-13:** all 20 core generation tests pass, including two new
proportion tests that pin realized topology, mass, appendage instructions,
original preservation, palette exhaustion and serialized replay. The native
proportion scenario passes in 71 frames with six captures: four alternatives
are admitted and all five cards draw. The retained hash is `aa8ae203bde513d6`;
alternative 1 is `333f1885cc2ef33c`, including after export/reopen. Reopen passes
in eight frames with two captures; a falsified expected hash exits 1 before
host creation. Shared-scale versus fit-each changes sampled pixels in all four
alternative previews (4,361; 4,377; 6,171; 4,762 pixels), while the original's
sample remains identical. These fixed native image regions exclude card labels
and selection borders; they are a bounded fixture, not a general scaling oracle.

The previous 167-frame/15-capture bench acceptance also passes. Both Mesocosm's
release and Isometry's separate root workspace all-features/all-targets checks
pass. The native build uses the committed Git dependencies without sibling
overrides. Compact receipts and source identities are under
`testing/bench/receipts/2026-09-13/proportions/`; PNGs and saved comparison files
remain under `Code/testing/specimen-proportions-2026-09-13/`.

### Generation wiring and gap filling (2026-09-13)

**Status:** generation wiring native-verified; species likeness remains open. Mark clarified that a proportion comparison is one
control within the intended generator, not its completion. Parallel audits
identified existing stretch graphs, appendage chains, archetype recipes and
material accounts whose vocabulary the creator was not exposing.

Core owns a new `Generated` body plan: seeded bounded parent trees, segment
counts, socket directions, admitted shape choices and serial appendage chains.
Axial and Branched retain their existing streams. Eight starting anatomies
provide raccoon-like, cat-like, horse-like, bird-like, fish-like, grass, shrub
and tree recipes. Starting forms reject developmental organ omissions; their
feeding roles still come from ordinary realized anatomy. These references do
not confer flying or swimming capabilities on the existing world simulation.

The host exposes body plan, starting anatomy, feeding role, seed entry, random
seed, suggested reproducible seeds, reroll, mass and size. Generation remains
on the existing worker with stale-result rejection. Refused candidates remain
visible. Save specimen includes the exact content, request and expected hash
through the same saved-comparison format.

Size belongs to the admitted content pack. Sizes 2 and 3 enlarge eligible
structural shapes and update voxel bytes, content addresses and palette bounds
together. Sensory and other role-sensitive detail retain their admitted size;
every resize derives from the unchanged base pack. Mass remains independently
editable paid tissue, with its own reserve and ecological consequences. Neither
control is a CSS scaling effect. Saves retain both the base pack and selected
size, and validate that resizing reproduces the saved content on reopen.

**Done when:** all eight starts admit and render; generated topology differs
across seeds and replays; seed/reroll and editable mass work through native
controls; size changes authoritative bounds while preserving paid mass; mass
changes accounts while holding geometry; saved resized specimens reconstruct;
and previous generation and bench acceptance remain green.

**Findings:** core supports ordered acyclic attachment graphs, but does not
currently enforce whole-body self-intersection freedom. The generated grammar
uses cardinal stretches and finite admitted shapes. Surface markings and
finished visual recognition need their own native inspection, not inference
from archetype labels. Native captures show distinct but still blocky structural
prototypes; finished species likeness and surface markings remain open. World
trials remain paused.

**Receipt, 2026-09-13:** 27 core generation tests and 10 content tests pass.
The native generation scenario passes in 106 frames with 16 captures, covering
all eight starting forms, generated topology reroll and seed replay, invalid
input, size and paid mass. The cat fixture changes from bounds `[16,19,30]`
to `[32,32,60]` at Size 2 while retaining 800 mg. Capacity changes from 1,522
to 7,429 mg. Raising mass to 1,600 mg preserves the larger bounds and all
474,216 sampled viewport pixels. This verifies independent controls, not a
constant-density growth rule.

Saved Size 2 reopens in 19 frames with three captures and the identical
`4e8f6296be013bcd` world hash. Restoring Size 1 recovers the base geometry.
Falsified size metadata and expected hash both refuse before host creation;
a zero-candidate request refuses saving without crashing. The existing native
bench scenario (167 frames/15 captures) and proportion scenario (71/6) pass.
The release all-features/all-targets workspace check passes on committed Git
dependencies. An unrelated task held the shared Cargo cache lock, so the final
frozen build used a separate offline lock directory over the already-present
sources, with cache cleanup disabled. Source identities, native receipts and
the one post-build formatting-only change are recorded under
`testing/bench/receipts/2026-09-13/generation/`; images remain under
`Code/testing/generation-wiring-2026-09-13/`.

### Composable anatomy (2026-09-13)

**Status:** native-verified. The named animals and plants are reference body plans,
not a closed roster. Core adds independently selected stretch arrangements
and appendage patterns to the existing generated recipe path. The host exposes
the choices, counts and seeded length bounds through the specimen bench.
Requests without structure settings retain their existing draw streams.

**Done when:** arrangements and organ patterns produce measurably distinct
developed anatomy; feasible combinations admit through ordinary founding;
seed replay, paid mass and saved restoration hold; native controls and captures
exercise animal and plant combinations; prior generation acceptance passes.
The first vocabulary is chain, cardinal radial branches, crown, mat, bent vine
and downward roots, combined with bare sites, legs, wings, fins, leaves or
feelers. These describe anatomy. Flight, swimming, climbing and root physiology
remain application capabilities. Shells, rounded or tapered part shapes and
surface markings remain further vocabulary work.

**Receipt:** 32 core generation tests pass, including all 96 compatible
layout/organ/feeding-role combinations on the bench's SpacedRoster palette
at seed 1 and six structural stretches. Tests cover paid founding, JSON replay,
sparse palettes and bounds. Review caught coincident fin pairs and converging
mat/root branches; their resolved-position fixtures now pass. These fixtures
do not establish general self-intersection freedom. Feeding supports remain
separate from structural stretch counts, and radial branches use a hub separate
from the feeding root so ordinary ecology does not mistake a downward mass
for a mouth.

Native composition acceptance passes in 75 frames with 11 captures. Saved
roots/leaves at length 2 reopens and regenerates the same `9c2153f6c1c90c07`
world hash. Command-line generation and native loading use the same rich
palette for explicit structure; a radial-feeler request matches the independently
observed native-control hash `50e8f3a3a6b6d22d`. Earlier command-line requests
retain their primitive palette for replay compatibility. All 14 deterministic
checkpoints in the previous generation receipt match, and the old resized save
still reopens. The ordinary native bench acceptance and release workspace
all-features/all-targets check pass. One host content-selection/roundtrip test
also passes.

Source identities and compact receipts are under
`testing/bench/receipts/2026-09-13/structure/`; captures remain under
`Code/testing/structure-generation-2026-09-13/`. The capture review confirms
distinct but blocky structures. This slice broadens combinations of existing
parts; it does not yet expand the finite voxel shape bank or qualify population
rendering cost. Shells, tapering, rounded forms and coherent surface patterns
are the next visual vocabulary candidates.

### Glyph effects and interaction experiment (2026-09-13)

**Status:** first planar experiment native-verified. Mark's working model mixes
a glyph-effect lexicon with spatial, rule-based behavior and stable generated
material/surface/item interactions. Strings form only when the selected glyph
and behavior support a relationship; individual marks remain valid outcomes.
Clouds are one arrangement among streams, boundaries, branches and inscriptions.

Three independent inputs are glyph vocabulary, spatial behavior and receiving
interaction profile. Generated choices are seeded and persisted so discoveries
can be repeated. A ruleset may guarantee an effect's existence, conditions,
outcome and interactions, reserving explicit slots for generated variation.
Appearance changes must not silently alter those guarantees. D&D 5e and PF2e
are motivating consumers, not implemented rule packs for this experiment.

Keep game adjudication with each product. Existing wing-functions supplies
part-bound functional blueprints and typed proposals; it does not become a
universal collision, magic or ecology evaluator. Visual and audio consumers
read accepted activity. Glyphs, meshes, strokes and surface patches can share
presentation recipes without becoming biological traits or gameplay authority.

**First slice:** an explicitly separate bench interaction experiment. Hold the
effect and seed fixed while changing a receiver, then hold the receiver while
changing spatial behavior. Show the resulting marks and relationships beside
a readable explanation. Support deterministic sampling, replay and bounded
counts. An authored guarantee must survive reseeding; allowed generated
interactions may vary. The first viewport may be planar and must say so;
shared-depth body attachment and world collision require their own rendering
and activity receipts.

**Done when:** native controls visibly distinguish stream, enclosure and
surface attachment; compatible marks form inspectable strings while incompatible
marks remain separate; receiver changes produce repeatable explained outcomes;
saved experiment inputs reconstruct; fixed guarantees resist reseeding; and
existing anatomy and bench acceptance remain green. Document presentation-only
inputs and all unimplemented gameplay semantics explicitly.

**Receipt, 2026-09-13:** `mesocosm-core::effect_experiment` owns a versioned,
bounded request, stable material-response selection and stateless integer
samples. The host draws procedural glyph strokes through the existing Sprigging
leaf path. No dependencies or platform pins changed. The demonstration has
quotes/slashes/backticks, stream/enclose/inscribe, and stone/metal/moss samples.
Guaranteed responses are authored demonstration laws, not imported game rules.
Generated lookup depends on the interaction seed and receiver, never the
appearance seed. Groups require glyph, behavior and response compatibility.

Seven focused core tests pass. Native acceptance passes in 125 frames with ten
captures: animation changes pixels, guaranteed interaction reseeding preserves
pixels, appearance reseeding changes marks while retaining the response, and
split/string relationships differ. Save/reopen and closing/reopening the view
reproduce all 786,384 sampled viewport pixels. Restart restoration, fixed-tick
playback and pause pass separately in 166 frames with four captures. Invalid
version, count and saved tick each refuse before host creation. The specimen
world hash stays unchanged through the experiment. Existing bench acceptance
(167 frames) and anatomy composition acceptance (75 frames), plus the release
workspace all-features/all-targets check, pass.

Native review changed the initial stream from simultaneous clumped marks to
staggered emissions with individual scripted arrival times. Saved requests and
ticks reopen with `--effect-experiment FILE`. Compact receipts and source
identities live at `testing/bench/receipts/2026-09-13/effects/`; images remain at
`Code/testing/effects-2026-09-13/final/`.

**Remaining scope:** the receiver is a material sample on a demonstration plane.
Contact age is scripted, not a collision result. Surface and item identities,
organ emission sites, scene interaction, CSS effect styling, sound, mixed fixed and
generated rule packs, and effect-to-effect composition remain subsequent work.
The first comparison proves controllable expression and reproducible responses,
not gameplay adjudication, camouflage or a populated-scene performance budget.

### Bench C. Visible disposable world trial

**Owner:** Mesocosm host/runtime. Reuse Prepared::enter and the existing runtime
driver, adding a narrow prepared-world trial boundary where necessary. Show
play/pause/reset/step over the disposable world, preserve the entry world and
parked game, and expose bounded accepted activity once per tick. Honor game
checkpoints as visible pauses. Scope selection and later sound emitters by
preview epoch, organism and part plus the geometry dependency revision;
allocation revision alone does not track attachment/severing.

**Done when:** the saved request and a fixed intent/tick sequence reproduce
initial/final hashes and activity; the scene visibly reflects actual movement
or feeding; redraw does not repeat an event; reroll/reset rejects stale worker,
selection and activity state. Runtime/history/flow remain the source of facts.

**Bounded idle trial implementation, 2026-09-13:** `mesocosm-runtime::Trial`
owns an exact clone of the displayed fresh specimen world, including proportion
and size choices. It reuses ordinary Runtime stepping and checkpoint handling;
it admits only fresh tick-zero, epoch-zero worlds with a living controlled body.
A later snapshot needs its runtime history/checkpoint and is refused here.
The trial applies at most 128 Idle intents. Its immutable baseline plus that
trace, rather than seed/count metadata alone, is the replay source.

The native bench exposes World trial, Step world (one tick), Play/Pause,
Reset and Exit. Trial rendering uses the habitat and accumulated terrain dirty
regions. Each successful step consumes a new accepted-history batch once;
redraw is read-only. Movement slashes use recorded departure/arrival positions;
feeding quotes use donor pre-step and eater post-step locations, including a
donor that disappears. These are organism-level activity indicators, not
physical mouth/foot contacts. Marks live for eight simulation ticks and are
bounded to 128; clipping and occlusion may hide them. Marker height (0/2/4/6/8,
default 4 world units) and size (0.7/1.4/2.8, default 1.4) are independent
presentation controls. The recorded interaction coordinates stay unchanged.

Start/reset/exit replace presentation identity; successful ticks expire part
selection; generation changes discard the trial. Saving a specimen during a
trial still refers to source generation, and exiting restores that source.
Playback pauses when hidden or when the planar experiment opens. The existing
spatial preview remains a separate presentation experiment.

The resource audit also corrected `TrialEvidence.subject_predation_mg`: it
must credit `Event::Fed.eater`, not the victim in `from`. A paired-subject test
checks both directions and excludes non-predatory feeding. Previous receipts
of that field reflect the earlier implementation; they are not reinterpreted
as verified incoming predation evidence.

**Native acceptance:** the seed-7 consumer's 48 Idle ticks reproduce 12 accepted
movements and 50 feeding events. Reset and a separate process reopening the
saved specimen reproduce the hashes and activity. Restyling changes viewport
pixels while preserving world state and activity; restoring styling restores
the viewport. Playback, pause, hidden/planar transitions and reroll cancellation
are exercised. See [receipts](../testing/bench/receipts/2026-09-13/world-trial/source.json).
This closes the bounded Idle trial, not broader controlled intents, contact
localization, material reactions, transparency, sound or integrated cost.
Checkpoint stopping has a focused injected-state test; a natural checkpoint
and donor removal are not separate native acceptance cases.

### Bench D. Variety and cost receipt

**Owner:** integrated bench host. Reuse the probe-only variety comparison above
and price the added document/style/input boundary. Control instance count independently from
distinct mesh count. Compare shared geometry with varied appearance, different
assemblies of reusable parts, and independently generated geometry. The current
content pack has at most 16 volumes. The follow-up probe has configurable
mesh capacity, measures up to 1,005 live keys and refuses an undersized cache
before GPU creation. Establish application working limits from actual bytes
and measured costs, with visible refusal rather than silently missing bodies.

**Done when:** equivalent camera, poses and pixel scale accompany correctness,
first-frame cost, median/p95/max work, mesh/instance uploads, texture/staging
cost and cache activity. Carry the equivalent-pose yaw baseline and document
the renderer's policy for coincident surfaces; count foreground coverage separately from background.
Record the source revisions and benchmark population, including unique geometry.

**Integrated population workload, 2026-09-13:** `--population FILE` is a
presentation-only bench input over the existing LiveBodyRenderer and the same
Cambium producer/Genet document boundary. It does not invent ecological
organisms or alter a saved specimen. Three versioned fixture families separate
instance colour, assembly placement, and seeded exterior occupancy notches.
Body count (1..1000), design count (1..body count), seed and mesh capacity
(1..4096) are explicit JSON inputs. Insufficient live-key capacity is refused
before GPU creation. Generated occupancy is not general anatomy generation.

The fixed orthographic camera and spiral grid preserve pixel scale and
population prefixes. Actual unique meshes, assemblies, part instances, quads,
CPU preparation and retained mesh counts are reported separately from GPU
cache builds/uploads/evictions. The population surface uses the existing body
renderer directly; ecological projection, terrain and active effects are
outside this workload. Texture bytes cover its RGBA8 colour and Depth32
attachments, not total process/GPU memory.

The pinned Cambium host already exposes CPU stage profiles. Named native
cost phases retain first-frame and median/p95/max values, separate producer
calls from actual redraws, and exclude capture profiles. GPU execution
duration and successful presentation of every callback are not inferred.
Independent ray/AABB tests derive silhouettes and nearest depth from explicit
voxel occupancy for all three families at two poses, including overlap. Their
magnified framing is separate from the native fixed-camera coverage receipt.

**Receipt:** [21 native configurations](../testing/bench/receipts/2026-09-13/population/matrix.json)
cover 1/16/128/1000 bodies and independent design counts through 1000.
All 4,746 frames' scenario runs pass; admitted runs submit all requested
parts without clipping, cache eviction or warm mesh uploads. At 1000 bodies,
shared appearance and assembly use three GPU mesh keys; 1000 occupancy
designs use 1002. Each redraw uploads 336,000 instance bytes. The two scene
attachments occupy 7,475,328 bytes at the measured viewport size.

Observed median producer CPU time during yaw was 1.02 ms for 1000 appearance
variants, 1.24 ms for 1000 assemblies, and 7.12 ms for 1000 distinct notched
torsos. Resolved CSS changes incurred roughly 2.1 ms of relayout in these
runs. GPU duration and sustained display throughput remain unmeasured. Process snapshots retain concurrent build
load; they do not establish application working limits. A later three-case
recheck measured 0.58/0.80/5.35 ms respectively. Both passes are retained,
including the receipt-only readiness correction between them.

Only 4.55-5.01% of the native viewport foreground is occupied at 1000 bodies.
This receipt prices sparse fixed-scale submissions and the document boundary.
That sparse receipt did not settle dense coverage/overdraw, real generated
anatomy with ecological projection, terrain/effects together, full memory
residency or application working limits. Initial CPU preparation also repeats shared part meshing
across assembly designs; it is reported honestly rather than treated as a
shared preparation cache. Existing yaw/coplanar policy receipts remain in force.

**Dense and layered receipt, 2026-09-13:** population controls now expose
camera half-height (8..2000), spacing (1..128), depth layers (1..32) and input
reversal. Defaults preserve the earlier layout; the v2 digest records these
settings. Layer groups occupy distinct positions 40 units apart along the
camera axis. Tight lateral packing is a renderer stress fixture, not admitted
ecological placement. Volume-key grouping still determines GPU batching.

[Twenty-three native cases](../testing/bench/receipts/2026-09-13/density/matrix.json)
pass the existing style/yaw/reset and warm-cache checks. Unclipped dense
layouts cover 33-35% of the viewport; tightly cropped layouts cover 93-95%.
The crop submits 960 bodies and reports 350-371 conservative bounds crossing
clip planes, including 154-165 wholly outside a plane. This is not a claim
that all submitted bodies are visible. One/four/eight-layer controls keep
the same projected group count and camera. Native reversal comparisons
change zero viewport pixels in all four tested pairs.

The independent oracle now tests four and eight separated layers. At its
magnified framing, 265 covered pixel rays intersect 1060 then 2120 bodies;
the silhouette stays unchanged. Stable interior depth and clear background
checks retain their tolerances. Both manual and configured input reversal
preserve exact RGBA/depth. These counts describe geometric overlap, not shader
invocations or early-depth rejection.

The native device confirms timestamp queries disabled. A separate ignored
diagnostic uses a private baseline-feature device, drains old queue work
outside each sample, and records CPU render/submit, completion wait and
serialized total with a ten-second bound. Its 23 cases and 24 steady samples
per case pass at the native attachment dimensions, 1236x756. It identifies
an NVIDIA RTX 4060 Laptop GPU using Vulkan; native adapter identity is not
exposed by the current host hook, so their timings are not subtracted.

Private-device median serialized totals were 0.56 ms wide / 0.65 ms cropped
for shared appearance, 0.57 / 0.68 ms for assemblies, and 1.15 / 1.27 ms for
122 geometry keys. The 962-key eight-layer case took 5.68 ms forward and
4.85 ms reversed. These are one bounded run's CPU-plus-completion times,
not GPU-only durations or application frame budgets. Native profiles and
available concurrent-build snapshots are retained separately.

This closes the bounded dense/layered fixture receipt. Real generated anatomy
through ecological projection, terrain and causal effects together, transparent
compositing, full memory accounting and application working limits remain open.
The raw capture/profile paths, source identities and repeat commands are in
[the source receipt](../testing/bench/receipts/2026-09-13/density/source.json).

**Integrated generated-world receipt (2026-09-13).** The existing Section path
now has a [three-source native matrix](../testing/bench/receipts/2026-09-13/integrated-world/matrix.json):
axial consumer, rooted producer, and size-2 cat-like consumer, reopened from
retained requests, palettes and volume content. Each passes 391 frames, eight
ordinary Idle steps, activity hiding/restoration, hidden-step reset/replay,
and autoplay/pause. Showing activity again restores identical captured viewport
pixels and identical history, event totals and world hash. The trial's new
Show/Hide activity setting affects presentation only and survives reset.

At tick eight the three worlds submit respectively 13/15/16 voxel bodies,
454/492/537 parts, and 27/39/35 activity glyphs. All report zero fallback,
budget omission, missing volume and projection failure. These are submitted
counts after Section filtering, not a visible-pixel census. The rooted world
has movement and feeding by its other organisms; uptake itself has no glyph.
Read-only Section probe fields now expose that distinction, complete body
stats, world organism count and ground revision. Terrain counters are explicitly
unavailable.

Forty alternating material changes force main-surface redraws at the same
paused poses, both with marks and without them. CPU producer medians span
1.05-1.86 ms in this run; paused frames cause zero actual scene redraws and
zero mesh/instance uploads. Warm phases upload no body mesh geometry. The
shown-first ordering and timing variation do not establish an incremental
glyph cost. Five comparison surfaces remain in the document: material changes
redraw the main surface, while visibility and simulation revisions redraw all
six. Aggregate profiles therefore remain document-level receipts. Manual-step
phases exclude some simulation CPU in the after-frame scenario callback;
a separate autoplay phase includes ordinary frame-hook stepping and records
its actual tick count. Neither is a GPU duration or an application budget.

The 39 Section tests, including pairwise body/terrain and body/glyph depth,
and the 190-frame trial lifecycle scenario pass. A three-way generated depth
oracle remains open. Default window captures clip the lower viewport, so
pixel checks cover its captured portion. Complete terrain accounting, larger
ecological populations, transparency, uptake/contact effects and full memory
remain next gates. Exact sources, executable identity, repeat command and
measurement limits are in the [source receipt](../testing/bench/receipts/2026-09-13/integrated-world/source.json).

The two immediate probes, Bench A and Bench B's bounded native integration
are complete. Generation comparison and the opaque spatial matrix have receipts. Bench C's
bounded idle trial has native reset/reopen and lifecycle receipts. Bench D has a bounded sparse-population document/style/input receipt;
dense/layered fixtures and three bounded generated worlds now have receipts;
scaled ecological scenes and application working limits remain open. T1 general
CSS 3D, T2's large-element sweep, T4 planar
retention, the L2 crate merger, portable body v1 and a general audio framework
do not block these slices.

**Soundscape and effects:** use Bench C's accepted movement, feeding, carving
and life-cycle activity plus habitat state. Tapping is not the causal model.
Per-foot contacts, impact forces and water interaction need their own evidenced
inputs before the effects claim them. Active Hocket lives under
woodshed/ports/hocket: Firewheel/device/sample work is useful prior art, while
its track/layer looper engine is not the bench's event model. A later local
audio consumer owns sound mappings, activity aggregation, audio-clock scheduling
and configurable voice/detail budgets. It shares facts with visual effects.

**Progress, 2026-09-13:** resource audit and owner-specific done conditions
recorded; stale L3/L5 and T3 embedding descriptions reconciled. Mark then
separated the cheap adapter/yaw and variety probes from bench integration.
Those probe slices now have source, tests, fresh performance/correctness JSON
and a reproducible receipt at `Code/testing/wing/l0c_2026-09-13/`, with the probe
and compact receipts retained at Mere cc4c079c. Bench A adds posed bounds,
preview framing and visible-part queries to the existing Section; its fresh
headless readbacks pass. Bench B adds the native viewport, style bridge and
part examiner, with fresh presented-document captures and failing controls.
The bounded idle world trial is implemented; broader world-intent trials and
the integrated population/cost receipt remain open.


### Spatial glyph preview (2026-09-13)

The bench now places opaque punctuation strokes in world space around the
posed specimen. Section draws one bounded glyph batch against its existing
body and terrain depth attachment, including isolated previews. Positions are
3D; glyph faces follow the orthographic camera. This is the first 2.5D view of
the spatial effect vocabulary. It does not yet lift the planar experiment's
receiver rules into world collisions.

**Spatial glyphs** enables an eighteen-mark orbit. Step, reset, play/pause,
glyph selection and oblique/across camera controls are presentation settings.
Hiding the specimen or opening the planar experiment pauses the orbit.
The body world and gameplay clock remain unchanged. Comparison cards retain
ordinary bodies and follow camera changes consistently.

The renderer admits at most 128 finite, opaque glyphs, reuses its GPU buffers,
and initializes only when first used. It shares the exact section matrix,
slab and bounds; no DOM element or depth key is needed per mark. No population
or frame-cost claim is made by this preview. Glyphs are decorative and do not
participate in the body's picking hook.

Done conditions: a real voxel-body GPU fixture verifies front and rear strokes
from Side and Oblique cameras over all covered pixels, including exposed
edges; native controls change orbit pixels; reset reproduces exact pixels;
camera changes preserve world identity; existing bench and planar scenarios
still pass. Native scenario: `testing/bench/spatial.scenario`. Receipts:
`testing/bench/receipts/2026-09-13/spatial/`.

### Spatial coverage extension (2026-09-13)

The admitted opaque spatial vocabulary now includes four forms. These are
presentation experiments; they do not yet lift the planar receiver rules into
world adjudication or claim that every glyph combination is a causal string.

| Form | Placement | Behavior | Attachment |
| --- | --- | --- | --- |
| Orbit | Camera-facing strokes in a 3D loop | Repeating world-space orbit | Posed body bounds |
| Surface | World-plane strokes | Stable seeded inscription | Largest actual meshed face per admitted part |
| Tether | Camera-facing strokes along a bowed path | Repeating orientation | Two distinct part anchors required |
| Emission | Camera-facing strokes moving outward | Staggered travel and shrinking lifetime | Actual face normal, optionally selected part |

`Section::glyph_anchors` returns at most 32 anchors in stable PartId order.
It uses the renderer's own `posed_quad` transform, including part placement,
continuous body yaw, origin and scale. A selected part must match the current
body projection revision. Meshed faces may be hidden by another part or ground;
shared depth determines visibility. An anchor does not assert that a face is
exposed to an ecological process. Surface strokes get a small world-space
normal offset to avoid coplanar depth ties.

Appearance seed and count are configurable. Counts cycle through 6, 18, 64 and
128 in the bench; saved requests admit 2 through 128. Sampling is stateless in
tick order. Framing uses a fixed form envelope so travel does not pump the zoom.
Save/Reopen spatial stores the form, glyph, appearance seed, count and tick;
replay requires the same specimen world hash, pose, camera, body/habitat mode
and complete part identity (organism, part and mesh revision).
It is a settings receipt for the current bench, not a standalone world save.

Coverage targets:

- All four forms, three glyphs and two native camera views (24 combinations).
- Counts 2/18/128, arbitrary sampling order, seed changes and replay in pure tests.
- All three glyphs with camera-facing and world-plane bases, body occlusion,
  exposed stroke edges, world bounds and slab clipping in GPU tests.
- Invalid finite/size/colour/orientation inputs preserve the prior renderer list;
  128 entries admitted, 129 refused, empty input clears.
- Actual sparse-mesh faces, exact continuous pose, part revision expiry and
  bounded anchors checked independently of the presentation sampler.
- Native seed/count changes, exact saved replay pixels, selected-part surface
  and emission, tether refusal for one anchor, and unchanged world identity.

Native receipt: the 24-combination matrix plus density, selected attachments,
replay and isolation-refusal checks passed in 336 frames (32 captures). Original
orbit controls passed in 207 frames, bench regression in 167, and planar effects
in 125. Native images are under `Code/testing/spatial-coverage-2026-09-13/`;
compact receipts are under `testing/bench/receipts/2026-09-13/spatial-coverage/`.

The complete effects system remains open: world collision and material/item
responses, gameplay rule packs and guarantees, effect composition, transparent
haze and overlap policy, surface editing/picking, soundscape event consumption,
perspective cameras, richer glyph assets and population cost measurement.
Those need their own owner-specific acceptance, rather than being implied by
this opaque spatial coverage matrix.

## CSS features and standards to earmark

Fast-track candidates for genet's
[standards-to-features ledger](../../../genet/design_docs/2026-09-07_standards_to_features_ledger.md),
which is the authority on where Livery stands. The status column here is a
file-mention count over the livery crates on 2026-09-11, a hint and not a
conformance claim; the ledger's census numbers win.

| Tier | Feature | Standard | Livery mentions | Unlocks here |
| --- | --- | --- | --- | --- |
| 0 | 3D transforms, `transform-style`, `backface-visibility`, individual transform properties | CSS Transforms 2 | 0 | L1 and optional DOM 3D presentation; independent of the shared-depth viewport |
| 0 | `image-rendering: pixelated` | CSS Images 3 | 1 | Isometry's nearest-neighbour lens for sprites and tiles |
| 0 | custom properties, `@property` | CSS Variables 1, Properties and Values API | 11, 1 | palette swap by variable; animatable typed yaw and tint |
| 0 | `color-mix()`, relative colour, `oklch` | CSS Color 4 and 5 | 10, 8 | per-face shading from one material colour, replacing `face_shade` in code |
| 1 | animations and transitions, `animation-composition` | CSS Animations 1, Transitions 1, Animations 2 | 2, 5 | part motion between poses; facing swaps |
| 1 | `clip-path`, `mask-image`, `mask-composite` | CSS Masking 1 | 9, 0 | cutaway of underground layers; damage masks on destructible parts |
| 1 | `content-visibility`, `contain` | CSS Containment 2 | 0 | off-screen layer culling for large maps |
| 1 | `will-change` | CSS Will Change 1 | 0 | the promotion hint for retained fragments |
| 1 | `mix-blend-mode`, `isolation`, `filter`, `backdrop-filter` | Compositing 1, Filter Effects 1 | 1, present | lighting tint, section haze; netrender already rasterizes filters |
| 2 | `offset-path`, `offset-distance` | Motion Path 1 | 0 | bodies moving along paths without script |
| 2 | anchor positioning | CSS Anchor Positioning 1 | 0 | labels and callouts pinned to bodies; Isometry's atlas labels |
| 2 | declarative Shadow DOM, custom elements | HTML, DOM | 6 | L8: a body as a self-contained element |
| 2 | `@scope`, nesting | CSS Cascading 6, Nesting 1 | 0 | per-body style scoping without class prefixes |
| 2 | View Transitions | CSS View Transitions 1 | 0 | epoch board and section transitions |
| 3 | `corner-shape`, `border-shape` | CSS Borders 4 | 0 | non-box shapes only; PolyCSS's triangle path; not needed for voxels |
| 3 | accessibility tree, ARIA | ledger row 3 | ledger | a screen-readable tabletop; the inspector as a test oracle |

Each row that opens a lane gets a ledger row in genet naming this doc as the
consumer. That edit is genet's, in the same session the lane opens.

## Prior art, what still transfers

| Source | Transfers | Does not |
| --- | --- | --- |
| PolyCSS | face culling, greedy rectangle merge, painter sort, baked per-normal shading, atlas slicing, snapshot as standalone HTML | `corner-shape` triangles; a JS dependency |
| Zdog | flat-shaded vector pseudo-3D under affine transforms, exactly the Vello lane | canvas/SVG runtime |
| Bonsai | relief lab loop, decoration tier, SDF layer brush editing, profiler-first culture | meshing, deferred shading, GPU-authoritative generation |
| Renderling | headless image-test pattern, slab allocation ideas for L6 | the runtime and its toolchain |
| GBA tactics games | baked sprites at a locked angle carry many animated bodies | free camera |
| Dwarf Fortress | z-level cutaway as the underground presentation | ASCII |
| Godot Voxel Tools, `block-mesh` | task pools, greedy meshing benchmarks for L2 | engine ownership |

## Novel angles, adopted

1. **GPU sprite bake on the resident lane** (L6). The resident chunk already
   feeds the tracer by GPU copy; the same allocation feeds a bake kernel whose
   output is a sprite atlas, not a mesh.
2. **Ground as generated tile layers** (L4). The version that reaches the
   browser with no GPU features at all.
3. **The hagiograph as a consumer** (L8). `CLAUDE.md` already assigns memorial
   events to the stack's procedural voxel engine; if that engine is genet plus
   the appearance crate, a memorial is a generated, styled, shareable document.
4. **Games as documents.** Save state stays in the core hash. Presentation is
   a DOM that cambium, accessibility, P2P peers, and the inspector already
   understand.

## Deliberate exclusions

- Perspective cameras in any vessel. Vello is affine; the tracer remains the
  only perspective-capable lens and is not load-bearing.
- A per-pixel depth buffer in the DOM. Interleave replaces it.
- Ground as DOM voxels. Ground is tiles per height step or the tracer.
- Per-face animation. Parts are rigid; animation is per part.
- A PolyCSS or Zdog dependency. Their cores are small enough to own.
- Any lane past L0 before the L0 receipt.
- Amending the execution graph plan from this doc.
- Editing genet's ledger or Livery plans from this doc.

## Open decisions for Mark

- Hybrid switch policy: which bodies count as "in focus" for live
  fragment-backed parts while genet's T2 is open.
- Whether the tracer keeps the Mesocosm section by default or the section
  also moves to tile layers once L4 lands.
- Whether L6's GPU bake targets the enhanced capability profile only, with
  CPU bake as the downlevel tier, or replaces the CPU bake outright.
- Where the merged appearance crate lives: the Isometry root workspace, or
  mere as a platform organ once a non-wing consumer appears.
- Poses and clips (raised 2026-09-12): whether a pose (a set of part
  transforms) and a clip (keyframes over poses) become body-document data
  played as CSS animations on the individual transform properties, with
  player-set facing slots as stored poses, and which wing plan owns which
  poses exist and when they fire. Presentation here; behaviour elsewhere.

## Receipts

Filled by lanes as they close. L0 first.

### L0a, netrender face throughput (2026-09-11)

Receipt: `Code/testing/wing/l0a_netrender_face_ceiling.{json,md}`; probe at
`mere/crates/probes/netrender-face-ceiling/` (untracked). RTX 4060 Laptop GPU,
Vulkan, driver 610.88, netrender `3961aca91`, 1920x1080 RGBA8, medians over
60 frames after 5 warmup. Three paths were measured: **static** (the same
flat-rectangle scene every frame through the tile cache), **live** (every
body's transform changes every frame, scene rebuilt, tile cache), and
**fragment** (each body a retained fragment placed through
`Scene::place_fragment` with a fresh 4x4 each frame).

| path | rectangles | frame ms |
| --- | ---: | ---: |
| static | 50,000 | 11.3 |
| static | 150,000 | 27.3 |
| live, flat | 5,000 | 6.5 |
| live, flat | 20,000 | 18.4 |
| live, flat | 50,000 | 50.9 |
| live, fragment | 50,000 | 6.8 |
| live, fragment | 200,000 | 10.0 |
| live, fragment | 600,000 | 36.2 |

Findings:

- **Retained fragments are the live path.** Under a 16 ms frame the flat live
  path holds roughly ten to fifteen thousand rectangles; the fragment path
  holds two hundred thousand measured and an interpolated ceiling near three
  hundred thousand. Fragment lowering stayed at one lower per body across
  every transform-changing frame, so a moving body costs a placement, not a
  re-encode. This is a twenty-fold lift and it decides L5: live faces are
  viable at Paredros scale provided every body is a fragment.
- **CPU path overhead dominates, not the GPU.** On the flat live path at the
  largest cell, dirty-tile rebuild is three quarters of netrender's total
  while Vello's render is a small fraction. The tile cache re-lowers every
  tile a moving body touches, which is why the flat path collapses.
- **The fragment path bypasses the tile cache**, so its two halves are master
  compose and Vello render, comparable at the largest cell. A face-batch
  scene op (L3) would attack the compose half; below two hundred thousand
  rectangles neither half is the constraint, so L3's batch op is not yet
  earned.
- **No API refusal.** Fragments already carry a per-instance transform.
  Qualified 2026-09-12: planar, and retained only at layer depth zero; see
  L0c above and genet T4.
- **Probe-workspace blocker, not fixed here:** `mere/.cargo/config.toml`
  patches the genet git source's `servo-paint` to a path that no longer
  exists, so any cargo run with a cwd inside mere fails at resolution. The
  probe was built with `--manifest-path` from outside mere. This will bite
  every probe under `mere/crates/probes` until the stale patch is removed.

L0a's half of the L0 done condition: static ceiling fifty thousand
rectangles through the tile path; live ceiling two hundred thousand through
fragments. The genet half (L0b) decides whether the DOM can supply that
many elements.

### L0b, genet element cost (2026-09-12)

Receipt: `Code/testing/wing/l0b_genet_element_ceiling.{json,md}`; fixtures,
generator, runner, and attribution variants under `Code/testing/wing/l0b/`.
Ryzen 9 7940HS, RTX 4060 Laptop, genet `546201874df` through the Ortet
release host, netrender path-patched at `3961aca91`. Elements are absolutely
positioned faces under 2D `matrix()` transforms (Livery has no 3D transforms
yet), grouped body > faces. Ortet exposes no per-frame timing and presents
with Fifo on a visible window, so steady cost is the slope between a long
and a one-frame run and cannot resolve below the refresh interval.

| elements | first frame over empty baseline | steady per frame |
| ---: | ---: | ---: |
| 1,000 | 0.2 s | 6 to 7 ms (refresh floor) |
| 5,000 | 4.6 to 4.9 s | 8 to 14 ms |
| 20,000 | 60 to 132 s | unresolvable |
| 50,000 | 15 to 17 min | unresolvable |

| variant, 5,000 elements | first frame over baseline |
| --- | ---: |
| faces hidden with `display:none` | 4.0 s |
| faces with transforms (the fixture) | 4.9 s |
| faces with no transform | 10.1 s |

Findings:

- **Steady state is not the problem.** Through five thousand elements the
  redraw sits at the display floor. At twenty and fifty thousand the
  run-to-run variance of a single document exceeds the per-frame slope, so
  steady state is unmeasurable by this method and a host-side harness with
  real timing is needed to say more.
- **The first frame is the wall, and it is superlinear.** Ten times the
  elements from five to fifty thousand costs roughly two hundred times the
  time. This is a genet defect in parse, style, or first layout, not a
  property of the DOM approach; a stock browser lays out fifty thousand
  positioned boxes in well under a second.
- **The quadratic term is absolute positioning, not parse and style.**
  Corrected 2026-09-12 by the one-factor attribution at
  `Code/testing/wing/l0b_first_frame_attribution.md`. Parse, style, and box
  construction are linear at about 0.13 ms per element (the `display:none`
  variant grows with exponent 1.05). Only `position:absolute` changes the
  exponent, to about 2.2: genet's positioned-box pass walks the whole
  fragment tree two to four times per positioned box
  (`apply_absolute_and_fixed_positioning` in genet-livery's
  `layout/positioned.rs` calling `FragmentTree::resize_leaf`,
  `translate_subtree`, and `recompute_overflow` in buckram's
  `fragment_tree.rs`). The untransformed variant was slower because
  `left`/`top` offsets make `translate_subtree` take its whole-document
  scan instead of its zero-offset early return, not because of overdraw.
  The earlier "parse plus style is the larger half" reading in the L0b
  receipt is withdrawn there and here.
- **Live mutation was not measured.** The Boa-scripted fixtures render
  byte-identical frames with no wake events, so this Ortet build does not
  present script mutations. The fifty-thousand live cell timed out at thirty
  minutes on its first frame. This matters less than it looks: live faces in
  this plan are driven by the Rust host mutating the DOM directly, so the
  right live measurement is a host-side harness, not page script.
- **No phase timing exists in genet.** Neither info nor debug logging emits
  parse, style, layout, or paint spans. Attribution had to be done by
  fixture variant.

### L0c, the three-path comparison (2026-09-12)

Receipt: `Code/testing/wing/l0c_three_paths.{json,md}`, frames under
`Code/testing/wing/l0c_frames/`; probe at `mere/crates/probes/wing-three-paths/`
(untracked). RTX 4060 Laptop GPU, Vulkan, driver 610.88, netrender
`3961aca91`, mesocosm at isometry `89752cd` through a detached worktree
(`Code/worktrees/isometry-l0c`, because the live checkout's `mesocosm-core`
was mid-edit by another session). 1920x1080, one orthographic camera, no
genet DOM; a CPU z-buffer oracle cross-checked against Mesocosm's depth pipeline.

The following tables preserve the original run. The 2026-09-13 follow-up
below supersedes its yaw and sprite-cache claims; the scale workloads differ.

Correctness, two bodies, worst interior mismatch over three frames:

| case | A fragments | B sprites | C depth |
| --- | ---: | ---: | ---: |
| crossing | 0.25% | 0.48% | 0.00% |
| articulated, tail over the ledge | 1.39% | 1.00% | 0.03% |
| terrain edit at the chunk seam | 0.22% | 0.49% | 0.00% |
| clipped viewport | 0.25% | 0.48% | 0.00% |
| material change | 0.47% | 0.75% | 0.00% |
| continuous yaw | 0.17% | 0.63% | 1.63% (adapter quantizes yaw) |

Cost, 1000 bodies (181k resident rectangles on A), median frame ms:

| case | A | B | C |
| --- | ---: | ---: | ---: |
| crossing | 13.7 | 17.2 | 2.7 |
| articulated | 11.2 | 31.0 | 3.8 |
| terrain edit | 7.9 | 16.9 | 2.6 |
| clipped | 91.9 | 22.0 | 3.0 |
| material change | 19.2 | 21.2 | 3.0 |
| continuous yaw | 223.5 | 18.0 | 3.4 |

Findings, in the receipt's numbering:

- **Path C has small residuals on most fixtures.** Original medians were
  2.6 to 3.8 ms at a thousand repeated bodies. Its yaw was quantized in this
  run; the follow-up adds continuous parent yaw. Rounded zero percentages
  do not mean exact pixels, and distinct geometry raises its cost.
- **Path A's re-lower is about 1.1 µs per rectangle, CPU-bound.** Roughly
  14,000 re-lowered rectangles fit under 16 ms, about a hundred checkered
  walkers turning; the rest of the frame is gone. Retained placement alone
  reproduces L0a's number.
- **T4's netrender fallback measured.** One clip layer costs path A 3.3x at 200 bodies and
  6.7x at 1000, with the lower count at zero: the fallback never retains.
- **Painter order fails on ordinary bodies.** A six-cell-wide walker spans
  twelve depth keys; placed at its base key the ground under its front edge
  paints over it, placed at its front key it paints over the wall. Both
  painter paths share the defect. The one-cell rule is a property of path
  A, not an application constraint, and it is retired (ruling 12).
- **The original per-body sprite cache duplicated content.** 1.5 GB RSS at a thousand articulated
  bodies at 22 px per voxel, and a 974 ms frame when every body's facing
  changes at once.
- **`mesocosm-mesh` `Quad::corners` winding is reversed for y faces**
  against its own doc comment. Mesocosm's `cull_mode: None` never noticed;
  the probe's first winding-based cull dropped every top face until path C
  disagreed. Recorded for `mesocosm-mesh`'s owner; nothing changed there.
- **Not priced:** CSS ownership of per-part classes, effects and hit
  testing, which never ran through genet. Under path C those become
  instance data and a picking hook; that boundary is genet T3's.

**L0c verdict.** Resident geometry with a depth attachment is the honest
occlusion mechanism for bodies and ground. Retained planar fragments keep
their place for planar, static content. Sprites are a bake for far or fixed
things at small pixel scale.

**Original qualifications (Mark's review, 2026-09-13):** the
continuous-yaw row is not an equivalent workload, since C snapped between
quarter turns while A re-projected, so C's advantage for continuous
rotation was unestablished until ruling 15 landed and the case reran on
identical poses; the sprite memory figure describes the probe's
per-body cache, not sprites as such; "exact" overstated it, C has one or
two residual pixels on most frames and 610 on the articulated case's
frame 30; T4's netrender fallback was measured and genet embedding was
not; and variety was not measured at all, since the swarm repeats a few
shapes and C was configured for 64 cached meshes.

**Follow-up (2026-09-13):** `Code/testing/wing/l0c_2026-09-13/README.md`
records finite parent yaw, matching A/C poses, content-keyed sprites and
independent shape counts. At 1,000 duplicate bodies, three yaw order runs
give A 135–148 ms and C 2.14–2.43 ms median, with zero geometry uploads.
One C repeat has a 20.71 ms p95, so this is not a deadline guarantee.
Across 1, 16, 128 and 1,000 actual shapes, crossing C rises from 2.09 to
8.43 ms and B from 9.53 to 10.53 ms. B holds 5.42 to 276.10 MiB of images;
turning 1,000 distinct shapes raises it to 819.64 MiB and a 612 ms spike.
The corrected swarm distribution is a new workload; old timings are not
mixed into these comparisons. The original yaw fixture now has two interior
residual pixels per sampled frame. All coloured residuals in the ordinary
articulated and eight-shape crossing frames lie on coincident surfaces
with competing checker materials; the oracle and GPU do not share a tie
policy. This explains the residual without claiming pixel-exact agreement.

**Specimen bench follow-on:** the [prerequisite assessment](#specimen-bench-prerequisites-2026-09-13)
now supplies the prior-work inventory, ownership and implementation slices.
Its activity/effects boundary follows bodies interacting with their world.

### L0a/L0b founding verdict (historical)

The measurements below remain scoped to their original fixtures. Their
hybrid and body-fragment default was superseded by rulings 13–15 and the
current L3/L5 scopes; it does not set specimen bench prerequisites.

L0's done condition is met with one named gap (live mutation through the
DOM is unmeasured until a host-side harness exists). The ceilings:

| path | largest count under a 16 ms frame |
| --- | ---: |
| netrender, retained fragments, live | 200,000 rectangles (measured), ~290,000 (interpolated) |
| netrender, flat rebuild, live | 10,000 to 15,000 rectangles |
| genet DOM, steady state | at least 5,000 elements, more unmeasurable |
| genet DOM, first frame within 5 s | about 5,000 elements |

The bottleneck is genet's first-frame scaling, not netrender. The honest
default is therefore the **hybrid**: live faces for bodies in focus and
during rotation, baked sprites elsewhere. Live-all-the-time at Paredros scale
is gated on an engine slice that makes genet's first frame linear (the
quadratic term is the per-positioned-box whole-tree rework named above; the
linear parse and style cost is already small) plus a host-side mutation
harness to measure the live path. Both are genet's T2 and T3.

**L5 shape, ruled 2026-09-12 (ruling 10):** put the DOM at body or part
granularity and keep faces below it. A body is one element whose paint is a
retained netrender fragment, the way genet already splices an iframe's paint
list into a replaced box. Element count drops to hundreds, rectangle count
stays with netrender at the level L0a proved, and CSS still owns classes,
transforms, hit testing, and styling per part. Per-face shading and masks
then apply per part rather than per face. This is the shape the hybrid was
always going to converge on; the genet first-frame slice is owed regardless
and belongs in L1's genet plan as a named consumer.

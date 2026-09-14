# wing-scene extraction plan

**Date:** 2026-09-14

**Status, 2026-09-14:** assessment. No code moved. Owns the execution of lane
L9 in [the orthographic voxel presentation plan](2026-09-11_orthographic_voxel_presentation_plan.md#l9-the-shared-scene-crate-wing-scene-founded-2026-09-14),
whose six done conditions are restated in §7 as tests and receipts. The lane's
sequencing precondition is met: `shared/wing-scenario` landed at cc7828f.

**Owns:** the move of Mesocosm's shared-depth scene out of `mesocosm-genet`
into the Isometry path crate `shared/wing-scene`, the product-neutral input
contract that replaces `mesocosm_core::World`, and the order of steps that
keeps both product workspaces compiling throughout.

**Does not own:** world truth, the appearance/volume/bake crate decision (still
open in the presentation plan), Paredros's P1 retarget itself (a Paredros lane
once this API is messaged over), and Isometry's board as a third consumer.

---

## 1. Section's current public surface

23 files, 4,569 lines, 39 `#[test]` functions. A parenthesised bare number is
the defining line; a path after it is a caller outside `src/section/` —
Mesocosm's bench (`src/app/bench/`), the section views (`src/app/frame.rs`,
`creator.rs`, `devtime.rs`, `receipts.rs`, `drive.rs`, `actions.rs`,
`config.rs`, `app.rs`) or `examples/`. No path means no outside caller.

**section.rs (557).** `SLAB_HALF_HEIGHT` (62, `app/config.rs:139`), `PAN_STEP` (67,
`app.rs:424-427`), `struct Pan` (75, `app.rs:100,276`),
`struct SectionFrame<'a>` with `world`/`volumes`/`ground`/`dirty`/`centre`/
`pose`/`roster` (83, `app/frame.rs:394`, `app/bench/producer.rs:310`),
`struct Section` (98), `half_height_or_default` (516, `frame.rs:37`),
`centre_on` (542, `frame.rs:39`). `Section`'s inherent methods: `new` (140,
`creator.rs:193,230,414`, `bench/producer.rs:195`), `resize` (188,
`bench/producer.rs:205`), `mode` (225, `bench/producer.rs:191`,
`receipts.rs:258`), `encoded_view` (232, `bench/producer.rs:340`),
`configure_bodies` (236, `frame.rs:152`, `bench/producer.rs:208`),
`body_stats` (247, `bench/producer.rs:329`), `terrain_diagnostics` (254,
`bench/producer.rs:324`, `bench/probe_state.rs:391`), `half_height` (259,
`receipts.rs:253`), `set_half_height` (264, `frame.rs:149`,
`bench/producer.rs:295`), `slab_window` (275, `frame.rs:195`),
`last_roster_members` (281, `receipts.rs:244`),
`last_roster_capsules_dropped` (294, `receipts.rs:248`), `render` (301,
`bench/producer.rs:310`), `display_texture` (448, `frame.rs:342`,
`receipts.rs:53`), `draw` (453, `frame.rs:394`, the chromeless fallback).

**section/camera.rs (493, 9 tests).** `SLAB_DEPTH` (21, `frame.rs:90`, `bench/producer.rs:233`),
`OBLIQUE_DEGREES` (35), `TERRARIUM_DEGREES` (38,
`config.rs:142`), `enum CameraMode` (43) with `ALL` (68), `is_terrarium` (78),
`quarter_turn` (86), `parse` (102), `name` (108), `forward` (121), `basis`
(147), `vertical_half` (166), `slab_reach` (184) — `CameraMode` itself is used
by `config.rs`, `actions.rs:460-473`, `creator.rs`, `bench/state.rs`,
`bench/spatial.rs`, `examples/camera_compare.rs`; `struct Framing` + `new`
(196, 204, `creator.rs:200,237,421`, `bench/producer.rs:195`);
`struct SlabWindow` + `new` + `holds` (228, 238, 253 — constructed inside the
crate, returned to `frame.rs:195`).

**section/view.rs (236, 2 tests).** `camera_basis(mode, pitch)` (218, `frame.rs:44,143`,
`bench/producer.rs:400`). `struct View` is `pub(super)`: the single set of
camera numbers everything else derives from (`trace`, `window`, `matrix`,
`clip`).

**section/terrarium.rs (164) and terrain.rs (77).** `BODY_SCALE` re-exported as `TERRARIUM_BODY_SCALE` (terrarium.rs:11,
`frame.rs:85`), `framed_habitat` (15, `app.rs:266`), `enum Cutaway` + `parse` +
`name` (31, 38, 46, `config.rs:70,143`, `frame.rs:60`), `fn occupied`
re-exported as `terrarium_occupied` (117, `frame.rs:62`, `drive.rs:405`),
`Section::set_mode` (127, `creator.rs:252,334`, `devtime.rs:48`),
`Section::configure_terrarium` (134, `frame.rs:52`); `enum TerrainStyle` +
`parse`/`name`/`resolved` (terrain.rs:10,53,62,70, `config.rs:72,144`),
`Section::configure_terrain` (terrain.rs:18, `frame.rs:150`).

**section/capsules.rs (112).** `pose_of` (16), `pose_of_scaled` (20, `frame.rs:170`), `roster_of` (40),
`roster_of_scaled` (48, `frame.rs:197`). **`pose_of` and `roster_of` have no
caller outside the module** — they are 1.0-scale wrappers kept for the
`follow.rs:19-20` doc comment. Dead surface; retire rather than move.

**section/bodies.rs (569, 3 tests) and its children.** `enum BodyMode` + `parse`/`name` (23,30,38, `config.rs:74,145`, `frame.rs:154`,
`bench/producer.rs:208`), `DEFAULT_BODY_BUDGET` (46, `config.rs:146`,
`bench/producer.rs:208`), `struct BodyFrameStats` (49, `bench/producer.rs:329`).
`BodyLayer` is `pub(super)`.
`anchors.rs`: `MAX_GLYPH_ANCHORS` (13), `struct GlyphAnchor` (16,
`bench/spatial.rs:62`), `Section::glyph_anchors` (32, `bench/producer.rs:270`).
`appearance.rs`: `Section::set_body_tint` (23, `bench/producer.rs:218`),
`Section::body_tint` (50, internal only).

**section/glyphs.rs (297), inspection.rs (233), capture.rs (106).** `MAX_SPATIAL_GLYPHS` (glyphs.rs:9, `bench/trial.rs:165`),
`enum GlyphOrientation` (14), `struct SpatialGlyph` (42) —
`bench/spatial/sampling.rs`, `bench/trial.rs`, `bench/trial/carving.rs`;
`Section::set_glyphs` (287, `bench/producer.rs:294`).
`struct BodySelection` (inspection.rs:23), `struct BodyPick` (32),
`enum BodyPickError` (44); `Section::set_body_preview` (66,
`bench/producer.rs:296`), `set_body_yaw` (81, `bench/producer.rs:212`),
`body_yaw` (95, internal), `presentation_bounds` (101, four calls in
`bench/producer.rs:237,248,267,284`), `pick_pixel` (111,
`bench/producer.rs:94`), `pick_ndc` (124, tests only), `validate_pick` (167,
`bench/producer.rs:100`), `select_part` (200, `bench/producer.rs:121`),
`validate_selection` (215, `bench/spatial/saved.rs:45,102`), `set_body_focus`
(228, `bench/producer.rs:226`).
`Section::capture` (capture.rs:13, tests and `section/picking_pixels.rs:21`),
`Section::capture_from` (23, `receipts.rs:88`).

---

## 2. The `World` coupling inventory

`grep -n '\bWorld\b'` over the module returns **34 lines** (the lane's "31" was
counted before the 2026-09-13 spatial files landed): 15 in non-test code and 19
in tests and fixtures. Non-test sites and their minimal product-neutral
replacement:

| Site | Reads | Replacement |
| --- | --- | --- |
| `section.rs:44` `use mesocosm_core::World` | — | deleted |
| `section.rs:84` `SectionFrame::world` | passed only to `bodies::prepare` and `fallback_all` | `bodies: &[SceneBody<'_>]` |
| `bodies.rs:6` import | — | deleted |
| `bodies.rs:299` `prepare(world, volumes, window)` | `world.organisms` (id, position, body(), phenotype, is_alive()), `world.controlled_id()`, `world.ruleset()` (via `materials::project`) | one `&[SceneBody]` per frame: document, pose, tint, subject id; plus optional `&[PartMaterial]` the product projects |
| `bodies.rs:252` `validate_selection(sel, world, volumes)` | re-finds the organism by id and re-projects it | `validate_selection(sel, body: &SceneBody)` — the caller already holds it |
| `bodies.rs:466` `fallback_all(world)` | re-finds each placed organism to rebuild capsule poses | the retained `Vec<SceneBody>` of the frame just prepared; no lookup |
| `capsules.rs:7,16,21,41,49` | `world.body()`, `world.position()`, `world.organisms`, `world.controlled_id()` | stays in mesocosm-genet: these are Mesocosm's capsule-roster policy, not scene API |
| `inspection.rs:9,170,218` | `validate_pick`/`validate_selection` forward to `bodies` | same as `bodies.rs:252` |
| `terrarium.rs:15` `framed_habitat(world)` | `world.terrarium_habitat()`, `world.organisms` positions and body aabbs | stays in mesocosm-genet; the crate takes the resulting world-space bounds, not the habitat |

Five further couplings are not `World` but are equally product-bound:

- `appearance.rs:60` calls `crate::app::look_of(organism)` — the host's kingdom
  palette. Becomes a caller-supplied `tint: [f32; 3]` per body (Paredros
  already supplies `Appearance::played`/`other`,
  `paredros-client/src/producer/bodies.rs:199-203`).
- `bodies.rs:359` `materials::project(&organism.phenotype, world.ruleset())` —
  Mesocosm's process mosaic. `materials.rs` stays in mesocosm-genet and the
  adapter hands the resulting `Vec<PartMaterial>` in per body. Paredros has no
  phenotype and supplies an empty slice
  (`paredros-client/src/producer.rs:38-39`).
- `bodies.rs:504` `body_origin(organism, scale, grounded)` reads
  `organism.position` (`[i32;3]`) and `body().aabb()`. The neutral form is a
  continuous `position: [f32;3]` plus a `ground_anatomy: bool`; Paredros
  already computes feet from `MotionPose` at `MOTION_SCALE`
  (`producer/bodies.rs:185-192`).
- `SectionFrame::ground: &Ground` — `mesocosm_core::places::Ground`. **Both
  products already use this exact type**: `paredros_world::World::ground`
  returns `&mesocosm_core::places::Ground`
  (`paredros/crates/paredros-world/src/world.rs:157`, importing it at line 7).
  It is a `mesocosm_core` type but not `World`, so it satisfies the lane's
  prohibition. It is still wrapped behind a trait (§3) so Isometry's map can be
  a third source without a fourth producer.
- `LiveBodyProjection.organism: OrganismId` (`mesocosm-mesh/src/live.rs:43`)
  and `MeshError::EmptyBodyProjection { organism }`. `OrganismId(pub u32)`
  (`mesocosm-core/src/organism.rs:44`) against Paredros's
  `SubjectId(pub u64)` (`paredros-identity/src/lib.rs:40`). See §6, risk 1.

---

## 3. The proposed wing-scene public API

Illustrative, not compile-ready: derives are omitted and exact error types
follow the move.

```rust
// ---- identity -------------------------------------------------------------
/// 64 bits, so a Paredros SubjectId fits without narrowing.
pub struct SubjectKey(pub u64);

/// A drawn part plus the revision that expires it (was `BodySelection`).
pub struct PartAddress {
    pub subject: SubjectKey,
    pub part: mesocosm_core::PartId,
    pub revision: mesocosm_mesh::BodyDependencyRevision,
}

// ---- inputs (condition 1) -------------------------------------------------
/// `yaw_radians` is ruling 15's continuous parent yaw.
pub struct Pose { pub position: [f32; 3], pub yaw_radians: f32 }

/// One body for one frame. No product world appears here.
pub struct SceneBody<'a> {
    pub subject: SubjectKey,
    pub document: &'a mesocosm_core::BodyDocument,
    pub pose: Pose,
    pub scale: f32,
    /// Stand the document's own floor on `pose.position.y`.
    pub grounded: bool,
    pub tint: [f32; 3],
    /// Product-projected per-part expression; empty is legal.
    pub materials: &'a [mesocosm_render::PartMaterial],
    /// In shot regardless of the window (Mesocosm's controlled critter).
    pub always_visible: bool,
}

// ---- volume source, with the declared-extent fallback (condition 1) -------
pub use mesocosm_mesh::{Volume, VolumeMap, VolumeRef, VolumeSource};

/// The declared-extent solid fallback: one `Volume::solid(half_extent * 2,
/// material)` per reference the frame's documents address, for anatomies that
/// carry no voxel data (every Paredros body today).
pub struct DeclaredExtentVolumes { /* volumes: VolumeMap, conflicts: usize */ }

impl DeclaredExtentVolumes {
    pub fn from_documents<'a>(docs: impl IntoIterator<Item = &'a mesocosm_core::BodyDocument>,
        material: u8) -> Self;
    /// One tag declared at two extents: counted, never drawn wrong silently.
    pub fn conflicts(&self) -> usize;
}
impl VolumeSource for DeclaredExtentVolumes { /* ... */ }

/// An enum, not `&dyn` — see risk 2.
pub enum SceneVolumes<'a> {
    Voxels(&'a VolumeMap),
    DeclaredSolid(&'a DeclaredExtentVolumes),
}

// ---- terrain source (condition 2) -----------------------------------------
/// Addressed by revision, so an unchanged ground costs no CPU walk.
pub trait TerrainSource {
    fn revision(&self) -> u64;
    fn brick_map(&self) -> Result<mesocosm_lens::BrickMap, String>;
    /// Slots to re-upload when only `dirty` bricks moved. `None` forces full.
    fn refresh(&self, map: &mut mesocosm_lens::BrickMap, dirty: &[[i16; 3]])
        -> Result<Option<Vec<u32>>, String>;
}
/// The impl both products get for free.
pub struct GroundTerrain<'a>(pub &'a mesocosm_core::places::Ground);
impl TerrainSource for GroundTerrain<'_> { /* ... */ }

// ---- the orthographic slab camera (condition 2) ---------------------------
/// One free forward vector, so Mesocosm's seven named modes and Paredros's
/// `CameraPolicy::forward` are both presets over it.
pub struct SlabCamera {
    pub centre: [f32; 3],
    /// Any direction with a horizontal component; straight down is refused.
    pub forward: [f32; 3],
    pub half_height: f32,
    pub aspect: f32,
    pub depth: f32,
    /// `None` is the plain slab between the two world-vertical walls.
    pub cutaway: Option<Cutaway>,
}

/// The world-space cutaway the presentation contract names.
pub enum Cutaway {
    /// Keep only what falls inside this world box, whatever the camera does.
    Bounds { min: [f32; 3], max: [f32; 3] },
    /// Drop everything on the near side of this world plane.
    Plane { normal: [f32; 3], distance: f32 },
}

impl SlabCamera {
    pub fn new(centre: [f32; 3], forward: [f32; 3], half_height: f32,
        aspect: f32, depth: f32) -> Option<Self>;
    /// Right, up, forward. World up defines the standing wall, not screen up.
    pub fn basis(self) -> Option<[[f32; 3]; 3]>;
    pub fn trace(self) -> Option<mesocosm_lens::TraceCamera>;
    /// The one `clip_from_world` terrain and bodies both consume.
    pub fn clip_from_world(self) -> Option<[[f32; 4]; 4]>;
    pub fn clip(self) -> mesocosm_render::ClipSlab;
    pub fn window(self) -> SlabWindow;
    /// Where a world point lands, so a test can name a pixel.
    pub fn ndc_of(self, point: [f32; 3]) -> Option<[f32; 2]>;
    pub fn pixel_of(self, point: [f32; 3], size: [u32; 2]) -> Option<[u32; 2]>;
    /// The input to a product's own bedrock clamp. See risk 5.
    pub fn vertical_half(self) -> f32;
}

/// The oriented box the camera shows; what a roster culls against.
pub struct SlabWindow { pub centre: [f32; 3], pub axes: [[f32; 3]; 3], pub half: [f32; 3] }
impl SlabWindow { pub fn holds(&self, at: [f32; 3]) -> bool; }

// ---- the scene ------------------------------------------------------------
pub struct SceneFrame<'a> {
    pub camera: SlabCamera,
    pub bodies: &'a [SceneBody<'a>],
    pub volumes: SceneVolumes<'a>,
    /// `None` is the isolated body preview: no trace, no terrain in the pick.
    pub terrain: Option<&'a dyn TerrainSource>,
    pub dirty: &'a [[i16; 3]],
    pub grade: mesocosm_lens::Grade,
    pub terrain_appearance: Option<mesocosm_lens::TerrainAppearance>,
    pub body_budget: usize,
}

pub struct Scene { /* tracer, map, bodies, glyphs, colour + depth targets */ }

impl Scene {
    pub fn new(device: wgpu::Device, queue: wgpu::Queue, width: u32, height: u32)
        -> Result<Self, String>;
    pub fn resize(&mut self, width: u32, height: u32);
    /// Bodies as instances first, then the tracer's depth join, both under one
    /// `clip_from_world`, then the glyph batch (conditions 2 and 3).
    pub fn render(&mut self, encoder: &mut wgpu::CommandEncoder, frame: SceneFrame<'_>)
        -> Result<SceneStats, String>;
    pub fn encoded_view(&self) -> &wgpu::TextureView;
    pub fn display_texture(&self) -> &wgpu::Texture;
    pub fn capture_from(&self, master: &wgpu::Texture,
        overlay: impl FnOnce(&mut wgpu::CommandEncoder, &wgpu::TextureView, wgpu::TextureFormat))
        -> Option<(u32, u32, Vec<u8>)>;
    pub fn terrain_diagnostics(&self) -> Option<mesocosm_lens::BrickDiagnostics>;

    // condition 3
    pub fn set_glyphs(&mut self, glyphs: Vec<SpatialGlyph>) -> Result<(), String>;

    // condition 4
    pub fn set_body_focus(&mut self, subject: Option<SubjectKey>, selected: Option<PartAddress>);
    pub fn presentation_bounds(&mut self, body: &SceneBody<'_>, volumes: SceneVolumes<'_>)
        -> Result<Option<([f32; 3], [f32; 3])>, String>;
    /// Per-part bounds: the oracle Paredros's severance and pose tests read.
    pub fn part_bounds(&self, subject: SubjectKey, part: mesocosm_core::PartId)
        -> Option<([f32; 3], [f32; 3])>;
    pub fn glyph_anchors(&mut self, body: &SceneBody<'_>, volumes: SceneVolumes<'_>,
        selected: Option<PartAddress>) -> Result<Vec<GlyphAnchor>, String>;
    pub fn pick_pixel(&self, pixel: [u32; 2]) -> Result<Option<BodyPick>, BodyPickError>;
    pub fn pick_ndc(&self, ndc: [f32; 2]) -> Result<Option<BodyPick>, BodyPickError>;
    pub fn validate_pick(&mut self, pick: BodyPick, body: &SceneBody<'_>,
        volumes: SceneVolumes<'_>) -> bool;
    pub fn select_part(&self, subject: SubjectKey, current: Option<PartAddress>,
        backwards: bool) -> Option<PartAddress>;
}

// ---- producer wrapper (condition 5) ---------------------------------------
/// What a product supplies each frame; never GPU code.
pub trait SceneSource {
    fn frame(&mut self, size: [u32; 2], aspect: f32) -> Result<SceneFrame<'_>, String>;
}
/// The unchanged-input skip plus the sRGB / straight-alpha contract the bench
/// settled. Implements `cambium_rootstock::TextureProducer`.
pub struct SceneProducer<S: SceneSource> { /* scene, source, signature, renders */ }
impl<S: SceneSource> SceneProducer<S> {
    pub fn new(source: S) -> Self;
    pub fn renders(&self) -> u64;
    pub fn presented_camera(&self) -> Option<SlabCamera>;
    /// The skip evidence Paredros's tests read: camera, size, terrain revision
    /// and each body's (subject, revision, pose).
    pub fn signature(&self) -> &SceneSignature;
    pub fn cached_bodies(&self) -> usize;
    pub fn last_error(&self) -> Option<&str>;
}
```

`SpatialGlyph`, `GlyphOrientation`, `MAX_SPATIAL_GLYPHS`, `GlyphAnchor`,
`MAX_GLYPH_ANCHORS`, `BodyPick` and `BodyPickError` move verbatim, with
`BodySelection` renamed `PartAddress` and `OrganismId` becoming `SubjectKey`.
`mesocosm_core::effect_experiment::Glyph`, which `SpatialGlyph` carries
(`section/glyphs.rs:7`), is a `mesocosm_core` enum and not `World`, so it moves
as-is; widening it to a `wing-glyphs` canon is a later layer.

### What stays in mesocosm-genet

`Section` becomes an adapter of roughly 250 lines over `Scene`. It keeps and
supplies, per frame:

- `CameraMode`'s seven presets, `camera_basis`, `Framing`, `SLAB_DEPTH`,
  `SLAB_HALF_HEIGHT`, `OBLIQUE_DEGREES`, `TERRARIUM_DEGREES`, `centre_on`,
  `half_height_or_default`, `Pan`, `PAN_STEP` — together a `SlabCamera` plus
  Mesocosm's bedrock clamp; none of it is shared.
- `terrarium.rs`: habitat framing, `framed_habitat`, `Cutaway` the *policy*
  enum with its occupied/always/never reading, the filtered `BrickMap` rebuild.
  It produces a `wing_scene::Cutaway::Bounds` and a `TerrainSource`.
- `terrain.rs` (`TerrainStyle`, the `Grade`/`TerrainAppearance` presets),
  `materials.rs` (phenotype to `Vec<PartMaterial>`), `capsules.rs` (the
  `CritterPose` roster and played pose, minus the retired `pose_of`/`roster_of`),
  `app::look_of` tinting, and `BodyMode` — which selects whether Mesocosm hands
  `Scene` bodies or hands the tracer a capsule roster, so it is host policy.
- The capsule fallback. `BodyLayer::add_fallback` (`section/bodies.rs:389-420`)
  projects a `BodyLensProjection` when a voxel projection fails. Mesocosm-only:
  `Scene` reports the failure and the adapter decides.

### What Paredros's P1 producer supplies

`SceneModel` (`paredros-client/src/producer/handle.rs:54`) keeps `camera`,
`appearance`, `terrain` and `follow_centre` and gains a `SceneSource` impl:
`game.bodies().all().filter(alive)` becomes `SceneBody` with
`SubjectKey(subject.0)`, `&record.document`, a `Pose` from `MotionPose` over
`MOTION_SCALE`, `tint` from `Appearance`, empty `materials`, `grounded: true`;
volumes from `DeclaredExtentVolumes::from_documents(.., BODY_MATERIAL)`;
`terrain: Some(&GroundTerrain(game.world().ground()))`; camera from
`CameraPolicy`. `producer/camera.rs` (216), `bodies.rs` (272) and `scene.rs`
(420) are retired; `handle.rs` and the fixture stay.

---

## 4. Dependency check

| Dependency | Mesocosm | Paredros | One version? |
| --- | --- | --- | --- |
| `mesocosm-lens` — `BrickTracer`, `BrickMap`, `TraceCamera`, `SlabWall`, `Grade`, `FRAME_FORMAT` | path (`mesocosm/Cargo.toml:89`) | path `../../../mesocosm/…`, **optional**, behind `r1-proof` (`paredros-client/Cargo.toml:41`, `:19`) | Yes — one on-disk crate. wing-scene declares it by relative path, as `mesocosm-mesh` already declares `wing-formats`. |
| `mesocosm-render` — `LiveBodyRenderer`, `LiveBody`, `ClipSlab`, `PartMaterial`, `pick_bodies`, `body_bounds`, `posed_quad` | path (`:93`) | path, not optional (`:43`) | Yes |
| `mesocosm-mesh` — `LiveBodyProjector`, `mesh_body`, `VolumeSource`, `VolumeMap`, `Volume`, `BodyDependencyRevision` | path (`:91`) | path (`:42`) | Yes |
| `mesocosm-core`, **not `World`** — `BodyDocument`, `PartId`, `VolumeRef`, `places::Ground`, `effect_experiment::Glyph` | path (`:88`) | path (`:40`); `paredros-world` takes it too (`paredros-world/Cargo.toml:17`) | Yes. lens, mesh and render all depend on it already, so it is unavoidable and is the de-facto shared body-document crate. |
| `wgpu` | workspace `30`, backends named (`mesocosm/Cargo.toml:116-125`) | `30`, same backend list (`paredros-client/Cargo.toml:57`) | Yes, one major. wing-scene declares `version = "30", default-features = false, features = ["std", "wgsl"]` and leaves backends to the leaf binaries. |
| `cambium-rootstock` — `TextureProducer`, `ProducedTexture`, `ProducerContext`, `SourceAlpha`, `SourceEncoding` | mere `4f4de1d0` (`mesocosm/Cargo.toml:64`) | mere `4f4de1d0` (`paredros/Cargo.toml:68`) | Yes, identical rev; `shared/wing-scenario/Cargo.toml` pins the same. |
| `netrender` | — | — | **Not needed.** `Section` never names netrender; the host imports `display_texture()` into netrender's graph (`app/frame.rs:342`). |
| `modulus` | workspace | optional under `r1-proof` | Transitive under `mesocosm-lens`; no direct row. |

**Product-crate check.** No proposed dependency reaches `mesocosm-runtime`,
`mesocosm-genet`, `mesocosm-views`, `paredros-world`, `paredros-identity` or
`isometry-core`. The only `mesocosm_core` items named are `BodyDocument`,
`PartId`, `VolumeRef`, `places::Ground` and `effect_experiment::Glyph`;
`World`, `Organism`, `OrganismId`, `BodyPhenotype`, `process::Registry` and
`world::TerrariumHabitat` all stay behind the adapter.

**Patch table.** wing-scene needs its own `[workspace]` and a restated
`[patch.crates-io]` exactly as `shared/wing-scenario/Cargo.toml` does (vello
fork, parley, genet-taffy `=0.14.0`, ipc-channel at genet `101d9e9a`), because
`[patch]` applies only from a workspace root and does not inherit through a
path dependency. It must also be added to the Isometry root workspace's
`exclude` list (`Cargo.toml:13`), beside `shared/wing-scenario` — the root
pins mere at `fb7e136b`, not `4f4de1d0`, so it must never be pulled in.

---

## 5. Move order

Each step ends with `cargo check --workspace --all-features --all-targets` and
`cargo test` green in **both** product workspaces.

1. **Found the crate.** `shared/wing-scene` with the `wing-scenario` manifest
   shape (own `[workspace]`, restated patch table, MPL-2.0, `publish = false`),
   added to the root `exclude`. Move `section/camera.rs` and `section/view.rs`
   into `src/camera.rs` (camera numbers, `SlabCamera`, `SlabWindow`, `Cutaway`)
   — **split, not verbatim**: `CameraMode`'s seven presets and `camera_basis`
   stay in mesocosm-genet and produce a `forward` vector; the `View` internals
   (`trace`, `matrix`, `clip`, `reach`, `window`) become `SlabCamera` methods.
   Move `mesocosm_mesh::VolumeSource` re-export plus the new
   `DeclaredExtentVolumes`, lifted from `paredros-client/src/producer/bodies.rs:210-237`.
   Nothing consumes it yet. Camera parity is proved by the 11 tests from
   `camera.rs` and `view.rs` re-pointed at `SlabCamera`.
2. **Move the body layer.** `src/bodies.rs` ← `section/bodies.rs` minus the
   `World` walk, the capsule fallback and `rendered_tint`; `src/anchors.rs` ←
   `section/anchors.rs` verbatim but keyed on `SceneBody`. `prepare` takes
   `&[SceneBody]` and the window. `section/bodies.rs` shrinks to the adapter
   that builds `SceneBody` from `world.organisms` and keeps `add_fallback`.
   `bodies.rs` at 569 lines splits: `bodies.rs` (prepare/draw) and
   `bodies/placement.rs` (origin, bounds, intersects, stats).
3. **Move the terrain join and the targets.** `src/scene.rs` ← the `render`
   body of `section.rs:301-444` plus `target()` and `copy_to_display`;
   `src/capture.rs` ← `section/capture.rs` verbatim. `TerrainSource` and
   `GroundTerrain` land here. `Section::render` becomes a call into
   `Scene::render` with a `SceneFrame`. **This is the step where Mesocosm's
   pixels can move**; it is gated on the capture comparison in §7.
4. **Move glyphs.** `src/glyphs.rs` + `glyphs.wgsl` ← `section/glyphs.rs`
   verbatim, with `View` replaced by `SlabCamera`. 297 lines, under the
   ceiling.
5. **Move the queries.** `src/query.rs` ← `section/inspection.rs`, with
   `BodySelection` → `PartAddress` and the two `&World` parameters replaced by
   `&SceneBody`. `section/picking_pixels.rs` and `section/picking_tests.rs`
   (588 lines of GPU tests) move with it and split one file per subject:
   `query_tests.rs` (pick/validate) and `mask_tests.rs` (the GPU ownership
   mask).
6. **Section is an adapter.** `section.rs` keeps `SectionFrame`, `Pan`,
   `centre_on`, `half_height_or_default` and every `Section::` method as a
   forwarder, so `app/frame.rs`, `creator.rs`, `receipts.rs` and
   `bench/producer.rs` are untouched at this step. `terrarium.rs`,
   `terrain.rs`, `materials.rs`, `capsules.rs`, `appearance.rs` **stay** and
   are rewritten only where they touched `BodyLayer` internals.
7. **Producer wrapper.** `src/producer.rs` with `SceneSource` and
   `SceneProducer`. `bench/producer.rs`'s `TextureProducer` impl
   (`app/bench/producer.rs:354-385`) becomes a `SceneSource` impl; its
   sRGB-to-linear tint conversion (`:361-367`) stays in the bench.
8. **Paredros retargets.** `paredros-client/src/producer/{camera,bodies,scene}.rs`
   deleted; `handle.rs` gains the `SceneSource` impl; the eight GPU tests in
   `producer/tests.rs` re-point at `SceneProducer` and
   `SceneProducer::presented_camera()` in place of `presented_view()`.
   `mesocosm-lens` can stop being optional there, retiring the `r1-proof` gate
   wart the P1 progress note records.

**Verbatim:** `capture.rs`, `glyphs.rs` + `glyphs.wgsl`, `anchors.rs`,
`anchors_tests.rs`, `depth_tests.rs`, `bodies_tests.rs`.
**Split (600-line ceiling):** `bodies.rs` (569 and growing once the fallback's
absence is replaced by richer stats), `picking_tests.rs` (404) with
`picking_pixels.rs` (184) — one test file per subject, per the ceiling rule.
**Stay:** `terrarium.rs`, `terrain.rs`, `materials.rs` + `materials/capture.rs`,
`capsules.rs`, `appearance.rs`, `appearance_tests.rs`, `inspection_tests.rs`
(it drives `BodyLayer` through Mesocosm worlds), `section/tests.rs`.

---

## 6. Risks and unknowns

1. **`OrganismId` is u32 and `SubjectId` is u64.** `LiveBodyProjector::project`
   takes an `OrganismId` (`mesocosm-mesh/src/live.rs:91`) and
   `LiveBodyProjection` carries it (`:43`), as does
   `MeshError::EmptyBodyProjection`. A `SubjectKey(u64)` cannot round-trip
   through it. Three ways out: wing-scene calls `mesh_body`
   (`mesocosm-mesh/src/lib.rs:213`) and owns its own per-`(SubjectKey,
   revision)` cache, as Paredros already does
   (`producer/bodies.rs:239-255`) — but that loses the per-`VolumeRef` mesh
   cache the projector gives Mesocosm; or mesocosm-mesh gains a
   `project_body(&BodyDocument, &impl VolumeSource) -> (BodyMesh,
   BodyDependencyRevision)` with `project` as a wrapper; or `OrganismId`
   widens to u64. **This is a decision for Mark**, and it edits mesocosm-mesh,
   which is outside the named owner tree.
2. **`mesh_body` and `project` take `&impl VolumeSource`, not `&dyn`.**
   `SceneVolumes` as an enum works; a `&'a dyn VolumeSource` field does not,
   without an `impl VolumeSource for &dyn VolumeSource` blanket in
   mesocosm-mesh. The API above uses the enum to avoid that edit; the cost is
   that a third product cannot add a volume source without a wing-scene change.
3. **Draw order is opposite in the two producers.** `Section::render`
   (`section.rs:336-407`) draws bodies, then glyphs *only in the isolated
   preview shortcut*, then the tracer, then glyphs again
   (`section.rs:428-439`). Paredros draws bodies then terrain and no glyphs
   (`producer/scene.rs:201-229`). The isolated-preview early return
   (`section.rs:376-389`) skips terrain entirely and completes a query frame
   with `terrain: false`. `Scene::render` must keep that branch or
   `set_body_preview` and three `bench/producer.rs` framing paths change
   behaviour. Modelled above as `SceneFrame::terrain: Option<...>`.
4. **The `View`-to-`SlabCamera` reduction is where pixels can move.**
   `View::reach` has a fast path that returns `mode.slab_reach(half, aspect)`
   when `pitch.is_none() && depth == SLAB_DEPTH` (`section/view.rs:50-57`) and
   otherwise builds a `SlabWall`. `CameraMode::slab_reach`
   (`section/camera.rs:184-187`) itself reads `SlabWall` with
   `map_or(SLAB_DEPTH * 0.5, ...)`. A single `SlabCamera::reach` that always
   builds the wall is arithmetically the same but not bit-identical in the
   level modes, and `a_level_camera_reaches_exactly_the_half_depth_it_always_did`
   (`section/camera.rs:420-438`) asserts exact equality. Keep the branch.
5. **`CameraMode::vertical_half` bakes `WIDEST_ASPECT = 1.0`**
   (`section/camera.rs:217`, used at `:168-170`) because the bedrock clamp is
   asked before a surface exists. `SlabCamera` carries a real aspect, so
   `vertical_half()` on it is not the same number. Mesocosm's clamp
   (`centre_on`, `section.rs:542`) must keep reading the aspect-free form, or
   `a_tilted_camera_declares_the_extra_height_it_frames`
   (`section/camera.rs:407-415`) and the follow centre both move. Keep
   `centre_on` and its helper in mesocosm-genet, as §5 step 6 does.
6. **Paredros's eight GPU tests** (`producer/tests.rs:105, 197, 290, 369, 480,
   538` plus two helpers-driven cases) read `producer.presented_view()`,
   `producer.bodies()` (`&BodyLayer` with `.bodies[i].subject`,
   `part_bounds`, `cached_bodies`, `signature`) and `body_stats()`. Three break
   as written: `presented_view() -> Option<SlabView>` becomes
   `Option<SlabCamera>`; `DrawnBody::part_bounds` (`producer/bodies.rs:100-121`)
   has no equivalent in the proposed API — wing-scene must expose a
   `part_bounds(subject, part)` or the severance and fractional-pose tests lose
   their oracle; and `cached_bodies()`/`signature()` are the skip test's
   evidence, so `SceneProducer` must expose the same two. Add all three to the
   API before step 8.
7. **The tracer paints pure black at a zero-distance hit on the slab's front
   wall**, recorded in Paredros's P1 progress note. It is a `mesocosm-lens`
   defect, not a scene one, but it will follow the scene into both products and
   should be named in the L9 receipt rather than discovered again.
8. **Mesocosm's bench acceptance is capture-based.** `spatial-coverage.scenario`
   passed in 336 frames over 32 captures, and the spatial save/replay receipt
   requires "the same specimen world hash, pose, camera, body/habitat mode and
   complete part identity". Any change to `BodySelection`'s field names reaches
   `bench/spatial/saved.rs` and the saved-request format. Renaming
   `BodySelection` → `PartAddress` therefore needs either a serde alias or a
   saved-request version bump; the DOC_POLICY §3 "no migration shims" rule
   argues for regenerating the receipts instead.
9. **`section/appearance.rs:60` reaches into `crate::app::look_of`** — a
   host function called from inside the body layer. Nothing in the current test
   suite pins the tint, so this is a silent-drift risk during step 2.
10. **Unknown: how much of `BodyFrameStats`' 20 fields survive.**
    `material_parts`, `secretory_parts`, `carcasses`, `fallback_bodies` and
    `fallback_parts_dropped` are Mesocosm meanings. `SceneStats` should carry
    only the neutral counters and let the adapter add the rest, but
    `bench/probe_state.rs` and `receipts.rs` read the merged struct today.

---

## 7. Done conditions, as tests and receipts

Restating L9's six conditions against tests that exist or must be written.

1. **Decoupled body inputs.** *Exists:* nothing.
   *Must be written:* a pure test in wing-scene building a `SceneFrame` from
   two hand-made `BodyDocument`s with no product crate in scope, asserting the
   crate's `deny` list holds — `cargo tree -p wing-scene` shows no
   `mesocosm-runtime`, `mesocosm-genet`, `paredros-world`, `paredros-identity`
   or `isometry-core`; and a `DeclaredExtentVolumes` test asserting one solid
   per addressed tag at `half_extent * 2` and a counted conflict when one tag
   is declared at two extents (ported from Paredros's declared limit in
   `producer/bodies.rs:19-24`).
2. **One depth attachment, one `clip_from_world`, orthographic slab, world
   cutaway.** *Exists:* `section/view.rs:134` and `:175` (rays start and end on
   the raster depth interval, every turn, three pitches);
   `producer/camera.rs:183` and `:197` (the same two, Paredros's arm);
   `section/depth_tests.rs` (1 test);
   `producer/tests.rs:197` (terrain hides a body, with a bodies-only control).
   *Must be written:* a `Cutaway::Plane` case — today only `Bounds` exists
   (`ClipSlab::bounds`, `live_body.rs:84`, fed from `TerrariumView::bounds`).
3. **Glyphs on that depth.** *Exists:* `section/glyphs.rs`'s own tests and the
   spatial-coverage native receipt (24 combinations, 336 frames, captures at
   `Code/testing/spatial-coverage-2026-09-13/`). *Must be written:* those tests
   re-pointed at `SlabCamera`; the receipt regenerated once, byte-compared
   against the 2026-09-13 captures.
4. **Queries address the same identities.** *Exists:*
   `section/picking_tests.rs` (4), `section/picking_pixels.rs` (3, including
   the GPU/CPU ownership mask at `:37`), `section/inspection_tests.rs` (3),
   `section/anchors_tests.rs` (3), `producer/tests.rs:538` (pick agreement).
   *Must be written:* one test that a `PartAddress` minted from a
   `SubjectKey(u64)` above `u32::MAX` survives a pick round trip — the
   guard against risk 1 being resolved by narrowing.
5. **Producer wrapper.** *Exists:* `producer/tests.rs:480` (an unchanged frame
   produces nothing; a suspension keeps the geometry) and
   `bench/producer.rs:354-385`'s `SourceAlpha::Straight` /
   `SourceEncoding::Srgb`. *Must be written:* one wing-scene test asserting the
   skip signature covers camera, size, terrain revision and every body's
   `(subject, revision, pose)`, and that `alpha`/`encoding` are fixed.
6. **Both products green; no product-world dependency.** *Exists:* Mesocosm's
   `cargo test` suite (39 section tests among it) and the bench scenarios under
   `testing/bench/` (`acceptance`, `spatial`, `spatial-coverage`, `habitat`,
   `population`, `effects`, `carving-*`, `structure-*`, `proportions-*`,
   `generation-*`, `integrated-world`, `failure`); Paredros's 45 client, 132
   world and 6 native tests plus the session smoke.
   *Receipt for the lane:* both workspaces' `--all-features --all-targets`
   check, both test suites, Mesocosm's `acceptance` and `spatial-coverage`
   scenarios rerun with captures compared against 2026-09-13, Paredros's eight
   GPU tests green against `wing_scene::SceneProducer`, and
   `paredros-client/src/producer/{camera,bodies,scene}.rs` deleted.

## Findings

- 2026-09-14: the `World` count is 34 lines, not 31 — 15 non-test, 19 in tests.
- 2026-09-14: `section::pose_of` and `section::roster_of` have no caller
  outside the module; only the `_scaled` pair is live (`app/frame.rs:170,197`).
- 2026-09-14: both products already share `mesocosm_core::places::Ground`
  (`paredros-world/src/world.rs:7,157`), so terrain needs no new format.
- 2026-09-14: `mesocosm_mesh::VolumeSource` already exists
  (`mesocosm-mesh/src/volume.rs:110`); the declared-extent fallback is a new
  impl of it, not a new trait.

## Progress

- **2026-09-14:** assessed and written. No code moved, no commit.

- **2026-09-14, step 1 done, uncommitted.** `shared/wing-scene` founded
  (Cargo.toml in the wing-scenario shape, src/lib.rs, src/camera.rs 276
  lines + camera/tests.rs 310, src/volumes.rs 176) with `SlabCamera`,
  `Cutaway`, `SlabWindow`, `DeclaredExtentVolumes` and the `VolumeSource`
  re-export; mesocosm-genet's `View` is now an alias for `SlabCamera` and
  `CameraMode`'s presets stay behind it; mesocosm-mesh gains
  `LiveBodyProjector::project_body(&BodyDocument, &impl VolumeSource) ->
  Result<(BodyMesh, BodyDependencyRevision), MeshError>` with `project`
  delegating to it and a new `MeshError::EmptyBody` (ruling on risk 1).
  Green: wing-scene 13 tests, mesocosm-mesh 62, mesocosm-genet 155 release,
  both workspace checks, bench acceptance 167 frames and spatial-coverage
  336 frames. Camera parity: all 32 spatial-coverage captures have
  byte-identical viewports against the 2026-09-13 receipt; the only
  differing pixels are the button strip reflowed by the uncommitted
  World-trial control. Deviations from §3: `clip_from_world` and `basis`
  are infallible (the old `View` was), `ndc_of`/`pixel_of` wait for step 5,
  `Cutaway::Plane` lowers provisionally as a replaced near wall until step
  3, and the isolated Oblique preview without a terrarium can differ from
  the old reach in the last bit because the forward vector is no longer
  renormalised there; every shipped path is bit-exact.

# isometer family plan

**Date:** 2026-09-14

**Status, 2026-09-14:** assessment. No code moved, no commit. Sequenced after
the mesocosm-lens ground-hit quantiser floor fix, the mesquite/taproot pin
bumps, and step 8 of
[the isometer extraction plan](2026-09-14_isometer_extraction_plan.md)
(Paredros's P1 retarget). Consumes, and does not restate, that plan's
inventory of `Section`.

**Owns:** turning `mesocosm-lens`, `mesocosm-render` and `mesocosm-mesh` from
Mesocosm-owned path dependencies into isometer's component crates; the
mesocosm-core back-dependency each of them carries today and where each type
lands; the workspace and naming shape of the family; and the resolution of the
presentation plan's open decision "where the merged appearance crate lives".

**Does not own:** the scene API itself (the extraction plan owns it), world
truth, Paredros's P1 retarget, Isometry's board, and **isomere** — see §0.

---

## 0. The isomere boundary

Mark ruled two names on 2026-09-14 (`paredros/design_docs/2026-09-13_genet_document_host_plan.md:176-189`).
**isometer** is the game-world scene family: the slab camera, the depth join,
voxel volumes, meshes, the body renderer, the brick tracer — everything that
answers "what is in the world and where does it land on this raster".
**isomere** is the wing-unique GUI layer: scene-plus-host chrome, overlays,
graphs, and the cambium applications that belong to the wing rather than to
one product. Nothing in this plan founds, sizes or moves isomere; the line
between them is that isometer never names a cambium widget and isomere never
names a voxel. Both were crates.io-free at the ruling, and a fresh read-only
query on 2026-09-14 returns zero crates for `isometer` (so `isometer-core`,
`isometer-lens`, `isometer-render` and `isometer-mesh` are free as well) and
two unrelated chemistry crates for `isomere`.

---

## 1. The graph as it stands

### 1.1 Who depends on whom

Verified from the manifests, 2026-09-14:

| Crate | Depends on | Manifest |
| --- | --- | --- |
| `mesocosm-mesh` | `mesocosm-core`, `wing-formats` (relative path), serde, postcard, blake3 | `mesocosm/crates/mesocosm-mesh/Cargo.toml:14-18` |
| `mesocosm-render` | `mesocosm-core`, `mesocosm-mesh`, `wgpu`, bytemuck, glam, pollster | `mesocosm-render/Cargo.toml:13-18`; dev-dep `mesocosm-runtime` (`:23`) |
| `mesocosm-lens` | `modulus`, `mesocosm-core`, `wgpu`, bytemuck, pollster, serde, postcard, `conatus`, optional `burn` | `mesocosm-lens/Cargo.toml:19-27`; dev-deps `mesocosm-mesh`, `mesocosm-render`, `mesocosm-runtime`, `netrender` (`:34-38`) |
| `isometer` | `mesocosm-core`, `mesocosm-mesh`, `mesocosm-lens`, `mesocosm-render`, `cambium-rootstock`, `wgpu`, serde | `shared/isometer/Cargo.toml:20-36` |

So the three components form a chain — mesh below render, both beside lens —
and isometer sits on all three. **No component depends on isometer**, which is
what makes the family shape available without inverting anything.

Consumers: `mesocosm-genet` takes all three plus isometer
(`mesocosm-genet/Cargo.toml:24-26,56`); `paredros-client` takes render and
mesh unconditionally and lens optionally behind `r1-proof`
(`paredros-client/Cargo.toml:41-43,19`); `shared/wing-integration` takes mesh
(`:15`); `mesocosm-runtime` and `mesocosm-views` take only core.
**Isometry's own crates take none of them** — the root workspace's grep for
`mesocosm-` returns nothing but the `exclude` line (`Cargo.toml:13`).

### 1.2 What each exports, and what each consumer actually uses

**`mesocosm-lens`** (`src/lib.rs:34-45,55,88,94,99,116,157,167`) exports the
body projection (`BodyLensProjection`, `BodyPlacement`, `BodyProjectionError`,
`BodyRevision`, `LensPart`), the brick map (`BrickMap`, `BrickMapError`,
`BrickProjectionRevision`, `BrickRayError`, `BrickRayHit`), the heightfield
march (`Capture`, `DirtyRect`, `FRAME_FORMAT`, `FrameDiagnostics`,
`FrameInput`, `Lens`, `LensError`, `MapChange`, `MapRevision`, `LensScene`,
`SceneCodecError`), the tracer (`BrickCapture`, `BrickChange`,
`BrickDiagnostics`, `BrickFrameInput`, `BrickRevision`, `BrickTraceError`,
`BrickTracer`, `LeasedAtlas`, `SlabWall`, `TraceCamera`) and the presentation
constants (`MAX_CAPSULES`, `MAX_ROSTER`, `MAX_ROSTER_CAPSULES`, `Grade`,
`TerrainAppearance`, `Flight`, `CritterPose`), plus `pub mod bricks/critter/maps`.
Of that, mesocosm-genet uses `BrickDiagnostics`, `BrickTracer`, `MAX_CAPSULES`,
`MAX_ROSTER`, `MAX_ROSTER_CAPSULES`, `SlabWall`, `TerrainAppearance` and
`TraceCamera::orthographic_slab`; paredros-client uses `BrickDiagnostics`,
`BrickTracer`, `BrickTracer::encode_with_depth`, `CritterPose`, `TraceCamera`;
isometer uses `BrickMap`, `BrickRayError`, `BrickRayHit`, `FRAME_FORMAT`,
`Grade::retro`, `TraceCamera::orthographic_slab`. **Nobody outside the crate
uses `Lens`, `LensScene`, `maps` or `critter::wander`.**

**`mesocosm-render`** (`src/lib.rs:34-45`) exports `Camera`; the geometry
helpers (`SceneItem`, `Vertex`, `build_scene_vertices`, `build_vertices`,
`deadened`, `face_shade`, `kingdom_colour`, `material_colour`,
`warning_colour`); the live body (`BodyDrawStats`, `ClipSlab`, `LiveBody`,
`LiveBodyError`, `LiveBodyRenderer`, `PartMaterial`); and `BACKGROUND`,
`RenderError`, `Frame`, `Renderer`. mesocosm-genet uses `Camera::default`,
`PartMaterial`, `RenderError::NoAdapter`, `composite::Composite`,
`live_body::LiveBodyError`; paredros-client uses `ClipSlab`,
`geometry::Vertex`, `live_body::{LiveBody, pick_bodies}`; isometer uses
`ClipSlab`, `PartMaterial`, `composite::Composite` and
`live_body::{BodyHit, BodyQueryError, body_bounds, pick_bodies, posed_quad}`.

**`mesocosm-mesh`** (`src/lib.rs:52-63,67,81,108,193,218`) exports the content
pack (`ContentEntry`, `ContentError`, `ContentPack`, six `MATERIAL_*`,
`MAX_PACK_VOLUMES`, `MAX_VOLUME_VOXELS`, `VOXEL_GRAMMAR_V1`), the two
projections (`Flattened`, `flatten`, `flatten_attributed`; `PartMesh`, `Quad`,
`mesh_volume`, `mesh_volume_naive`; `Placement`, `BodyMesh`, `place_point`,
`mesh_body`, `MeshError`), the live projector (`BodyDependencyRevision`,
`DEFAULT_MESH_CACHE_CAPACITY`, `LiveBodyProjection`, `LiveBodyProjector`), the
wire profile (`BodyProfile`, `PROFILE_SCHEMA`, `PROFILE_VERSION`,
`ProfileError`) and the volume seam (`Volume`, `VolumeError`, `VolumeMap`,
`VolumeSource`). mesocosm-genet uses `BodyDependencyRevision`, `BodyMesh`,
`ContentPack`, `ContentPack::generate`, `VolumeMap`, `mesh_body`;
paredros-client uses `MeshError`, `mesh_body`, `place_point`; isometer uses
`BodyDependencyRevision`, `Volume` and the `VolumeSource`/`VolumeMap` re-export.

### 1.3 The mesocosm-core back-dependency, site by site

This is the heart of the problem: a neutral family cannot depend on Mesocosm's
core. Non-test, non-example sites only — examples are dealt with in §4 step 1.

| Crate and site | Types named |
| --- | --- |
| `mesocosm-lens/src/body.rs:15-18` | `BodyDocument`, `Part`, `PartId`, `Provenance`, `VolumeRef`, `snapshot::{encode, hash_bytes}` |
| `mesocosm-lens/src/bricks.rs:16` | `places::{BRICK, Ground}` |
| `mesocosm-lens/src/tracer/counters.rs:5` | `places::{Ground, Places}` |
| `mesocosm-lens/src/maps.rs:16` | `Places`, `Rng` |
| `mesocosm-render/src/live_body.rs:19-20` | `Yaw`, `PartId`, `VolumeRef` |
| `mesocosm-render/src/live_body/materials.rs:11` | `PartId`, `process::Process` |
| `mesocosm-render/src/live_body/pose.rs:10` | `Yaw` |
| `mesocosm-render/src/live_body/query.rs:10` | `PartId` |
| `mesocosm-mesh/src/lib.rs:50,93` | `BodyDocument`, `PartId`, `Provenance`, `VolumeRef`, `Yaw`; **`OrganismId`** in `MeshError::EmptyBodyProjection` |
| `mesocosm-mesh/src/live.rs:17,220-221` | `BodyDocument`, **`OrganismId`**, `Provenance`, `VolumeRef`, `Yaw`, `Origin` |
| `mesocosm-mesh/src/flatten.rs:20,159` | `BodyDocument`, `Yaw`, `PartId` |
| `mesocosm-mesh/src/content.rs:23` | `PartPalette`, `PartTemplate`, `Role`, `RoleShapes`, `VolumeRef`, `classify` |
| `mesocosm-mesh/src/profile.rs:51` | `BodyDocument`, `PartOrigin`, `wire` |
| `mesocosm-mesh/src/volume.rs:17` | `VolumeRef` |

Each type, its meaning, and where it should land:

| Type | Defined | Neutral? | Lands |
| --- | --- | --- | --- |
| `PartId(u32)` | `core/src/body.rs:23` | Yes — an index into a body, no organism in it | `isometer-core` |
| `VolumeRef([u8;32])` | `body.rs:32` | Yes — a content address of voxels | `isometer-core` |
| `Yaw` | `body.rs:46` | Yes — a quarter turn | `isometer-core` |
| `Part` | `body.rs:136` | Yes — geometry plus attachment plus provenance | `isometer-core` |
| `Aabb` | `body.rs:164` | Yes | `isometer-core` |
| `BodyDocument` | `body.rs:211` | Yes — already the wing's shared organ; Paredros stores one per subject with no Mesocosm world anywhere | `isometer-core` |
| `Provenance`, `Origin` | `body.rs:106,94` | Yes — lineage of a *part*, not of an organism | `isometer-core` |
| `PartOrigin` | `chronicle` re-export | Yes — a part's authored/incorporated flag on the wire profile | `isometer-core` |
| `places::Ground`, `places::BRICK` | `places/bricks.rs:68` | Yes — a sparse voxel field with revisions. Both products already share it (`paredros-world/src/world.rs:7,157`) | `isometer-core` |
| `snapshot::{encode, hash_bytes}`, `wire` | `core/src/snapshot.rs`, `wire.rs` | Yes — postcard framing and blake3 hashing, no world semantics | `isometer-core` |
| `Role`, `classify` | `plan.rs:82,110` | Yes — shape classification from a half-extent | `isometer-core` |
| `PartTemplate`, `RoleShapes`, `PartPalette` | `development.rs:39,59,117` | **Mixed** — the vocabulary is neutral, the defaults are Mesocosm's development model | Mesocosm-owned adapter: `isometer-mesh::ContentPack::generate` takes them as arguments; see §4 step 3 |
| `process::Process` | `process.rs:64` (`flow.rs:150` is a second, unrelated `Process`) | **No** — metabolic vocabulary | Stays in mesocosm-core; `PartMaterial` takes a neutral material code and Mesocosm's `section/materials.rs` maps |
| `OrganismId(u32)` | `organism.rs:44` | **No** — an organism identity | Stays; `mesocosm-mesh` stops naming it (§4 step 2) |
| `Places`, `Rng` | `places.rs:91`, `rng.rs:15` | **No** — Mesocosm's biome partition and its seeded RNG | Stays; `lens/maps.rs` leaves the family (§4 step 4), and `tracer/counters.rs:5` takes a caller-supplied biome slice |
| `World` | `world.rs` | **No** | Never named by any component today, and must not be |

Two of these are already inside isometer as well:
`shared/isometer/src` names `mesocosm_core::process::Process::Secrete` and
`mesocosm_core::effect_experiment::Glyph`, so the `Process` decision above and
a later `wing-glyphs` canon reach isometer too, not only render.

---

## 2. Target layout

```text
shared/isometer/                  one Cargo workspace, MPL-2.0, publish = false
  Cargo.toml                      facade package + [workspace] members
  src/                            the facade: SlabCamera, Scene, query, producer
  crates/isometer-core/           body document, part identity, voxel address,
                                  yaw, provenance, Ground, postcard/blake3 seam
  crates/isometer-mesh/           ← mesocosm-mesh, plus isometry-voxel (L2)
  crates/isometer-render/         ← mesocosm-render
  crates/isometer-lens/           ← mesocosm-lens
```

- **Umbrella vs facade.** `isometer` stays the package at the workspace root
  (`shared/isometer/Cargo.toml` already declares `[workspace] members = ["."]`
  at `:13-15`), widened to `members = [".", "crates/*"]`. It is both: the
  workspace root and a facade that re-exports `isometer::{core, lens, mesh,
  render}`, so a product takes one dependency and one version. Keeping the
  facade at the root rather than moving it to `crates/isometer/` leaves
  `mesocosm-genet/Cargo.toml:56`'s path untouched — the smaller reversible step.
- **Component names.** `isometer-lens`, `isometer-render`, `isometer-mesh`,
  `isometer-core`. "lens", "render" and "mesh" are kept as the plain working
  words; the prefix is what carries the family, and no collision forces a
  change. Checked read-only against the crates.io search API on 2026-09-14:
  the query `isometer` returns **zero** crates in total, so all four names and
  the umbrella are free.
- **Licence.** MPL-2.0 throughout, as all four crates carry today
  (`mesocosm-lens/Cargo.toml:5`, `mesocosm-render:5`, `mesocosm-mesh:5`,
  `shared/isometer/Cargo.toml:5`), and as Mesocosm's licensing boundary rules
  for a promoted reusable library. **`isometry-voxel` is the exception and a
  decision for Mark**: it inherits the Isometry root workspace's
  `license = "MIT OR Apache-2.0"` (root `Cargo.toml:38`), so folding it into
  `isometer-mesh` relicenses it. See §3 and §5 risk 5.
- **Resolution, today.** Every product resolves the family by path inside this
  one repository: Mesocosm through `mesocosm/Cargo.toml`'s
  `[workspace.dependencies]` rows, Paredros through
  `paredros-client/Cargo.toml:41-43`, and Isometry's board through a path into
  `shared/isometer` once it exists. The family keeps its own `[workspace]` and
  its own restated `[patch.crates-io]` and stays in the root workspace's
  `exclude` list (root `Cargo.toml:13`) — **because the root pins mere at
  `fb7e136b` while mesocosm, paredros and isometer pin `4f4de1d0`**
  (`mesocosm/Cargo.toml:34-74`, root `Cargo.toml:63-105`). That divergence is
  the family's real blocker for Isometry, not the crate names. See §5 risk 4.
- **What publication would need.** All four are `publish = false` today. A real
  publish needs one lockstep version across the four; `wing-formats` published
  or vendored, since `isometer-mesh` takes it by relative path
  (`mesocosm-mesh/Cargo.toml:15`); the `conatus`, `modulus` and
  `cambium-rootstock` git pins turned into versions, which is mere's decision
  and not this plan's; and a `version =` beside every `path =`.

---

## 3. The merged appearance crate (presentation plan lane L2)

The presentation plan's L2 (`2026-09-11_orthographic_voxel_presentation_plan.md:263-280`)
owes a merge of `isometry-voxel` (recipe, palette, `bake_facing`, `Sheet`,
`load_vox`, `Voxels`; `crates/isometry-voxel/src/lib.rs:27-31`) and
`mesocosm-mesh` into one wing-neutral crate emitting either a sprite sheet or
a face list from the same volume. Its open decision reads "where the merged
appearance crate lives: the Isometry root workspace, or mere as a platform
organ once a non-wing consumer appears", and it explicitly excludes the scene
crate, which L9 already ruled (`:1413-1417`).

**Resolution under the family: mesh *is* that crate.** `isometer-mesh` already
owns the face-list half (`Quad`, `PartMesh`, `mesh_volume`, `flatten`) and the
volume seam (`Volume`, `VolumeMap`, `VolumeSource`); `isometry-voxel` owns the
sprite half over the same voxel data. Merging them anywhere else would leave
two crates that both mesh a `Volume`. The answer to "the Isometry root
workspace, or mere" is **neither**: `shared/isometer/crates/isometer-mesh`.
It satisfies L2's own done condition — "the crate has no engine or wgpu
dependency" — which mesh already meets by construction
(`mesocosm-mesh/Cargo.toml:10-12`) and which is why mesh, not render, is the
right home. Mere remains available later if a non-wing consumer appears; that
was always the second half of the decision, and nothing here forecloses it.

Two consequences. `dot_vox` (`isometry-voxel/Cargo.toml:14`) becomes a family
dependency and should be feature-gated (`vox = ["dep:dot_vox"]`, default off).
And `isometry-voxel::BodyProfile`/`BODY_SCHEMA`/`PartOrigin`
(`isometry-voxel/src/lib.rs:28`) collide by name with
`mesocosm_mesh::BodyProfile`/`PROFILE_SCHEMA` (`mesocosm-mesh/src/lib.rs:62`);
they are two encodings of the same idea, and picking one is a schema change
and therefore a receipt change.

---

## 4. Move order

Preconditions, all outside this plan: the mesocosm-lens ground-hit quantiser
floor fix lands (uncommitted in `src/tracer.wgsl` and `tracer_tests.rs` as of
this writing); the mesquite and taproot pin bumps land in mesocosm and
paredros; and the extraction plan's step 8, Paredros's P1 retarget onto
`isometer::SceneProducer`, lands. Nothing below starts while `tracer.wgsl` or
`paredros-client/src/producer/` is dirty.

Each step ends green in **all three** product workspaces:
`cargo check --workspace --all-features --all-targets` and `cargo test` in
`mesocosm/`, `paredros/` and the Isometry root, plus `cargo test` in
`shared/isometer`. Steps 1-5 move no crate and are each revertible by one
`git revert`.

1. **Widen the family workspace.** `shared/isometer/Cargo.toml` gains
   `members = [".", "crates/*"]` and an empty `crates/`. No crate exists there
   yet; nothing else changes. *Also in this step:* move the four
   `mesocosm-lens` examples that name `World`
   (`burrow_run.rs:15`, `burrow_watch.rs:40`, `g4_frame/*`, `v2_projection.rs`)
   and the two `mesocosm-runtime` dev-dependencies
   (`mesocosm-lens/Cargo.toml:37`, `mesocosm-render/Cargo.toml:23`) into
   `mesocosm-genet`'s `examples/`, since a component cannot keep a dev-dep on a
   product runtime. This is the largest single file move in the plan and is
   deliberately first, while nothing has renamed.
2. **Sever `OrganismId` from mesh.** `LiveBodyProjector::project_body` already
   exists (added by extraction step 1, `mesocosm-mesh/src/live.rs:121`).
   Retire `project` (`:92`), `LiveBodyProjection::organism` (`:43`) and
   `MeshError::EmptyBodyProjection` (`lib.rs:93`); mesocosm-genet stamps its
   own `OrganismId` on the result. Done when `grep OrganismId` over
   `mesocosm-mesh/src` returns nothing outside tests.
3. **Sever the development vocabulary from mesh.** `content.rs:23`'s
   `PartPalette`, `PartTemplate` and `RoleShapes` become parameters of
   `ContentPack::generate`; `Role` and `classify` move to `isometer-core` in
   step 6. Mesocosm's callers pass what they pass today. Done when
   `content.rs` names only `VolumeRef`, `Role` and `classify`.
4. **Sever `Places`/`Rng` from lens.** `src/maps.rs` (the probe biome
   synthesiser, no caller outside the crate's own examples) moves to
   `mesocosm-genet`; `tracer/counters.rs:5` takes the ground and a caller-built
   biome slice instead of `Places`. Done when `grep 'Places\|Rng'` over
   `mesocosm-lens/src` returns nothing outside tests.
5. **Sever `Process` from render.** `live_body/materials.rs:11`'s
   `PartMaterial` takes a neutral `material: u8` (the grammar
   `isometer-mesh::MATERIAL_*` already defines) plus the appearance fields it
   already carries; Mesocosm's `section/materials.rs` maps
   `process::Process` to it. The same edit reaches
   `shared/isometer/src/bodies.rs`, which names `Process::Secrete` directly.
   **This step can move pixels** and is gated on the spatial-coverage capture
   comparison used throughout the extraction plan.
6. **Found `isometer-core`.** `git mv` `core/src/body.rs`,
   `core/src/places/bricks.rs`, `core/src/snapshot.rs`'s `encode`/`hash_bytes`,
   `core/src/wire.rs`, and `Role`/`classify` from `core/src/plan.rs` into
   `shared/isometer/crates/isometer-core`. `mesocosm-core` gains a dependency
   on it and **re-exports every moved item at its current path**, so
   `mesocosm_core::BodyDocument` and `mesocosm_core::places::Ground` still
   resolve for `mesocosm-runtime`, `mesocosm-views`, `paredros-world`,
   `paredros-social`, `paredros-sortie` and `wing-integration` — none of those
   change a line. Done when the state hash, every snapshot receipt and
   `BodyProfile`'s `PROFILE_SCHEMA` bytes are unchanged (§6).
7. **Rename and move mesh.** `git mv mesocosm/crates/mesocosm-mesh
   shared/isometer/crates/isometer-mesh`, package renamed, imports rewritten
   `mesocosm_core::` → `isometer_core::` for the moved types. Path rows updated
   in `mesocosm/Cargo.toml:90`, `paredros-client/Cargo.toml:42`,
   `wing-integration/Cargo.toml:15`, `shared/isometer/Cargo.toml:23` and
   `mesocosm-render/Cargo.toml:14`. **Source is otherwise verbatim.**
8. **Rename and move render.** Same shape; `mesocosm/Cargo.toml:92`,
   `paredros-client/Cargo.toml:43`, `shared/isometer/Cargo.toml:27`. Verbatim.
9. **Rename and move lens.** Same shape; `mesocosm/Cargo.toml:89`,
   `paredros-client/Cargo.toml:41`, `shared/isometer/Cargo.toml:25`. Verbatim,
   minus what steps 1 and 4 already removed. `mesocosm-lens`'s `conatus`,
   `modulus` and optional `burn` rows come with it and are restated against the
   family's own mere pin.
10. **Facade.** `shared/isometer/src/lib.rs` gains
    `pub use isometer_{core,lens,mesh,render} as {core,lens,mesh,render}`.
    Products may then drop their individual path rows and take `isometer`
    alone; Mesocosm and Paredros do so in this step, which is where
    `paredros-client`'s `r1-proof` optional-lens gate retires for good.
11. **L2: merge `isometry-voxel` into `isometer-mesh`.** Recipe, palette,
    `bake_facing`, `bake_strip`, `Sheet`, `Voxels`, `Rgb` and the `vox` ingest
    behind a default-off feature; the two `BodyProfile` encodings reconciled to
    one. `crates/isometry-voxel` is retired from the Isometry root workspace.
    Gated on the licence decision in §5 risk 5.

**Renames verbatim:** lens, render, mesh — package name, directory and import
prefix only.
**Splits:** `mesocosm-core` (loses `body.rs` 629 lines, `places/bricks.rs` 470,
the `encode`/`hash_bytes` seam, `wire.rs`, and `Role`/`classify` from
`plan.rs` 292; gains a dependency on `isometer-core` and a re-export block, and
gains nothing else); `mesocosm-mesh`'s `content.rs` (loses the development
defaults to its callers); `isometry-voxel` (dissolves into `isometer-mesh`).
**Stays:** `World`, `OrganismId`, `Organism`, `process::Process`, `flow`,
`phenotype`, `Places`, `Rng`, `rules`, `history`, `chronicle`, `score`,
`record` — all of mesocosm-core's world model.

---

## 5. Risks and unknowns

1. **Hash and replay stability is the whole gate on step 6.** `state_hash`
   (`mesocosm-core/src/snapshot.rs:116`) hashes the world, bodies included;
   `mesocosm-mesh`'s `BodyProfile` carries `PROFILE_SCHEMA`/`PROFILE_VERSION`
   (`lib.rs:62`) over `mesocosm_core::wire`; `VolumeRef` is a blake3 content
   address (`body.rs:32`); `BodyRevision` in lens hashes an encoded document
   (`lens/src/body.rs:15-18`). Moving `BodyDocument` and `Ground` across a
   crate boundary is byte-neutral **only if** field order, field names and
   derive order are untouched — postcard is positional but serde's derive is
   not blind to attribute changes, and a stray `#[serde(rename)]` or reordered
   variant silently rewrites every saved world. Mesocosm already carries a
   known fixture drift here: `structure-cli.scenario` fails at HEAD on a
   generated-start hash (`50e8f3a3a6b6d22d` expected, `8d1d3676ecf24452` got),
   recorded in the extraction plan's step-5 progress note. That must be
   resolved or explicitly excluded *before* step 6, or it will mask a real
   regression.
2. **The lens tracer is under a live edit.** `mesocosm-lens/src/tracer.wgsl`
   and `tracer_tests.rs` are uncommitted right now: the ground-hit quantiser
   floor, a 24-line change that lifts any graded ground colour under the first
   rung. Step 9 `git mv`s that file. A rename on top of an unlanded WGSL edit
   is the cheapest possible way to lose a shader fix, and the fix itself
   changes ground pixels, so every capture receipt this plan compares against
   must be regenerated after it lands and before step 1.
3. **Paredros's P1 retarget is in flight against isometer.** Extraction step 8
   deletes `paredros-client/src/producer/{camera,bodies,scene}.rs` and
   re-points eight GPU tests at `SceneProducer`. Steps 7-10 here rewrite the
   same manifest's rows `:41-43`. Running them concurrently churns Paredros
   twice and makes any failure ambiguous between the two lanes. Sequence, do
   not overlap.
4. **Isometry is a third consumer that does not exist.** No Isometry crate
   depends on any of the four today. Two things stand between it and the
   family: the mere revision split (root `fb7e136b` vs family `4f4de1d0`),
   which is why `shared/isometer` is in the root's `exclude` list
   (`Cargo.toml:13`) and which no crate rename fixes; and Mesocosm's own rule
   that a portable profile is extracted after two real consumers, never
   declared in advance. **This plan has two consumers, not three.** Whether
   that is enough to found the family now, or whether it waits on Isometry's
   board, is Mark's call — the family shape is defensible either way, and the
   mere-pin alignment is separately worth doing.
5. **Licence, and it is a real one.** `isometry-voxel` is `MIT OR Apache-2.0`
   by workspace inheritance (`crates/isometry-voxel/Cargo.toml:5`, root
   `Cargo.toml:38`); the family is MPL-2.0. Step 11 folds permissively
   licensed code into a copyleft crate, which is allowed for the copyright
   holder but is a one-way door for that code's downstream availability.
   Mark decides before step 11 whether `isometer-mesh` stays MPL-2.0 and
   absorbs it, or whether the bake half stays a separate MIT/Apache crate.
6. **crates.io is checked but not claimed.** The 2026-09-14 read-only query
   returns zero crates for `isometer`, so nothing is taken — but nothing is
   held either, and the naming ledger's rule is that a name is claimed by a
   real publish. All four crates are `publish = false`; the names are free and
   unbanked until that changes.
7. **`wing-formats` by relative path.** `isometer-mesh` keeps
   `{ path = "../../../shared/wing-formats" }` after the move, a path that
   leaves the family directory. Harmless in this repository, fatal to a
   publish, and it means the family is not self-contained on disk.
8. **Unknown: how much of lens actually belongs.** `Lens`, `LensScene`,
   `maps` and `critter::wander` have no consumer outside the crate. Lens is
   the largest component and roughly a third of its public surface is the
   heightfield-march probe that the brick tracer superseded. Whether the family
   takes that dead lane along or the move is also a retirement is not settled
   here, and settling it would shrink every step from 9 onward.

---

## 6. Done conditions, as tests and receipts

1. **No component depends on mesocosm-core.** *Must be written:* a check in
   the family's CI lane asserting `cargo tree -p isometer-lens`,
   `-p isometer-render`, `-p isometer-mesh` and `-p isometer-core` name none of
   `mesocosm-core`, `mesocosm-runtime`, `mesocosm-genet`, `mesocosm-views`,
   `paredros-world`, `paredros-identity`, `paredros-client` or `isometry-core`.
   This generalises the extraction plan's condition 1, which asserted the same
   list for `isometer` alone.
2. **The hash and the wire are unmoved.** *Exists:* `state_hash`
   (`mesocosm-core/src/snapshot.rs:116`) and its suite; `PROFILE_SCHEMA` /
   `PROFILE_VERSION` round trips (`mesocosm-mesh/src/profile.rs`);
   `mesocosm-mesh`'s 62 tests; Mesocosm's bench scenarios under
   `mesocosm/testing/bench/`. *Must be written:* one test that encodes a fixed
   `BodyDocument` and a fixed `Ground` and asserts the exact postcard byte
   string and blake3 digest, committed **before** step 6 so the move is
   compared against a pin rather than against itself.
3. **Mesocosm builds against the family with unchanged receipts.** *Exists:*
   mesocosm-genet's 150 release tests, the `acceptance` (167 frames),
   `spatial` (207), `spatial-coverage` (336 frames / 32 captures),
   `world-trial`, `proportions` and `uptake-world` scenarios; the 2026-09-13
   capture receipt at `Code/testing/spatial-coverage-2026-09-13/` that the
   extraction plan compared all 32 viewports against byte-for-byte.
   *Receipt:* the same 32 viewports byte-identical after step 10, against a
   baseline regenerated after risk 2's shader fix lands.
4. **paredros-client builds against the family with unchanged receipts.**
   *Exists:* 45 client, 132 world and 6 native tests, the session smoke, and
   the eight GPU tests in `producer/tests.rs`. *Receipt:* all green after step
   10 with `paredros-client/Cargo.toml` naming `isometer` and nothing else from
   the family, and with the `r1-proof` optional-lens feature deleted.
5. **Isometry builds against the family.** *Must be written:* the Isometry
   root workspace takes `isometer` by path and its existing suite stays green —
   which requires the mere-pin alignment in risk 4 first. Until that lands this
   condition is **blocked, not failed**, and the lane says so rather than
   quietly dropping the third consumer.
6. **L2 closed.** *Exists:* `isometry-voxel`'s own suite and Isometry's tileset
   bake. *Must be written:* a pixel-identical sheet comparison and a
   quad-identical mesh comparison against today's outputs, per L2's own done
   condition, plus a `cargo tree -p isometer-mesh` assertion that no wgpu and
   no engine crate appears.
7. **One version of each component in every product graph.** *Must be
   written:* a check that `cargo tree -d` in each of the three workspaces
   reports no duplicate of `isometer-core`, `isometer-mesh`, `isometer-render`,
   `isometer-lens` or `wgpu`.

## Findings

- 2026-09-14: the three components form a chain, not a cycle — mesh below
  render, lens beside both, and **no component depends on isometer**
  (`mesocosm-render/Cargo.toml:14`, `mesocosm-lens/Cargo.toml:34-35` is a
  dev-dependency only). The family shape needs no inversion.
- 2026-09-14: no Isometry crate depends on lens, render, mesh or isometer
  today; the only mention in the root manifest is the `exclude` line
  (`Cargo.toml:13`).
- 2026-09-14: the root workspace pins mere at `fb7e136b` while mesocosm,
  paredros and isometer pin `4f4de1d0`. This, not naming, is what keeps
  Isometry out of the family.
- 2026-09-14: `isometry-voxel` is `MIT OR Apache-2.0` by root workspace
  inheritance while every family crate is MPL-2.0, so L2 is a relicensing
  decision as well as a merge.
- 2026-09-14: `mesocosm-lens`'s `Lens`, `LensScene`, `maps` and
  `critter::wander` have no caller outside the crate; lens and render both keep
  `mesocosm-runtime` as a dev-dependency (`:37`, `:23`).
- 2026-09-14: crates.io returns zero crates for the query `isometer`, so the
  umbrella and all four component names are free; `isomere` returns two
  unrelated chemistry crates.
- 2026-09-14, later: **the mere revision split is resolved.** The umbrella
  push e1f5c7c repinned the root workspace to one revision per family
  (mere 3675a352, genet 7baa554c, netrender 3961aca9, cleromancy 3617583b),
  and isometer's cambium-rootstock line moved to mere 3675a352 in the same
  series (419f6c9, f35661f). The root, mesocosm, paredros and isometer now
  share one mere revision, so the risk recorded above at `fb7e136b` against
  `4f4de1d0` no longer holds. isometer stays in the root's `exclude` list
  because it carries its own `[workspace]`, which is a layout choice this
  plan's move order can revisit, not a resolution constraint; "Isometry
  builds against the family" is unblocked at the pin level and waits only
  on a consumer in the tabletop crates.
- 2026-09-14, later: **isometry-voxel is MPL-2.0 (Mark's ruling).** The
  crate no longer inherits the root workspace's MIT OR Apache-2.0; its
  manifest declares MPL-2.0, its sources carry the MPL header block the
  Mesocosm crates use, and the MPL text sits beside it. The licence risk
  to lane L2's fold into isometer-mesh is closed.
- 2026-09-14, ruled (Mark, relayed by the Paredros session as he said it
  would be): **shape 1, the physical split with mesocosm-core
  re-exporting.** `shared/isometer/crates/isometer-core` takes body.rs,
  places/bricks.rs (`Ground`, `BRICK`), snapshot.rs's seam, wire.rs, `Role`
  and `classify` from plan.rs, and `SpeciesId`, added from the import scan
  because it rides on every `BodyDocument` and both products import it.
  mesocosm-core depends on isometer-core and re-exports every moved item at
  its current path; lens, render, mesh and isometer switch to isometer-core
  and drop mesocosm-core. Two named preconditions: isometer's own
  `process::Process::Secrete` and `effect_experiment::Glyph` references are
  cut first (step 5 and the wing-glyphs canon), or isometer keeps the
  back-dependency. Gate: Mesocosm's world hash and Paredros's v3 to v6
  archive restores byte-identical across the move. Generation (`Places`,
  `Grown`, `Rng`) stays Mesocosm's; Paredros keeps mesocosm-core for that
  one seam and takes body and ground from isometer-core, a product-side
  change the Paredros session makes when step 6 lands. Sequence: steps 1 to
  5, then the split. The heightfield-march lane and the chain critter are
  kept as unconsumed components pending Mark's word, since the march is the
  large-scale lens and the chain a locomotion model an airborne critter
  would use.

## Progress

- **2026-09-14:** assessed and written. No code moved, no commit.
- **2026-09-14, lane started** on the ruling above; step 1 running.
- **2026-09-14, step 1 done.** `shared/isometer` is a workspace with
  `members = [".", "crates/*"]` and an empty `crates/`. Twenty-three
  example files moved verbatim by rename from mesocosm-lens to
  mesocosm-genet: the four the plan named (burrow_run, burrow_watch,
  g4_frame, v2_projection), t1_picking (the sole consumer of lens's
  mesocosm-runtime dev-dependency), and, as the one deviation, v1_frame
  and g2_frame, whose `app.rs` and `gpu.rs` the moved examples include by
  path; moving the family beat a cross-crate path include or a duplicate.
  The mesocosm-runtime dev-dependency rows are gone from lens and render
  (render's was already dead); mesocosm-genet gains a postcard
  dev-dependency; `cargo tree` shows no runtime or genet edge under lens,
  render or mesh. Green: mesocosm workspace check, lens 60 single-threaded,
  render 32 + 5, genet 150, all seven moved examples build, isometer 28,
  paredros check, root check with Cargo.lock unchanged. burrow_run and
  v2_projection panic on world-fixture assertions identically before and
  after the move (positive control run in place), the same drift risk 1
  records for structure-cli; not chased.
- **2026-09-14, step 4 done (run ahead of steps 2 and 3, which wait on
  another session's live edits in mesocosm-genet's section files).** The
  probe biome synthesiser left the lens for `mesocosm-genet/src/maps.rs`
  (168 lines) with its four tests and the three examples that call it
  (flyover, lope, menagerie, by rename); `BiomeMaps`, the renderer's input
  type, stays at `mesocosm_lens::maps::BiomeMaps` so no consumer moved a
  line, and the lens's five in-crate test call sites use a 55-line
  test-only `maps::probe` fixture instead of a second copy of the
  generator. `tracer/counters.rs` needed nothing: it is a single
  `#[cfg(test)]` function using the same `Places::grown` fixture as four
  other lens test modules, and the done condition exempts tests. No
  production `Places` or `Rng` site remains in mesocosm-lens; its
  mesocosm-core dependency stays for the types step 6 moves. Green:
  mesocosm workspace check, lens 56 single-threaded, genet 154, both
  example sets build, isometer 28, paredros check, root check with both
  locks byte-identical.
- **2026-09-14, step 2 done.** `OrganismId` is out of mesocosm-mesh's
  production code: `LiveBodyProjector::project`, `LiveBodyProjection` and
  `MeshError::EmptyBodyProjection` are gone (the struct outright, since
  nothing constructed it once `project` left), every caller uses
  `project_body`, and Mesocosm's one product caller,
  section/materials/capture.rs, names the organism itself on failure.
  isometer's own error receipt already reconstructed the subject id, so
  every receipt is byte-identical by construction. Mesh still names, in
  production, `PartPalette`/`PartTemplate`/`RoleShapes` (step 3) and
  `PartOrigin` from wire (step 6's list), plus the move-list types; its
  tests name `Attachment`, `Founding`, `Recipe`, `Soma`, `develop_body`,
  `Kingdom`, `Organism`, `Stage`, which step 6 must carry or rewrite.
  Green: mesocosm check, mesh 60 (two comparison tests folded into their
  survivors), genet 154, runtime 44, views 46, isometer 28, paredros check,
  root check, all four locks unchanged. Note for later steps:
  mesocosm-genet carries pre-existing rustfmt drift in seven files; a
  `cargo fmt` there is not a no-op and must not ride into a family commit.
- **2026-09-14, step 3 done.** mesocosm-mesh's `content.rs` names from
  mesocosm-core only `Role`, `VolumeRef` and `classify`. The plan's
  "parameters of `generate`" became a trait: `PartPalette` is a serialized
  field of `ContentPack`, so `ContentPack<P: Palette>` with a two-method
  mesh-local `Palette` (`admitted`, `admit`) and a `Shape` view; the impl
  lives on a `#[serde(transparent)]` newtype `DevelopmentPalette` in
  mesocosm-genet's `generation_content.rs` because the orphan rule forbids
  the impl in either crate that owns a side, and `mesocosm-core` cannot
  depend on mesh. Persisted packs, replay traces and saved specimens are
  byte-unchanged; `generate-start` output is byte-identical before and
  after, and the pinned sensor content address holds. Mesh's tests keep a
  test-only impl on `PartPalette` so fixtures stay literal. Every
  mesocosm-core item mesh still names outside tests is on step 6's list:
  `BodyDocument`, `PartId`, `PartOrigin`, `Provenance`, `Origin`, `Role`,
  `VolumeRef`, `Yaw`, `classify`, `wire`. Green: mesocosm check, mesh 60,
  genet 154, runtime 44, views 46, isometer 28, paredros check, root
  check, all four locks unchanged.
- **2026-09-14, step 5 done, gate passed.** `PartMaterial.material` is a
  neutral tissue-channel index (`u8`, `TISSUE_CHANNELS = 5`) and
  mesocosm-render names `process::Process` nowhere, tests included;
  Mesocosm's `section/materials.rs` maps `Process` to the channel from
  `Process::ALL`'s position so the shader order holds by construction;
  isometer's `Process::Secrete` filter is gone and `secretory_parts` is
  host-written through `SceneHost::prepared` beside `carcasses`, with
  `BodyFrameStats`'s serde shape unchanged because the played receipt
  serializes it verbatim. The shader changed in comments and local names
  only. The channel values are not mesh's `MATERIAL_*` palette codes,
  which sit on a different axis. Gate: all 32 spatial-coverage captures
  byte-identical to the 2026-09-14 reference over the whole frame, and an
  inverted-channel control moved 2.2 million viewport pixels, so the
  comparison is sensitive to exactly the ordering preserved. Green:
  mesocosm check, render 32 + 5, genet 154, runtime 44, views 46,
  isometer 28, paredros check (it names neither type), root check, locks
  unchanged. population.scenario fails identically at HEAD; not chased.
  Precondition one of the ruling is met: isometer names no
  `mesocosm_core::process` item.
- **2026-09-14, step 6a done: the split's two preconditions.** isometer's
  glyph renderer has its own `Stroke { Quotes, Slashes, Backticks }`,
  bit-identical strokes, and names nothing from `effect_experiment` or
  `process`; Mesocosm maps `effect_experiment::Glyph` to it in one
  function in section.rs, and the saved spatial request still serializes
  the Mesocosm type unchanged. Everything isometer still names from
  mesocosm-core is on step 6's list, plus `Attachment` in a test fixture,
  which body.rs carries anyway; the move list should name `Attachment`,
  `AttachError` and `BodyPlan` explicitly. The byte-pin test lives at
  mesocosm-core/tests/move_pins.rs (six tests): `state_hash` of
  `World::new(0, 0)` at tick 0 = `df397e7c183eec55`; `PROFILE_SCHEMA`
  `mesocosm.body/v0`, version 0, magic `MESOBODY`, header 10; a fixture
  body document's 105 postcard bytes and its framed wire bytes as literal
  arrays; the lens body revision `f4abe81ee0b320a4`; `VolumeRef` as 32
  raw bytes. The blake3 content-address pin already exists literally in
  mesocosm-mesh's content tests and is cited, not copied. **Finding:**
  the glyph journey receipt (`testing/glyphs/journey.json`, committed
  ac38887) records baseline `765c055b377dfb62`, but the same example at
  e088d7e prints `df397e7c183eec55` and a different final hash; the
  simulation moved between those commits, not this lane, and that
  receipt is stale in the way risk 1 describes. Gate: all 32
  spatial-coverage captures identical to the 2026-09-14 set over the
  whole frame; spatial, world-trial and uptake-world pass. Green:
  mesocosm check, core 558 plus the six pins, genet 154, views 46,
  isometer 28, paredros check, root check, locks unchanged.
- **2026-09-14, step 6 done, every gate held.** `shared/isometer/crates/
  isometer-core` exists (MPL-2.0; body.rs 491 with body/tests.rs 191,
  anatomy.rs 249, plan.rs 291, ground.rs 495, snapshot.rs 73, wire.rs 12,
  lib.rs 48; 32 tests; dependencies wing-formats, serde, postcard only;
  `cargo tree` names no product crate). mesocosm-core depends on it and
  re-exports every moved item at its old path, so mesocosm-runtime,
  mesocosm-views, mesocosm-genet, paredros-world, paredros-social,
  paredros-sortie and wing-integration compile unchanged; paredros-world's
  tests pass with no Paredros edit. lens, render, mesh and isometer take
  isometer-core; isometer names mesocosm-core nowhere; the three
  components keep it as a dev-dependency only, for the generation
  fixtures the ruling leaves Mesocosm's. Deviations: plan.rs moved whole
  because `BodyDocument.plan` is a `BodyPlan` over `Role`, `Facing` and
  `Symmetry`; anatomy.rs moved because body.rs calls `living()`;
  bricks.rs split, with a neutral `Terrain` trait and `Cavity` in
  isometer-core so `Ground::grow` keeps its inherent form and Mesocosm's
  `Grown` implements it beside `nest_entry` in a new
  mesocosm-core/src/places/bricks.rs; `SnapshotError` stayed and a
  two-variant `CodecError` carries the codec seam; five inherent impls on
  now-foreign types became extension traits (`ShapeProcesses`,
  `BodyProcesses`, `BodyOrgans`) with one `use` line in 21 mesocosm-core
  files and no consumer change; one anatomy test folds over extent
  instead of `reach`, whose assertion lives in process tests. Gates: the
  six pins unchanged; the sensor content address; all 32 spatial-coverage
  captures identical to the 2026-09-14 set and to a baseline captured at
  a736009; acceptance's 15 captures and every non-timing receipt field
  identical to that baseline. Green everywhere: isometer 28, isometer-core
  32, core 535 plus suites, mesh 60, render 32 + 5, lens 56, runtime 44,
  views 46, genet 154, paredros check and world tests, root check with the
  root lock unchanged. wing-integration's tracked lock changed: isometer-
  core entered its graph, and two stale mere lines refreshed to 3675a352,
  drift left by the umbrella repin, recorded rather than reverted.
- **2026-09-14, step 6 gate closed on the Paredros side.** At 5dfc721,
  with no Paredros edit: paredros-world 143 tests including motion_compat's
  real v3 and v4 archive restores and the v5 and v6 save-restore tests,
  paredros-client 45 lib tests with the eight GPU receipts, session
  acceptance ok in 106 frames; byte-identity holds on every restored
  fixture (Paredros's state hash is not versioned by its save header, so
  the restores are the pin). Steps 7 to 9 are pre-agreed with the Paredros
  session: this lane changes only the three mesocosm-lens, -render and
  -mesh path rows in paredros-client/Cargo.toml, one per step, runs the
  Paredros workspace check before each commit, and touches nothing else
  under paredros/. Their product-side switch to isometer-core runs in
  parallel in paredros-world and paredros-client source and dependency
  rows, committed by pathspec on their side.
- **2026-09-15, step 7 done.** mesocosm-mesh is `shared/isometer/crates/
  isometer-mesh`, moved by rename with source verbatim beyond the import
  prefix; its manifest keeps mesocosm-core as a dev-dependency by path for
  the generation fixtures and restates no patch table, since the family
  workspace root carries it. The central `[workspace.dependencies]` row in
  mesocosm/Cargo.toml is `isometer-mesh`, so lens, render and genet
  renamed their keys too; shared/isometer and wing-integration point at
  the new path, and wing-integration's tracked lock renames the package
  block only. Paredros's one row is flipped in the working tree and
  committed by the Paredros session with its own manifest edits, since
  that file is dirty on their side. Gates: the sensor content-address pin
  in the moved crate, the six core pins, all 32 spatial-coverage captures
  and 15 acceptance captures byte-identical to the 2026-09-14 set and to
  step 6's run. Green: isometer 28, isometer-core 32, isometer-mesh 60 +
  4 + 2, mesocosm check, core suites, render 32 + 5, lens 56, runtime 44,
  genet 154, wing-integration check, root check with the root lock
  unchanged.
- **2026-09-15, steps 8 and 9 done, one commit.** mesocosm-render is
  `shared/isometer/crates/isometer-render` and mesocosm-lens is
  `shared/isometer/crates/isometer-lens`, both moved by rename with source
  verbatim beyond the import prefix; each manifest keeps mesocosm-core as
  a dev-dependency by path and expands the wgpu, serde and postcard rows
  to mesocosm's exact specs; lens restates conatus and modulus at the
  family's mere revision 3675a352, burn verbatim, and netrender at
  3961aca9 for its tests, and its five remaining examples came with it.
  mesocosm/crates now holds core, genet, phenotype, runtime and views;
  mesocosm's central rows point into the family, and the unused modulus
  row is dropped. Paredros's render and lens rows are flipped in the
  working tree for the Paredros session to commit with its manifest,
  together with its own import prefixes (mesh, render, lens). Gates: the
  six core pins; all 32 spatial-coverage captures byte-identical to the
  2026-09-14 set; acceptance and world-trial, which draws the traced
  ground, pass. Green: isometer 28, isometer-core 32, isometer-mesh 60 +
  4 + 2, isometer-render 32 + 5, isometer-lens 56 single-threaded,
  mesocosm check, runtime 44, genet 154, wing-integration check, root
  check with the root lock unchanged. A `paredros-client` feature still
  gates the optional lens (`r1-proof`); step 10 retires it on the
  Paredros side.

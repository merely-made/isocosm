# scenevm, read for the games wing

**Status, 2026-10-06:** read; ruled a donor of techniques only (ruling 622).
kiss3d stays the lit body tenant (471, 606).

**Subject.** `scenevm`, the renderer crate of
[markusmoenig/Eldiron](https://github.com/markusmoenig/Eldiron/tree/master/crates/scenevm),
read at `master` (`5fb0d1e888f6fcfa35dd3a15148237219a640018` when checked).
On crates.io it is 0.95.0 (2026-10-02), MIT; it has been released every few
weeks since 0.9.0 (2026-03-08), with 941 downloads, from one maintainer.

**How it was read.** Web pages only: `Cargo.toml`, the README, the `src/` and
`embedded/` listings, `src/lib.rs`, `src/vm.rs`, `src/vm/raster3d.rs`,
`embedded/3d_header.wgsl` and `embedded/3d_body.wgsl`, through a summarising
fetch. One read of `vm.rs` came back partial. One summary called the compute
pipelines placeholders; the shader files showed otherwise, and this brief
follows the shaders. "Not found" below means not found in what was read. No
code was copied.

Mark asked how it compares with kiss3d as a tenant, and how it composes with
the structure read from balaur (`2026-10-06_balaur_wing_brief.md`).

## 1. What it is

- **A layer renderer with two 3D paths.** One is a raster path
  (`src/vm/raster3d.rs`, about 450 lines of layout and targets). It has a
  shadow-mapped sun (9-tap PCF), up to 8 point lights, and an irradiance probe
  grid.
- **The other is a compute ray and path tracer.** `embedded/3d_body.wgsl`
  (about 1,200 lines, one `@compute` entry, `cs_main`) and `3d_header.wgsl`
  (about 1,050 lines) cover:
  - primary rays through a grid or BVH of triangles (`sv_trace_grid`,
    `trace_unified`);
  - traced shadows (`trace_shadow`);
  - `path_trace_indirect` with up to 5 bounces;
  - ambient occlusion (`compute_ao`).

  Output is a `texture_storage_2d<rgba8unorm>`. Each layer reads the one
  before it (`prev_layer`) and is composited in order.
- **Eldiron's scene model, not a general one:**
  - a tile atlas with a 256-entry palette;
  - tile animation in storage buffers (`tile_anims`, `tile_frames`);
  - per-texel material ids into a material table;
  - avatars;
  - per-face surface paint;
  - "organic" vegetation billboards.

  A 2D raster path and an optional UI layer sit beside it.
- **Shaders are embedded WGSL constants.** Despite the crate's "configurable
  compute shaders", no runtime shader replacement was found.
- **It owns its device.** It creates its own wgpu instance, adapter and
  device, shared through `static GLOBAL_GPU: OnceLock` (a `thread_local` on
  wasm). It renders into its own `Texture`, a winit or Metal surface, or CPU
  readback (`render_frame` downloads the pixels). No entry taking a caller's
  device or target was found.
- **It is app-shaped.** It is on wgpu **29.0.3**, and its defaults bring
  winit, rfd, fontdue, rust-embed, image and uuid.

## 2. Against kiss3d as the lit body tenant

kiss3d 0.46 is on wgpu 30 and BSD-3. It takes the host's device but keeps a
thread-local `Context` and wants a window (471's evidence).

To serve as a tenant, scenevm would need 471's seams and more:
- a wgpu 30 port, since two wgpu versions cannot share one device;
- the global device removed;
- rendering into caller targets;
- its own sun, lights and probes given up for the stack-owned light block
  (472);
- an input path for conatus's resident buffers instead of Eldiron's polys,
  tiles and chunks;
- depth for the join with the tracer's terrain (L3). Its compute path was
  seen writing colour only.

kiss3d's raster feature set is the broader one for bodies: PBR/IBL,
skinning and morphs, SSAO, OIT, transmission. So 471 stands.

## 3. Against balaur's structure

- **Render only observes.** scenevm is a retained renderer that would sit
  under the projection fed by the record's receipts (483). That composes.
- **The path tracer is Monte Carlo.** Its seed takes the frame counter
  (`U.anim_counter`), so two captures differ. Balaur's three render modes agree
  bit for bit, and genet's receipts compare capture digests. A traced capture
  would need seeding by pixel, bounce and sample index, not by frame. This
  concerns render captures only; the sim's determinism (604 to 607) is
  untouched.
- **Its 2D path would be a second 2D renderer.** The VTT renders tiles as
  genet DOM elements, and 476 keeps lit 2D on kiss3d behind the scene
  contract.

## 4. What is worth learning

Each item is a technique, taken by name under the MIT licence, with a licence
row, when a lane needs it.

1. **A small, capped probe grid for indirect light.**
   - The probes are built against a small CPU BVH of occluders
     (`IrradianceOcclusionTree`, binary, at most 1,024 triangles), with at
     most 64 light sources and cells of about 3 units, capped at 18×8×18
     probes.
   - Each probe holds one `vec3` of radiance.
   - Shading samples the probe at the point and one step along the normal
     (`sample_irradiance_grid`), which curbs light leaking through thin
     walls.
   - Where the bake runs was not traced.
   - *For the wing:* a probe grid could live in 472's light and environment
     block, filled by the voxel tracer instead of a triangle BVH, and read by
     both the tracer and the rasteriser. That would give bodies indirect light
     from the terrain without per-frame path tracing.
2. **Progressive accumulation for stills.** One sample per pixel per frame,
   5 bounces, and a running mean through the previous layer
   (`prev + (cur − prev) / n`), with no denoiser. That is cheap quality for a
   still camera: bench captures, a photo mode, the VTT's prepared maps. If its
   output is to serve as a receipt, seed it deterministically, as §3 says.
3. **An environment block's fields.** Its generic uniforms (`gp0` to `gp6`)
   carry sky colour; sun colour, intensity, direction and an enable flag;
   ambient colour and strength; fog colour and density; AO samples and
   radius; bump strength and transparency bounces; and maximum shadow and
   sky distances, shadow steps and reflection samples. That is a concrete
   field list to check 472's block contract against.
4. **A caution about traced vegetation billboards.** They are
   camera-facing, alpha-tested sprites (`trace_organic_primary` against a
   sprite atlas). They appear in primary rays only, so they cast no shadow and
   take no part in occlusion. A terrarium's plants would want both.
5. **One scene, two renderers, chosen by quality.** The same scene buffers
   (`verts3d`, `indices3d`, atlas and material table) are bound to both the
   tracer and the rasteriser. The wing splits by content instead: terrain is
   traced, bodies are rasterised, joined by depth. This confirms that one
   scene contract can feed both kinds of renderer. It does not suggest
   changing the split.

Per-texel material ids into a table match the wing's own voxel material
scheme, compact ids into world-local definitions. That is convergence, not
something new.

## 5. Ruling

Recorded as ruling 622 in the wing design record: scenevm is a donor of
techniques only.

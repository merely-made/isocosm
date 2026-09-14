// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The join: bodies rastered into a depth attachment, terrain traced against
//! that same depth, both under one `clip_from_world`.
//!
//! A [`Scene`] owns the device-side frame — the tracer, the brick map, the
//! body layer and the two colour targets — and nothing that could name a
//! product's world. Each frame arrives as a [`SceneFrame`]: where the camera
//! is, which bodies are in it, where the terrain comes from, and how it is
//! graded. What a host still decides for itself it supplies through
//! [`SceneHost`].

use mesocosm_lens::{
    BrickChange, BrickDiagnostics, BrickFrameInput, BrickMap, BrickRevision, BrickTracer,
    CritterPose, FRAME_FORMAT, Grade, TerrainAppearance,
};

use crate::bodies::{BodyFrameStats, BodyLayer, SceneBody, SceneVolumes};
use crate::camera::SlabCamera;
use crate::glyphs::GlyphLayer;
use crate::query::PresentedFrame;

mod terrain;
pub use terrain::{GroundTerrain, HostTerrain, TerrainRefresh, TerrainSource};

/// Captures and the display twin are written in this format regardless of any
/// surface's, so a PNG encoder never has to swizzle a BGRA frame.
pub(crate) const CAPTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

/// Capsule stand-ins handed straight to the tracer instead of a raster body
/// pass.
///
/// A frame that carries this draws no meshes and takes the tracer's plain
/// encode: it is a host's cheaper presentation mode, not a fallback. The
/// fallback for a body that would not project is [`SceneHost::fallback`].
#[derive(Clone, Copy)]
pub struct CapsuleFrame<'a> {
    pub roster: &'a [CritterPose],
    /// The always-visible body, at full capsule fidelity.
    pub played: Option<&'a CritterPose>,
}

/// One frame's inputs. No product world appears here.
pub struct SceneFrame<'a> {
    pub camera: SlabCamera,
    /// The bodies to raster, in any order; the layer culls and sorts them.
    pub bodies: &'a [SceneBody<'a>],
    pub volumes: SceneVolumes<'a>,
    /// `None` is the isolated body preview: no trace, no terrain in the pick.
    pub terrain: Option<&'a dyn TerrainSource>,
    /// The host's drain of the world's changed bricks. The slots they map to
    /// are the only region the tracer re-uploads, so a carve costs its own
    /// bricks and not the enclosure.
    pub dirty: &'a [[i16; 3]],
    pub grade: Grade,
    pub terrain_appearance: Option<TerrainAppearance>,
    pub body_budget: usize,
    /// Host policy: draw bodies as tracer capsules instead of rastering their
    /// meshes. `Some` skips the body pass and the depth join entirely.
    pub capsules: Option<CapsuleFrame<'a>>,
}

/// What one encoded frame did.
#[derive(Clone, Copy, Debug, Default)]
pub struct SceneStats {
    /// The tracer's terrain work, or `None` when the frame drew no terrain.
    pub terrain: Option<BrickDiagnostics>,
    /// Whether the terrain join ran. A query receipt records it, because a
    /// pick against a frame without terrain must not consult the brick map.
    pub terrain_drawn: bool,
}

/// The host policy a frame calls back into.
///
/// Every method here is a seam of this extraction rather than a permanent
/// one: the capsule fallback is Mesocosm's alone — the scene reports that a
/// body would not project and the host decides what the frame shows instead.
pub trait SceneHost {
    /// Before the frame's bodies are projected.
    fn begin(&mut self) {}

    /// One body whose projection or bounds failed, with the frame's counters
    /// so a substitute presentation still spends the frame's budget.
    fn fallback(&mut self, _body: &SceneBody<'_>, _stats: &mut BodyFrameStats) {}

    /// After the projection pass, for counters only the host can read back.
    fn prepared(&mut self, _bodies: &mut BodyLayer) {}

    /// Capsule stand-ins for everything but the always-visible body.
    fn roster(&self) -> &[CritterPose] {
        &[]
    }

    /// The always-visible body's stand-in, at full capsule fidelity.
    fn played(&self) -> Option<&CritterPose> {
        None
    }
}

pub struct Scene {
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    tracer: BrickTracer,
    map: Option<BrickMap>,
    /// A CPU map change that has not reached a successful terrain encode.
    /// Isolated previews and failed frames must not consume its upload.
    terrain_upload_pending: bool,
    terrain_diagnostics: Option<BrickDiagnostics>,
    bodies: BodyLayer,
    /// The scene's own chrome, on the scene's own depth. Built on first use,
    /// because a frame that never sets a glyph pays for no pipeline.
    pub(crate) glyphs: Option<GlyphLayer>,
    pub(crate) width: u32,
    pub(crate) height: u32,
    /// What the tracer writes: display-encoded values in a linear-tagged
    /// format, exactly as the lens's own captures read them back.
    traced: wgpu::Texture,
    traced_view: wgpu::TextureView,
    /// The same texels in an sRGB-tagged twin. Sampling the raw bytes into an
    /// sRGB target would encode a second time and wash the scene out; the
    /// twin's decode-on-sample cancels the target's encode.
    display: wgpu::Texture,
    display_view: wgpu::TextureView,
    /// The query receipt of the last complete encode. Every visual change a
    /// host owns rather than a frame drops it through
    /// [`Scene::invalidate_query`].
    pub(crate) presented: Option<PresentedFrame>,
}

impl Scene {
    pub fn new(
        device: wgpu::Device,
        queue: wgpu::Queue,
        width: u32,
        height: u32,
    ) -> Result<Self, String> {
        let tracer =
            BrickTracer::with_format(device.clone(), queue.clone(), width, height, FRAME_FORMAT);
        let bodies = BodyLayer::new(&device, width, height, 1.0);
        let (traced, traced_view) = target(&device, width, height, FRAME_FORMAT, "traced scene");
        let (display, display_view) =
            target(&device, width, height, CAPTURE_FORMAT, "traced scene srgb");
        Ok(Self {
            device,
            queue,
            tracer,
            map: None,
            terrain_upload_pending: true,
            terrain_diagnostics: None,
            bodies,
            glyphs: None,
            width,
            height,
            traced,
            traced_view,
            display,
            display_view,
            presented: None,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.invalidate_query();
        self.width = width.max(1);
        self.height = height.max(1);
        self.tracer.resize(self.width, self.height);
        self.bodies.resize(&self.device, self.width, self.height);
        (self.traced, self.traced_view) = target(
            &self.device,
            self.width,
            self.height,
            FRAME_FORMAT,
            "traced scene",
        );
        (self.display, self.display_view) = target(
            &self.device,
            self.width,
            self.height,
            CAPTURE_FORMAT,
            "traced scene srgb",
        );
    }

    pub fn bodies(&self) -> &BodyLayer {
        &self.bodies
    }

    pub fn bodies_mut(&mut self) -> &mut BodyLayer {
        &mut self.bodies
    }

    /// Distinct part geometries the body layer retains across frames.
    pub fn cached_bodies(&self) -> usize {
        self.bodies.cached_bodies()
    }

    /// Display-encoded sRGB bytes in an unorm view, for a document compositor
    /// that samples encoded values directly. Alpha is opaque. The caller must
    /// submit the scene's encoder before staging this same-device image.
    pub fn encoded_view(&self) -> &wgpu::TextureView {
        &self.traced_view
    }

    /// The completed same-device texture a frame graph imports.
    pub fn display_texture(&self) -> &wgpu::Texture {
        &self.display
    }

    pub fn display_view(&self) -> &wgpu::TextureView {
        &self.display_view
    }

    /// Most recent encode's terrain work. `None` means terrain was skipped or
    /// the encode failed, never a retained prior receipt.
    pub fn terrain_diagnostics(&self) -> Option<BrickDiagnostics> {
        self.terrain_diagnostics
    }

    /// The tracer's own last receipt, which survives a skipped frame. What a
    /// capsule roster's counters are read from.
    pub fn last_trace_diagnostics(&self) -> Option<BrickDiagnostics> {
        self.tracer.last_diagnostics()
    }

    /// The brick map the next frame traces, once one has been supplied.
    pub fn terrain_map(&self) -> Option<&BrickMap> {
        self.map.as_ref()
    }

    /// A CPU map change no encode has consumed yet.
    pub fn terrain_upload_pending(&self) -> bool {
        self.terrain_upload_pending
    }

    /// Binds a map the host built itself, pending a full upload.
    pub fn set_terrain_map(&mut self, map: BrickMap) {
        self.map = Some(map);
        self.terrain_upload_pending = true;
    }

    /// Traces one ray against the bound terrain, for a pick that must know
    /// whether ground occludes a body hit.
    pub fn terrain_ray(
        &self,
        origin: [f32; 3],
        direction: [f32; 3],
        far: f32,
    ) -> Result<Option<mesocosm_lens::BrickRayHit>, mesocosm_lens::BrickRayError> {
        match &self.map {
            Some(map) => map.trace_ray(origin, direction, far),
            None => Ok(None),
        }
    }

    /// Encodes one frame into the tenant-owned display texture.
    ///
    /// Bodies raster first into the shared depth, then the tracer joins the
    /// terrain against it under the same `clip_from_world`, then the glyph
    /// batch against that same depth, then the display copy. A frame with no terrain stops after the
    /// bodies: that is the isolated preview, and it completes with
    /// `terrain_drawn` false so a pick does not consult the brick map.
    pub fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        frame: SceneFrame<'_>,
        host: &mut dyn SceneHost,
    ) -> Result<SceneStats, String> {
        // A partial encode may have changed terrain or body projections. It
        // cannot retain a query receipt from a different completed frame.
        self.invalidate_query();
        self.terrain_diagnostics = None;
        let mut full = self.terrain_upload_pending;
        let mut slots = Vec::new();
        if let Some(source) = frame.terrain {
            if self.map.is_none() {
                self.map = Some(source.brick_map()?);
                full = true;
                self.terrain_upload_pending = true;
            } else {
                // A refresh that fails part-way, and one that succeeds whose
                // encode then fails, both leave a full retry pending.
                self.terrain_upload_pending = true;
                let map = self.map.as_mut().expect("map present");
                match source.refresh(map, frame.dirty)? {
                    TerrainRefresh::Current => self.terrain_upload_pending = full,
                    TerrainRefresh::Slots(moved) => slots = moved,
                    TerrainRefresh::Full => full = true,
                }
            }
        }
        let change = if full {
            BrickChange::Full
        } else {
            BrickChange::Slots(&slots)
        };
        let camera = frame.camera;
        let trace_camera = camera.trace().ok_or("invalid scene camera")?;

        if frame.capsules.is_none() {
            let matrix = self.draw_bodies(encoder, &frame, host);
            if frame.terrain.is_none()
                || (self.bodies.isolated && self.bodies.stats.fallback_bodies == 0)
            {
                self.draw_glyphs(encoder, camera);
                self.copy_to_display(encoder);
                self.complete_query_frame(camera, false);
                return Ok(SceneStats::default());
            }
            let revision = frame.terrain.expect("terrain present").revision();
            let Self {
                tracer,
                map,
                bodies,
                traced_view,
                ..
            } = &mut *self;
            let mut input = BrickFrameInput::for_camera(
                map.as_ref().expect("map present"),
                BrickRevision(revision),
                trace_camera,
                &frame.grade,
            )
            .changed(change)
            .with_clip_from_world(matrix)
            .with_roster(host.roster());
            input.terrain_appearance = frame.terrain_appearance;
            if let Some(pose) = host.played() {
                input = input.with_pose(pose);
            }
            self.terrain_diagnostics = Some(
                tracer
                    .encode_with_depth(encoder, traced_view, &bodies.depth_view, input)
                    .map_err(|error| error.to_string())?,
            );
        } else {
            let capsules = frame.capsules.expect("capsule frame");
            let revision = frame
                .terrain
                .ok_or("a capsule frame still needs terrain")?
                .revision();
            let Self {
                tracer,
                map,
                traced_view,
                ..
            } = &mut *self;
            let mut input = BrickFrameInput::for_camera(
                map.as_ref().expect("map present"),
                BrickRevision(revision),
                trace_camera,
                &frame.grade,
            )
            .changed(change)
            .with_roster(capsules.roster);
            input.terrain_appearance = frame.terrain_appearance;
            if let Some(pose) = capsules.played {
                input = input.with_pose(pose);
            }
            self.terrain_diagnostics = Some(
                tracer
                    .encode(encoder, traced_view, input)
                    .map_err(|error| error.to_string())?,
            );
        }
        self.terrain_upload_pending = false;
        if frame.capsules.is_none() {
            self.draw_glyphs(encoder, camera);
        }
        self.copy_to_display(encoder);
        // A capsule frame rasters no meshes, so it leaves no part identity to
        // query: the host's stand-ins are not selectable surfaces.
        if frame.capsules.is_none() {
            self.complete_query_frame(camera, true);
        }
        Ok(SceneStats {
            terrain: self.terrain_diagnostics,
            terrain_drawn: true,
        })
    }

    /// The bounded glyph batch, drawn into the scene's own colour against the
    /// depth the bodies and the terrain join already wrote.
    fn draw_glyphs(&self, encoder: &mut wgpu::CommandEncoder, camera: SlabCamera) {
        if let Some(glyphs) = &self.glyphs {
            glyphs.draw(
                &self.queue,
                encoder,
                &self.traced_view,
                &self.bodies.depth_view,
                camera,
            );
        }
    }

    /// Projects, culls and rasters the frame's bodies, clearing the shared
    /// attachments first. Returns the `clip_from_world` the terrain join
    /// reuses, which is the camera's whether the draw succeeded or not.
    fn draw_bodies(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        frame: &SceneFrame<'_>,
        host: &mut dyn SceneHost,
    ) -> [[f32; 4]; 4] {
        let camera = frame.camera;
        let window = camera.window();
        host.begin();
        let Self { bodies, .. } = &mut *self;
        bodies.budget = frame.body_budget.max(1);
        bodies.prepare(frame.bodies, frame.volumes, window, |body, stats| {
            host.fallback(body, stats)
        });
        host.prepared(&mut self.bodies);
        {
            let _clear = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("clear shared scene attachments"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.traced_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.bodies.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
        }
        let matrix = camera.clip_from_world();
        let Self {
            device,
            queue,
            bodies,
            traced_view,
            ..
        } = &mut *self;
        if let Err(error) = bodies.draw(device, queue, encoder, traced_view, camera) {
            eprintln!("{error}; using capsule fallback");
            bodies.stats.last_error = Some(error);
            for subject in bodies.take_drawn() {
                if let Some(body) = frame.bodies.iter().find(|body| body.subject == subject) {
                    host.fallback(body, &mut bodies.stats);
                }
            }
        }
        matrix
    }

    fn copy_to_display(&self, encoder: &mut wgpu::CommandEncoder) {
        encoder.copy_texture_to_texture(
            self.traced.as_image_copy(),
            self.display.as_image_copy(),
            wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
        );
    }
}

pub(crate) fn target(
    device: &wgpu::Device,
    width: u32,
    height: u32,
    format: wgpu::TextureFormat,
    label: &str,
) -> (wgpu::Texture, wgpu::TextureView) {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_SRC
            | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let view = texture.create_view(&Default::default());
    (texture, view)
}

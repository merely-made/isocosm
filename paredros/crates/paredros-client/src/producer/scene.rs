// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The `TextureProducer` over one played `GameState`.
//!
//! Terrain is `mesocosm-lens`'s brick tracer over a `BrickMap` built from the
//! game's own `Ground`; bodies are `mesocosm-render`'s live body renderer.
//! Both write the one depth attachment this module owns and both consume the
//! same `clip_from_world`, which is what makes them occlude each other per
//! pixel (ruling 13). Bodies are instances, never per-body surfaces (ruling 14).

use cambium_rootstock::{
    ProducedTexture, ProducerContext, SourceAlpha, SourceEncoding, TextureProducer,
};
use mesocosm_core::PartId;
use mesocosm_lens::{
    BrickChange, BrickFrameInput, BrickMap, BrickRevision, BrickTracer, FRAME_FORMAT, Grade,
};
use mesocosm_render::live_body::{BodyDrawStats, LiveBodyRenderer};
use paredros_identity::{BodyRevisionId, SubjectId};
use paredros_world::MotionPose;

use super::bodies::BodyLayer;
use super::camera::SlabView;
use super::handle::SceneHandle;

/// Palette depth of the retro grade the terrain is shaded with.
const PALETTE: u32 = 3;
/// Distinct volume boxes the body renderer may retain. One per addressed tag.
const MESH_BUDGET: usize = 256;

/// Everything the last completed frame's pixels depend on. Equality here is
/// the whole skip rule.
#[derive(Clone, PartialEq)]
struct Rendered {
    size: [u32; 2],
    ground_revision: u64,
    framed: SubjectId,
    terrain: bool,
    view: SlabView,
    bodies: Vec<(SubjectId, BodyRevisionId, MotionPose)>,
}

struct Targets {
    colour: wgpu::Texture,
    colour_view: wgpu::TextureView,
    depth_view: wgpu::TextureView,
    size: [u32; 2],
}

/// One Paredros scene, produced into a Cambium document leaf.
pub struct SceneProducer {
    handle: SceneHandle,
    tracer: Option<BrickTracer>,
    tracer_size: [u32; 2],
    renderer: Option<LiveBodyRenderer>,
    targets: Option<Targets>,
    map: Option<BrickMap>,
    map_revision: Option<u64>,
    /// A CPU map change that has not reached a successful terrain encode.
    terrain_upload_pending: bool,
    grade: Grade,
    layer: BodyLayer,
    rendered: Option<Rendered>,
    presented: Option<SlabView>,
    renders: u64,
    stats: BodyDrawStats,
    error: Option<String>,
}

impl SceneProducer {
    pub fn new(handle: SceneHandle) -> Self {
        Self {
            handle,
            tracer: None,
            tracer_size: [0; 2],
            renderer: None,
            targets: None,
            map: None,
            map_revision: None,
            terrain_upload_pending: true,
            grade: Grade::retro(PALETTE),
            layer: BodyLayer::default(),
            rendered: None,
            presented: None,
            renders: 0,
            stats: BodyDrawStats::default(),
            error: None,
        }
    }

    pub fn handle(&self) -> &SceneHandle {
        &self.handle
    }

    /// Frames produced. This is the `generation` the document sees.
    pub fn renders(&self) -> u64 {
        self.renders
    }

    /// The last draw's receipts, including whether static geometry uploaded.
    pub fn body_stats(&self) -> BodyDrawStats {
        self.stats
    }

    pub fn last_error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    /// The camera the last completed frame used, for a caller naming a pixel.
    pub fn presented_view(&self) -> Option<SlabView> {
        self.presented
    }

    pub fn bodies(&self) -> &BodyLayer {
        &self.layer
    }

    /// The completed colour texture, for a caller reading it back.
    pub fn colour_texture(&self) -> Option<&wgpu::Texture> {
        self.targets.as_ref().map(|targets| &targets.colour)
    }

    /// Nearest drawn body surface along the ray through `ndc`.
    ///
    /// **Terrain is not consulted.** The tracer's depth lives on the GPU and
    /// reading it back would cost a stall per click, so a body standing behind
    /// a ridge is still picked here even though the frame does not show it.
    /// Hence the name; a terrain-aware pick is a separate, GPU-side query.
    pub fn pick_body_ignoring_terrain(&self, ndc: [f32; 2]) -> Option<(SubjectId, PartId)> {
        let view = self.presented?;
        let camera = view.trace()?;
        let (origin, direction) = camera.ray_at(ndc)?;
        let live: Vec<_> = self.layer.bodies.iter().map(|body| body.live()).collect();
        let hit = mesocosm_render::live_body::pick_bodies(
            &live,
            origin,
            direction,
            camera.far(),
            Some(view.clip()),
        )
        .ok()??;
        Some((self.layer.bodies[hit.body_index].subject, hit.part))
    }

    /// Renders one frame, or answers `None` when nothing the picture depends
    /// on has moved.
    pub fn render_scene(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        size: [u32; 2],
        needs_frame: bool,
    ) -> Result<Option<wgpu::TextureView>, String> {
        if size[0] == 0 || size[1] == 0 {
            return Err("viewport has no pixels".into());
        }
        let aspect = size[0] as f32 / size[1] as f32;
        let (view, framed, terrain, ground_revision) = {
            let model = self.handle.borrow();
            let centre = model
                .follow_centre()
                .map_err(|error| format!("{error:?}"))?;
            let view = SlabView::new(model.camera, centre, aspect).ok_or("invalid scene camera")?;
            self.layer
                .prepare(model.game(), model.played(), model.appearance);
            (
                view,
                model.played(),
                model.terrain,
                model.game().world().ground().revision(),
            )
        };
        let current = Rendered {
            size,
            ground_revision,
            framed,
            terrain,
            view,
            bodies: self.layer.signature(),
        };
        if !needs_frame && self.targets.is_some() && self.rendered.as_ref() == Some(&current) {
            return Ok(None);
        }
        let clip_from_world = view.clip_from_world().ok_or("invalid scene basis")?;
        self.ensure_targets(device, size);
        if terrain {
            self.ensure_map(ground_revision)?;
        }
        let (colour_view, depth_view) = {
            let targets = self.targets.as_ref().expect("targets ensured");
            (targets.colour_view.clone(), targets.depth_view.clone())
        };
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("paredros scene"),
        });
        clear(&mut encoder, &colour_view, &depth_view);
        let live: Vec<_> = self.layer.bodies.iter().map(|body| body.live()).collect();
        let renderer = self
            .renderer
            .get_or_insert_with(|| LiveBodyRenderer::new(device, FRAME_FORMAT, MESH_BUDGET));
        self.stats = renderer
            .draw(
                device,
                queue,
                &mut encoder,
                &colour_view,
                &depth_view,
                clip_from_world,
                Some(view.clip()),
                &live,
            )
            .map_err(|error| format!("live bodies: {error:?}"))?;
        if terrain {
            self.trace_terrain(
                device,
                queue,
                &mut encoder,
                &colour_view,
                &depth_view,
                size,
                view,
                clip_from_world,
                ground_revision,
            )?;
        }
        queue.submit(Some(encoder.finish()));
        self.renders = self
            .renders
            .checked_add(1)
            .expect("scene frame generation exhausted");
        self.rendered = Some(current);
        self.presented = Some(view);
        Ok(Some(colour_view))
    }

    #[allow(clippy::too_many_arguments)]
    fn trace_terrain(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        colour: &wgpu::TextureView,
        depth: &wgpu::TextureView,
        size: [u32; 2],
        view: SlabView,
        clip_from_world: [[f32; 4]; 4],
        ground_revision: u64,
    ) -> Result<(), String> {
        let camera = view.trace().ok_or("invalid slab camera")?;
        let map = self.map.as_ref().expect("map ensured");
        let tracer = self.tracer.get_or_insert_with(|| {
            BrickTracer::with_format(
                device.clone(),
                queue.clone(),
                size[0],
                size[1],
                FRAME_FORMAT,
            )
        });
        if self.tracer_size != size {
            tracer.resize(size[0], size[1]);
            self.tracer_size = size;
        }
        let change = if self.terrain_upload_pending {
            BrickChange::Full
        } else {
            BrickChange::Slots(&[])
        };
        let input =
            BrickFrameInput::for_camera(map, BrickRevision(ground_revision), camera, &self.grade)
                .changed(change)
                .with_clip_from_world(clip_from_world);
        tracer
            .encode_with_depth(encoder, colour, depth, input)
            .map_err(|error| format!("terrain trace: {error:?}"))?;
        self.terrain_upload_pending = false;
        Ok(())
    }

    fn ensure_targets(&mut self, device: &wgpu::Device, size: [u32; 2]) {
        if self.targets.as_ref().is_some_and(|t| t.size == size) {
            return;
        }
        let extent = wgpu::Extent3d {
            width: size[0],
            height: size[1],
            depth_or_array_layers: 1,
        };
        let colour = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("paredros scene colour"),
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: FRAME_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let depth = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("paredros scene depth"),
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let colour_view = colour.create_view(&Default::default());
        let depth_view = depth.create_view(&Default::default());
        self.targets = Some(Targets {
            colour,
            colour_view,
            depth_view,
            size,
        });
        self.terrain_upload_pending = true;
    }

    /// The brick map is rebuilt only when the ground's own revision moves, so
    /// an ordinary frame costs no CPU walk of the world.
    fn ensure_map(&mut self, ground_revision: u64) -> Result<(), String> {
        if self.map.is_some() && self.map_revision == Some(ground_revision) {
            return Ok(());
        }
        let map = {
            let model = self.handle.borrow();
            BrickMap::from_ground(model.game().world().ground())
                .map_err(|error| format!("brick map: {error}"))?
        };
        self.map = Some(map);
        self.map_revision = Some(ground_revision);
        self.terrain_upload_pending = true;
        Ok(())
    }

    /// Releases GPU targets and pipelines while keeping the body geometry
    /// cache, so coming back costs an allocation rather than a rebuild.
    pub fn release_gpu(&mut self) {
        self.targets = None;
        self.tracer = None;
        self.tracer_size = [0; 2];
        self.renderer = None;
        self.map = None;
        self.map_revision = None;
        self.terrain_upload_pending = true;
        self.rendered = None;
        self.presented = None;
    }
}

fn clear(
    encoder: &mut wgpu::CommandEncoder,
    colour: &wgpu::TextureView,
    depth: &wgpu::TextureView,
) {
    let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("clear paredros scene attachments"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: colour,
            depth_slice: None,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
            view: depth,
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

impl TextureProducer for SceneProducer {
    fn render(&mut self, cx: &ProducerContext<'_>) -> Option<ProducedTexture> {
        match self.render_scene(
            cx.device,
            cx.queue,
            cx.frame.physical_size,
            cx.frame.needs_frame,
        ) {
            Ok(view) => {
                self.error = None;
                view.map(|view| ProducedTexture {
                    view,
                    generation: self.renders,
                    alpha: SourceAlpha::Straight,
                    encoding: SourceEncoding::Srgb,
                })
            },
            Err(why) => {
                self.error = Some(why);
                None
            },
        }
    }

    fn suspend(&mut self) {
        self.release_gpu();
    }

    fn retire(&mut self) {
        self.release_gpu();
        self.stats = BodyDrawStats::default();
    }
}

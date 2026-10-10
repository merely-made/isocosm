// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The main view: the ruled terrarium section, brick-traced.
//!
//! [`isometer::Scene`] projects bodies and traces the terrain depth. The lit
//! body tenant reads that pre-pass; Netrender layers its transparent target
//! over the traced colour. What stays here is the vessel's policy over it:
//! which Ground the map binds, where the slab sits, which body is posed, and
//! how the traced texture reaches the surface. The input is a native
//! [`SiteScene`]: the played critter's lifted site and its placed bodies.
//!
//! Which way it looks is [`camera`]'s, a host flag; none of the modes moves
//! the hash.

mod bodies;
mod camera;
mod capsules;
mod tenant;
pub use capsules::{pose_of, roster_of};
mod terrain;
pub use terrain::TerrainStyle;
mod inspection;
mod view;

pub use bodies::{BodyMode, DEFAULT_BODY_BUDGET};
pub use inspection::BodySelection;
/// The frame receipt is `isometer`'s; re-exported for the receipts.
pub use isometer::BodyFrameStats;

pub use camera::{CameraMode, Framing, OBLIQUE_DEGREES, SLAB_DEPTH, TERRARIUM_DEGREES};
/// The cull window is `isometer`'s; re-exported for the roster.
pub use isometer::SlabWindow;

use isometer::core::BodyDocument;
use isometer::core::ground::Ground;
use isometer::lens::{BrickMap, CritterPose, Grade};
use isometer::render::composite::Composite;
use isometer::{
    CapsuleFrame, DeclaredExtentVolumes, GroundTerrain, Scene, SceneFrame, SceneVolumes,
};

/// The default slab half-height, ruled 2026-08-29; a host may carry another.
pub const SLAB_HALF_HEIGHT: f32 = 28.0;
const PALETTE: u32 = 3;

/// Voxels one arrow-key press shifts the section by. Presentation only: this
/// number never reaches an intent.
pub const PAN_STEP: f32 = 2.0;

/// A presentation-only offset added to the follow centre.
#[derive(Clone, Copy, Debug, Default)]
pub struct Pan {
    pub x: f32,
    pub y: f32,
}

/// One body the section draws, standing where the host placed it.
#[derive(Clone, Debug)]
pub struct PlacedBody {
    pub id: u64,
    pub document: BodyDocument,
    /// A presentation spot, never a world position (see `played::layout`).
    pub at: [i32; 3],
    pub tint: [f32; 3],
    pub alive: bool,
}

/// The section's native input: a lifted site, its bodies, and who is played.
pub struct SiteScene {
    pub site: u64,
    pub ground: Ground,
    /// The lifted window the ground came from, which places patches (783).
    pub window: isometer::space::volume::Volume,
    pub bodies: Vec<PlacedBody>,
    pub played: Option<u64>,
    /// Declared-extent boxes for the bodies, which carry no voxel content.
    pub volumes: DeclaredExtentVolumes,
}

impl SiteScene {
    /// Where body `id` stands in this scene, if it is here.
    pub fn at(&self, id: u64) -> Option<[i32; 3]> {
        self.body(id).map(|b| b.at)
    }

    pub fn body(&self, id: u64) -> Option<&PlacedBody> {
        self.bodies.iter().find(|b| b.id == id)
    }
}

/// The host's presentation reads for one frame. None of it is world state.
#[derive(Clone, Copy)]
pub struct SectionFrame<'a> {
    pub scene: &'a SiteScene,
    /// Bricks changed since the ground was bound; empty for a rebound one.
    pub dirty: &'a [[i16; 3]],
    pub centre: [f32; 3],
    /// The controlled critter, at full capsule fidelity.
    pub pose: Option<&'a CritterPose>,
    /// Everything else alive in the slab.
    pub roster: &'a [CritterPose],
}

pub struct Section {
    device: wgpu::Device,
    queue: wgpu::Queue,
    /// Shared projection, queries and traced terrain.
    scene: Scene,
    body_tenant: tenant::BodyTenant,
    grade: Grade,
    terrain_appearance: Option<isometer::lens::TerrainAppearance>,
    width: u32,
    height: u32,
    /// How much world this section frames. Presentation, so it lives beside the
    /// device rather than in the world.
    half_height: f32,
    /// Which way it looks. Presentation, beside the half-height and for the
    /// same reason: it frames the world and decides nothing in it.
    mode: CameraMode,
    body_mode: BodyMode,
    body_budget: usize,
    /// The Mesocosm facts the scene is handed rather than the ones it derives.
    host_bodies: bodies::HostBodies,
    composite: Composite,
}

impl Section {
    /// Binds a lifted site's Ground and builds the scene on the host's own
    /// device. `format` is the surface's, for the composite that lands
    /// the traced frame under the HUD.
    pub fn new(
        host: &::tenant::HostDevice,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
        ground: &Ground,
        framing: Framing,
    ) -> Result<Self, String> {
        let (device, queue) = (host.device.clone(), host.queue.clone());
        let map = BrickMap::from_ground(ground).map_err(|error| error.to_string())?;
        let composite = Composite::new(&device, format);
        let mut scene = Scene::new(device.clone(), queue.clone(), width, height)?;
        scene.set_terrain_map(map);
        scene.bodies_mut().preview_depth = SLAB_DEPTH;
        Ok(Self {
            device,
            queue,
            scene,
            body_tenant: tenant::BodyTenant::new(host, [width, height]),
            grade: Grade::retro(PALETTE),
            terrain_appearance: None,
            width,
            height,
            half_height: half_height_or_default(framing.half_height),
            mode: framing.mode,
            body_mode: BodyMode::default(),
            body_budget: DEFAULT_BODY_BUDGET,
            host_bodies: bodies::HostBodies::new(),
            composite,
        })
    }

    /// Rebinds the terrain to another lifted Ground: a new site, or new edits.
    pub fn set_ground(&mut self, ground: &Ground) -> Result<(), String> {
        let map = BrickMap::from_ground(ground).map_err(|error| error.to_string())?;
        self.scene.set_terrain_map(map);
        self.invalidate_query();
        Ok(())
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width.max(1);
        self.height = height.max(1);
        self.scene.resize(self.width, self.height);
        self.body_tenant
            .resize(&self.device, [self.width, self.height]);
    }

    /// Aspect comes from the window rather than a fixed 16:9, so a resized
    /// section stretches nothing.
    fn aspect(&self) -> f32 {
        self.width as f32 / self.height.max(1) as f32
    }

    /// Which way this section is looking. The receipt names it, so a capture
    /// can be told apart from the next one.
    pub fn mode(&self) -> CameraMode {
        self.mode
    }

    pub fn configure_bodies(&mut self, mode: BodyMode, budget: usize) {
        if self.body_mode != mode || self.body_budget != budget.max(1) {
            self.invalidate_query();
        }
        self.body_mode = mode;
        self.body_budget = budget.max(1);
        if mode == BodyMode::Capsules {
            self.scene.bodies_mut().clear_inspection();
        }
    }

    pub fn body_stats(&self) -> BodyFrameStats {
        self.scene.bodies().stats.clone()
    }

    pub fn set_body_light(&mut self, intensity: f32) {
        self.body_tenant.set_light(intensity);
    }

    /// Transparent lit bodies, layered over the terrain by Netrender.
    pub fn body_view(&self) -> Option<wgpu::TextureView> {
        (self.body_mode == BodyMode::Voxels).then(|| self.body_tenant.view())
    }

    /// How much world the section frames, in voxels of half-height.
    pub fn half_height(&self) -> f32 {
        self.half_height
    }

    /// Host-owned framing, shared by terrain rays, body depth and culling.
    pub fn set_half_height(&mut self, half: f32) {
        let half = half_height_or_default(half);
        if self.half_height != half {
            self.invalidate_query();
            self.half_height = half;
        }
    }

    /// The world box this camera actually shows, from the camera's own
    /// numbers. What falls outside cannot reach a pixel, so it is what the
    /// roster culls against.
    pub fn slab_window(&self, centre: [f32; 3]) -> SlabWindow {
        self.view(centre).window()
    }

    /// Roster members the last traced frame drew. The receipt's evidence that
    /// the section shows the ecology rather than one body.
    pub fn last_roster_members(&self) -> u32 {
        let stats = &self.scene.bodies().stats;
        if self.body_mode == BodyMode::Voxels {
            return (stats.voxel_bodies + stats.fallback_bodies)
                .saturating_sub(usize::from(stats.controlled_drawn)) as u32;
        }
        self.scene
            .last_trace_diagnostics()
            .map_or(0, |diagnostics| diagnostics.roster_members)
    }

    /// Capsules the last frame's roster members carried past their budget. The
    /// widest are kept, so this is detail spent rather than bodies lost.
    pub fn last_roster_capsules_dropped(&self) -> u32 {
        self.scene
            .last_trace_diagnostics()
            .map_or(0, |diagnostics| diagnostics.roster_capsules_dropped)
    }

    /// Traces one frame into the tenant-owned display texture.
    pub fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        frame: SectionFrame<'_>,
    ) -> Result<(), String> {
        let terrain = GroundTerrain(&frame.scene.ground);
        let camera = self.view(frame.centre);
        let voxels = self.body_mode == BodyMode::Voxels;
        let scene_bodies = if voxels {
            self.scene_bodies(frame.scene)
        } else {
            Vec::new()
        };
        let (grade, appearance, budget) = (self.grade, self.terrain_appearance, self.body_budget);
        let Self {
            scene,
            host_bodies,
            body_tenant,
            ..
        } = self;
        let mut host = SectionHost {
            bodies: host_bodies,
            scene: frame.scene,
            tenant: body_tenant,
        };
        scene.render(
            encoder,
            SceneFrame {
                camera,
                bodies: &scene_bodies,
                volumes: SceneVolumes::DeclaredSolid(&frame.scene.volumes),
                terrain: Some(&terrain),
                dirty: frame.dirty,
                grade,
                terrain_appearance: appearance,
                body_budget: budget,
                capsules: (!voxels).then_some(CapsuleFrame {
                    roster: frame.roster,
                    played: frame.pose,
                }),
            },
            &mut host,
        )?;
        Ok(())
    }

    /// The completed terrain texture imported by Netrender's frame graph.
    pub fn display_texture(&self) -> &wgpu::Texture {
        self.scene.display_texture()
    }

    /// Reads the most recently traced frame back as RGBA8, with `overlay`
    /// given the chance to composite chrome over it first. The staging buffer
    /// and the row padding are the scene's.
    pub fn capture(
        &self,
        overlay: impl FnOnce(&mut wgpu::CommandEncoder, &wgpu::TextureView, wgpu::TextureFormat),
    ) -> Option<(u32, u32, Vec<u8>)> {
        self.scene.capture(|encoder, view, format| {
            if let Some(body) = self.body_view() {
                Composite::new(&self.device, format).draw(
                    &self.device,
                    &self.queue,
                    encoder,
                    view,
                    &body,
                    (0.0, 0.0, self.width as f32, self.height as f32),
                    (self.width, self.height),
                );
            }
            overlay(encoder, view, format);
        })
    }

    /// Reads a completed frame master back as RGBA8. RG3 uses this route so
    /// its evidence crosses the same imported-tenant master as the live
    /// window.
    pub fn capture_from(
        &self,
        master: &wgpu::Texture,
        overlay: impl FnOnce(&mut wgpu::CommandEncoder, &wgpu::TextureView, wgpu::TextureFormat),
    ) -> Option<(u32, u32, Vec<u8>)> {
        self.scene.capture_from(master, overlay)
    }

    /// Chromeless fallback: trace and composite directly into the surface.
    pub fn draw(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        surface: &wgpu::TextureView,
        frame: SectionFrame<'_>,
    ) -> Result<(), String> {
        self.render(encoder, frame)?;
        self.composite.draw(
            &self.device,
            &self.queue,
            encoder,
            surface,
            self.scene.display_view(),
            (0.0, 0.0, self.width as f32, self.height as f32),
            (self.width, self.height),
        );
        if let Some(body) = self.body_view() {
            self.composite.draw(
                &self.device,
                &self.queue,
                encoder,
                surface,
                &body,
                (0.0, 0.0, self.width as f32, self.height as f32),
                (self.width, self.height),
            );
        }
        Ok(())
    }
}

/// The host policy one frame calls back into: Mesocosm's capsule stand-in for
/// a body that would not project, and the counters only the scene can supply.
struct SectionHost<'a> {
    bodies: &'a mut bodies::HostBodies,
    scene: &'a SiteScene,
    tenant: &'a mut tenant::BodyTenant,
}

impl isometer::SceneHost for SectionHost<'_> {
    fn uses_body_tenant(&self) -> bool {
        true
    }

    fn encode_body_tenant(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        bodies: &mut isometer::BodyLayer,
        camera: isometer::SlabCamera,
    ) -> Result<(), String> {
        self.tenant.encode(encoder, bodies, camera)
    }

    fn begin(&mut self) {
        self.bodies.fallback.clear();
        self.bodies.played_fallback = None;
    }

    fn fallback(&mut self, body: &isometer::SceneBody<'_>, stats: &mut BodyFrameStats) {
        self.bodies.add_fallback(body, stats);
    }

    fn prepared(&mut self, layer: &mut isometer::BodyLayer) {
        layer.stats.body_scale = self.bodies.scale;
        let scene = self.scene;
        layer.stats.carcasses = layer
            .drawn()
            .filter(|subject| scene.body(subject.0).is_some_and(|b| !b.alive))
            .count();
    }

    fn roster(&self) -> &[CritterPose] {
        &self.bodies.fallback
    }

    fn played(&self) -> Option<&CritterPose> {
        self.bodies.played_fallback.as_ref()
    }
}

/// The half-height a host actually frames with: its own, or the default when
/// it named none. One reading, so the camera and the follow centre's clamp
/// cannot disagree about how tall the frame is.
pub fn half_height_or_default(configured: f32) -> f32 {
    if configured > 0.0 {
        configured
    } else {
        SLAB_HALF_HEIGHT
    }
}

/// Where the slab sits: on the followed body plus the pan, clamped so the
/// frame never dips below bedrock. Presentation only.
pub fn centre_on(at: [i32; 3], pan: Pan, half_height: f32, mode: CameraMode) -> [f32; 3] {
    let floor = mode.vertical_half(half_height_or_default(half_height));
    [
        at[0] as f32 + pan.x,
        (at[1] as f32 + pan.y).max(floor),
        at[2] as f32,
    ]
}

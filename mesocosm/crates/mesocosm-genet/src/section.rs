// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The main view: the ruled terrarium section, brick-traced.
//!
//! The join itself — bodies rastered into a depth attachment, terrain traced
//! against it, the display twin and the capture read-back — is
//! [`isometer::Scene`]'s now. What stays here is the vessel's policy over
//! it: which Ground the map binds, where the slab sits, which body is posed,
//! and how the traced texture reaches the surface the HUD then composites on.
//!
//! Which way it looks is [`camera`]'s, and stays a host flag: the shallow
//! oblique Mark ruled on 2026-09-04 is the default, and the two level arms the
//! measured slice compared it against are still there behind `--camera` (DC4,
//! Q9). None of the three moves the hash.

mod bodies;
mod camera;
mod capsules;
#[cfg(test)]
use capsules::pose_at;
pub use capsules::{pose_of, pose_of_scaled, roster_of, roster_of_scaled};
mod materials;
mod terrain;
mod terrarium;
pub use terrain::TerrainStyle;
mod view;
pub use terrarium::{
    BODY_SCALE as TERRARIUM_BODY_SCALE, Cutaway, framed_habitat, occupied as terrarium_occupied,
};
pub use view::camera_basis;
mod inspection;

pub use bodies::{BodyMode, DEFAULT_BODY_BUDGET};
pub use inspection::{BodyPick, BodySelection};
/// The glyph batch, its attachments and the frame receipt are `isometer`'s
/// now; re-exported so the host's bench and receipts keep one import path.
pub use isometer::{
    BodyFrameStats, BodyPickError, GlyphAnchor, GlyphOrientation, MAX_GLYPH_ANCHORS,
    MAX_SPATIAL_GLYPHS, SpatialGlyph, Stroke,
};

/// The experiment's glyph choice as the renderer's stroke shape. `isometer`
/// draws shapes; the effect-experiment vocabulary is Mesocosm's, and this is
/// the one place the two meet.
pub fn stroke(glyph: mesocosm_core::effect_experiment::Glyph) -> Stroke {
    use mesocosm_core::effect_experiment::Glyph;
    match glyph {
        Glyph::Quotes => Stroke::Quotes,
        Glyph::Slashes => Stroke::Slashes,
        Glyph::Backticks => Stroke::Backticks,
    }
}

pub use camera::{CameraMode, Framing, OBLIQUE_DEGREES, SLAB_DEPTH, TERRARIUM_DEGREES};
/// The cull window is `isometer`'s now; re-exported so the host's roster
/// and bench keep one import path.
pub use isometer::SlabWindow;

use isometer::{
    CapsuleFrame, GroundTerrain, HostTerrain, Scene, SceneFrame, SceneVolumes, TerrainSource,
};
use mesocosm_core::World;
use mesocosm_core::places::Ground;
use mesocosm_lens::{BrickMap, CritterPose, Grade};
use mesocosm_render::composite::Composite;

/// The G2 slab: section depth and the palette depth of the retro grade, plus
/// the half-height Mark ruled on 2026-08-29. Kept as defaults, not as constants
/// a camera policy may not vary.
///
/// **28, framing the content's height and letting the width follow.** The world
/// is 5.2:1 (129 voxels against a 25-voxel band) inside a 1.78:1 window, so no
/// half-height both frames the width and fills the height; 28 shows 77% of the
/// world and reads a 9-voxel limb at 9% of frame. A host may still carry
/// another ([`HostConfig::slab_half_height`](crate::HostConfig)), and every
/// framing replays to one hash. See the scale plan, "The framing".
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

/// The host's presentation reads for one frame, taken off the stepped world
/// before the device is borrowed. None of it is world state.
#[derive(Clone, Copy)]
pub struct SectionFrame<'a> {
    pub world: &'a World,
    pub volumes: &'a mesocosm_mesh::VolumeMap,
    pub ground: &'a Ground,
    /// The host's drain of the world's changed bricks. The slots they map to
    /// are the only region the tracer re-uploads, so a carve costs its own
    /// bricks and not the enclosure.
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
    /// The shared join. Everything device-side lives in here.
    scene: Scene,
    grade: Grade,
    terrain_appearance: Option<mesocosm_lens::TerrainAppearance>,
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
    terrarium: Option<terrarium::TerrariumView>,
    composite: Composite,
}

impl Section {
    /// Binds the live Ground at genesis and builds the scene on the host's
    /// own device. `format` is the surface's, for the composite that lands
    /// the traced frame under the HUD.
    pub fn new(
        device: wgpu::Device,
        queue: wgpu::Queue,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
        ground: &Ground,
        framing: Framing,
    ) -> Result<Self, String> {
        let map = BrickMap::from_ground(ground).map_err(|error| error.to_string())?;
        let composite = Composite::new(&device, format);
        let mut scene = Scene::new(device.clone(), queue.clone(), width, height)?;
        scene.set_terrain_map(map);
        scene.bodies_mut().preview_depth = SLAB_DEPTH;
        Ok(Self {
            device,
            queue,
            scene,
            grade: Grade::retro(PALETTE),
            terrain_appearance: None,
            width,
            height,
            half_height: half_height_or_default(framing.half_height),
            mode: framing.mode,
            body_mode: BodyMode::default(),
            body_budget: DEFAULT_BODY_BUDGET,
            host_bodies: bodies::HostBodies::new(),
            terrarium: None,
            composite,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width.max(1);
        self.height = height.max(1);
        self.scene.resize(self.width, self.height);
    }

    /// Aspect comes from the window rather than a fixed 16:9, so a resized
    /// section stretches nothing.
    fn aspect(&self) -> f32 {
        self.width as f32 / self.height.max(1) as f32
    }

    /// The camera the last completed frame drew with, for a caller naming a
    /// pixel of it.
    pub fn presented_camera(&self) -> Option<isometer::SlabCamera> {
        self.scene.presented_camera()
    }

    /// Distinct part geometries the body layer is retaining. What a producer's
    /// suspension keeps.
    pub fn cached_bodies(&self) -> usize {
        self.scene.cached_bodies()
    }

    /// Which way this section is looking. The receipt names it, so a capture
    /// can be told apart from the next one.
    pub fn mode(&self) -> CameraMode {
        self.mode
    }

    /// Display-encoded sRGB bytes in an unorm view, for a document compositor
    /// that samples encoded values directly. Alpha is opaque. The caller must
    /// submit the section's encoder before staging this same-device image.
    pub fn encoded_view(&self) -> &wgpu::TextureView {
        self.scene.encoded_view()
    }

    /// Replaces the complete bounded presentation list. Invalid input leaves
    /// the previous list intact. Empty input removes all glyphs. The batch
    /// itself is the scene's, drawn against the depth it already wrote.
    pub fn set_glyphs(&mut self, glyphs: Vec<SpatialGlyph>) -> Result<(), String> {
        self.scene.set_glyphs(glyphs)
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

    /// Most recent Section encode's terrain work. None means terrain was
    /// skipped or the encode failed, never a retained prior terrain receipt.
    /// Encoding counters do not assert queue completion or visible pixels.
    pub fn terrain_diagnostics(&self) -> Option<mesocosm_lens::BrickDiagnostics> {
        self.scene.terrain_diagnostics()
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
    ///
    /// The terrarium's filtered map is rebuilt here, because only the host
    /// knows which bricks its cutaway policy hides; everything past that is
    /// the scene's, handed one [`SceneFrame`].
    pub fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        frame: SectionFrame<'_>,
    ) -> Result<(), String> {
        let rebuilt = match &mut self.terrarium {
            Some(view) => view.refresh(frame.ground, self.mode)?,
            None => None,
        };
        let ground_terrain = GroundTerrain(frame.ground);
        let host_terrain = HostTerrain::new(frame.ground.revision(), rebuilt);
        // The terrarium's map is the host's to rebuild; a plain section's is
        // refreshed from the ground's own dirty bricks.
        let terrain: &dyn TerrainSource = if self.terrarium.is_some() {
            &host_terrain
        } else {
            &ground_terrain
        };
        let camera = self.view(frame.centre);
        let voxels = self.body_mode == BodyMode::Voxels;
        // The mosaics outlive the scene bodies that borrow them.
        let materials = if voxels {
            self.scene_materials(frame.world, camera.window())
        } else {
            Vec::new()
        };
        let scene_bodies = if voxels {
            self.scene_bodies(frame.world, &materials)
        } else {
            Vec::new()
        };
        let (grade, appearance, budget) = (self.grade, self.terrain_appearance, self.body_budget);
        let Self {
            scene, host_bodies, ..
        } = self;
        let mut host = SectionHost {
            bodies: host_bodies,
            world: frame.world,
        };
        scene.render(
            encoder,
            SceneFrame {
                camera,
                bodies: &scene_bodies,
                volumes: SceneVolumes::Voxels(frame.volumes),
                terrain: Some(terrain),
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

    /// The completed same-device texture imported by Netrender's frame graph.
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
        self.scene.capture(overlay)
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
        Ok(())
    }
}

/// The host policy one frame calls back into: Mesocosm's capsule stand-in for
/// a body that would not project, and the counters only a world can supply.
struct SectionHost<'a> {
    bodies: &'a mut bodies::HostBodies,
    world: &'a World,
}

impl isometer::SceneHost for SectionHost<'_> {
    fn begin(&mut self) {
        self.bodies.fallback.clear();
        self.bodies.played_fallback = None;
    }

    fn fallback(&mut self, body: &isometer::SceneBody<'_>, stats: &mut BodyFrameStats) {
        self.bodies.add_fallback(body, stats);
    }

    fn prepared(&mut self, layer: &mut isometer::BodyLayer) {
        layer.stats.body_scale = self.bodies.scale;
        // The scene carries neutral material channels; which of them is the
        // expressed gland is Mesocosm's vocabulary, so the count is read back
        // here rather than named inside the scene.
        let secrete = materials::channel(mesocosm_core::process::Process::Secrete);
        layer.stats.secretory_parts = layer
            .drawn_materials()
            .map(|materials| {
                materials
                    .iter()
                    .filter(|material| material.material == secrete)
                    .count()
            })
            .sum();
        // The scene knows the documents, not which of them belong to a dead
        // organism, so the carcass count is read back here.
        let world = self.world;
        layer.stats.carcasses = layer
            .drawn()
            .filter(|subject| {
                let id = bodies::organism_of(*subject);
                world
                    .organisms
                    .iter()
                    .any(|organism| organism.id == id && !organism.is_alive())
            })
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

/// Where the slab sits: on the controlled critter, plus the pan, **clamped so
/// the frame never dips below bedrock**.
///
/// The z component takes no pan, because the section's whole claim is that it
/// cuts through whoever is being played.
///
/// The clamp is the companion ruled with the half-height (2026-08-29): the
/// world's lowest brick is y = 0, so a frame centred below `half_height` shows
/// void along the bottom, which reads as a slab floating rather than a
/// terrarium. Past roughly 24 the shipped follow centre does exactly that.
/// It is presentation, like the framing — it moves no intent, so it cannot
/// reach the trace.
///
/// The floor is the camera's own vertical reach rather than the half-height
/// (DC4): a tilted section leans its slab depth into the vertical and frames
/// about a voxel more than a level one, so reading the half-height would let
/// `oblique` show exactly the strip of void the clamp exists to refuse. The
/// two level modes are unchanged, to the bit.
pub fn centre_on(at: [i32; 3], pan: Pan, half_height: f32, mode: CameraMode) -> [f32; 3] {
    let floor = mode.vertical_half(half_height_or_default(half_height));
    [
        at[0] as f32 + pan.x,
        (at[1] as f32 + pan.y).max(floor),
        at[2] as f32,
    ]
}

#[cfg(test)]
mod depth_tests;
#[cfg(test)]
mod inspection_tests;
#[cfg(test)]
mod query_tests;
#[cfg(test)]
mod tests;

// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! One played `GameState` as an [`isometer::SceneSource`].
//!
//! Everything device-side — the tracer, the body raster, the shared depth
//! attachment, the skip, the queries — is the shared crate's. What lives here
//! is the translation: which subjects are drawn, where their anatomies stand,
//! how the camera preset becomes a [`SlabCamera`], and where the terrain comes
//! from.

use isometer::core::{BodyDocument, PartId};
use isometer::lens::{BrickMap, Grade};
use isometer::{
    BodyFrameStats, BodySignature, DeclaredExtentVolumes, FrameRequest, GroundTerrain, Pose, Scene,
    SceneBody, SceneFrame, SceneHost, SceneSignature, SceneSource, SceneVolumes, SlabCamera,
    SubjectKey, TerrainSource,
};
use eponym_identity::SubjectId;
use eponym_world::{AnatomyRecord, GameState, MOTION_SCALE, MotionPose};

use super::handle::{SceneHandle, SceneModel};

/// Palette depth of the retro grade the terrain is shaded with.
const PALETTE: u32 = 3;
/// Bodies one frame may project. The fixture worlds draw a handful; the cap
/// exists so a populated settlement degrades by omission rather than by cost.
const BODY_BUDGET: usize = 256;

/// Eponym keeps no host presentation beside the scene: no capsule roster, no
/// substitute for a body that would not project, no second counter set.
struct PlainHost;
impl SceneHost for PlainHost {}

/// The shared session handle as a scene source.
///
/// Cheap to clone and always the same session: the model behind it is the one
/// the DOM panels and the keyboard path read.
pub struct SceneModelSource {
    handle: SceneHandle,
}

impl SceneModelSource {
    pub fn new(handle: SceneHandle) -> Self {
        Self { handle }
    }

    pub fn handle(&self) -> &SceneHandle {
        &self.handle
    }

    /// The last draw's body receipts, including whether static geometry
    /// uploaded.
    pub fn body_stats(&self) -> BodyFrameStats {
        self.handle
            .borrow()
            .scene
            .as_ref()
            .map(|scene| scene.bodies().stats.clone())
            .unwrap_or_default()
    }

    /// Intact parts of a subject's current anatomy, in document order.
    pub fn drawn_parts(&self, subject: SubjectId) -> Vec<PartId> {
        let model = self.handle.borrow();
        anatomy(model.game(), subject)
            .map(|record| record.document.living().map(|part| part.id).collect())
            .unwrap_or_default()
    }

    /// World bounds of one drawn part, posed exactly as the last frame posed
    /// it.
    pub fn part_bounds(&self, subject: SubjectId, part: PartId) -> Option<([f32; 3], [f32; 3])> {
        self.handle
            .borrow()
            .scene
            .as_ref()?
            .part_bounds(SubjectKey(subject.0), part)
    }

    /// World bounds of every drawn part of one subject. `None` when the last
    /// frame did not draw it.
    pub fn body_bounds(&self, subject: SubjectId) -> Option<([f32; 3], [f32; 3])> {
        let mut bounds: Option<([f32; 3], [f32; 3])> = None;
        for part in self.drawn_parts(subject) {
            let Some((min, max)) = self.part_bounds(subject, part) else {
                continue;
            };
            bounds = Some(match bounds {
                None => (min, max),
                Some((low, high)) => (
                    [0, 1, 2].map(|i| low[i].min(min[i])),
                    [0, 1, 2].map(|i| high[i].max(max[i])),
                ),
            });
        }
        bounds
    }

    /// Nearest drawn body surface along the ray through `ndc`.
    ///
    /// Unlike the retired `pick_body_ignoring_terrain`, the shared query does
    /// consult the terrain when the frame drew any: a body standing behind a
    /// ridge is no longer picked through it.
    pub fn pick_body(&self, ndc: [f32; 2]) -> Option<(SubjectId, PartId)> {
        let model = self.handle.borrow();
        let pick = model.scene.as_ref()?.pick_ndc(ndc).ok().flatten()?;
        Some((SubjectId(pick.address.subject.0), pick.address.part))
    }

    /// The last completed frame read back as RGBA8, for a caller asserting on
    /// pixels.
    pub fn capture(&self) -> Option<(u32, u32, Vec<u8>)> {
        self.handle.borrow().scene.as_ref()?.capture(|_, _, _| {})
    }
}

impl SceneSource for SceneModelSource {
    fn inputs(&mut self, request: &FrameRequest<'_>) -> Result<SceneSignature, String> {
        self.handle.borrow().inputs(request)
    }

    fn frame(&mut self, request: &FrameRequest<'_>) -> Result<Option<wgpu::TextureView>, String> {
        self.handle.borrow_mut().frame(request)
    }

    fn presented_camera(&self) -> Option<SlabCamera> {
        self.handle.borrow().scene.as_ref()?.presented_camera()
    }

    fn cached_bodies(&self) -> usize {
        self.handle
            .borrow()
            .scene
            .as_ref()
            .map_or(0, Scene::cached_bodies)
    }

    /// Suspension keeps the scene, and with it the projected geometry: the
    /// producer's own banked signature is what makes the next request redraw.
    fn suspend(&mut self) {}

    /// Retirement is this host's whole GPU release.
    fn retire(&mut self) {
        let mut model = self.handle.borrow_mut();
        model.scene = None;
        model.scene_size = [0; 2];
        model.terrain_map_revision = None;
    }
}

/// The anatomy a frame draws: the current one, falling back to the last
/// admitted record when injury has made it stale. An out-of-date body is a
/// truer picture than a missing one.
fn anatomy(game: &GameState, subject: SubjectId) -> Option<&AnatomyRecord> {
    game.current_anatomy(subject)
        .ok()
        .or_else(|| game.anatomies().get(subject))
}

/// Every living subject that has both an anatomy and a place to stand.
fn drawable(game: &GameState) -> Vec<(SubjectId, &AnatomyRecord, MotionPose)> {
    let subjects: Vec<_> = game
        .bodies()
        .all()
        .filter(|body| body.alive())
        .map(|body| body.subject)
        .collect();
    let mut out = Vec::with_capacity(subjects.len());
    for subject in subjects {
        let Some(record) = anatomy(game, subject) else {
            continue;
        };
        let Some(pose) = game
            .pose(subject)
            .or_else(|| game.movement().position(subject).map(MotionPose::at_cell))
        else {
            continue;
        };
        out.push((subject, record, pose));
    }
    out
}

/// Where a document's boxes stand for one motion pose.
///
/// The declared boxes are stood on the pose's feet and centred on its column,
/// so the picture sits where the solver put the body. The vertical half of that
/// is `grounded` on the [`SceneBody`], which stands the document's own floor on
/// `position[1]`; the horizontal centring has no shared spelling and is applied
/// to the position itself.
fn scene_pose(document: &BodyDocument, pose: MotionPose, scale: f32) -> Pose {
    let feet = [0, 1, 2].map(|i| pose.position[i] as f32 / MOTION_SCALE as f32);
    let aabb = document.aabb();
    let centred = |axis: usize| feet[axis] - (aabb.min[axis] + aabb.max[axis]) as f32 * 0.5 * scale;
    Pose {
        position: [centred(0), feet[1], centred(2)],
        yaw_radians: 0.0,
    }
}

impl SceneModel {
    /// The tint one subject is drawn in.
    fn tint(&self, subject: SubjectId) -> [f32; 3] {
        if subject == self.played() {
            self.appearance.played
        } else {
            self.appearance.other
        }
    }

    /// Everything the frame this request would draw depends on. No projection
    /// and no device work: the walk is the same one [`Self::frame`] makes.
    pub(super) fn inputs(&self, request: &FrameRequest<'_>) -> Result<SceneSignature, String> {
        let centre = self.follow_centre().map_err(|error| format!("{error:?}"))?;
        let camera = self
            .camera
            .camera(centre, request.aspect)
            .ok_or("invalid scene camera")?;
        let game = self.game();
        let scale = self.appearance.scale;
        let bodies = drawable(game)
            .into_iter()
            .map(|(subject, record, pose)| BodySignature {
                subject: SubjectKey(subject.0),
                revision: record.revision.0,
                pose: scene_pose(&record.document, pose, scale),
                scale,
                grounded: true,
                tint: self.tint(subject),
                always_visible: subject == self.played(),
            })
            .collect();
        Ok(SceneSignature {
            size: request.size,
            render_scale: request.render_scale,
            camera: Some(camera),
            terrain_revision: self.terrain.then(|| game.world().ground().revision()),
            bodies,
            // The framed subject: the camera follows it, and which body wears
            // the played tint turns on it.
            host: vec![self.played().0],
        })
    }

    /// Encodes one frame into the scene's own targets and answers the view to
    /// present.
    pub(super) fn frame(
        &mut self,
        request: &FrameRequest<'_>,
    ) -> Result<Option<wgpu::TextureView>, String> {
        let centre = self.follow_centre().map_err(|error| format!("{error:?}"))?;
        let camera = self
            .camera
            .camera(centre, request.aspect)
            .ok_or("invalid scene camera")?;
        let terrain_on = self.terrain;
        let material = self.appearance.material;
        let scale = self.appearance.scale;
        let played = self.played();
        let tints = (self.appearance.played, self.appearance.other);
        self.ensure_scene(request)?;

        let SceneModel {
            held,
            scene,
            terrain_map_revision,
            ..
        } = self;
        let scene = scene.as_mut().ok_or("the scene was not built")?;
        let game = held.session().game();
        let ground = game.world().ground();
        if terrain_on && *terrain_map_revision != Some(ground.revision()) {
            scene.set_terrain_map(
                BrickMap::from_ground(ground).map_err(|error| format!("brick map: {error}"))?,
            );
            *terrain_map_revision = Some(ground.revision());
        }

        let drawn = drawable(game);
        let volumes = DeclaredExtentVolumes::from_documents(
            drawn.iter().map(|(_, record, _)| &record.document),
            material,
        );
        let bodies: Vec<SceneBody<'_>> = drawn
            .iter()
            .map(|(subject, record, pose)| SceneBody {
                subject: SubjectKey(subject.0),
                document: &record.document,
                pose: scene_pose(&record.document, *pose, scale),
                scale,
                grounded: true,
                tint: if *subject == played { tints.0 } else { tints.1 },
                materials: &[],
                always_visible: *subject == played,
            })
            .collect();
        let terrain = terrain_on.then(|| GroundTerrain(ground));

        let mut encoder = request
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("paredros scene"),
            });
        scene.render(
            &mut encoder,
            SceneFrame {
                camera,
                bodies: &bodies,
                volumes: SceneVolumes::DeclaredSolid(&volumes),
                terrain: terrain.as_ref().map(|source| source as &dyn TerrainSource),
                dirty: &[],
                grade: Grade::retro(PALETTE),
                terrain_appearance: None,
                body_budget: BODY_BUDGET,
                capsules: None,
            },
            &mut PlainHost,
        )?;
        request.queue.submit(Some(encoder.finish()));
        // The encoded twin, not the sRGB-tagged one: the leaf declares
        // `SCENE_ENCODING`, so the compositor samples these bytes directly.
        Ok(Some(scene.encoded_view().clone()))
    }

    /// Builds the scene on the first frame and resizes it afterwards.
    fn ensure_scene(&mut self, request: &FrameRequest<'_>) -> Result<(), String> {
        let size = [request.size[0].max(1), request.size[1].max(1)];
        match &mut self.scene {
            None => {
                self.scene = Some(Scene::new(
                    request.device.clone(),
                    request.queue.clone(),
                    size[0],
                    size[1],
                )?);
                self.terrain_map_revision = None;
            },
            Some(scene) if self.scene_size != size => scene.resize(size[0], size[1]),
            Some(_) => {},
        }
        self.scene_size = size;
        Ok(())
    }
}

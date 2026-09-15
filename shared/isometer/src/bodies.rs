// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Addressed voxel bodies in the same camera and depth target as the terrain.
//!
//! The inputs are product-neutral by construction: a [`SceneBody`] carries a
//! body document, where it stands, how big it is and what colour it takes, and
//! nothing that could name a world. A host keeps its own organisms, subjects or
//! tokens behind a [`SubjectKey`] and hands this layer one slice per frame.

use isometer_core::{BodyDocument, PartId};
use mesocosm_mesh::{BodyDependencyRevision, BodyMesh, LiveBodyProjector, MeshError, VolumeMap};
use mesocosm_render::live_body::{LiveBody, LiveBodyRenderer};

use crate::camera::{SlabCamera, SlabWindow};
use crate::volumes::DeclaredExtentVolumes;

mod placement;
pub use placement::BodyFrameStats;
pub(crate) use placement::{body_origin, depth_target, distance, intersects};

/// A host's identity for one drawn body.
///
/// 64 bits, so Paredros's `SubjectId(u64)` fits without narrowing and
/// Mesocosm's `OrganismId(u32)` widens losslessly. The scene never interprets
/// it; it only carries it back out of a pick.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SubjectKey(pub u64);

/// A drawn part, plus the revision that expires the address.
///
/// The revision makes an address stale when attachment geometry changes. It is
/// presentation state, never a world address or a trace input.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PartAddress {
    pub subject: SubjectKey,
    pub part: PartId,
    pub revision: BodyDependencyRevision,
}

/// Where a body stands and which way it faces.
///
/// `yaw_radians` is the continuous parent yaw: a host's own presentation
/// rotation, not the document's quarter-turn attachment yaw.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Pose {
    pub position: [f32; 3],
    pub yaw_radians: f32,
}

/// One body for one frame. No product world appears here.
#[derive(Clone, Copy, Debug)]
pub struct SceneBody<'a> {
    pub subject: SubjectKey,
    pub document: &'a BodyDocument,
    pub pose: Pose,
    pub scale: f32,
    /// Stand the document's own floor on `pose.position[1]`.
    pub grounded: bool,
    pub tint: [f32; 3],
    /// Product-projected per-part expression; empty is legal.
    pub materials: &'a [mesocosm_render::PartMaterial],
    /// In shot regardless of the window, and first in draw order. Mesocosm's
    /// controlled critter; a host with no such body leaves this false.
    pub always_visible: bool,
}

impl SceneBody<'_> {
    /// Where this body's mesh origin lands in the world: its pose, with the
    /// document's own floor stood on `pose.position[1]` when grounded.
    pub fn origin(&self) -> [f32; 3] {
        body_origin(self)
    }
}

/// Where a frame's part geometry is resolved from.
///
/// An enum rather than a `&dyn VolumeSource`, because `mesh_body` and
/// `project_body` take `&impl VolumeSource`; see the extraction plan, risk 2.
#[derive(Clone, Copy, Debug)]
pub enum SceneVolumes<'a> {
    /// Authored voxel data, addressed by [`crate::VolumeRef`].
    Voxels(&'a VolumeMap),
    /// The declared-extent boxes, for anatomies that carry no voxel data.
    DeclaredSolid(&'a DeclaredExtentVolumes),
}

/// One projected body held for the next draw and the next query.
struct PlacedBody {
    subject: SubjectKey,
    revision: BodyDependencyRevision,
    mesh: BodyMesh,
    materials: Vec<mesocosm_render::PartMaterial>,
    origin: [f32; 3],
    scale: f32,
    yaw_radians: f32,
    tint: [f32; 3],
}

impl PlacedBody {
    fn live(&self) -> LiveBody<'_> {
        LiveBody {
            mesh: &self.mesh,
            materials: &self.materials,
            origin: self.origin,
            scale: self.scale,
            yaw_radians: self.yaw_radians,
            tint: self.tint,
            focused: false,
            selected_part: None,
        }
    }
}

/// The scene's body pass: projection, instance draw, and the part queries that
/// read the same poses the draw used.
pub struct BodyLayer {
    projector: LiveBodyProjector,
    renderer: LiveBodyRenderer,
    placed: Vec<PlacedBody>,
    pub stats: BodyFrameStats,
    pub budget: usize,
    /// Draw only the always-visible body, with no terrain behind it.
    pub isolated: bool,
    pub preview_depth: f32,
    focus_subject: Option<SubjectKey>,
    selected: Option<PartAddress>,
    depth: wgpu::Texture,
    pub depth_view: wgpu::TextureView,
}

impl BodyLayer {
    /// `preview_depth` is the host's ordinary slab depth: the isolated preview
    /// starts at the depth the shipped camera already uses.
    pub fn new(device: &wgpu::Device, width: u32, height: u32, preview_depth: f32) -> Self {
        let (depth, depth_view) = depth_target(device, width, height);
        Self {
            projector: LiveBodyProjector::default(),
            renderer: LiveBodyRenderer::new(device, mesocosm_lens::FRAME_FORMAT, 256),
            placed: Vec::new(),
            stats: BodyFrameStats::default(),
            budget: 1,
            isolated: false,
            preview_depth,
            focus_subject: None,
            selected: None,
            depth,
            depth_view,
        }
    }

    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        (self.depth, self.depth_view) = depth_target(device, width, height);
    }

    /// The depth texture itself, for a host that binds it as a read target.
    pub fn depth_texture(&self) -> &wgpu::Texture {
        &self.depth
    }

    /// Distinct part geometries the projector is retaining. One per addressed
    /// [`crate::VolumeRef`] rather than one per body, because that is the
    /// cache a body's parts are actually resolved out of; it is what survives
    /// a producer's suspension.
    pub fn cached_bodies(&self) -> usize {
        self.projector.cached_mesh_count()
    }

    pub fn clear_inspection(&mut self) {
        self.placed.clear();
        self.selected = None;
    }

    /// Subjects the last prepare actually projected, in draw order.
    pub fn drawn(&self) -> impl Iterator<Item = SubjectKey> + '_ {
        self.placed.iter().map(|body| body.subject)
    }

    /// The material slice each projected body kept, in the same draw order as
    /// [`BodyLayer::drawn`]. A host reads this back for the counters whose
    /// meaning is its own vocabulary and not the scene's.
    pub fn drawn_materials(&self) -> impl Iterator<Item = &[mesocosm_render::PartMaterial]> + '_ {
        self.placed.iter().map(|body| body.materials.as_slice())
    }

    /// Drops every projected body, returning the subjects that had been drawn
    /// so a host can replace them with its own fallback presentation.
    ///
    /// The voxel counters clear with them; the host's own fallback counters do
    /// not, because the host is what writes those.
    pub fn take_drawn(&mut self) -> Vec<SubjectKey> {
        let subjects: Vec<_> = self.drawn().collect();
        self.stats.voxel_bodies = 0;
        self.stats.voxel_parts = 0;
        self.stats.material_parts = 0;
        self.stats.secretory_parts = 0;
        self.stats.projection_failures += 1;
        self.placed.clear();
        subjects
    }

    /// Exact transformed quad bounds, before viewport and cutaway clipping.
    /// Uses the same pose the next body draw would.
    pub fn presentation_bounds(
        &mut self,
        body: &SceneBody<'_>,
        volumes: SceneVolumes<'_>,
    ) -> Result<Option<([f32; 3], [f32; 3])>, String> {
        let (mesh, _) = self
            .project(body, volumes)
            .map_err(|error| format!("body projection: {error:?}"))?;
        mesocosm_render::live_body::body_bounds(posed(body, &mesh))
            .map_err(|error| format!("body bounds: {error:?}"))
    }

    /// World bounds of one drawn part, under the same part matrix the draw
    /// used — parent rotation, pivot and continuous yaw included.
    ///
    /// Read off the meshed faces rather than the declared volume box, so a
    /// sparse anatomy reports what it actually occupies.
    pub fn part_bounds(&self, subject: SubjectKey, part: PartId) -> Option<([f32; 3], [f32; 3])> {
        let placed = self.placed.iter().find(|body| body.subject == subject)?;
        let placement = placed.mesh.placements.iter().find(|p| p.part == part)?;
        let quads = placed.mesh.mesh_for(placement.volume)?.quads.len();
        let live = placed.live();
        let mut min = [f32::INFINITY; 3];
        let mut max = [f32::NEG_INFINITY; 3];
        for index in 0..quads {
            for corner in mesocosm_render::live_body::posed_quad(live, part, index).ok()? {
                for axis in 0..3 {
                    min[axis] = min[axis].min(corner[axis]);
                    max[axis] = max[axis].max(corner[axis]);
                }
            }
        }
        (min[0] <= max[0]).then_some((min, max))
    }

    pub fn pick(
        &self,
        origin: [f32; 3],
        direction: [f32; 3],
        far: f32,
        clip: mesocosm_render::ClipSlab,
    ) -> Result<
        Option<(PartAddress, mesocosm_render::live_body::BodyHit)>,
        mesocosm_render::live_body::BodyQueryError,
    > {
        let bodies: Vec<_> = self.placed.iter().map(PlacedBody::live).collect();
        mesocosm_render::live_body::pick_bodies(&bodies, origin, direction, far, Some(clip)).map(
            |hit| {
                hit.map(|hit| {
                    let body = &self.placed[hit.body_index];
                    (
                        PartAddress {
                            subject: body.subject,
                            part: hit.part,
                            revision: body.revision,
                        },
                        hit,
                    )
                })
            },
        )
    }

    /// Walks the parts of the last successful draw for `subject`.
    pub fn select_part(
        &self,
        subject: SubjectKey,
        current: Option<PartAddress>,
        backwards: bool,
    ) -> Option<PartAddress> {
        let body = self.placed.iter().find(|body| body.subject == subject)?;
        let parts: Vec<_> = body
            .mesh
            .placements
            .iter()
            .filter(|part| drawable(&body.mesh, part.part))
            .collect();
        if parts.is_empty() {
            return None;
        }
        let current = current.filter(|address| address.subject == subject);
        let index = current
            .and_then(|address| parts.iter().position(|part| part.part == address.part))
            .map(|index| {
                if backwards {
                    index.checked_sub(1).unwrap_or(parts.len() - 1)
                } else {
                    (index + 1) % parts.len()
                }
            })
            .unwrap_or_else(|| if backwards { parts.len() - 1 } else { 0 });
        Some(PartAddress {
            subject,
            part: parts[index].part,
            revision: body.revision,
        })
    }

    /// Confirms a part address is still both drawn and geometrically current.
    /// The caller already holds the body, so nothing is looked up here.
    pub fn validate_address(
        &mut self,
        address: PartAddress,
        body: &SceneBody<'_>,
        volumes: SceneVolumes<'_>,
    ) -> bool {
        if body.subject != address.subject {
            return false;
        }
        let Some(revision) =
            self.placed
                .iter()
                .find(|placed| {
                    placed.subject == address.subject
                        && placed.revision == address.revision
                        && placed.mesh.placements.iter().any(|part| {
                            part.part == address.part && drawable(&placed.mesh, part.part)
                        })
                })
                .map(|placed| placed.revision)
        else {
            return false;
        };
        self.project(body, volumes).is_ok_and(|(mesh, current)| {
            current == revision
                && mesh
                    .placements
                    .iter()
                    .any(|part| part.part == address.part && drawable(&mesh, part.part))
        })
    }

    pub fn set_focus(
        &mut self,
        subject: Option<SubjectKey>,
        selected: Option<PartAddress>,
    ) -> bool {
        let changed = self.focus_subject != subject || self.selected != selected;
        self.focus_subject = subject;
        self.selected = selected;
        changed
    }

    pub(crate) fn project(
        &mut self,
        body: &SceneBody<'_>,
        volumes: SceneVolumes<'_>,
    ) -> Result<(BodyMesh, BodyDependencyRevision), MeshError> {
        match volumes {
            SceneVolumes::Voxels(map) => self.projector.project_body(body.document, map),
            SceneVolumes::DeclaredSolid(solid) => self.projector.project_body(body.document, solid),
        }
    }

    /// Projects and culls one frame's bodies, nearest first with the
    /// always-visible body ahead of the rest.
    ///
    /// `fallback` is called for each body whose projection or bounds failed,
    /// with the frame's counters, so a host can substitute its own cheaper
    /// presentation and still spend the frame's budget on it.
    pub fn prepare(
        &mut self,
        bodies: &[SceneBody<'_>],
        volumes: SceneVolumes<'_>,
        window: SlabWindow,
        mut fallback: impl FnMut(&SceneBody<'_>, &mut BodyFrameStats),
    ) {
        self.placed.clear();
        self.stats = BodyFrameStats::default();
        let mut candidates: Vec<_> = bodies
            .iter()
            .filter(|body| !self.isolated || body.always_visible)
            .filter(|body| body.document.living().next().is_some())
            .collect();
        candidates.sort_by(|a, b| {
            (!a.always_visible)
                .cmp(&!b.always_visible)
                .then_with(|| {
                    distance(a.pose.position, window.centre)
                        .total_cmp(&distance(b.pose.position, window.centre))
                })
                .then(a.subject.cmp(&b.subject))
        });
        self.stats.candidates = candidates.len();
        let candidate_count = candidates.len();
        for (index, body) in candidates.into_iter().enumerate() {
            if self.placed.len() + self.stats.fallback_bodies >= self.budget {
                self.stats.omitted_bodies += candidate_count - index;
                break;
            }
            match self.project(body, volumes) {
                Ok((mesh, revision)) => {
                    let mut placed = PlacedBody {
                        subject: body.subject,
                        revision,
                        mesh,
                        materials: Vec::new(),
                        origin: body_origin(body),
                        scale: body.scale,
                        yaw_radians: body.pose.yaw_radians,
                        tint: body.tint,
                    };
                    match mesocosm_render::live_body::body_bounds(placed.live()) {
                        Ok(Some(bounds)) if body.always_visible || intersects(bounds, window) => {},
                        Ok(_) => {
                            self.stats.candidates -= 1;
                            continue;
                        },
                        Err(error) => {
                            self.stats.last_error = Some(format!("body bounds: {error:?}"));
                            self.stats.projection_failures += 1;
                            fallback(body, &mut self.stats);
                            continue;
                        },
                    }
                    self.stats.material_parts += body
                        .materials
                        .iter()
                        .map(|m| m.part)
                        .collect::<std::collections::BTreeSet<_>>()
                        .len();
                    self.stats.controlled_drawn |= body.always_visible;
                    self.stats.voxel_parts += placed.mesh.placement_count();
                    self.stats.voxel_bodies += 1;
                    placed.materials = body.materials.to_vec();
                    self.placed.push(placed);
                },
                Err(error) => {
                    self.stats.last_error = Some(format!("critter {}: {error:?}", body.subject.0));
                    self.stats.projection_failures += 1;
                    self.stats.missing_volumes +=
                        usize::from(matches!(error, MeshError::MissingVolume { .. }));
                    fallback(body, &mut self.stats);
                },
            }
        }
    }

    /// Draws the prepared bodies into `colour` and the layer's own depth.
    /// Returns the `clip_from_world` the terrain join must reuse.
    pub fn draw(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        colour: &wgpu::TextureView,
        camera: SlabCamera,
    ) -> Result<[[f32; 4]; 4], String> {
        let matrix = camera.clip_from_world();
        let bodies: Vec<_> = self
            .placed
            .iter()
            .map(|body| {
                let mut live = body.live();
                live.focused = self.focus_subject == Some(body.subject);
                live.selected_part = self.selected.and_then(|address| {
                    (address.subject == body.subject && address.revision == body.revision)
                        .then_some(address.part)
                });
                live
            })
            .collect();
        let stats = self
            .renderer
            .draw(
                device,
                queue,
                encoder,
                colour,
                &self.depth_view,
                matrix,
                Some(camera.clip()),
                &bodies,
            )
            .map_err(|error| format!("voxel bodies: {error:?}"))?;
        self.stats.mesh_builds = stats.mesh_builds;
        self.stats.mesh_upload_bytes = stats.mesh_upload_bytes as u64;
        self.stats.instance_upload_bytes = stats.instance_upload_bytes as u64;
        self.stats.frame_upload_bytes = stats.frame_upload_bytes as u64;
        self.stats.draw_parts = stats.draw_parts;
        Ok(matrix)
    }
}

/// The body as the next draw would pose it.
pub(crate) fn posed<'a>(body: &SceneBody<'_>, mesh: &'a BodyMesh) -> LiveBody<'a> {
    let mut live = LiveBody::new(mesh, body_origin(body));
    live.scale = body.scale;
    live.yaw_radians = body.pose.yaw_radians;
    live
}

/// Whether a part has any meshed face to draw, select or attach to.
pub(crate) fn drawable(mesh: &BodyMesh, part: PartId) -> bool {
    mesh.placements
        .iter()
        .find(|placement| placement.part == part)
        .and_then(|placement| mesh.mesh_for(placement.volume))
        .is_some_and(|mesh| !mesh.quads.is_empty())
}

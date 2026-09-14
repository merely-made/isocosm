// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Addressed voxel bodies in the same camera and depth target as the terrain.

use mesocosm_core::{Organism, OrganismId, World};
use mesocosm_lens::{BodyLensProjection, BodyPlacement, CritterPose, MAX_ROSTER};
use mesocosm_mesh::{LiveBodyProjection, LiveBodyProjector, VolumeMap};
use mesocosm_render::live_body::{LiveBody, LiveBodyRenderer};
use serde::Serialize;
use std::collections::BTreeMap;

use super::{BodySelection, SlabWindow};
#[cfg(test)]
use super::{CameraMode, SLAB_DEPTH};

#[path = "anchors.rs"]
pub(super) mod anchors;
#[path = "appearance.rs"]
mod appearance;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BodyMode {
    Capsules,
    #[default]
    Voxels,
}

impl BodyMode {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "capsules" => Some(Self::Capsules),
            "voxels" => Some(Self::Voxels),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Capsules => "capsules",
            Self::Voxels => "voxels",
        }
    }
}

pub const DEFAULT_BODY_BUDGET: usize = MAX_ROSTER + 1;

#[derive(Clone, Debug, Default, Serialize)]
pub struct BodyFrameStats {
    pub material_parts: usize,
    pub secretory_parts: usize,
    pub candidates: usize,
    pub body_scale: f32,
    pub voxel_bodies: usize,
    pub voxel_parts: usize,
    pub carcasses: usize,
    pub fallback_bodies: usize,
    pub fallback_parts_dropped: usize,
    pub omitted_bodies: usize,
    pub missing_volumes: usize,
    pub projection_failures: usize,
    pub last_error: Option<String>,
    pub controlled_drawn: bool,
    pub mesh_builds: usize,
    pub mesh_upload_bytes: u64,
    pub instance_upload_bytes: u64,
    pub frame_upload_bytes: u64,
    pub draw_parts: usize,
}

struct PlacedBody {
    projection: LiveBodyProjection,
    materials: Vec<mesocosm_render::PartMaterial>,
    origin: [f32; 3],
    scale: f32,
    yaw_radians: f32,
    tint: [f32; 3],
}

impl PlacedBody {
    fn live(&self) -> LiveBody<'_> {
        LiveBody {
            mesh: &self.projection.mesh,
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

pub(super) struct BodyLayer {
    projector: LiveBodyProjector,
    renderer: LiveBodyRenderer,
    placed: Vec<PlacedBody>,
    yaw: BTreeMap<OrganismId, f32>,
    tints: BTreeMap<OrganismId, [f32; 3]>,
    pub fallback: Vec<CritterPose>,
    pub played_fallback: Option<CritterPose>,
    pub stats: BodyFrameStats,
    pub budget: usize,
    pub scale: f32,
    pub ground_anatomy: bool,
    pub isolated: bool,
    pub preview_depth: f32,
    focus_subject: Option<OrganismId>,
    selected: Option<BodySelection>,
    depth: wgpu::Texture,
    pub depth_view: wgpu::TextureView,
}

impl BodyLayer {
    pub fn new(device: &wgpu::Device, width: u32, height: u32) -> Self {
        let (depth, depth_view) = depth_target(device, width, height);
        Self {
            projector: LiveBodyProjector::default(),
            renderer: LiveBodyRenderer::new(device, mesocosm_lens::FRAME_FORMAT, 256),
            placed: Vec::new(),
            yaw: BTreeMap::new(),
            tints: BTreeMap::new(),
            fallback: Vec::new(),
            played_fallback: None,
            stats: BodyFrameStats::default(),
            budget: DEFAULT_BODY_BUDGET,
            scale: 1.0,
            ground_anatomy: false,
            isolated: false,
            preview_depth: super::SLAB_DEPTH,
            focus_subject: None,
            selected: None,
            depth,
            depth_view,
        }
    }

    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        (self.depth, self.depth_view) = depth_target(device, width, height);
    }

    pub fn clear_inspection(&mut self) {
        self.placed.clear();
        self.selected = None;
    }

    pub fn yaw(&self, subject: OrganismId) -> f32 {
        self.yaw.get(&subject).copied().unwrap_or(0.0)
    }

    pub fn set_yaw(&mut self, subject: OrganismId, radians: f32) -> bool {
        if self.yaw(subject) == radians {
            return false;
        }
        if radians == 0.0 {
            self.yaw.remove(&subject);
        } else {
            self.yaw.insert(subject, radians);
        }
        true
    }

    pub fn presentation_bounds(
        &mut self,
        organism: &Organism,
        volumes: &VolumeMap,
    ) -> Result<Option<([f32; 3], [f32; 3])>, String> {
        let projection = self
            .projector
            .project(organism.id, organism.body(), volumes)
            .map_err(|error| format!("body projection: {error:?}"))?;
        let mut body = LiveBody::new(
            &projection.mesh,
            body_origin(organism, self.scale, self.ground_anatomy),
        );
        body.scale = self.scale;
        body.yaw_radians = self.yaw(organism.id);
        mesocosm_render::live_body::body_bounds(body)
            .map_err(|error| format!("body bounds: {error:?}"))
    }

    pub fn pick(
        &self,
        origin: [f32; 3],
        direction: [f32; 3],
        far: f32,
        clip: mesocosm_render::ClipSlab,
    ) -> Result<
        Option<(BodySelection, mesocosm_render::live_body::BodyHit)>,
        mesocosm_render::live_body::BodyQueryError,
    > {
        let bodies: Vec<_> = self.placed.iter().map(PlacedBody::live).collect();
        mesocosm_render::live_body::pick_bodies(&bodies, origin, direction, far, Some(clip)).map(
            |hit| {
                hit.map(|hit| {
                    let body = &self.placed[hit.body_index];
                    (
                        BodySelection {
                            organism: body.projection.organism,
                            part: hit.part,
                            revision: body.projection.revision,
                        },
                        hit,
                    )
                })
            },
        )
    }

    pub fn select_part(
        &self,
        subject: OrganismId,
        current: Option<BodySelection>,
        backwards: bool,
    ) -> Option<BodySelection> {
        let body = self
            .placed
            .iter()
            .find(|body| body.projection.organism == subject)?;
        let parts: Vec<_> = body
            .projection
            .mesh
            .placements
            .iter()
            .filter(|part| drawable(&body.projection, part.part))
            .collect();
        if parts.is_empty() {
            return None;
        }
        let current = current.filter(|selection| selection.organism == subject);
        let index = current
            .and_then(|selection| parts.iter().position(|part| part.part == selection.part))
            .map(|index| {
                if backwards {
                    index.checked_sub(1).unwrap_or(parts.len() - 1)
                } else {
                    (index + 1) % parts.len()
                }
            })
            .unwrap_or_else(|| if backwards { parts.len() - 1 } else { 0 });
        Some(BodySelection {
            organism: subject,
            part: parts[index].part,
            revision: body.projection.revision,
        })
    }

    pub fn validate_selection(
        &mut self,
        selection: BodySelection,
        world: &World,
        volumes: &VolumeMap,
    ) -> bool {
        let Some(revision) = self
            .placed
            .iter()
            .find(|body| {
                body.projection.organism == selection.organism
                    && body.projection.revision == selection.revision
                    && body.projection.mesh.placements.iter().any(|part| {
                        part.part == selection.part && drawable(&body.projection, part.part)
                    })
            })
            .map(|body| body.projection.revision)
        else {
            return false;
        };
        let Some(organism) = world
            .organisms
            .iter()
            .find(|organism| organism.id == selection.organism)
        else {
            return false;
        };
        self.projector
            .project(organism.id, organism.body(), volumes)
            .is_ok_and(|current| {
                current.revision == revision
                    && current
                        .mesh
                        .placements
                        .iter()
                        .any(|part| part.part == selection.part && drawable(&current, part.part))
            })
    }

    pub fn set_focus(
        &mut self,
        subject: Option<OrganismId>,
        selected: Option<BodySelection>,
    ) -> bool {
        let changed = self.focus_subject != subject || self.selected != selected;
        self.focus_subject = subject;
        self.selected = selected;
        changed
    }

    pub fn prepare(&mut self, world: &World, volumes: &VolumeMap, window: SlabWindow) {
        self.placed.clear();
        self.fallback.clear();
        self.played_fallback = None;
        self.stats = BodyFrameStats {
            body_scale: self.scale,
            ..Default::default()
        };
        let controlled = world.controlled_id();
        let mut candidates: Vec<_> = world
            .organisms
            .iter()
            .filter(|o| !self.isolated || Some(o.id) == controlled)
            .filter(|o| o.body().living().next().is_some())
            .collect();
        candidates.sort_by(|a, b| {
            let priority = |o: &Organism| Some(o.id) != controlled;
            priority(a)
                .cmp(&priority(b))
                .then_with(|| {
                    distance(a.position, window.centre)
                        .total_cmp(&distance(b.position, window.centre))
                })
                .then(a.id.cmp(&b.id))
        });
        self.stats.candidates = candidates.len();
        let candidate_count = candidates.len();
        for (index, organism) in candidates.into_iter().enumerate() {
            if self.placed.len() + self.stats.fallback_bodies >= self.budget {
                self.stats.omitted_bodies += candidate_count - index;
                break;
            }
            let tint = self.rendered_tint(organism);
            match self
                .projector
                .project(organism.id, organism.body(), volumes)
            {
                Ok(projection) => {
                    let mut body = PlacedBody {
                        projection,
                        materials: Vec::new(),
                        origin: body_origin(organism, self.scale, self.ground_anatomy),
                        scale: self.scale,
                        yaw_radians: self.yaw(organism.id),
                        tint,
                    };
                    match mesocosm_render::live_body::body_bounds(body.live()) {
                        Ok(Some(bounds))
                            if Some(organism.id) == controlled || intersects(bounds, window) => {},
                        Ok(_) => {
                            self.stats.candidates -= 1;
                            continue;
                        },
                        Err(error) => {
                            self.stats.last_error = Some(format!("body bounds: {error:?}"));
                            self.stats.projection_failures += 1;
                            self.add_fallback(organism, controlled == Some(organism.id), tint);
                            continue;
                        },
                    }
                    let materials = super::materials::project(&organism.phenotype, world.ruleset());
                    self.stats.material_parts += materials
                        .iter()
                        .map(|m| m.part)
                        .collect::<std::collections::BTreeSet<_>>()
                        .len();
                    self.stats.secretory_parts += materials
                        .iter()
                        .filter(|m| m.process == mesocosm_core::process::Process::Secrete)
                        .count();
                    self.stats.controlled_drawn |= Some(organism.id) == controlled;
                    self.stats.voxel_parts += body.projection.mesh.placement_count();
                    self.stats.voxel_bodies += 1;
                    self.stats.carcasses += usize::from(!organism.is_alive());
                    body.materials = materials;
                    self.placed.push(body);
                },
                Err(error) => {
                    self.stats.last_error = Some(format!("critter {}: {error:?}", organism.id.0));
                    self.stats.projection_failures += 1;
                    self.stats.missing_volumes += usize::from(matches!(
                        error,
                        mesocosm_mesh::MeshError::MissingVolume { .. }
                    ));
                    self.add_fallback(organism, controlled == Some(organism.id), tint);
                },
            }
        }
    }

    fn add_fallback(&mut self, organism: &Organism, controlled: bool, tint: [f32; 3]) {
        let at = body_origin(organism, self.scale, self.ground_anatomy);
        let placement = BodyPlacement {
            ground: [
                at[0],
                at[1] + organism.body().aabb().min[1] as f32 * self.scale,
                at[2],
            ],
            scale: self.scale,
            tint,
        };
        match BodyLensProjection::project_truncated(organism.body(), placement) {
            Ok((body, dropped)) if controlled || self.fallback.len() < MAX_ROSTER => {
                self.stats.fallback_bodies += 1;
                self.stats.controlled_drawn |= controlled;
                self.stats.fallback_parts_dropped += dropped;
                if !controlled {
                    self.stats.fallback_parts_dropped += body
                        .pose
                        .capsules
                        .len()
                        .saturating_sub(mesocosm_lens::MAX_ROSTER_CAPSULES);
                }
                if controlled {
                    self.played_fallback = Some(body.pose);
                } else {
                    self.fallback.push(body.pose);
                }
            },
            _ => self.stats.omitted_bodies += 1,
        }
    }

    pub fn draw(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        colour: &wgpu::TextureView,
        camera: super::view::View,
    ) -> Result<[[f32; 4]; 4], String> {
        let matrix = camera.clip_from_world();
        let bodies: Vec<_> = self
            .placed
            .iter()
            .map(|body| {
                let mut live = body.live();
                live.focused = self.focus_subject == Some(body.projection.organism);
                live.selected_part = self.selected.and_then(|selection| {
                    (selection.organism == body.projection.organism
                        && selection.revision == body.projection.revision)
                        .then_some(selection.part)
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

    pub fn fallback_all(&mut self, world: &World) {
        let subjects: Vec<_> = self
            .placed
            .iter()
            .map(|body| body.projection.organism)
            .collect();
        for subject in subjects {
            if let Some(organism) = world.organisms.iter().find(|o| o.id == subject) {
                self.add_fallback(
                    organism,
                    Some(subject) == world.controlled_id(),
                    self.rendered_tint(organism),
                );
            }
        }
        self.stats.voxel_bodies = 0;
        self.stats.voxel_parts = 0;
        self.stats.material_parts = 0;
        self.stats.secretory_parts = 0;
        self.stats.projection_failures += 1;
        self.placed.clear();
    }
}

fn drawable(projection: &LiveBodyProjection, part: mesocosm_core::PartId) -> bool {
    projection
        .mesh
        .placements
        .iter()
        .find(|placement| placement.part == part)
        .and_then(|placement| projection.mesh.mesh_for(placement.volume))
        .is_some_and(|mesh| !mesh.quads.is_empty())
}

fn distance(at: [i32; 3], centre: [f32; 3]) -> f32 {
    (0..3).map(|i| (at[i] as f32 - centre[i]).powi(2)).sum()
}

fn body_origin(organism: &Organism, scale: f32, grounded: bool) -> [f32; 3] {
    let mut origin = organism.position.map(|v| v as f32);
    if grounded {
        origin[1] -= organism.body().aabb().min[1] as f32 * scale;
    }
    origin
}

fn intersects((min, max): ([f32; 3], [f32; 3]), window: SlabWindow) -> bool {
    let middle = [0, 1, 2].map(|i| (min[i] + max[i]) * 0.5 - window.centre[i]);
    let half = [0, 1, 2].map(|i| (max[i] - min[i]) * 0.5);
    (0..3).all(|axis| {
        let extent: f32 = (0..3).map(|i| half[i] * window.axes[axis][i].abs()).sum();
        dot(middle, window.axes[axis]).abs() <= window.half[axis] + extent
    })
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a.into_iter().zip(b).map(|(a, b)| a * b).sum()
}

#[cfg(test)]
pub(super) fn clip_from_world(
    mode: CameraMode,
    centre: [f32; 3],
    half: f32,
    aspect: f32,
) -> [[f32; 4]; 4] {
    super::view::slab_camera(mode, centre, half, aspect).clip_from_world()
}

fn depth_target(
    device: &wgpu::Device,
    width: u32,
    height: u32,
) -> (wgpu::Texture, wgpu::TextureView) {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("section body and terrain depth"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    let view = texture.create_view(&Default::default());
    (texture, view)
}

#[cfg(test)]
#[path = "bodies_tests.rs"]
mod tests;

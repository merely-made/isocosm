// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Mesocosm's half of the body layer.
//!
//! The projection, the instance draw, the cull and the part queries are
//! `isometer`'s now. What stays here is everything that needs a Mesocosm
//! world to mean anything: which organism is controlled, what colour the
//! kingdom palette gives it, how the terrarium scales and grounds anatomies,
//! and the capsule roster a body falls back to when its voxels will not
//! project.

use mesocosm_core::{Organism, OrganismId, World};
use mesocosm_lens::{BodyLensProjection, BodyPlacement, CritterPose, MAX_ROSTER};
use mesocosm_render::PartMaterial;
use std::collections::BTreeMap;
use isometer::{BodyFrameStats, Pose, SceneBody, SlabWindow, SubjectKey};

#[cfg(test)]
use super::{CameraMode, SLAB_DEPTH};

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

/// The host's side of one body frame: the presentation facts isometer is
/// handed rather than the ones it derives.
pub(super) struct HostBodies {
    /// Terrarium body scale, applied to every drawn anatomy.
    pub scale: f32,
    /// Stand each anatomy's own floor on its world position.
    pub ground_anatomy: bool,
    /// Host-owned continuous yaw, outside the world and its hash.
    pub yaw: BTreeMap<OrganismId, f32>,
    /// Host tint overrides; without one, the kingdom palette answers.
    pub tints: BTreeMap<OrganismId, [f32; 3]>,
    /// Capsule stand-ins for bodies whose voxels would not project.
    pub fallback: Vec<CritterPose>,
    pub played_fallback: Option<CritterPose>,
}

impl HostBodies {
    pub fn new() -> Self {
        Self {
            scale: 1.0,
            ground_anatomy: false,
            yaw: BTreeMap::new(),
            tints: BTreeMap::new(),
            fallback: Vec::new(),
            played_fallback: None,
        }
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

    /// One organism as the scene reads it. `materials` is the phenotype's
    /// process mosaic where the caller needs it drawn, and empty where only
    /// geometry is being asked about.
    pub fn scene_body<'a>(
        &self,
        organism: &'a Organism,
        materials: &'a [PartMaterial],
        controlled: Option<OrganismId>,
    ) -> SceneBody<'a> {
        SceneBody {
            subject: key(organism.id),
            document: organism.body(),
            pose: Pose {
                position: organism.position.map(|v| v as f32),
                yaw_radians: self.yaw(organism.id),
            },
            scale: self.scale,
            grounded: self.ground_anatomy,
            tint: self.rendered_tint(organism),
            materials,
            always_visible: Some(organism.id) == controlled,
        }
    }

    /// The capsule stand-in. Mesocosm-only: the scene reports a projection
    /// failure and this decides what the frame shows instead.
    pub fn add_fallback(&mut self, body: &SceneBody<'_>, stats: &mut BodyFrameStats) {
        let at = body.origin();
        let controlled = body.always_visible;
        let placement = BodyPlacement {
            ground: [
                at[0],
                at[1] + body.document.aabb().min[1] as f32 * body.scale,
                at[2],
            ],
            scale: body.scale,
            tint: body.tint,
        };
        match BodyLensProjection::project_truncated(body.document, placement) {
            Ok((projected, dropped)) if controlled || self.fallback.len() < MAX_ROSTER => {
                stats.fallback_bodies += 1;
                stats.controlled_drawn |= controlled;
                stats.fallback_parts_dropped += dropped;
                if !controlled {
                    stats.fallback_parts_dropped += projected
                        .pose
                        .capsules
                        .len()
                        .saturating_sub(mesocosm_lens::MAX_ROSTER_CAPSULES);
                }
                if controlled {
                    self.played_fallback = Some(projected.pose);
                } else {
                    self.fallback.push(projected.pose);
                }
            },
            _ => stats.omitted_bodies += 1,
        }
    }
}

/// Mesocosm's organism identity, widened into the scene's subject key.
pub(super) fn key(id: OrganismId) -> SubjectKey {
    SubjectKey(u64::from(id.0))
}

/// The scene subject back as an organism id. Every key the scene returns was
/// minted by [`key`] above, so the narrowing is exact.
pub(super) fn organism_of(subject: SubjectKey) -> OrganismId {
    OrganismId(subject.0 as u32)
}

impl super::Section {
    /// One frame's bodies, with their process mosaics, in world order. The
    /// scene does its own culling and ordering over this slice.
    pub(super) fn scene_bodies<'a>(
        &self,
        world: &'a World,
        materials: &'a [Vec<PartMaterial>],
    ) -> Vec<SceneBody<'a>> {
        let controlled = world.controlled_id();
        world
            .organisms
            .iter()
            .zip(materials)
            .map(|(organism, materials)| {
                self.host_bodies.scene_body(organism, materials, controlled)
            })
            .collect()
    }

    /// The process mosaics behind those bodies, held by the caller so the
    /// scene bodies can borrow them for the frame.
    ///
    /// Only the organisms the scene could still draw are projected: a mosaic
    /// is a walk of a phenotype against the ruleset, and a body the cull or
    /// the budget already excluded never has its materials read. The order and
    /// the cut are `BodyLayer::prepare`'s own, so the slice every drawn body
    /// receives is exactly what it received before the fold; the rest get an
    /// empty slice, which was already legal.
    pub(super) fn scene_materials(
        &self,
        world: &World,
        window: SlabWindow,
    ) -> Vec<Vec<PartMaterial>> {
        let controlled = world.controlled_id();
        let isolated = self.scene.bodies().isolated;
        let distance = |organism: &Organism| {
            (0..3)
                .map(|axis| (organism.position[axis] as f32 - window.centre[axis]).powi(2))
                .sum::<f32>()
        };
        let mut candidates: Vec<usize> = world
            .organisms
            .iter()
            .enumerate()
            .filter(|(_, organism)| !isolated || Some(organism.id) == controlled)
            .filter(|(_, organism)| organism.body().living().next().is_some())
            .map(|(index, _)| index)
            .collect();
        candidates.sort_by(|&a, &b| {
            let (a, b) = (&world.organisms[a], &world.organisms[b]);
            (Some(a.id) != controlled)
                .cmp(&(Some(b.id) != controlled))
                .then_with(|| distance(a).total_cmp(&distance(b)))
                .then(a.id.0.cmp(&b.id.0))
        });
        candidates.truncate(self.body_budget.max(1));
        let drawn: std::collections::BTreeSet<usize> = candidates.into_iter().collect();
        world
            .organisms
            .iter()
            .enumerate()
            .map(|(index, organism)| {
                if drawn.contains(&index) {
                    super::materials::project(&organism.phenotype, world.ruleset())
                } else {
                    Vec::new()
                }
            })
            .collect()
    }
}

#[cfg(test)]
pub(super) fn clip_from_world(
    mode: super::CameraMode,
    centre: [f32; 3],
    half: f32,
    aspect: f32,
) -> [[f32; 4]; 4] {
    super::view::slab_camera(mode, centre, half, aspect).clip_from_world()
}

#[cfg(test)]
#[path = "bodies_tests.rs"]
mod tests;

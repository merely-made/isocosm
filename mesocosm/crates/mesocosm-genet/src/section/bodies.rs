// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Mesocosm's half of the body layer.
//!
//! The projection, the instance draw, the cull and the part queries are
//! `isometer`'s. What stays here is what needs the scene to mean anything:
//! which body is played, its tint, and the capsule stand-in a body falls back
//! to when its voxels will not project.

use std::num::NonZeroU64;

use isometer::hybrid::{HybridPolicy, HybridSettings};
use isometer::lens::{BodyLensProjection, BodyPlacement, CritterPose, MAX_ROSTER};
use isometer::{BodyFrameStats, Pose, SceneBody, SubjectKey};

use super::{PlacedBody, SiteScene};

mod materials;
pub(crate) use materials::{body_volumes, skin_parts};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BodyMode {
    #[default]
    Smooth,
    Capsules,
    Boxes,
    Voxels,
    /// Shared voxel shading for live and baked images.
    Hybrid,
}

impl BodyMode {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "smooth" => Some(Self::Smooth),
            "capsules" => Some(Self::Capsules),
            "boxes" => Some(Self::Boxes),
            "voxels" => Some(Self::Voxels),
            "hybrid" => Some(Self::Hybrid),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Smooth => "smooth",
            Self::Capsules => "capsules",
            Self::Boxes => "boxes",
            Self::Voxels => "voxels",
            Self::Hybrid => "hybrid",
        }
    }

    /// Mesocosm's default quality (case 1172, 9e31bce4's values).
    pub(super) fn hybrid_settings(self) -> Option<HybridSettings> {
        if self != Self::Hybrid {
            return None;
        }
        let policy = HybridPolicy::new(
            [[10.0, 14.0], [18.0, 22.0], [30.0, 42.0]],
            [12, 4, 2].map(|frames| NonZeroU64::new(frames).unwrap()),
        )
        .expect("Mesocosm hybrid bands and periods are ordered");
        Some(HybridSettings::new(policy, 24, 8 * 1024 * 1024, 8))
    }
}

pub const DEFAULT_BODY_BUDGET: usize = MAX_ROSTER + 1;

/// The quantum the hybrid rung's sun is rounded to (case 1170): a tuning
/// value, measured in the M4.7 receipt, not a ruled default.
pub const DEFAULT_LIGHT_STEP: f32 = 1.0 / 64.0;

pub(super) struct HostBodies {
    /// Body scale, applied to every drawn anatomy.
    pub scale: f32,
    /// Capsule stand-ins for bodies whose voxels would not project.
    pub fallback: Vec<CritterPose>,
    pub played_fallback: Option<CritterPose>,
    /// Subjects examination keeps live besides the played one; see
    /// [`super::host::examined_subjects`].
    pub examined: Vec<u64>,
    /// See [`DEFAULT_LIGHT_STEP`].
    pub light_step: f32,
}

impl HostBodies {
    pub fn new() -> Self {
        Self {
            scale: 1.0,
            fallback: Vec::new(),
            played_fallback: None,
            examined: Vec::new(),
            light_step: DEFAULT_LIGHT_STEP,
        }
    }

    /// One placed body with the native close-up's function allocations.
    pub fn scene_body<'a>(&self, body: &'a PlacedBody, played: Option<u64>) -> SceneBody<'a> {
        SceneBody {
            subject: SubjectKey(body.id),
            document: &body.document,
            pose: Pose {
                position: body.at.map(|v| v as f32),
                yaw_radians: body
                    .physical
                    .as_ref()
                    .map_or(0.0, |p| super::capsules::yaw_radians(p.yaw)),
            },
            scale: self.scale,
            grounded: body.physical.is_none(),
            tint: body.tint,
            materials: &body.materials,
            always_visible: Some(body.id) == played,
        }
    }

    /// The capsule stand-in: the scene reports a projection failure and this
    /// decides what the frame shows instead.
    pub fn add_fallback(&mut self, body: &SceneBody<'_>, stats: &mut BodyFrameStats) {
        let at = body.origin();
        let played = body.always_visible;
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
            Ok((mut projected, dropped)) if played || self.fallback.len() < MAX_ROSTER => {
                super::capsules::rotate_pose(
                    &mut projected.pose,
                    body.origin(),
                    body.pose.yaw_radians,
                );
                stats.fallback_bodies += 1;
                stats.controlled_drawn |= played;
                stats.fallback_parts_dropped += dropped;
                if played {
                    self.played_fallback = Some(projected.pose);
                } else {
                    stats.fallback_parts_dropped += projected
                        .pose
                        .capsules
                        .len()
                        .saturating_sub(isometer::lens::MAX_ROSTER_CAPSULES);
                    self.fallback.push(projected.pose);
                }
            },
            _ => stats.omitted_bodies += 1,
        }
    }
}

impl super::Section {
    /// The hybrid sun's quantum; zero keeps the light exact.
    pub fn set_light_step(&mut self, step: f32) {
        self.host_bodies.light_step = step;
    }

    /// One frame's bodies in scene order; the scene culls and orders them.
    /// Each body's skin colour table, registered against its subject.
    pub(super) fn register_palettes(&mut self, scene: &SiteScene) {
        let layer = self.scene.bodies_mut();
        for body in &scene.bodies {
            let subject = isometer::SubjectKey(body.id);
            if layer.palette(subject) != Some(body.palette.as_slice()) {
                layer.set_palette(subject, Some(body.palette.clone()));
            }
        }
    }

    pub(super) fn scene_bodies<'a>(&self, scene: &'a SiteScene) -> Vec<SceneBody<'a>> {
        scene
            .bodies
            .iter()
            .map(|body| {
                let mut body = self.host_bodies.scene_body(body, scene.played);
                if self.body_mode == BodyMode::Hybrid {
                    // Shared voxel face shading is fixed. The same product
                    // intensity reaches the live draw and the sprite key.
                    body.tint = body.tint.map(|channel| channel * self.body_tenant.light());
                }
                body
            })
            .collect()
    }
}

// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Mesocosm's half of the body layer.
//!
//! The projection, the instance draw, the cull and the part queries are
//! `isometer`'s. What stays here is what needs the scene to mean anything:
//! which body is played, its tint, and the capsule stand-in a body falls back
//! to when its voxels will not project.

use isometer::lens::{BodyLensProjection, BodyPlacement, CritterPose, MAX_ROSTER};
use isometer::{BodyFrameStats, Pose, SceneBody, SubjectKey};

use super::{PlacedBody, SiteScene};

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

pub(super) struct HostBodies {
    /// Body scale, applied to every drawn anatomy.
    pub scale: f32,
    /// Capsule stand-ins for bodies whose voxels would not project.
    pub fallback: Vec<CritterPose>,
    pub played_fallback: Option<CritterPose>,
}

impl HostBodies {
    pub fn new() -> Self {
        Self {
            scale: 1.0,
            fallback: Vec::new(),
            played_fallback: None,
        }
    }

    /// One placed body as the scene reads it. Native bodies carry no process
    /// mosaic yet, so no materials.
    pub fn scene_body<'a>(&self, body: &'a PlacedBody, played: Option<u64>) -> SceneBody<'a> {
        SceneBody {
            subject: SubjectKey(body.id),
            document: &body.document,
            pose: Pose {
                position: body.at.map(|v| v as f32),
                yaw_radians: 0.0,
            },
            scale: self.scale,
            grounded: true,
            tint: body.tint,
            materials: &[],
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
            Ok((projected, dropped)) if played || self.fallback.len() < MAX_ROSTER => {
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
    /// One frame's bodies in scene order; the scene culls and orders them.
    pub(super) fn scene_bodies<'a>(&self, scene: &'a SiteScene) -> Vec<SceneBody<'a>> {
        scene
            .bodies
            .iter()
            .map(|body| self.host_bodies.scene_body(body, scene.played))
            .collect()
    }
}

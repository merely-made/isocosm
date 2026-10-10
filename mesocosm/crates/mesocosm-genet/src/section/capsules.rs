// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The capsule comparison mode's poses, over the placed bodies of a scene.

use super::{PlacedBody, SiteScene, SlabWindow};
use isometer::lens::{BodyLensProjection, BodyPlacement, CritterPose, MAX_ROSTER};

/// The played body's pose and how many of its parts the capsule budget
/// dropped.
pub fn pose_of(scene: &SiteScene, scale: f32) -> Option<(CritterPose, u32)> {
    let body = scene.body(scene.played?)?;
    pose_at(body, scale)
}

/// Everything else living in the slab window, capped at the lens's roster.
pub fn roster_of(scene: &SiteScene, window: SlabWindow, scale: f32) -> Vec<CritterPose> {
    scene
        .bodies
        .iter()
        .filter(|body| {
            body.alive && Some(body.id) != scene.played && window.holds(body.at.map(|v| v as f32))
        })
        .filter_map(|body| pose_at(body, scale).map(|(pose, _)| pose))
        .take(MAX_ROSTER)
        .collect()
}

fn pose_at(body: &PlacedBody, scale: f32) -> Option<(CritterPose, u32)> {
    let placement = BodyPlacement {
        ground: body.at.map(|v| v as f32),
        scale,
        tint: body.tint,
    };
    BodyLensProjection::project_truncated(&body.document, placement)
        .ok()
        .map(|(projected, dropped)| (projected.pose, dropped as u32))
}

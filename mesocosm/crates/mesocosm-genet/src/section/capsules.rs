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
            Some(body.id) != scene.played && window.holds(body.at.map(|v| v as f32))
        })
        .filter_map(|body| pose_at(body, scale).map(|(pose, _)| pose))
        .take(MAX_ROSTER)
        .collect()
}

fn pose_at(body: &PlacedBody, scale: f32) -> Option<(CritterPose, u32)> {
    let root = body.at.map(|v| v as f32);
    let ground = if body.physical.is_some() {
        [root[0], root[1] + body.document.aabb().min[1] as f32 * scale, root[2]]
    } else { root };
    let placement = BodyPlacement {
        ground,
        scale,
        tint: body.tint,
    };
    BodyLensProjection::project_truncated(&body.document, placement)
        .ok()
        .map(|(mut projected, dropped)| {
            if let Some(physical) = &body.physical {
                rotate_pose(&mut projected.pose, root, yaw_radians(physical.yaw));
            }
            (projected.pose, dropped as u32)
        })
}

pub(super) fn yaw_radians(yaw: isometer::core::Yaw) -> f32 {
    use isometer::core::Yaw;
    let turns = match yaw { Yaw::Zero => 0, Yaw::Quarter => 1, Yaw::Half => 2, Yaw::ThreeQuarter => 3 };
    turns as f32 * std::f32::consts::FRAC_PI_2
}

/// The capsule tier follows the same root-pivot yaw as the voxel body.
pub(super) fn rotate_pose(pose: &mut CritterPose, root: [f32; 3], yaw: f32) {
    if yaw == 0.0 { return; }
    let (sin, cos) = yaw.sin_cos();
    let rotate = |at: [f32; 3]| {
        let [x, y, z] = [0, 1, 2].map(|i| at[i] - root[i]);
        [root[0] + x * cos + z * sin, root[1] + y, root[2] - x * sin + z * cos]
    };
    for capsule in &mut pose.capsules { capsule.a = rotate(capsule.a); capsule.b = rotate(capsule.b); }
    for eye in &mut pose.eyes {
        let at = rotate([eye[0], eye[1], eye[2]]);
        eye[..3].copy_from_slice(&at);
    }
    pose.bounds_centre = rotate(pose.bounds_centre);
}

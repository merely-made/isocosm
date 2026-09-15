// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Legacy capsule projections and counted body fallbacks.

use super::SlabWindow;
use isometer::lens::{BodyLensProjection, BodyPlacement, CritterPose, MAX_ROSTER};
use mesocosm_core::{BodyDocument, Organism, World};
/// The controlled critter's pose, through the landed V2 projection.
///
/// It stays the tracer's single pose rather than a roster member, because a
/// member's capsule budget is smaller than the played body's: see
/// [`isometer::lens::MAX_ROSTER`].
/// The pose comes back with the count of parts the capsule budget dropped, so
/// a truncated player is reported rather than merely smaller.
pub fn pose_of(world: &World, tint: [f32; 3]) -> Option<(CritterPose, u32)> {
    pose_of_scaled(world, tint, 1.0, false)
}

pub fn pose_of_scaled(
    world: &World,
    tint: [f32; 3],
    scale: f32,
    grounded: bool,
) -> Option<(CritterPose, u32)> {
    let body = world.body()?;
    let mut at = world.position()?.map(|v| v as f32);
    if grounded {
        at[1] -= body.aabb().min[1] as f32 * scale;
    }
    pose_at_origin(body, at, tint, scale)
}

/// Every other living organism the window holds, posed and tinted.
///
/// The scan is a bounds test per organism and a projection only for those
/// inside, so the frame's cost tracks organisms in the slab rather than
/// organisms in the world. The lens truncates whatever exceeds its own cap;
/// the take here just stops projecting once the cap is met.
pub fn roster_of(
    world: &World,
    window: SlabWindow,
    tint: impl Fn(&Organism) -> [f32; 3],
) -> Vec<CritterPose> {
    roster_of_scaled(world, window, tint, 1.0, false)
}

pub fn roster_of_scaled(
    world: &World,
    window: SlabWindow,
    tint: impl Fn(&Organism) -> [f32; 3],
    scale: f32,
    grounded: bool,
) -> Vec<CritterPose> {
    let controlled = world.controlled_id();
    world
        .organisms
        .iter()
        .filter(|organism| {
            organism.is_alive()
                && Some(organism.id) != controlled
                && window.holds(organism.position.map(|v| v as f32))
        })
        .filter_map(|organism| {
            let body = organism.body();
            let mut at = organism.position.map(|v| v as f32);
            if grounded {
                at[1] -= body.aabb().min[1] as f32 * scale;
            }
            pose_at_origin(body, at, tint(organism), scale).map(|(pose, _)| pose)
        })
        .take(MAX_ROSTER)
        .collect()
}

/// One body placed where it stands, and the parts its capsule budget could
/// not carry.
///
/// Body space is world voxels — the raster lane draws a part's voxels at
/// `position + v` — so the scale is 1 and the projection's floor subtraction
/// is undone, or the two views would disagree about where the same voxel is.
///
/// **A body past the lens's capsule limit is drawn truncated, never dropped.**
/// Until DC3 this swallowed the refusal with `.ok()` and the frame simply had
/// no body in it, which is how a played critter could disappear while alive.
/// The overflow is now a number the host puts in its receipt.
#[cfg(test)]
pub(super) fn pose_at(
    body: &BodyDocument,
    at: [i32; 3],
    tint: [f32; 3],
    scale: f32,
) -> Option<(CritterPose, u32)> {
    pose_at_origin(body, at.map(|v| v as f32), tint, scale)
}

fn pose_at_origin(
    body: &BodyDocument,
    at: [f32; 3],
    tint: [f32; 3],
    scale: f32,
) -> Option<(CritterPose, u32)> {
    let floor = body.aabb().min[1] as f32 * scale;
    let placement = BodyPlacement {
        ground: [at[0], at[1] + floor, at[2]],
        scale,
        tint,
    };
    BodyLensProjection::project_truncated(body, placement)
        .ok()
        .map(|(projected, dropped)| (projected.pose, dropped as u32))
}

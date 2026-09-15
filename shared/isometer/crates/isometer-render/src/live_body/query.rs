// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! CPU queries over the actual double-sided faces submitted by the renderer.

use glam::{DMat4, DVec3, Vec3};
use isometer_core::PartId;

use super::pose::{clip_contains, model_matrix, validate_body, validate_clip};
use super::{ClipSlab, LiveBody, LiveBodyError};

/// One addressed surface in the caller's submitted body slice.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BodyHit {
    pub body_index: usize,
    pub part: PartId,
    /// Distance in world units, independent of the input direction's length.
    pub distance: f32,
    pub point: [f32; 3],
    /// Distinct part identities hit at the same reported f32 distance.
    /// The lowest (body_index, PartId) wins. This stable CPU policy does not
    /// promise the GPU's colour owner where surfaces coincide.
    pub tied: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BodyQueryError {
    /// Ray values must be finite, direction nonzero, and far nonnegative.
    InvalidRay,
    Scene(LiveBodyError),
}

impl From<LiveBodyError> for BodyQueryError {
    fn from(error: LiveBodyError) -> Self {
        Self::Scene(error)
    }
}

/// World bounds of posed geometry, before clipping; empty geometry has none.
/// These use the same part matrices as drawing, including parent rotation.
pub fn body_bounds(body: LiveBody<'_>) -> Result<Option<([f32; 3], [f32; 3])>, LiveBodyError> {
    validate_body(body)?;
    let mut min = Vec3::splat(f32::INFINITY);
    let mut max = Vec3::splat(f32::NEG_INFINITY);
    let mut seen = false;
    for placement in &body.mesh.placements {
        let model = model_matrix(body, placement.yaw, placement.pivot, placement.pivot_at);
        let mesh = body
            .mesh
            .mesh_for(placement.volume)
            .expect("validated mesh");
        for quad in &mesh.quads {
            for corner in quad.corners() {
                let point = model.transform_point3(Vec3::from_array(corner.map(|v| v as f32)));
                if !point.is_finite() {
                    return Err(LiveBodyError::InvalidBody);
                }
                min = min.min(point);
                max = max.max(point);
                seen = true;
            }
        }
    }
    Ok(seen.then_some((min.to_array(), max.to_array())))
}

/// One actual meshed face, transformed by the same part matrix used to draw.
/// This is before clipping or occlusion, and is not a whole-body exterior test.
pub fn posed_quad(
    body: LiveBody<'_>,
    part: PartId,
    quad_index: usize,
) -> Result<[[f32; 3]; 4], LiveBodyError> {
    validate_body(body)?;
    let placement = body
        .mesh
        .placements
        .iter()
        .find(|p| p.part == part)
        .ok_or(LiveBodyError::InvalidBody)?;
    let quad = body
        .mesh
        .mesh_for(placement.volume)
        .and_then(|mesh| mesh.quads.get(quad_index))
        .ok_or(LiveBodyError::InvalidBody)?;
    let model = model_matrix(body, placement.yaw, placement.pivot, placement.pivot_at);
    let points = quad.corners().map(|corner| {
        model
            .transform_point3(Vec3::from_array(corner.map(|v| v as f32)))
            .to_array()
    });
    if points.iter().flatten().any(|v| !v.is_finite()) {
        return Err(LiveBodyError::InvalidBody);
    }
    Ok(points)
}

/// Nearest submitted surface along a ray, including distance zero and far.
///
/// Query only the successfully drawn projection snapshot. This function does
/// not know omitted bodies, terrain rendered elsewhere, or document clipping.
/// Slabs and bounds reject actual faces; they do not create cut caps. Quad
/// edges are inclusive, and rays parallel to a face do not hit that face.
/// Distances are compared at returned f32 precision, with ties documented on
/// `BodyHit`. No framebuffer coverage equivalence is claimed at raster edges.
pub fn pick_bodies(
    bodies: &[LiveBody<'_>],
    origin: [f32; 3],
    direction: [f32; 3],
    far: f32,
    clip: Option<ClipSlab>,
) -> Result<Option<BodyHit>, BodyQueryError> {
    if !origin.iter().chain(direction.iter()).all(|v| v.is_finite())
        || !far.is_finite()
        || far < 0.0
        || direction == [0.0; 3]
    {
        return Err(BodyQueryError::InvalidRay);
    }
    validate_clip(clip)?;
    for body in bodies {
        validate_body(*body)?;
    }
    // Use double precision for the inverse and intersection, but invert the
    // exact f32 matrix sent to the GPU. Tiny positive body scales remain valid.
    let origin = DVec3::from_array(origin.map(f64::from));
    let direction = DVec3::from_array(direction.map(f64::from)).normalize();
    let mut best: Option<BodyHit> = None;
    for (body_index, body) in bodies.iter().copied().enumerate() {
        for placement in &body.mesh.placements {
            let model = model_matrix(body, placement.yaw, placement.pivot, placement.pivot_at);
            let inverse = DMat4::from_cols_array(&model.to_cols_array().map(f64::from)).inverse();
            if !inverse.is_finite() {
                return Err(LiveBodyError::InvalidBody.into());
            }
            let local_origin = inverse.transform_point3(origin);
            // Do not normalize again: the parameter must remain world distance.
            let local_direction = inverse.transform_vector3(direction);
            let mesh = body
                .mesh
                .mesh_for(placement.volume)
                .expect("validated mesh");
            for quad in &mesh.quads {
                let axis = usize::from(quad.axis);
                if local_direction[axis] == 0.0 {
                    continue;
                }
                // Match the f32 local vertex coordinates uploaded by drawing.
                let plane = f64::from(quad.origin[axis] as f32);
                let distance = (plane - local_origin[axis]) / local_direction[axis];
                if distance < 0.0 || distance > f64::from(far) || !distance.is_finite() {
                    continue;
                }
                let local = local_origin + distance * local_direction;
                if !quad
                    .plane_axes()
                    .into_iter()
                    .enumerate()
                    .all(|(index, axis)| {
                        let low = f64::from(quad.origin[axis] as f32);
                        let high = f64::from((quad.origin[axis] + quad.size[index] as i32) as f32);
                        local[axis] >= low && local[axis] <= high
                    })
                {
                    continue;
                }
                let point = (origin + distance * direction).as_vec3().to_array();
                if !point.iter().all(|value| value.is_finite()) {
                    return Err(LiveBodyError::InvalidBody.into());
                }
                if !clip_contains(clip, point) {
                    continue;
                }
                let candidate = BodyHit {
                    body_index,
                    part: placement.part,
                    distance: distance as f32,
                    point,
                    tied: false,
                };
                select_hit(&mut best, candidate);
            }
        }
    }
    Ok(best)
}

fn select_hit(best: &mut Option<BodyHit>, mut candidate: BodyHit) {
    let Some(current) = best else {
        *best = Some(candidate);
        return;
    };
    if candidate.distance < current.distance {
        *current = candidate;
    } else if candidate.distance == current.distance {
        let current_key = (current.body_index, current.part);
        let candidate_key = (candidate.body_index, candidate.part);
        let distinct = current_key != candidate_key;
        candidate.tied = current.tied || distinct;
        if candidate_key < current_key {
            *current = candidate;
        } else {
            current.tied |= distinct;
        }
    }
}

#[cfg(test)]
#[path = "query_tests.rs"]
mod tests;

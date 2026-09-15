// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Shared draw/query placement and admission checks.

use glam::{Mat4, Vec3};
use isometer_core::Yaw;

use super::{ClipSlab, LiveBody, LiveBodyError, materials::valid_materials};

pub(super) fn model_matrix(
    body: LiveBody<'_>,
    yaw: Yaw,
    pivot: [i32; 3],
    pivot_at: [i32; 3],
) -> Mat4 {
    let angle = match yaw {
        Yaw::Zero => 0.0,
        Yaw::Quarter => core::f32::consts::FRAC_PI_2,
        Yaw::Half => core::f32::consts::PI,
        Yaw::ThreeQuarter => -core::f32::consts::FRAC_PI_2,
    };
    Mat4::from_translation(Vec3::from_array(body.origin))
        * Mat4::from_scale(Vec3::splat(body.scale))
        * Mat4::from_rotation_y(body.yaw_radians)
        * Mat4::from_translation(Vec3::new(
            pivot_at[0] as f32,
            pivot_at[1] as f32,
            pivot_at[2] as f32,
        ))
        * Mat4::from_rotation_y(angle)
        * Mat4::from_translation(Vec3::new(
            -pivot[0] as f32,
            -pivot[1] as f32,
            -pivot[2] as f32,
        ))
}

pub(super) fn validate_body(body: LiveBody<'_>) -> Result<(), LiveBodyError> {
    if !body.origin.iter().all(|value| value.is_finite())
        || !body.scale.is_finite()
        || body.scale <= 0.0
        || !body.yaw_radians.is_finite()
        || !body.tint.iter().all(|value| value.is_finite())
    {
        return Err(LiveBodyError::InvalidBody);
    }
    for placement in &body.mesh.placements {
        if body.mesh.mesh_for(placement.volume).is_none() {
            return Err(LiveBodyError::MissingMesh {
                volume: placement.volume,
            });
        }
    }
    if !valid_materials(body.materials) {
        return Err(LiveBodyError::InvalidMaterials);
    }
    Ok(())
}

pub(super) fn validate_clip(clip: Option<ClipSlab>) -> Result<(), LiveBodyError> {
    let Some(slab) = clip else { return Ok(()) };
    let invalid_bounds = slab.bounds.is_some_and(|(min, max)| {
        (0..3).any(|axis| !min[axis].is_finite() || !max[axis].is_finite() || min[axis] > max[axis])
    });
    if !slab.normal.iter().all(|value| value.is_finite())
        || !slab.min.is_finite()
        || !slab.max.is_finite()
        || slab.min > slab.max
        || slab.normal.iter().map(|value| value * value).sum::<f32>() <= f32::EPSILON
        || invalid_bounds
    {
        return Err(LiveBodyError::InvalidClip);
    }
    Ok(())
}

pub(super) fn clip_contains(clip: Option<ClipSlab>, point: [f32; 3]) -> bool {
    let Some(slab) = clip else { return true };
    let distance: f32 = (0..3).map(|axis| slab.normal[axis] * point[axis]).sum();
    distance >= slab.min
        && distance <= slab.max
        && slab.bounds.is_none_or(|(min, max)| {
            (0..3).all(|axis| point[axis] >= min[axis] && point[axis] <= max[axis])
        })
}

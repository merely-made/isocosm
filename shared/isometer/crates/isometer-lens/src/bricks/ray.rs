// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! CPU queries of the presented material map, including its cutaway filtering.

use super::BrickMap;

/// A terrain sample from the same material atlas the tracer reads.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BrickRayHit {
    pub material: u8,
    pub voxel: [i32; 3],
    pub distance: f32,
    pub point: [f32; 3],
    pub normal: [f32; 3],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BrickRayError {
    InvalidRay,
    /// The shader's 1,024-cell traversal budget was exhausted. Do not treat
    /// an unexamined interval as clear terrain when deciding what was hit.
    TraversalLimit,
}

impl std::fmt::Display for BrickRayError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidRay => f.write_str("invalid presented-terrain ray"),
            Self::TraversalLimit => f.write_str("presented-terrain traversal budget exhausted"),
        }
    }
}

impl std::error::Error for BrickRayError {}

impl BrickMap {
    /// Nearest non-air sample in this presentation map. Direction is
    /// normalized once; distance and `far` are world units. Feed the camera's
    /// front-wall ray and far interval so hidden foreground never occludes.
    ///
    /// Calls modulus's shared CPU walk of `BRICK_DDA_WGSL`, including its
    /// clamped start voxel and crossings measured from the ray origin.
    /// This adapter validates and normalizes the ray, reports world-space
    /// hit points, and makes budget exhaustion an error rather than a
    /// clear-space claim.
    pub fn trace_ray(
        &self,
        origin: [f32; 3],
        direction: [f32; 3],
        far: f32,
    ) -> Result<Option<BrickRayHit>, BrickRayError> {
        if !origin.into_iter().chain(direction).all(f32::is_finite)
            || !far.is_finite()
            || far <= 0.0
        {
            return Err(BrickRayError::InvalidRay);
        }
        let length = direction
            .iter()
            .map(|&v| f64::from(v).powi(2))
            .sum::<f64>()
            .sqrt();
        if length == 0.0 {
            return Err(BrickRayError::InvalidRay);
        }
        let direction = direction.map(|v| (f64::from(v) / length) as f32);
        match self.shared().trace(origin, direction, far) {
            modulus::BrickTrace::Hit(hit) => Ok(Some(BrickRayHit {
                material: hit.material,
                voxel: hit.voxel,
                distance: hit.t,
                point: [0, 1, 2].map(|i| origin[i] + direction[i] * hit.t),
                normal: hit.normal,
            })),
            modulus::BrickTrace::Clear => Ok(None),
            modulus::BrickTrace::Exhausted => Err(BrickRayError::TraversalLimit),
        }
    }
}

#[cfg(test)]
#[path = "ray_tests.rs"]
mod tests;

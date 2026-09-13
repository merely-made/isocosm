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
    /// Mirrors modulus::BRICK_DDA_WGSL: 0.0001 start offset, 1e-6 parallel
    /// threshold, X/Y/Z boundary tie order and 1,024-cell budget. Unlike the
    /// shader, an interval shorter than the start offset is explicitly empty
    /// and budget exhaustion is an error rather than a clear-space claim.
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
        let low = self
            .origin()
            .map(|v| f32::from(v) * modulus::BRICK_EDGE as f32);
        let high = [0, 1, 2].map(|i| low[i] + self.pointer_extent()[i] as f32 * 8.0);
        let mut enter = 0.0f32;
        let mut exit = far;
        for axis in 0..3 {
            let d = direction[axis];
            if d.abs() < 1e-6 {
                if origin[axis] < low[axis] || origin[axis] >= high[axis] {
                    return Ok(None);
                }
                continue;
            }
            let a = (low[axis] - origin[axis]) / d;
            let b = (high[axis] - origin[axis]) / d;
            enter = enter.max(a.min(b));
            exit = exit.min(a.max(b));
        }
        let start_t = enter.max(0.0) + 0.0001;
        if enter > exit || start_t > exit {
            return Ok(None);
        }
        let start = [0, 1, 2].map(|i| origin[i] + direction[i] * start_t);
        let mut voxel = start.map(|v| v.floor() as i32);
        let step = direction.map(|v| if v >= 0.0 { 1 } else { -1 });
        let mut crossing = [0, 1, 2].map(|i| {
            if direction[i] > 1e-6 {
                start_t + ((voxel[i] + 1) as f32 - start[i]) / direction[i]
            } else if direction[i] < -1e-6 {
                start_t + (voxel[i] as f32 - start[i]) / direction[i]
            } else {
                1e30
            }
        });
        let delta = direction.map(|v| if v.abs() > 1e-6 { 1.0 / v.abs() } else { 1e30 });
        let mut distance = start_t;
        let mut normal = [0.0, 1.0, 0.0];
        for _ in 0..1024 {
            // Match the shader's bounds check before the shared accessor
            // narrows a world cell to an i16 brick key. At the coordinate
            // limit, f32 can round the start epsilon onto an outside cell.
            let inside = (0..3).all(|i| voxel[i] as f32 >= low[i] && (voxel[i] as f32) < high[i]);
            let material = if inside { self.material_at(voxel) } else { 0 };
            if material != 0 {
                return Ok(Some(BrickRayHit {
                    material,
                    voxel,
                    distance,
                    point: [0, 1, 2].map(|i| origin[i] + direction[i] * distance),
                    normal,
                }));
            }
            let axis = if crossing[0] <= crossing[1] && crossing[0] <= crossing[2] {
                0
            } else if crossing[1] <= crossing[2] {
                1
            } else {
                2
            };
            distance = crossing[axis];
            crossing[axis] += delta[axis];
            voxel[axis] += step[axis];
            normal = [0.0; 3];
            normal[axis] = -step[axis] as f32;
            if distance > exit || distance > far {
                return Ok(None);
            }
        }
        Err(BrickRayError::TraversalLimit)
    }
}

#[cfg(test)]
#[path = "ray_tests.rs"]
mod tests;

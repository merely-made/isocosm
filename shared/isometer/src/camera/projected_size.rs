// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::{SlabCamera, clip_xyw};

/// Larger of the projected width and height of a world AABB, in pixels.
/// The bounds can extend past the viewport: panning across the screen edge
/// should not reduce a subject's detail. Perspective bounds are clipped at
/// near and far before dividing by depth. Returns `None` for invalid inputs
/// or bounds entirely beyond either perspective depth plane.
pub fn projected_size(
    camera: SlabCamera,
    (min, max): ([f32; 3], [f32; 3]),
    viewport: [u32; 2],
) -> Option<f32> {
    if viewport.contains(&0)
        || !camera.framable()
        || (0..3).any(|i| !min[i].is_finite() || !max[i].is_finite() || min[i] > max[i])
    {
        return None;
    }

    let matrix = camera.clip_from_world();
    let corners: [[f32; 3]; 8] = std::array::from_fn(|corner| {
        let world = std::array::from_fn(|axis| {
            if corner & (1 << axis) == 0 {
                min[axis]
            } else {
                max[axis]
            }
        });
        // Same clip x, y and w as the raster. For a slab w is always 1.
        clip_xyw(matrix, world)
    });
    if !corners.iter().flatten().all(|v| v.is_finite()) {
        return None;
    }

    let mut low = [f32::INFINITY; 2];
    let mut high = [f32::NEG_INFINITY; 2];
    let mut include = |[x, y, w]: [f32; 3]| {
        for (axis, ndc) in [x / w, y / w].into_iter().enumerate() {
            low[axis] = low[axis].min(ndc);
            high[axis] = high[axis].max(ndc);
        }
    };

    if let Some(frustum) = camera.frustum() {
        for (corner, point) in corners.into_iter().enumerate() {
            if (frustum.near..=frustum.far).contains(&point[2]) {
                include(point);
            }
            // Each edge runs from an unset corner bit to the same bit set.
            for axis in 0..3 {
                let other = corner | (1 << axis);
                if other == corner {
                    continue;
                }
                let end = corners[other];
                for plane in [frustum.near, frustum.far] {
                    if (point[2] < plane && end[2] > plane) || (end[2] < plane && point[2] > plane)
                    {
                        let t = (plane - point[2]) / (end[2] - point[2]);
                        include([
                            point[0] * (1.0 - t) + end[0] * t,
                            point[1] * (1.0 - t) + end[1] * t,
                            plane,
                        ]);
                    }
                }
            }
        }
    } else {
        corners.into_iter().for_each(include);
    }

    let [width, height] = [0, 1].map(|axis| (high[axis] - low[axis]) * 0.5 * viewport[axis] as f32);
    (width.is_finite() && height.is_finite()).then_some(width.max(height))
}

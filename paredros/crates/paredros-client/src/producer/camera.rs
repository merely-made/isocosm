// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! One set of camera numbers for terrain rays, raster depth and CPU picking.
//!
//! Ruling 1 of Mesocosm's presentation plan: the section is orthographic. The
//! slab stands between two world-vertical walls, so a tilted view still cuts
//! the world upright. Terrain rays come from [`TraceCamera::orthographic_slab`]
//! and body raster depth comes from [`SlabView::clip_from_world`]; both are
//! derived from the same basis here so the two agree per pixel.

use mesocosm_lens::{SlabWall, TraceCamera};
use mesocosm_render::ClipSlab;

/// Presentation policy the host owns. None of it reaches an intent.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CameraPolicy {
    /// Which way the section looks. Any direction with a horizontal component;
    /// straight down has no wall to stand a slab on and is refused.
    pub forward: [f32; 3],
    /// Half the world height the frame shows, in voxels.
    pub half_height: f32,
    /// How much world the slab keeps between its two vertical cuts.
    pub depth: f32,
    /// Added to the follow centre, in world voxels.
    pub pan: [f32; 3],
}

impl Default for CameraPolicy {
    fn default() -> Self {
        Self {
            // A shallow oblique: enough lean to read relief, shallow enough
            // that a standing body keeps its height.
            forward: [0.70, -0.28, 0.70],
            half_height: 10.0,
            depth: 48.0,
            pan: [0.0; 3],
        }
    }
}

/// The camera numbers for one frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SlabView {
    pub centre: [f32; 3],
    pub forward: [f32; 3],
    pub half_height: f32,
    pub aspect: f32,
    pub depth: f32,
}

impl SlabView {
    pub fn new(policy: CameraPolicy, centre: [f32; 3], aspect: f32) -> Option<Self> {
        let centre = [0, 1, 2].map(|i| centre[i] + policy.pan[i]);
        let view = Self {
            centre,
            forward: normalize(policy.forward)?,
            half_height: policy.half_height,
            aspect,
            depth: policy.depth,
        };
        let finite = view.centre.iter().chain(view.forward.iter()).copied();
        let positive = [view.half_height, view.aspect, view.depth];
        (finite.chain(positive).all(f32::is_finite) && positive.iter().all(|v| *v > 0.0))
            .then_some(view)
    }

    /// Screen right, screen up, and forward. World up defines the standing
    /// wall, not merely the screen basis.
    pub fn basis(self) -> Option<[[f32; 3]; 3]> {
        let right = normalize(cross(self.forward, WORLD_UP))?;
        Some([right, cross(right, self.forward), self.forward])
    }

    /// Half the forward extent the seeded slab actually reaches.
    fn reach(self) -> f32 {
        SlabWall::new(
            self.forward,
            WORLD_UP,
            self.half_height,
            self.aspect,
            self.depth,
        )
        .map_or(self.depth * 0.5, |wall| wall.reach)
    }

    pub fn trace(self) -> Option<TraceCamera> {
        TraceCamera::orthographic_slab(
            self.centre,
            self.forward,
            WORLD_UP,
            self.half_height,
            self.aspect,
            self.depth,
        )
    }

    /// Column-major world-to-clip, the convention the tracer's depth join and
    /// the live body renderer both consume.
    pub fn clip_from_world(self) -> Option<[[f32; 4]; 4]> {
        let [right, up, forward] = self.basis()?;
        let x = right.map(|v| v / (self.half_height * self.aspect));
        let y = up.map(|v| v / self.half_height);
        let z = forward.map(|v| v / (2.0 * (self.reach() + 1.0)));
        Some([
            [x[0], y[0], z[0], 0.0],
            [x[1], y[1], z[1], 0.0],
            [x[2], y[2], z[2], 0.0],
            [
                -dot(x, self.centre),
                -dot(y, self.centre),
                0.5 - dot(z, self.centre),
                1.0,
            ],
        ])
    }

    /// The world-vertical cut the body raster shares with the traced slab.
    pub fn clip(self) -> ClipSlab {
        let length = self.forward[0].hypot(self.forward[2]);
        let normal = if length > 0.0 {
            [self.forward[0] / length, 0.0, self.forward[2] / length]
        } else {
            [0.0, 0.0, 1.0]
        };
        let middle = dot(normal, self.centre);
        ClipSlab::new(normal, middle - self.depth * 0.5, middle + self.depth * 0.5)
    }

    /// Where a world point lands in normalized device coordinates, positive Y
    /// upward. The same arithmetic the GPU runs, so a test can name a pixel.
    pub fn ndc_of(self, point: [f32; 3]) -> Option<[f32; 2]> {
        let [right, up, _] = self.basis()?;
        let offset = [0, 1, 2].map(|i| point[i] - self.centre[i]);
        Some([
            dot(right, offset) / (self.half_height * self.aspect),
            dot(up, offset) / self.half_height,
        ])
    }

    /// The pixel a world point lands on, in the produced texture's own frame
    /// (origin top-left). `None` when the point falls outside the frame.
    pub fn pixel_of(self, point: [f32; 3], size: [u32; 2]) -> Option<[u32; 2]> {
        let ndc = self.ndc_of(point)?;
        let x = (ndc[0] * 0.5 + 0.5) * size[0] as f32;
        let y = (0.5 - ndc[1] * 0.5) * size[1] as f32;
        (x >= 0.0 && y >= 0.0 && x < size[0] as f32 && y < size[1] as f32)
            .then(|| [x as u32, y as u32])
    }
}

const WORLD_UP: [f32; 3] = [0.0, 1.0, 0.0];

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a.into_iter().zip(b).map(|(a, b)| a * b).sum()
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn normalize(v: [f32; 3]) -> Option<[f32; 3]> {
    let length = dot(v, v).sqrt();
    (length.is_finite() && length > 0.0).then(|| v.map(|c| c / length))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn view() -> SlabView {
        SlabView::new(CameraPolicy::default(), [12.0, 30.0, -4.0], 16.0 / 9.0).unwrap()
    }

    #[test]
    fn the_projection_matrix_and_the_ndc_query_agree() {
        let view = view();
        let matrix = view.clip_from_world().unwrap();
        for point in [[12.0, 30.0, -4.0], [3.0, 26.5, 7.25], [20.0, 33.0, -11.0]] {
            let clip = [0, 1].map(|row| {
                matrix[3][row] + (0..3).map(|col| matrix[col][row] * point[col]).sum::<f32>()
            });
            let ndc = view.ndc_of(point).unwrap();
            assert!((clip[0] - ndc[0]).abs() < 1e-5, "{clip:?} vs {ndc:?}");
            assert!((clip[1] - ndc[1]).abs() < 1e-5, "{clip:?} vs {ndc:?}");
        }
    }

    #[test]
    fn traced_rays_start_and_end_on_the_raster_depth_interval() {
        let view = view();
        let camera = view.trace().unwrap();
        let matrix = view.clip_from_world().unwrap();
        let clip = view.clip();
        for ndc in [[0.0, 0.0], [-0.8, 0.6], [0.75, -0.75]] {
            let (origin, direction) = camera.ray_at(ndc).unwrap();
            let end = [0, 1, 2].map(|i| origin[i] + direction[i] * camera.far());
            assert!(
                (dot(clip.normal, origin) - clip.min).abs() < 1e-3,
                "near wall"
            );
            assert!((dot(clip.normal, end) - clip.max).abs() < 1e-3, "far wall");
            for point in [origin, end] {
                let depth = matrix[3][2] + (0..3).map(|i| matrix[i][2] * point[i]).sum::<f32>();
                assert!((0.0..=1.0).contains(&depth), "slab must fit raster depth");
            }
        }
    }
}

// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Eponym's presentation policy over the shared slab.
//!
//! The camera arithmetic itself is `isometer`'s [`SlabCamera`]: one
//! orthographic slab between two world-vertical walls, one `clip_from_world`
//! for the terrain rays and the body raster alike. What is Eponym's own is
//! the *preset* — which way the section looks, how much world it shows, how
//! deep it cuts, where it is panned — and how a subject is coloured and
//! scaled. None of it reaches an intent.

use isometer::SlabCamera;

/// The material every part is drawn in. 245 is the core's flat white tone, so
/// the colour a pixel carries is the body's tint times a face shade and
/// nothing else. Configurable, because the palette is presentation.
pub const BODY_MATERIAL: u8 = 245;

/// Body-document voxel units per world voxel. The anatomy's units are its own:
/// the fixture's root is four units tall inside a two-voxel motion envelope, so
/// a quarter puts a body at roughly the height the solver collides at.
pub const BODY_SCALE: f32 = 0.25;

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

impl CameraPolicy {
    /// This preset as one shared slab camera, or `None` when the numbers
    /// cannot frame anything.
    ///
    /// The forward vector is normalized here rather than handed over raw:
    /// [`SlabCamera`] takes a host's vector exactly as supplied, and its reach
    /// is read off a wall built from that vector, so an unnormalized preset
    /// would cut the slab at a different depth than this policy declares.
    pub fn camera(self, centre: [f32; 3], aspect: f32) -> Option<SlabCamera> {
        let centre = [0, 1, 2].map(|i| centre[i] + self.pan[i]);
        SlabCamera::new(
            centre,
            normalize(self.forward)?,
            self.half_height,
            aspect,
            self.depth,
        )
    }
}

/// How a subject is coloured. Identity, not decoration: the tests read it back.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Appearance {
    /// The played subject.
    pub played: [f32; 3],
    /// Everyone else.
    pub other: [f32; 3],
    pub material: u8,
    pub scale: f32,
}

impl Default for Appearance {
    fn default() -> Self {
        Self {
            played: [1.0, 0.42, 0.12],
            other: [0.20, 0.62, 1.0],
            material: BODY_MATERIAL,
            scale: BODY_SCALE,
        }
    }
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a.into_iter().zip(b).map(|(a, b)| a * b).sum()
}

fn normalize(v: [f32; 3]) -> Option<[f32; 3]> {
    let length = dot(v, v).sqrt();
    (length.is_finite() && length > 0.0).then(|| v.map(|c| c / length))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn camera() -> SlabCamera {
        CameraPolicy::default()
            .camera([12.0, 30.0, -4.0], 16.0 / 9.0)
            .unwrap()
    }

    #[test]
    fn the_projection_matrix_and_the_ndc_query_agree() {
        let camera = camera();
        let matrix = camera.clip_from_world();
        for point in [[12.0, 30.0, -4.0], [3.0, 26.5, 7.25], [20.0, 33.0, -11.0]] {
            let clip = [0, 1].map(|row| {
                matrix[3][row] + (0..3).map(|col| matrix[col][row] * point[col]).sum::<f32>()
            });
            let ndc = camera.ndc_of(point).unwrap();
            assert!((clip[0] - ndc[0]).abs() < 1e-5, "{clip:?} vs {ndc:?}");
            assert!((clip[1] - ndc[1]).abs() < 1e-5, "{clip:?} vs {ndc:?}");
        }
    }

    #[test]
    fn traced_rays_start_and_end_on_the_raster_depth_interval() {
        let camera = camera();
        let trace = camera.trace().unwrap();
        let matrix = camera.clip_from_world();
        let clip = camera.clip();
        for ndc in [[0.0, 0.0], [-0.8, 0.6], [0.75, -0.75]] {
            let (origin, direction) = trace.ray_at(ndc).unwrap();
            let end = [0, 1, 2].map(|i| origin[i] + direction[i] * trace.far());
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

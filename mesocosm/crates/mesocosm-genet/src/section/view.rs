// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Mesocosm's presets, turned into the one shared camera.
//!
//! The camera numbers themselves — trace, raster matrix, cut slab, reach and
//! cull window — live in `wing-scene` now. What stays here is the product's
//! own reading: which [`CameraMode`] is selected, whether a terrarium view is
//! overriding the pitch and the depth, and which habitat box is cut away.

use super::CameraMode;
use wing_scene::{Cutaway, SlabCamera};

/// The section's camera, for as long as `Section` is still the thing that
/// builds it. Retired with the adapter step.
pub(super) type View = SlabCamera;

impl super::Section {
    pub(super) fn view(&self, centre: [f32; 3]) -> View {
        let pitch = self.terrarium.as_ref().map(|view| view.pitch());
        SlabCamera {
            centre,
            // The un-pitched preset hands its own exact vector over; a pitched
            // terrarium hands the rotated one. Both are what the slab wall was
            // built from before this crate existed, so the reach, the cut
            // normal and the window are unchanged to the bit.
            forward: forward_of(self.mode, pitch),
            half_height: self.half_height,
            aspect: self.aspect(),
            depth: if self.bodies.isolated {
                self.bodies.preview_depth
            } else {
                self.terrarium
                    .as_ref()
                    .map_or(super::SLAB_DEPTH, |view| view.depth())
            },
            cutaway: if self.bodies.isolated {
                None
            } else {
                self.terrarium
                    .as_ref()
                    .map(|view| view.bounds())
                    .map(|(min, max)| Cutaway::Bounds { min, max })
            },
        }
    }
}

/// Which way a mode looks once the terrarium's pitch has had its say.
fn forward_of(mode: CameraMode, pitch: Option<f32>) -> [f32; 3] {
    match pitch {
        None => mode.forward(),
        Some(_) => camera_basis(mode, pitch)[2],
    }
}

/// A camera in `mode` at the section's shipped slab depth, level. The
/// fixtures' constructor; the host itself always goes through [`Section::view`].
#[cfg(test)]
pub(super) fn slab_camera(
    mode: CameraMode,
    centre: [f32; 3],
    half_height: f32,
    aspect: f32,
) -> SlabCamera {
    SlabCamera {
        centre,
        forward: mode.forward(),
        half_height,
        aspect,
        depth: super::SLAB_DEPTH,
        cutaway: None,
    }
}

pub fn camera_basis(mode: CameraMode, pitch: Option<f32>) -> [[f32; 3]; 3] {
    let [right, _, forward] = mode.basis();
    let Some(pitch) = pitch.filter(|_| mode.is_terrarium()) else {
        return mode.basis();
    };
    let pitch = pitch.to_radians();
    let length = forward[0].hypot(forward[2]);
    let forward = [
        forward[0] / length * pitch.cos(),
        -pitch.sin(),
        forward[2] / length * pitch.cos(),
    ];
    let up = [
        right[1] * forward[2] - right[2] * forward[1],
        right[2] * forward[0] - right[0] * forward[2],
        right[0] * forward[1] - right[1] * forward[0],
    ];
    [right, up, forward]
}

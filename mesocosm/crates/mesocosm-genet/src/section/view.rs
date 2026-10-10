// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Mesocosm's presets, turned into the one shared camera.
//!
//! The camera numbers themselves (trace, raster matrix, cut slab, reach and
//! cull window) live in `isometer`. What stays here is which [`CameraMode`]
//! is selected.

use super::CameraMode;
use isometer::SlabCamera;

impl super::Section {
    pub(super) fn view(&self, centre: [f32; 3]) -> SlabCamera {
        SlabCamera {
            centre,
            forward: self.mode.forward(),
            half_height: self.half_height,
            aspect: self.aspect(),
            depth: super::SLAB_DEPTH,
            cutaway: None,
        }
    }

    pub fn set_mode(&mut self, mode: CameraMode) {
        if self.mode != mode {
            self.invalidate_query();
            self.mode = mode;
        }
    }
}

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

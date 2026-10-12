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
            forward: self.custom_forward.unwrap_or_else(|| self.mode.forward()),
            half_height: self.half_height,
            aspect: self.aspect(),
            depth: super::SLAB_DEPTH,
            cutaway: None,
        }
    }

    /// Optional host-local orientation over the same terrain/body/cull camera.
    pub fn set_forward(&mut self, forward: Option<[f32; 3]>) -> Result<(), String> {
        if forward.is_some_and(|f| SlabCamera::new([0.0;3], f, self.half_height, 1.0, super::SLAB_DEPTH).is_none()) {
            return Err("invalid local section orientation".into());
        }
        if self.custom_forward != forward { self.invalidate_query(); self.custom_forward = forward; }
        Ok(())
    }

    pub fn set_mode(&mut self, mode: CameraMode) {
        if self.mode != mode {
            self.invalidate_query();
            self.mode = mode;
            self.custom_forward = None;
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

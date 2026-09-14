// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The host's door onto `wing-scene`'s frame read-back. The staging buffer,
//! the row padding and the chrome composite are the scene's now.

use super::Section;

impl Section {
    /// Reads the most recently traced frame back as RGBA8, with `overlay`
    /// given the chance to composite chrome over it first.
    pub fn capture(
        &self,
        overlay: impl FnOnce(&mut wgpu::CommandEncoder, &wgpu::TextureView, wgpu::TextureFormat),
    ) -> Option<(u32, u32, Vec<u8>)> {
        self.scene.capture(overlay)
    }

    /// Reads a completed frame master back as RGBA8. RG3 uses this route so
    /// its evidence crosses the same imported-tenant master as the live
    /// window.
    pub fn capture_from(
        &self,
        master: &wgpu::Texture,
        overlay: impl FnOnce(&mut wgpu::CommandEncoder, &wgpu::TextureView, wgpu::TextureFormat),
    ) -> Option<(u32, u32, Vec<u8>)> {
        self.scene.capture_from(master, overlay)
    }
}

// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The renderer's public draw entry points; each records through `draw_inner`.

use super::*;

impl LiveBodyRenderer {
    /// Records a `Load`/`Load` pass into caller-owned display-encoded colour
    /// and `Depth32Float` attachments. The supplied matrix must be the same
    /// world-to-clip convention used by the terrain tracer.
    pub fn draw(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        colour: &wgpu::TextureView,
        depth: &wgpu::TextureView,
        clip_from_world: [[f32; 4]; 4],
        clip_slab: Option<ClipSlab>,
        bodies: &[LiveBody<'_>],
    ) -> Result<BodyDrawStats, LiveBodyError> {
        self.draw_inner(
            device,
            Some(queue),
            encoder,
            colour,
            depth,
            clip_from_world,
            clip_slab,
            bodies,
            &[],
            &[],
        )
    }

    /// Draw the live side of a hybrid transition. Supply one sprite weight
    /// in 0..=16 per body, with 0 keeping the body entirely live.
    pub fn draw_hybrid(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        colour: &wgpu::TextureView,
        depth: &wgpu::TextureView,
        clip_from_world: [[f32; 4]; 4],
        clip_slab: Option<ClipSlab>,
        bodies: &[LiveBody<'_>],
        sprite_coverage: &[u32],
    ) -> Result<BodyDrawStats, LiveBodyError> {
        if sprite_coverage.len() != bodies.len()
            || sprite_coverage.iter().any(|&weight| weight > 16)
        {
            return Err(LiveBodyError::InvalidSpriteCoverage);
        }
        self.draw_styled(
            device,
            queue,
            encoder,
            colour,
            depth,
            clip_from_world,
            clip_slab,
            bodies,
            &[],
            sprite_coverage,
        )
    }

    /// [`Self::draw_hybrid`] with one style per body; a body past the end
    /// of `styles` takes the renderer's [`Self::set_display`] style.
    pub fn draw_styled(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        colour: &wgpu::TextureView,
        depth: &wgpu::TextureView,
        clip_from_world: [[f32; 4]; 4],
        clip_slab: Option<ClipSlab>,
        bodies: &[LiveBody<'_>],
        styles: &[BodyDisplay],
        sprite_coverage: &[u32],
    ) -> Result<BodyDrawStats, LiveBodyError> {
        self.draw_inner(
            device,
            Some(queue),
            encoder,
            colour,
            depth,
            clip_from_world,
            clip_slab,
            bodies,
            styles,
            sprite_coverage,
        )
    }

    /// In an atlas refresh each layer clears and uses the same upload buffers
    /// in encoder order. A queue write would be visible to all earlier passes.
    pub(crate) fn bake(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        colour: &wgpu::TextureView,
        depth: &wgpu::TextureView,
        clip_from_world: [[f32; 4]; 4],
        body: LiveBody<'_>,
    ) -> Result<BodyDrawStats, LiveBodyError> {
        self.draw_inner(
            device,
            None,
            encoder,
            colour,
            depth,
            clip_from_world,
            None,
            &[body],
            &[],
            &[],
        )
    }
}

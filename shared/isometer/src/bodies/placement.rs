// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Where a body stands, what it costs, and whether the camera can see it.
//!
//! Split out of `bodies.rs` under the 600-line ceiling: everything here is
//! arithmetic over a pose and a window, with no device and no projector.

use super::{SceneBody, SlabWindow};

/// What one prepared frame of bodies cost and contained.
///
/// Some counters are still Mesocosm's meanings (`carcasses`, the two capsule
/// `fallback_*` fields) and are written by a host after [`BodyLayer::prepare`]
/// rather than by the layer. They stay on one struct while `Section` is still
/// the thing that publishes the receipt.
///
/// [`BodyLayer::prepare`]: super::BodyLayer::prepare
#[derive(Clone, Debug, Default, serde::Serialize)]
pub struct BodyFrameStats {
    pub material_parts: usize,
    pub secretory_parts: usize,
    pub candidates: usize,
    pub body_scale: f32,
    pub voxel_bodies: usize,
    pub voxel_parts: usize,
    pub carcasses: usize,
    pub fallback_bodies: usize,
    pub fallback_parts_dropped: usize,
    pub omitted_bodies: usize,
    pub missing_volumes: usize,
    pub projection_failures: usize,
    pub last_error: Option<String>,
    pub controlled_drawn: bool,
    pub mesh_builds: usize,
    pub mesh_upload_bytes: u64,
    pub instance_upload_bytes: u64,
    pub frame_upload_bytes: u64,
    pub draw_parts: usize,
}

/// World origin of a body's mesh: its pose, with the document's own floor
/// stood on `pose.position[1]` when the body is grounded.
pub fn body_origin(body: &SceneBody<'_>) -> [f32; 3] {
    let mut origin = body.pose.position;
    if body.grounded {
        origin[1] -= body.document.aabb().min[1] as f32 * body.scale;
    }
    origin
}

/// Squared distance from a pose to the window centre; the draw order's key.
pub fn distance(at: [f32; 3], centre: [f32; 3]) -> f32 {
    (0..3).map(|i| (at[i] - centre[i]).powi(2)).sum()
}

/// Whether an axis-aligned world box meets the camera's oriented slab.
pub fn intersects((min, max): ([f32; 3], [f32; 3]), window: SlabWindow) -> bool {
    let middle = [0, 1, 2].map(|i| (min[i] + max[i]) * 0.5 - window.centre[i]);
    let half = [0, 1, 2].map(|i| (max[i] - min[i]) * 0.5);
    (0..3).all(|axis| {
        let extent: f32 = (0..3).map(|i| half[i] * window.axes[axis][i].abs()).sum();
        dot(middle, window.axes[axis]).abs() <= window.half[axis] + extent
    })
}

pub fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a.into_iter().zip(b).map(|(a, b)| a * b).sum()
}

/// The one depth attachment bodies, terrain and glyphs share.
pub fn depth_target(
    device: &wgpu::Device,
    width: u32,
    height: u32,
) -> (wgpu::Texture, wgpu::TextureView) {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("section body and terrain depth"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    let view = texture.create_view(&Default::default());
    (texture, view)
}

// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::HybridPolicy;

/// Product-selected pixel quality and separate image residency/work budgets.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HybridSettings {
    pub policy: HybridPolicy,
    /// Width and height of one colour/depth image.
    pub resolution: u32,
    pub max_resident_bytes: u64,
    /// Maximum bakes encoded per scene frame.
    pub max_refreshes: u32,
    /// Maximum duration of the ordered dither, clamped to 1–16 frames.
    /// Quantizing the weight to sixteenths can complete it sooner.
    pub transition_frames: u32,
}

impl HybridSettings {
    pub const fn new(
        policy: HybridPolicy,
        resolution: u32,
        max_resident_bytes: u64,
        max_refreshes: u32,
    ) -> Self {
        Self {
            policy,
            resolution,
            max_resident_bytes,
            max_refreshes,
            transition_frames: 8,
        }
    }

    pub const fn with_transition_frames(mut self, frames: u32) -> Self {
        self.transition_frames = frames;
        self
    }
}

/// Body-image work for the last encoded scene frame. Terrain has a separate
/// receipt. Bytes cover the allocated colour/depth atlas, including free
/// slots. Shared mesh buffers and the existing scene targets are separate.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HybridStats {
    pub retained_bytes: u64,
    pub capacity: u32,
    pub resident_slots: u32,
    pub hits: u32,
    pub misses: u32,
    pub refreshes: u32,
    /// Uploads requested by image bakes, separate from the ordinary live pass.
    pub bake_upload_bytes: u64,
    /// Placement/dither data uploaded by the sprite compositor.
    pub sprite_instance_upload_bytes: u64,
    /// Instance storage and cached bake mesh storage are distinct from the
    /// atlas's texel budget.
    pub sprite_instance_buffer_bytes: u64,
    pub bake_mesh_bytes: u64,
    pub evictions: u32,
    pub sprite_bodies: u32,
    /// Bodies that submitted both representations on this frame.
    pub transition_bodies: u32,
    pub live_fallbacks: u32,
    /// Previous atlas awaiting a queue fence after a quality change. No new
    /// atlas is allocated until the older submissions finish.
    pub retiring_bytes: u64,
}

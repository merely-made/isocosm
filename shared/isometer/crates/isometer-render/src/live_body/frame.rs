// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The pass's per-frame uniform, and the sun it may light bodies by.

use std::hash::{DefaultHasher, Hash, Hasher};

use bytemuck::{Pod, Zeroable};

use super::ClipSlab;

/// The scene's sun for live bodies and their baked images (case 1170):
/// `ambient + sun * max(n·toward_sun, 0)`, the terrain tracer's Lambert term.
/// Without one the pass keeps its fixed per-axis face shade.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BodyLight {
    pub toward_sun: [f32; 3],
    pub ambient: f32,
    pub sun: [f32; 3],
}

/// A body's presentation style: how its lit colour reaches the
/// display-encoded target. Chosen per body, so a host can style each class
/// of thing differently (cases 1176, 1178).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum BodyDisplay {
    /// Exact sRGB with no tonemap: the baseline, like the terrain (1178).
    #[default]
    Srgb,
    /// Khronos PBR Neutral, then a 2.2 power curve: the look kiss3d's
    /// resolve gave Mesocosm's retired voxel tenant, kept as a style.
    NeutralGamma22,
}

impl BodyDisplay {
    /// The instance lane value the shader reads.
    pub(super) fn lane(self) -> f32 {
        match self {
            Self::Srgb => 0.0,
            Self::NeutralGamma22 => 1.0,
        }
    }
}

impl BodyLight {
    /// Every term rounded to a multiple of `step`; a step of zero or less, or
    /// a non-finite one, leaves the light exact. The direction is
    /// renormalized after rounding.
    pub fn quantized(self, step: f32) -> Self {
        if !(step.is_finite() && step > 0.0) {
            return self;
        }
        let round = |value: f32| (value / step).round() * step;
        let toward = self.toward_sun.map(round);
        let length = toward.iter().map(|v| v * v).sum::<f32>().sqrt();
        Self {
            toward_sun: if length > 0.0 {
                toward.map(|v| v / length)
            } else {
                self.toward_sun
            },
            ambient: round(self.ambient),
            sun: self.sun.map(round),
        }
    }

    /// A revision for baked images: equal lights give equal stamps.
    pub fn stamp(self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.toward_sun.map(f32::to_bits).hash(&mut hasher);
        self.ambient.to_bits().hash(&mut hasher);
        self.sun.map(f32::to_bits).hash(&mut hasher);
        // Zero stays "unlit".
        hasher.finish().max(1)
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(super) struct FrameUniform {
    clip_from_world: [[f32; 4]; 4],
    slab_normal_min: [f32; 4],
    slab_max_enabled: [f32; 4],
    bounds_min: [f32; 4],
    bounds_max: [f32; 4],
    /// Toward the sun, then ambient.
    light_toward_ambient: [f32; 4],
    /// Sun weight per channel, then 1 when lit.
    light_sun_enabled: [f32; 4],
}

impl FrameUniform {
    pub(super) fn new(
        clip_from_world: [[f32; 4]; 4],
        clip_slab: Option<ClipSlab>,
        light: Option<BodyLight>,
    ) -> Self {
        let slab = clip_slab.unwrap_or(ClipSlab::new([0.0; 3], 0.0, 0.0));
        let (toward, ambient, sun, lit) = light.map_or(([0.0; 3], 0.0, [0.0; 3], 0.0), |l| {
            (l.toward_sun, l.ambient, l.sun, 1.0)
        });
        Self {
            clip_from_world,
            bounds_min: slab
                .bounds
                .map_or([0.0; 4], |(min, _)| [min[0], min[1], min[2], 1.0]),
            bounds_max: slab
                .bounds
                .map_or([0.0; 4], |(_, max)| [max[0], max[1], max[2], 0.0]),
            slab_normal_min: [slab.normal[0], slab.normal[1], slab.normal[2], slab.min],
            slab_max_enabled: [
                slab.max,
                0.0,
                0.0,
                if clip_slab.is_some() { 1.0 } else { 0.0 },
            ],
            light_toward_ambient: [toward[0], toward[1], toward[2], ambient],
            light_sun_enabled: [sun[0], sun[1], sun[2], lit],
        }
    }
}

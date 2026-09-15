// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::{Form, Spatial};
use crate::section::{GlyphAnchor, GlyphOrientation, SpatialGlyph, stroke};

// Stateless appearance noise. It never chooses a receiver rule or mutates World.
fn noise(seed: u64, id: u64) -> f32 {
    let mut n = seed.wrapping_add(id.wrapping_mul(0x9e3779b97f4a7c15));
    n = (n ^ (n >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    n = (n ^ (n >> 27)).wrapping_mul(0x94d049bb133111eb);
    ((n ^ (n >> 31)) >> 40) as f32 / 16777216.0
}
impl Spatial {
    pub fn marks(
        &self,
        bounds: ([f32; 3], [f32; 3]),
        anchors: &[GlyphAnchor],
    ) -> Vec<SpatialGlyph> {
        if !self.enabled {
            return Vec::new();
        }
        let (min, max) = bounds;
        let centre = [0, 1, 2].map(|i| (min[i] + max[i]) * 0.5);
        let radius = ((max[0] - min[0]).max(max[2] - min[2]) * 0.6).max(2.0);
        let height = (max[1] - min[1]).max(2.0);
        let mut result = Vec::new();
        for id in 0..self.count {
            let jitter = noise(self.seed, u64::from(id));
            let phase = (id as f32 * 360.0 / self.count as f32 + self.tick as f32 + jitter * 8.0)
                .to_radians();
            let mut mark = SpatialGlyph {
                centre,
                size: height * 0.18,
                angle: phase * 0.25,
                glyph: stroke(self.glyph),
                orientation: GlyphOrientation::CameraFacing,
                color: if jitter < 0.5 {
                    [0.35, 0.95, 0.72, 1.0]
                } else {
                    [1.0, 0.76, 0.3, 1.0]
                },
            };
            match self.form {
                Form::Orbit => {
                    mark.centre = [
                        centre[0] + radius * phase.cos(),
                        centre[1] + height * 0.22 * (phase * 2.0).sin(),
                        centre[2] + radius * phase.sin(),
                    ]
                },
                Form::Surface => {
                    if anchors.is_empty() {
                        break;
                    }
                    let anchor = &anchors[id as usize % anchors.len()];
                    let per_face = (self.count as usize).div_ceil(anchors.len());
                    let side = (per_face as f32).sqrt().ceil() as usize;
                    let slot = id as usize / anchors.len();
                    let x = ((slot % side) as f32 + 0.5) / side as f32 - 0.5;
                    let y = ((slot / side) as f32 + 0.5) / side as f32 - 0.5;
                    mark.size = anchor.extent[0].min(anchor.extent[1]) * 0.7 / side as f32;
                    mark.centre = [0, 1, 2].map(|i| {
                        anchor.centre[i]
                            + anchor.right[i] * x * anchor.extent[0]
                            + anchor.up[i] * y * anchor.extent[1]
                            + anchor.normal[i] * 0.01
                    });
                    mark.orientation = GlyphOrientation::WorldPlane {
                        right: anchor.right,
                        up: anchor.up,
                    };
                    mark.angle = (jitter - 0.5) * 0.25;
                },
                Form::Tether => {
                    if anchors.len() < 2 {
                        break;
                    }
                    let a = &anchors[0];
                    let b = &anchors[anchors.len() - 1];
                    let t = (id as f32 + 0.5) / self.count as f32;
                    mark.centre = [0, 1, 2].map(|i| a.centre[i] * (1.0 - t) + b.centre[i] * t);
                    mark.centre[1] += (max[1] - a.centre[1].min(b.centre[1]) + height * 0.3)
                        * (t * std::f32::consts::PI).sin();
                    mark.size = height * 0.09;
                    mark.angle = (self.tick as f32 * 0.04 + t * 6.0).sin() * 0.2;
                },
                Form::Emission => {
                    if anchors.is_empty() {
                        break;
                    }
                    let anchor = &anchors[id as usize % anchors.len()];
                    let age =
                        ((self.tick as f32 / 120.0 + id as f32 / self.count as f32) % 1.0).max(0.0);
                    mark.centre = [0, 1, 2].map(|i| {
                        anchor.centre[i]
                            + anchor.normal[i] * (0.02 + age * height * 0.65)
                            + anchor.right[i] * (jitter - 0.5) * age * height * 0.1
                    });
                    mark.size = height * 0.12 * (1.0 - age * 0.7);
                },
            }
            result.push(mark);
        }
        result
    }
}

#[cfg(test)]
mod tests;

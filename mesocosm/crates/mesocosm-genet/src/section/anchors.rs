// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Bounded effect attachments on actual meshed part faces. These faces may be
//! hidden by another part or terrain. The depth renderer owns that occlusion.

use super::{BodyLayer, BodySelection, body_origin};
use crate::section::Section;
use mesocosm_core::{Organism, PartId};
use mesocosm_mesh::{LiveBodyProjection, VolumeMap};
use mesocosm_render::live_body::{LiveBody, posed_quad};

pub const MAX_GLYPH_ANCHORS: usize = 32;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GlyphAnchor {
    pub selection: BodySelection,
    pub part: PartId,
    pub centre: [f32; 3],
    /// Orthonormal face axes. `right × up` points along `normal`.
    pub right: [f32; 3],
    pub up: [f32; 3],
    pub normal: [f32; 3],
    /// Full width and height of the chosen meshed rectangle in world units.
    pub extent: [f32; 2],
}

impl Section {
    /// Largest meshed face per living part, ordered by PartId and capped at
    /// 32. An explicit selection must match this body's current mesh revision.
    /// Uses the configured pose for the next draw, before cutaway/occlusion.
    pub fn glyph_anchors(
        &mut self,
        organism: &Organism,
        volumes: &VolumeMap,
        selected: Option<BodySelection>,
    ) -> Result<Vec<GlyphAnchor>, String> {
        self.bodies.glyph_anchors(organism, volumes, selected)
    }
}

impl BodyLayer {
    fn glyph_anchors(
        &mut self,
        organism: &Organism,
        volumes: &VolumeMap,
        selected: Option<BodySelection>,
    ) -> Result<Vec<GlyphAnchor>, String> {
        let projection = self
            .projector
            .project(organism.id, organism.body(), volumes)
            .map_err(|error| format!("body projection: {error:?}"))?;
        let mut body = LiveBody::new(
            &projection.mesh,
            body_origin(organism, self.scale, self.ground_anatomy),
        );
        body.scale = self.scale;
        body.yaw_radians = self.yaw(organism.id);
        anchors(&projection, body, selected)
    }
}

fn anchors(
    projection: &LiveBodyProjection,
    body: LiveBody<'_>,
    selected: Option<BodySelection>,
) -> Result<Vec<GlyphAnchor>, String> {
    if selected
        .is_some_and(|s| s.organism != projection.organism || s.revision != projection.revision)
    {
        return Err("glyph attachment selection has expired".into());
    }
    let mut placements: Vec<_> = projection
        .mesh
        .placements
        .iter()
        .filter(|p| selected.is_none_or(|s| s.part == p.part))
        .collect();
    placements.sort_by_key(|p| p.part);
    if selected.is_some() && placements.is_empty() {
        return Err("glyph attachment part is unavailable".into());
    }
    let mut result = Vec::new();
    for placement in placements {
        let mesh = projection
            .mesh
            .mesh_for(placement.volume)
            .ok_or("missing glyph attachment mesh")?;
        // Stable first-face tie-break: mesher axis/direction/slice order.
        let mut largest = None;
        for (index, quad) in mesh.quads.iter().enumerate() {
            let area = u64::from(quad.size[0]) * u64::from(quad.size[1]);
            if largest.is_none_or(|(_, best)| area > best) {
                largest = Some((index, area));
            }
        }
        let Some((index, _)) = largest else {
            continue;
        };
        let quad = &mesh.quads[index];
        let corners = posed_quad(body, placement.part, index)
            .map_err(|error| format!("glyph attachment pose: {error:?}"))?;
        let centre = [0, 1, 2].map(|axis| corners.iter().map(|p| p[axis]).sum::<f32>() * 0.25);
        let right_edge = difference(corners[if quad.positive { 1 } else { 3 }], corners[0]);
        let up_edge = difference(corners[if quad.positive { 3 } else { 1 }], corners[0]);
        let width = length(right_edge);
        let height = length(up_edge);
        if width <= f32::EPSILON || height <= f32::EPSILON {
            continue;
        }
        let right = right_edge.map(|v| v / width);
        let mut up = up_edge.map(|v| v / height);
        // Quad::corners has reversed y-face winding; axis and polarity remain
        // authoritative, so do not infer an outward normal from that winding.
        if quad.positive == (quad.axis == 1) {
            up = up.map(|v| -v);
        }
        let normal = cross(right, up);
        result.push(GlyphAnchor {
            selection: BodySelection {
                organism: projection.organism,
                part: placement.part,
                revision: projection.revision,
            },
            part: placement.part,
            centre,
            right,
            up,
            normal,
            extent: [width, height],
        });
        if result.len() == MAX_GLYPH_ANCHORS {
            break;
        }
    }
    if selected.is_some() && result.is_empty() {
        return Err("glyph attachment part has no meshed face".into());
    }
    Ok(result)
}

fn difference(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [0, 1, 2].map(|i| a[i] - b[i])
}
fn length(a: [f32; 3]) -> f32 {
    a.iter().map(|v| v * v).sum::<f32>().sqrt()
}
fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[cfg(test)]
#[path = "anchors_tests.rs"]
mod tests;

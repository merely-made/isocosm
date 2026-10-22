// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::{Primitive, Surface};
use crate::{LiveBody, material_colour};
use glam::{Mat4, Quat, Vec3};
use isometer_core::{BodyDocument, Role};

#[derive(Clone, Copy, Debug, Default)]
pub struct AnatomyStats {
    pub joints: usize,
    /// Reachable IK sole targets, distinct from an admitted rigid support.
    pub contacts: usize,
    /// Part-box contacts retained in the softened skin.
    pub supports: usize,
}

/// Two fixed-length links in the plane of the target and pole. Reach is
/// clamped, including folded, coincident and overextended targets.
pub fn two_bone(root: Vec3, target: Vec3, pole: Vec3, upper: f32, lower: f32) -> (Vec3, Vec3) {
    let delta = target - root;
    let direction = delta.try_normalize().unwrap_or(-Vec3::Y);
    let distance = delta.length().clamp(
        (upper - lower).abs() + 1e-4,
        (upper + lower - 1e-4).max(1e-4),
    );
    let along = (upper * upper - lower * lower + distance * distance) / (2.0 * distance);
    let bend = (pole - direction * direction.dot(pole))
        .try_normalize()
        .unwrap_or_else(|| direction.any_orthonormal_vector());
    (
        root + direction * along + bend * (upper * upper - along * along).max(0.0).sqrt(),
        root + direction * distance,
    )
}

/// A graph-derived rest stance. Long downward parts become two fixed-length
/// presentation links within that declared limb, not new saved anatomy. The
/// graph supplies attachments; the host supplies actual terrain contacts.
/// There is deliberately no clock-driven idle motion or saved animation.
pub fn from_anatomy(
    document: &BodyDocument,
    body: LiveBody<'_>,
    mut ground: impl FnMut([f32; 3]) -> Option<f32>,
) -> Result<(Surface, AnatomyStats), String> {
    let mut surface = Surface::default();
    let mut stats = AnatomyStats::default();
    let mut smallest = f32::INFINITY;
    let bounds = document.aabb();
    let trunk = if i64::from(bounds.max[2]) - i64::from(bounds.min[2])
        >= i64::from(bounds.max[0]) - i64::from(bounds.min[0])
    {
        Vec3::Z
    } else {
        Vec3::X
    };
    let bend_pole = Quat::from_rotation_y(body.yaw_radians) * trunk;
    for placement in &body.mesh.placements {
        let part = document
            .part(placement.part)
            .ok_or("posed part absent from anatomy")?;
        if part.half_extent.iter().any(|h| *h <= 0) {
            continue;
        }
        let matrix = Mat4::from_cols_array_2d(
            &crate::live_body::part_transform(body, part.id)
                .map_err(|e| format!("part pose: {e:?}"))?,
        );
        let (scale, rotation, _) = matrix.to_scale_rotation_translation();
        let local_centre = Vec3::from_array(part.half_extent.map(|v| v as f32));
        let centre = matrix.transform_point3(local_centre);
        let half = local_centre * scale.abs();
        let role = isometer_core::classify(part.half_extent);
        let support = part.attachment.is_some_and(|a| a.offset[1] < 0)
            && !document
                .living()
                .any(|p| p.attachment.is_some_and(|a| a.parent == part.id))
            && matches!(role, Role::Mass | Role::Limb)
            && i64::from(part.half_extent[1]) * 2 >= i64::from(part.half_extent[0])
            && i64::from(part.half_extent[1]) * 2 >= i64::from(part.half_extent[2]);
        let radius = half.min_element()
            * if support {
                0.35
            } else if role == Role::Mass {
                0.8
            } else {
                0.45
            };
        smallest = smallest.min(radius);
        let material = body
            .mesh
            .mesh_for(placement.volume)
            .and_then(|m| m.quads.first())
            .map_or(1, |q| q.material);
        let base = body
            .palette
            .and_then(|p| p.linear(material))
            .unwrap_or_else(|| material_colour(material));
        let mut colour =
            [0, 1, 2].map(|i| base[i] * body.tint[i] * if body.focused { 1.08 } else { 1.0 });
        if body.selected_part == Some(part.id) {
            colour = [0, 1, 2].map(|i| colour[i] * 0.72 + [1.0, 0.55, 0.1][i] * 0.28);
        }
        // Generated native bodies also express supports as a terminal cube
        // attached below its parent. Read that graph edge, not just aspect.
        let axis = if support {
            1
        } else {
            (0..3).max_by_key(|i| part.half_extent[*i]).unwrap()
        };
        let along = rotation * Vec3::AXES[axis];
        let span = half[axis] - radius;
        let top = centre + along * span;
        let bottom = centre - along * span;
        if (role == Role::Limb || support)
            && axis == 1
            && along.y > 0.7
            && part.attachment.is_some()
            && centre.y < body.origin[1] - radius
        {
            let mut target = bottom;
            let contact = ground(bottom.to_array());
            if let Some(height) = contact {
                target.y = (height + radius).max(bottom.y);
            }
            let pole = bend_pole;
            // Slightly folded rest links fit inside the limb's thickness.
            // The foot is not lifted just to manufacture an idle pose.
            let reserve = half.min_element() - radius;
            let link = (span * span + reserve * reserve).sqrt();
            let (knee, foot) = two_bone(top, target, pole, link, link);
            if contact.is_some_and(|height| (foot.y - radius - height).abs() < 0.025) {
                stats.contacts += 1;
            }
            surface
                .parts
                .push(Primitive::capsule(part.id, top, knee, radius, colour));
            surface
                .parts
                .push(Primitive::capsule(part.id, knee, foot, radius, colour));
            stats.joints += 1;
        } else {
            // Preserve broad mass and thin plates; soften the corners within
            // each declared envelope rather than giving every part one blob.
            surface.parts.push(Primitive::rounded_box(
                part.id, centre, half, rotation, radius, colour,
            ));
        }
        if let Some(attachment) = part.attachment {
            if let Some(parent) = document.part(attachment.parent) {
                let parent_matrix = Mat4::from_cols_array_2d(
                    &crate::live_body::part_transform(body, parent.id)
                        .map_err(|e| format!("attachment pose: {e:?}"))?,
                );
                let parent_centre = parent_matrix
                    .transform_point3(Vec3::from_array(parent.half_extent.map(|v| v as f32)));
                // Consecutive masses form a continuous trunk. Thin attachments
                // keep a narrower neck so limbs remain distinct from the trunk.
                let join = if !support
                    && role == Role::Mass
                    && isometer_core::classify(parent.half_extent) == Role::Mass
                {
                    half.min_element()
                        .min(parent.half_extent.into_iter().min().unwrap_or(0) as f32 * body.scale)
                        * 0.9
                } else {
                    radius * 0.8
                };
                surface.parts.push(Primitive::capsule(
                    part.id,
                    parent_centre,
                    centre,
                    join,
                    colour,
                ));
            }
        }
        if super::contacts::preserve(
            &mut surface,
            part.id,
            matrix,
            local_centre,
            colour,
            &mut ground,
        ) {
            stats.supports += 1;
        }
    }
    surface.blend = if smallest.is_finite() {
        smallest * 0.35
    } else {
        0.0
    };
    Ok((surface, stats))
}

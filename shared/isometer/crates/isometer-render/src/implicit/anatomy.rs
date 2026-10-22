// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::{Gait, Primitive, Surface};
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
    pub planted: usize,
    pub swinging: usize,
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
    anatomy(document, body, &mut ground, None)
}

/// Contact witnesses come from the native physical placement. The disposable
/// gait keeps their world anchors across admitted root movements. No witness
/// is synthesized for a hanging limb or an unsupported part.
pub fn walking_anatomy(
    document: &BodyDocument,
    body: LiveBody<'_>,
    ground: &mut impl FnMut([f32; 3]) -> Option<f32>,
    contacts: &std::collections::BTreeMap<isometer_core::PartId, Vec3>,
    gait: &mut Gait,
    elapsed: f32,
) -> Result<(Surface, AnatomyStats), String> {
    gait.begin(Vec3::from_array(body.origin), elapsed);
    let result = anatomy(document, body, ground, Some((contacts, &mut *gait)));
    gait.finish();
    result
}

fn anatomy(
    document: &BodyDocument,
    body: LiveBody<'_>,
    ground: &mut impl FnMut([f32; 3]) -> Option<f32>,
    mut walking: Option<(
        &std::collections::BTreeMap<isometer_core::PartId, Vec3>,
        &mut Gait,
    )>,
) -> Result<(Surface, AnatomyStats), String> {
    let mut surface = Surface::default();
    let mut stats = AnatomyStats::default();
    let mut smallest = f32::INFINITY;
    let parents: std::collections::BTreeSet<_> = document
        .living()
        .filter_map(|part| part.attachment.map(|a| a.parent))
        .collect();
    let bounds = document.aabb();
    let trunk = if i64::from(bounds.max[2]) - i64::from(bounds.min[2])
        >= i64::from(bounds.max[0]) - i64::from(bounds.min[0])
    {
        Vec3::Z
    } else {
        Vec3::X
    };
    let bend_pole = Quat::from_rotation_y(body.yaw_radians) * trunk;
    let handled = if let Some((contacts, gait)) = walking.as_mut() {
        let (chain_surface, chain_stats, handled) =
            super::chains::pose(document, body, contacts, gait, ground)?;
        surface.parts.extend(chain_surface.parts);
        stats = chain_stats;
        handled
    } else {
        std::collections::BTreeSet::new()
    };
    for placement in &body.mesh.placements {
        if handled.contains(&placement.part) {
            continue;
        }
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
        let terminal = !parents.contains(&part.id);
        let support = part.attachment.is_some_and(|a| a.offset[1] < 0)
            && terminal
            && matches!(role, Role::Mass | Role::Limb)
            && i64::from(part.half_extent[1]) * 2 >= i64::from(part.half_extent[0])
            && i64::from(part.half_extent[1]) * 2 >= i64::from(part.half_extent[2]);
        let radius = half.min_element()
            * if support {
                0.35
            } else if role == Role::Mass {
                0.35
            } else {
                0.45
            };
        smallest = smallest.min(radius);
        let colour = colour(body, part.id);
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
        let articulated = terminal
            && (role == Role::Limb || support)
            && axis == 1
            && along.y > 0.7
            && part.attachment.is_some()
            && centre.y < body.origin[1] - radius;
        let contact_pose = if let Some((contacts, gait)) = walking.as_mut() {
            contacts.get(&part.id).map(|point| {
                let [min, mut max] = gait.envelope(super::contacts::bounds(matrix, local_centre));
                // A cell-centred native witness gives a pad half a cell of
                // room. Its whole rounded end remains in the part envelope.
                let pad_radius = (half.min_element() * 0.15).min(0.2 * body.scale);
                let contact_radius = if articulated { radius } else { pad_radius };
                if articulated {
                    // Bound lift by the space available for fixed links to fold.
                    let reserve = half.min_element() - radius;
                    let minimum = 2. * (span * span - reserve * reserve).max(0.).sqrt();
                    let lift = (2. * span - minimum).max(0.) * 0.7;
                    max.y = max.y.min(point.y + 2. * radius + lift);
                }
                let (at, planted) = gait.foot(part.id, *point, [min, max], contact_radius, ground);
                (at, planted, pad_radius)
            })
        } else {
            None
        };
        if articulated {
            let mut target = bottom;
            let contact = ground(bottom.to_array());
            if let Some(height) = contact {
                target.y = (height + radius).max(bottom.y);
            }
            let reserve = half.min_element() - radius;
            let link = (span * span + reserve * reserve).sqrt();
            let mut posed = None;
            if let Some((point, _, _)) = contact_pose {
                let planted_target = point + Vec3::Y * radius;
                // Respect both fixed lengths and the admitted part envelope.
                // Try the anatomy's bend plane in both directions, then its
                // perpendicular, rather than pushing a knee through terrain.
                let [min, max] = walking
                    .as_ref()
                    .unwrap()
                    .1
                    .envelope(super::contacts::bounds(matrix, local_centre));
                let inside = |p: Vec3| {
                    p.cmpge(min + radius - 0.001).all() && p.cmple(max - radius + 0.001).all()
                };
                if top.distance(planted_target) < link * 2. - 1e-4 {
                    let across = bend_pole.cross(Vec3::Y);
                    for pole in [
                        bend_pole,
                        -bend_pole,
                        across,
                        -across,
                        bend_pole + across,
                        bend_pole - across,
                        -bend_pole + across,
                        -bend_pole - across,
                    ] {
                        let candidate = two_bone(top, planted_target, pole, link, link);
                        if inside(candidate.0) && inside(candidate.1) {
                            posed = Some(candidate);
                            break;
                        }
                    }
                }
            }
            // Slightly folded rest links fit inside the limb's thickness.
            // The foot is not lifted just to manufacture an idle pose.
            let (knee, foot) =
                posed.unwrap_or_else(|| two_bone(top, target, bend_pole, link, link));
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
                        * 0.45
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
        if let Some((point, planted, radius)) = contact_pose {
            super::contacts::pad(&mut surface, part.id, centre, point, radius, colour);
            stats.supports += usize::from(planted);
        } else if walking.is_none() {
            if super::contacts::preserve(
                &mut surface,
                part.id,
                matrix,
                local_centre,
                colour,
                ground,
            ) {
                stats.supports += 1;
            }
        }
    }
    surface.blend = if smallest.is_finite() {
        smallest * 0.2
    } else {
        0.0
    };
    if let Some((_, gait)) = walking {
        stats.planted = gait.planted;
        stats.swinging = gait.swinging;
    }
    Ok((surface, stats))
}

pub(super) fn colour(body: LiveBody<'_>, id: isometer_core::PartId) -> [f32; 3] {
    let material = body
        .mesh
        .mesh_for(
            body.mesh
                .placements
                .iter()
                .find(|p| p.part == id)
                .expect("anatomy placement")
                .volume,
        )
        .and_then(|m| m.quads.first())
        .map_or(1, |q| q.material);
    let base = body
        .palette
        .and_then(|p| p.linear(material))
        .unwrap_or_else(|| material_colour(material));
    let mut colour =
        [0, 1, 2].map(|i| base[i] * body.tint[i] * if body.focused { 1.08 } else { 1.0 });
    colour = crate::tissue_average(colour, body.materials, id);
    if body.selected_part == Some(id) {
        colour = [0, 1, 2].map(|i| colour[i] * 0.72 + [1.0, 0.55, 0.1][i] * 0.28);
    }
    colour
}

// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Draw the same fixed-point paid links native clearance admitted.
use super::{AnatomyStats, Gait, Primitive, Surface, anatomy::colour};
use crate::LiveBody;
use glam::{Quat, Vec3};
use isometer_core::{
    BodyDocument, PartId,
    legs::{self, UNIT},
};
use std::collections::BTreeSet;

pub(super) fn pose(
    doc: &BodyDocument,
    body: LiveBody<'_>,
    gait: &mut Gait,
    ground: &mut impl FnMut([f32; 3]) -> Option<f32>,
) -> Result<(Surface, AnatomyStats, BTreeSet<PartId>), String> {
    let mut surface = Surface::default();
    let mut stats = AnatomyStats::default();
    let mut handled = BTreeSet::new();
    let feet = gait.physical.as_ref().ok_or("physical sole map absent")?;
    let rotation = Quat::from_rotation_y(body.yaw_radians);
    let origin = Vec3::from_array(body.origin);
    let point = |p: [i64; 3]| {
        origin + rotation * Vec3::from_array(p.map(|v| v as f32 / UNIT as f32)) * body.scale
    };
    for leg in legs::chains(doc) {
        let Some(&sole) = feet.get(&leg.foot) else {
            continue;
        };
        let skin = leg
            .skin(sole)
            .ok_or("displayed sole exceeds admitted chain reach")?;
        let knee = skin.upper.b;
        let ankle = skin.lower.b;
        let radius = leg.radius as f32 / UNIT as f32 * body.scale;
        let pad = point([sole[0], sole[1] + leg.half[1], sole[2]]);
        surface.parts.extend([
            Primitive::capsule(
                leg.upper,
                point(leg.shoulder),
                point(knee),
                radius,
                colour(body, leg.upper),
            ),
            Primitive::capsule(
                leg.lower,
                point(knee),
                point(ankle),
                radius,
                colour(body, leg.lower),
            ),
            Primitive::rounded_box(
                leg.foot,
                pad,
                Vec3::from_array(leg.half.map(|v| v as f32 / UNIT as f32)) * body.scale,
                rotation,
                radius,
                colour(body, leg.foot),
            ),
            Primitive::capsule(
                leg.foot,
                point(ankle),
                pad,
                skin.connector.radius as f32 / UNIT as f32 * body.scale,
                colour(body, leg.foot),
            ),
        ]);
        let centre = point(sole);
        // A rigid pad can touch a high voxel at its edge, away from its centre.
        // Read actual supporting columns, as native foot admission does.
        let h = Vec3::from_array(leg.half.map(|v| v as f32 / UNIT as f32)) * body.scale;
        let extent = (rotation * Vec3::X).abs() * h.x + (rotation * Vec3::Z).abs() * h.z;
        let planted = (centre.x - extent.x).floor() as i32..(centre.x + extent.x).ceil() as i32;
        let planted = planted.into_iter().any(|x| {
            ((centre.z - extent.z).floor() as i32..(centre.z + extent.z).ceil() as i32).any(|z| {
                ground([x as f32 + 0.5, centre.y, z as f32 + 0.5])
                    .is_some_and(|y| (y - centre.y).abs() < 0.002)
            })
        });
        stats.joints += 2;
        stats.contacts += 1;
        stats.supports += usize::from(planted);
        gait.planted += usize::from(planted);
        gait.swinging += usize::from(!planted);
        handled.extend([leg.upper, leg.lower, leg.foot]);
    }
    Ok((surface, stats, handled))
}

// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Preserve the support points of the admitted part box when rounding its skin.
use super::{Primitive, Surface};
use glam::{Mat4, Vec3};
use isometer_core::PartId;

pub(super) fn bounds(matrix: Mat4, half: Vec3) -> [Vec3; 2] {
    let mut min = Vec3::splat(f32::INFINITY);
    let mut max = -min;
    for x in [0., half.x * 2.] {
        for y in [0., half.y * 2.] {
            for z in [0., half.z * 2.] {
                let p = matrix.transform_point3(Vec3::new(x, y, z));
                min = min.min(p);
                max = max.max(p);
            }
        }
    }
    [min, max]
}

pub(super) fn pad(
    surface: &mut Surface,
    part: PartId,
    centre: Vec3,
    point: Vec3,
    radius: f32,
    colour: [f32; 3],
) {
    surface.parts.push(Primitive::capsule(
        part,
        centre,
        point + Vec3::Y * radius,
        radius,
        colour,
    ));
}

pub(super) fn preserve(
    surface: &mut Surface,
    part: PartId,
    matrix: Mat4,
    half: Vec3,
    colour: [f32; 3],
    ground: &mut impl FnMut([f32; 3]) -> Option<f32>,
) -> bool {
    let centre = matrix.transform_point3(half);
    let inverse = matrix.inverse();
    let scale = matrix.transform_vector3(Vec3::X).length();
    let [min, max] = bounds(matrix, half);
    let mut best: Option<(f32, Vec3, f32)> = None;
    let mut consider = |x: f32, z: f32| {
        let Some(height) = ground([x, min.y, z]) else {
            return;
        };
        if (height - min.y).abs() > 0.025 {
            return;
        }
        let point = Vec3::new(x, height, z);
        let local = inverse.transform_point3(point);
        if local.cmplt(Vec3::splat(-0.001)).any() || local.cmpgt(half * 2. + 0.001).any() {
            return;
        }
        let margin = local
            .x
            .min(half.x * 2. - local.x)
            .min(local.z.min(half.z * 2. - local.z))
            * scale;
        let radius = (half.min_element() * scale * 0.2).min(margin * 0.8);
        if radius <= 0.001 {
            return;
        }
        let distance = (point.x - centre.x).powi(2) + (point.z - centre.z).powi(2);
        if best.is_none_or(|(d, _, _)| distance < d) {
            best = Some((distance, point, radius));
        }
    };
    consider(centre.x, centre.z);
    let width = (max.x.ceil() - min.x.floor()).max(1.) as usize;
    let depth = (max.z.ceil() - min.z.floor()).max(1.) as usize;
    let stride = ((width as f64 * depth as f64 / 256.).sqrt().ceil() as usize).max(1);
    for x in (0..width).step_by(stride) {
        for z in (0..depth).step_by(stride) {
            consider(
                min.x.floor() + x as f32 + 0.5,
                min.z.floor() + z as f32 + 0.5,
            );
        }
    }
    let Some((_, point, radius)) = best else {
        return false;
    };
    if surface.sample(point.to_array()).0 > 0.015 {
        // Both centres and the capsule radius fit in the admitted part box.
        // A rounded corner may remove material from a real supporting cell;
        // this short pad restores that contact under the same part identity.
        pad(surface, part, centre, point, radius, colour);
    }
    true
}

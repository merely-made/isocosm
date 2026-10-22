// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Real upper/lower/foot graph links. The saved boxes still own clearance;
//! pose candidates must stay inside their union and its native-cleared sweep.
use super::{
    AnatomyStats, Gait, Primitive, Surface,
    anatomy::{colour, two_bone},
};
use crate::{LiveBody, live_body::part_transform};
use glam::{Mat4, Vec3};
use isometer_core::{BodyDocument, PartId, Role, classify};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn pose(
    doc: &BodyDocument,
    body: LiveBody<'_>,
    contacts: &BTreeMap<PartId, Vec3>,
    gait: &mut Gait,
    ground: &mut impl FnMut([f32; 3]) -> Option<f32>,
) -> Result<(Surface, AnatomyStats, BTreeSet<PartId>), String> {
    if gait.physical.is_some() {
        return super::physical_chains::pose(doc, body, gait, ground);
    }
    let mut surface = Surface::default();
    let mut stats = AnatomyStats::default();
    let mut handled = BTreeSet::new();
    let parents: BTreeSet<_> = doc
        .living()
        .filter_map(|p| p.attachment.map(|a| a.parent))
        .collect();
    for foot in doc.living().filter(|p| !parents.contains(&p.id)) {
        let Some(ankle) = foot.attachment else {
            continue;
        };
        let Some(lower) = doc.part(ankle.parent).filter(|p| !p.severed) else {
            continue;
        };
        let Some(knee) = lower.attachment else {
            continue;
        };
        let Some(upper) = doc.part(knee.parent).filter(|p| !p.severed) else {
            continue;
        };
        let Some(hip) = upper.attachment else {
            continue;
        };
        if [upper, lower, foot]
            .iter()
            .any(|p| p.half_extent.iter().any(|h| *h <= 0)
                || !body.mesh.placements.iter().any(|placed| placed.part == p.id)
                || classify(p.half_extent) != Role::Limb)
            || lower.half_extent[1] <= lower.half_extent[0].max(lower.half_extent[2])
            || knee.offset[1] >= 0
            || ankle.offset[1] >= 0
            || hip.offset[0] == 0
            || handled.contains(&upper.id)
        {
            continue;
        }
        let mut matrices = Vec::new();
        let mut centres = Vec::new();
        let mut halves = Vec::new();
        let mut bounds = Vec::new();
        for part in [upper, lower, foot] {
            let matrix = Mat4::from_cols_array_2d(
                &part_transform(body, part.id).map_err(|e| format!("chain frame: {e:?}"))?,
            );
            let h = Vec3::from_array(part.half_extent.map(|v| v as f32));
            centres.push(matrix.transform_point3(h));
            halves.push(h * body.scale);
            bounds.push(gait.envelope(super::contacts::bounds(matrix, h)));
            matrices.push(matrix);
        }
        let outward =
            matrices[0].transform_vector3(Vec3::X).normalize() * (hip.offset[0] as f32).signum();
        let radius = halves
            .iter()
            .map(|h| h.min_element())
            .fold(f32::INFINITY, f32::min)
            * 0.35;
        let shoulder = centres[0] - outward * (halves[0].x - radius);
        let rest_knee = centres[0] + outward * (halves[0].x - 1.5 * radius);
        let rest_ankle = Vec3::new(centres[1].x, centres[2].y + 2. * radius, centres[1].z);
        let upper_length = shoulder.distance(rest_knee);
        let lower_length = rest_knee.distance(rest_ankle);
        let mut posed = (rest_knee, rest_ankle);
        let mut foot_half = halves[2];
        foot_half.y = radius * 1.5;
        let mut foot_centre = centres[2] - Vec3::Y * (halves[2].y - foot_half.y);
        let mut planted = false;
        let corridors = corridors(&bounds);
        if !segment_fits(shoulder, rest_knee, &corridors, radius)
            || !segment_fits(rest_knee, rest_ankle, &corridors, radius)
            || !segment_fits(rest_ankle, foot_centre, &corridors, radius * 0.6)
        { continue; }
        if let Some(&witness) = contacts.get(&foot.id) {
            let (sole, stance) = gait.foot(foot.id, witness, bounds[2], radius, ground);
            let delta = sole - witness + gait.native_offset(body.origin.into());
            let target = rest_ankle + delta;
            let pole = rest_knee - shoulder;
            let candidate = two_bone(shoulder, target, pole, upper_length, lower_length);
            // Keep the foot a real rigid part. The two paid limb links bend
            // with fixed lengths only where the existing native envelope permits.
            let fits = |a, b| segment_fits(a, b, &corridors, radius);
            let shifted = foot_centre + delta;
            let (_, rotation, _) = matrices[2].to_scale_rotation_translation();
            let extent = (rotation * Vec3::X).abs() * foot_half.x
                + Vec3::Y * foot_half.y + (rotation * Vec3::Z).abs() * foot_half.z;
            let foot_fits = (shifted - extent).cmpge(bounds[2][0] - 0.002).all()
                && (shifted + extent).cmple(bounds[2][1] + 0.002).all();
            if candidate.1.distance(target) < 0.01
                && fits(shoulder, candidate.0)
                && fits(candidate.0, candidate.1)
                && segment_fits(candidate.1, shifted, &corridors, radius * 0.6) && foot_fits
            {
                posed = candidate;
                foot_centre += delta;
                planted = stance;
                stats.contacts += 1;
            } else {
                gait.reject(foot.id, stance);
            }
        }
        surface.parts.push(Primitive::capsule(
            upper.id,
            shoulder,
            posed.0,
            radius,
            colour(body, upper.id),
        ));
        surface.parts.push(Primitive::capsule(
            lower.id,
            posed.0,
            posed.1,
            radius,
            colour(body, lower.id),
        ));
        let (_, rotation, _) = matrices[2].to_scale_rotation_translation();
        surface.parts.push(Primitive::rounded_box(
            foot.id,
            foot_centre,
            foot_half,
            rotation,
            radius,
            colour(body, foot.id),
        ));
        surface.parts.push(Primitive::capsule(
            foot.id,
            posed.1,
            foot_centre,
            radius * 0.6,
            colour(body, foot.id),
        ));
        stats.joints += 2;
        stats.supports += usize::from(planted);
        handled.extend([upper.id, lower.id, foot.id]);
    }
    Ok((surface, stats, handled))
}

#[cfg(test)]
mod tests;

// Rectangular corridors across overlapping or face-adjacent admitted boxes.
// Each corridor is a subset of their union, including its shared face.
fn corridors(boxes: &[[Vec3; 2]]) -> Vec<[Vec3; 2]> {
    let mut out = boxes.to_vec();
    for a in boxes { for b in boxes { for axis in 0..3 {
        if a[0][axis] > b[1][axis] || b[0][axis] > a[1][axis] { continue; }
        let mut lo = a[0].max(b[0]);
        let mut hi = a[1].min(b[1]);
        lo[axis] = a[0][axis].min(b[0][axis]);
        hi[axis] = a[1][axis].max(b[1][axis]);
        if lo.cmplt(hi).all() { out.push([lo,hi]); }
    }}}
    out
}

// Exact coverage of the entire capsule centreline by eroded clear boxes.
fn segment_fits(a: Vec3, b: Vec3, boxes: &[[Vec3; 2]], radius: f32) -> bool {
    let delta = b - a;
    let mut spans = Vec::new();
    for [lo,hi] in boxes {
        let lo = *lo + radius - 0.002;
        let hi = *hi - radius + 0.002;
        let (mut start, mut end) = (0f32, 1f32);
        for axis in 0..3 {
            if delta[axis].abs() < 1e-7 {
                if a[axis] < lo[axis] || a[axis] > hi[axis] { end = -1.; break; }
            } else {
                let p = (lo[axis]-a[axis])/delta[axis];
                let q = (hi[axis]-a[axis])/delta[axis];
                start = start.max(p.min(q)); end = end.min(p.max(q));
            }
        }
        if start <= end { spans.push((start,end)); }
    }
    spans.sort_by(|a,b|a.0.total_cmp(&b.0));
    let mut covered = 0f32;
    for (start,end) in spans {
        if start > covered + 0.0001 { return false; }
        covered = covered.max(end);
    }
    covered >= 1. - 0.0001
}

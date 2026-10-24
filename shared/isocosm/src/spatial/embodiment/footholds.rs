// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Nearby dry sole targets, using the same height cache and paid chain checks.
use super::*;
use isometer_core::legs::{Leg, UNIT};

pub(super) fn targets(
    g: &mut Grounding<'_>,
    leg: &Leg,
    x: i64,
    z: i64,
    yaw: Yaw,
    radius: i64,
) -> Result<Vec<[i64; 3]>> {
    let half = super::super::birth::rotate(yaw, leg.half)?.map(i64::abs);
    let mut targets = Vec::new();
    for dx in -radius..=radius {
        for dz in -radius..=radius {
            let local = [leg.sole[0] + dx * UNIT, 0, leg.sole[2] + dz * UNIT];
            let p = super::super::birth::rotate(yaw, local)?;
            let centre = [x * UNIT + p[0], 0, z * UNIT + p[2]];
            let mut heights = Vec::new();
            let mut dry = true;
            for xx in
                (centre[0] - half[0]).div_euclid(UNIT)..-(-(centre[0] + half[0])).div_euclid(UNIT)
            {
                for zz in (centre[2] - half[2]).div_euclid(UNIT)
                    ..-(-(centre[2] + half[2])).div_euclid(UNIT)
                {
                    let Ok(y) = articulation::top(g, xx, zz) else {
                        dry = false;
                        continue;
                    };
                    if !contacts::supported(g.volume, [xx, y, zz]) {
                        dry = false;
                    }
                    heights.push(y + 1);
                }
            }
            if !dry || heights.is_empty() {
                continue;
            }
            let height = *heights.iter().max().unwrap();
            let coverage = heights.iter().filter(|h| **h == height).count();
            targets.push((
                std::cmp::Reverse(coverage),
                dx.abs() + dz.abs(),
                dx,
                dz,
                [local[0], height * UNIT, local[2]],
            ));
        }
    }
    targets.sort();
    if targets.is_empty() {
        return Err("foot footprint lacks nearby dry ground".into());
    }
    Ok(targets.into_iter().map(|(_, _, _, _, p)| p).collect())
}

pub(super) fn poses(
    g: &Grounding<'_>,
    doc: &BodyDocument,
    chains: &[Leg],
    options: &BTreeMap<isometer_core::PartId, Vec<[i64; 3]>>,
    at: [i64; 3],
    yaw: Yaw,
    tried: &mut usize,
) -> Result<Option<Pose>> {
    let mut choices = Vec::new();
    for leg in chains {
        let local: Vec<_> = options[&leg.foot]
            .iter()
            .filter_map(|p| {
                let sole = [p[0], p[1] - at[1] * UNIT, p[2]];
                if leg.solve([sole[0], sole[1] + UNIT, sole[2]]).is_none()
                    || leg.solve([sole[0], sole[1] - UNIT, sole[2]]).is_none()
                {
                    return None;
                }
                if g.walking.is_some()
                    && narrow::leg_clear(g.volume, leg, sole, at.map(|v| v * UNIT), yaw).is_err()
                {
                    return None;
                }
                Some(sole)
            })
            .take(if g.walking.is_some() { 4 } else { 1 })
            .collect();
        if local.is_empty() {
            return Ok(None);
        }
        choices.push((leg.foot, local));
    }
    let base: BTreeMap<_, _> = choices.iter().map(|(id, ps)| (*id, ps[0])).collect();
    let mut variants = vec![base.clone()];
    if g.walking.is_some() {
        for (id, ps) in choices {
            for p in ps.into_iter().skip(1) {
                let mut feet = base.clone();
                feet.insert(id, p);
                variants.push(feet);
            }
        }
    }
    for feet in variants {
        *tried += 1;
        if g.walking.is_some_and(|r| *tried > usize::from(r.postures)) {
            return Err("nearby foothold posture budget exhausted".into());
        }
        let pose = Pose { at, yaw, feet };
        if check(g.volume, doc, &pose).is_ok() {
            return Ok(Some(pose));
        }
    }
    Ok(None)
}

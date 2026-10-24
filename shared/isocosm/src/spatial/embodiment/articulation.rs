// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Terrain-grounded paid chains. Root and soles are admitted native state;
//! occupied bounds and the renderer use the same integer chain geometry.
use super::*;
use isometer_core::{
    PartId,
    legs::{self, UNIT},
};

pub fn articulated(sim: &Simulation) -> bool {
    sim.genesis()
        .rules
        .spatial_templates
        .as_ref()
        .is_some_and(|t| t.version >= 4)
}

pub fn foot_points(pose: &Pose) -> Result<BTreeMap<PartId, [i64; 3]>> {
    pose.feet
        .iter()
        .map(|(&id, &local)| {
            let offset = super::super::birth::rotate(pose.yaw, local)?;
            let point = [0, 1, 2].map(|k| {
                pose.at[k]
                    .checked_mul(UNIT)
                    .and_then(|v| v.checked_add(offset[k]))
                    .ok_or("sole coordinate overflow")
            });
            Ok((
                id,
                [point[0].clone()?, point[1].clone()?, point[2].clone()?],
            ))
        })
        .collect()
}

pub(super) fn part_boxes(document: &BodyDocument, pose: &Pose) -> Result<BTreeMap<PartId, Box3>> {
    if pose.feet.is_empty() {
        return Ok(BTreeMap::new());
    }
    if pose
        .feet
        .values()
        .flatten()
        .any(|v| v.unsigned_abs() > (1u64 << 42))
        || pose
            .at
            .iter()
            .any(|v| v.unsigned_abs() > (i64::MAX / UNIT / 2) as u64)
    {
        return Err("articulated coordinates exceed bounded arithmetic".into());
    }
    let chains = legs::chains(document);
    if chains.len() != pose.feet.len() {
        return Err("placed feet differ from living paid chains".into());
    }
    let mut out = BTreeMap::new();
    for leg in chains {
        let sole = *pose
            .feet
            .get(&leg.foot)
            .ok_or("paid foot has no admitted sole")?;
        for (id, bounds) in leg.occupied(sole).ok_or("sole exceeds paid chain reach")? {
            let mut lo = [i64::MAX; 3];
            let mut hi = [i64::MIN; 3];
            for x in [bounds[0][0], bounds[1][0]] {
                for y in [bounds[0][1], bounds[1][1]] {
                    for z in [bounds[0][2], bounds[1][2]] {
                        let p = super::super::birth::rotate(pose.yaw, [x, y, z])?;
                        for k in 0..3 {
                            lo[k] = lo[k].min(p[k]);
                            hi[k] = hi[k].max(p[k]);
                        }
                    }
                }
            }
            let lower = [0, 1, 2].map(|k| {
                pose.at[k]
                    .checked_add(lo[k].div_euclid(UNIT))
                    .ok_or("limb bound overflow")
            });
            let upper = [0, 1, 2].map(|k| {
                pose.at[k]
                    .checked_add(-(-hi[k]).div_euclid(UNIT))
                    .ok_or("limb bound overflow")
            });
            out.insert(
                id,
                [
                    [lower[0].clone()?, lower[1].clone()?, lower[2].clone()?],
                    [upper[0].clone()?, upper[1].clone()?, upper[2].clone()?],
                ],
            );
        }
    }
    Ok(out)
}

pub(super) fn ground(
    g: &mut Grounding<'_>,
    doc: &BodyDocument,
    x: i64,
    z: i64,
    yaw: Yaw,
    height: Option<i64>,
) -> Result<Pose> {
    let chains = legs::chains(doc);
    let limb_ids: BTreeSet<_> = chains
        .iter()
        .flat_map(|l| [l.upper, l.lower, l.foot])
        .collect();
    let rest = boxes(
        doc,
        &Pose {
            at: [x, 0, z],
            yaw,
            ..Default::default()
        },
    )?;
    let parts: Vec<_> = doc
        .living()
        .filter(|p| p.half_extent.iter().all(|h| *h > 0))
        .collect();
    let mut min_y = i64::MIN;
    for (p, [lo, hi]) in parts.into_iter().zip(rest) {
        if limb_ids.contains(&p.id) {
            continue;
        }
        for xx in lo[0]..hi[0] {
            for zz in lo[2]..hi[2] {
                min_y = min_y.max(top(g, xx, zz)? + 1 - lo[1]);
            }
        }
    }
    let mut options = BTreeMap::new();
    for leg in &chains {
        options.insert(leg.foot, super::footholds::targets(g, leg, x, z, yaw,
            g.walking.map_or(0, |r| i64::from(r.foot_search)))?);
    }
    let soles: BTreeMap<_, _> = options.iter().map(|(id, ps)| (*id, ps[0])).collect();
    // Search root height from body clearance upward, bounded by paid reach.
    // The first valid posture is crouched rather than balanced on a trunk edge.
    let reach = chains
        .iter()
        .map(|l| (l.lengths[0] + l.lengths[1]) / UNIT)
        .max()
        .unwrap_or(0);
    let mut preferred: Vec<_> = chains
        .iter()
        .map(|l| (soles[&l.foot][1] - l.sole[1]) / UNIT)
        .collect();
    preferred.sort();
    if min_y == i64::MIN {
        min_y = preferred[0] - reach;
    }
    let preferred = height
        .unwrap_or(preferred[preferred.len() / 2])
        .clamp(min_y, min_y + reach);
    let heights =
        std::iter::once(preferred).chain((1..=reach).flat_map(|d| [preferred - d, preferred + d]));
    let mut tried = 0;
    for y in heights.filter(|y| *y >= min_y && *y <= min_y + reach) {
        if let Some(pose) = super::footholds::poses(g, doc, &chains, &options, [x, y, z], yaw, &mut tried)? {
            return Ok(pose);
        }
    }
    Err("no clear supported posture within paid leg reach".into())
}

pub(super) fn top(g: &mut Grounding<'_>, x: i64, z: i64) -> Result<i64> {
    if !g.volume.holds(x, z) {
        return Err("foot or body leaves terrain window".into());
    }
    Ok(*g.tops.entry([x, z]).or_insert_with(|| {
        g.volume.surface(x,z).unwrap_or(g.volume.datum-1)
    }))
}

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

pub(super) fn support(volume: &Volume, document: &BodyDocument, pose: &Pose) -> Result<()> {
    // The body's volume centre must lie over actual supporting terrain cells,
    // including edge contacts on slopes. No force constant is used.
    let bearing = pose.feet.keys().copied().collect();
    support_at(
        volume,
        document,
        &pose.feet,
        pose.at.map(|v| v * UNIT),
        pose.yaw,
        &bearing,
    )
}

pub(super) fn support_at(
    volume: &Volume,
    document: &BodyDocument,
    feet: &BTreeMap<PartId, [i64; 3]>,
    origin: [i64; 3],
    yaw: Yaw,
    bearing: &BTreeSet<PartId>,
) -> Result<()> {
    let mass_centre = document.centre_of_mass(|id| {
        document.part(id).map_or(0, |p| {
            p.half_extent
                .iter()
                .fold(8u64, |n, h| n.saturating_mul((*h).max(0) as u64))
        })
    });
    let centre = super::super::birth::rotate(yaw, mass_centre.map(|v| i64::from(v) * UNIT))?;
    let centre = [centre[0] + origin[0], centre[2] + origin[2]];
    let mut points = Vec::new();
    for (id, [lo, hi]) in super::narrow::pads(document, origin, yaw, feet)? {
        if !bearing.contains(&id) {
            continue;
        }
        if !(lo[0]..hi[0])
            .any(|x| (lo[2]..hi[2]).any(|z| contacts::supported(volume, [x, lo[1] - 1, z])))
        {
            return Err("paid foot has no dry terrain support".into());
        }
        for x in lo[0]..hi[0] {
            for z in lo[2]..hi[2] {
                if !contacts::supported(volume, [x, lo[1] - 1, z]) {
                    continue;
                }
                for dx in [0, 1] {
                    for dz in [0, 1] {
                        points.push([(x + dx) * UNIT, (z + dz) * UNIT]);
                    }
                }
            }
        }
    }
    points.sort();
    points.dedup();
    if points.len() < 3 {
        return Err("no ground-bearing foot polygon".into());
    }
    let cross = |a: [i64; 2], b: [i64; 2], c: [i64; 2]| {
        i128::from(b[0] - a[0]) * i128::from(c[1] - a[1])
            - i128::from(b[1] - a[1]) * i128::from(c[0] - a[0])
    };
    let mut hull = Vec::new();
    for &p in &points {
        while hull.len() >= 2 && cross(hull[hull.len() - 2], hull[hull.len() - 1], p) <= 0 {
            hull.pop();
        }
        hull.push(p);
    }
    let lower = hull.len();
    for &p in points.iter().rev().skip(1) {
        while hull.len() > lower && cross(hull[hull.len() - 2], hull[hull.len() - 1], p) <= 0 {
            hull.pop();
        }
        hull.push(p);
    }
    if hull
        .windows(2)
        .any(|pair| cross(pair[0], pair[1], centre) < 0)
    {
        return Err("body volume centre leaves its bearing-foot polygon".into());
    }
    Ok(())
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
    let mut soles = BTreeMap::new();
    for leg in &chains {
        let p = super::super::birth::rotate(yaw, leg.sole)?;
        let h = super::super::birth::rotate(yaw, leg.half)?.map(i64::abs);
        let centre = [x * UNIT + p[0], 0, z * UNIT + p[2]];
        let mut height = i64::MIN;
        for xx in (centre[0] - h[0]).div_euclid(UNIT)..-(-(centre[0] + h[0])).div_euclid(UNIT) {
            for zz in (centre[2] - h[2]).div_euclid(UNIT)..-(-(centre[2] + h[2])).div_euclid(UNIT) {
                let y = top(g, xx, zz)?;
                if !contacts::supported(g.volume, [xx, y, zz]) {
                    return Err("foot footprint lacks dry ground".into());
                }
                height = height.max(y + 1);
            }
        }
        soles.insert(leg.foot, [leg.sole[0], height * UNIT, leg.sole[2]]);
    }
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
    for y in heights.filter(|y| *y >= min_y && *y <= min_y + reach) {
        let feet = soles
            .iter()
            .map(|(&id, &p)| (id, [p[0], p[1] - y * UNIT, p[2]]))
            .collect();
        let pose = Pose {
            at: [x, y, z],
            yaw,
            feet,
        };
        if chains.iter().any(|leg| {
            let sole = pose.feet[&leg.foot];
            leg.solve([sole[0], sole[1] + UNIT, sole[2]]).is_none()
                || leg.solve([sole[0], sole[1] - UNIT, sole[2]]).is_none()
        }) {
            continue;
        }
        if check(g.volume, doc, &pose).is_ok() {
            return Ok(pose);
        }
    }
    Err("no clear supported posture within paid leg reach".into())
}

fn top(g: &mut Grounding<'_>, x: i64, z: i64) -> Result<i64> {
    if !g.volume.holds(x, z) {
        return Err("foot or body leaves terrain window".into());
    }
    Ok(*g.tops.entry([x, z]).or_insert_with(|| {
        g.volume.surface(x,z).unwrap_or(g.volume.datum-1)
    }))
}

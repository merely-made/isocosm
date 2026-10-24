// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! The existing volume-centre support law, shared by standing and weight shifts.
use super::*;
use isometer_core::{
    PartId,
    legs::{UNIT, support as polygon},
};

pub(super) fn centre(document: &BodyDocument, yaw: Yaw) -> Result<[i64; 3]> {
    let p = document.centre_of_mass(|id| {
        document.part(id).map_or(0, |p| {
            p.half_extent
                .iter()
                .fold(8u64, |n, h| n.saturating_mul((*h).max(0) as u64))
        })
    });
    super::super::birth::rotate(yaw, p.map(|v| i64::from(v) * UNIT))
}
pub(super) fn bearing(
    volume: &Volume,
    document: &BodyDocument,
    feet: &BTreeMap<PartId, [i64; 3]>,
    origin: [i64; 3],
    yaw: Yaw,
    bearing: &BTreeSet<PartId>,
) -> Result<Vec<[i64; 2]>> {
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
    if points.is_empty() {
        return Err("no ground-bearing foot polygon".into());
    }
    Ok(polygon::hull(points))
}
pub(super) fn check(
    volume: &Volume,
    document: &BodyDocument,
    feet: &BTreeMap<PartId, [i64; 3]>,
    origin: [i64; 3],
    yaw: Yaw,
    bearing_feet: &BTreeSet<PartId>,
) -> Result<()> {
    let centre = centre(document, yaw)?;
    let polygon = bearing(volume, document, feet, origin, yaw, bearing_feet)?;
    if !polygon::contains(&polygon, [centre[0] + origin[0], centre[2] + origin[2]]) {
        return Err("body volume centre leaves its bearing-foot polygon".into());
    }
    Ok(())
}
pub(super) fn standing(volume: &Volume, document: &BodyDocument, pose: &Pose) -> Result<()> {
    check(
        volume,
        document,
        &pose.feet,
        pose.at.map(|v| v * UNIT),
        pose.yaw,
        &pose.feet.keys().copied().collect(),
    )
}

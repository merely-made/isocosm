// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! The existing volume-centre support law, shared by standing and weight shifts.
use super::*;
use isometer_core::{
    PartId,
    legs::{Rotation, UNIT, support as polygon},
};

pub(super) fn centre(document: &BodyDocument, yaw: Yaw) -> Result<[i64; 3]> {
    centre_rotated(document, Rotation::at(yaw, 0))
}
pub(super) fn centre_rotated(document: &BodyDocument, rotation: Rotation) -> Result<[i64; 3]> {
    rotation
        .rotate(centre_local(document))
        .ok_or("turn centre overflow".into())
}
pub(super) fn centre_local(document: &BodyDocument) -> [i64; 3] {
    let p = document.centre_of_mass(|id| {
        document.part(id).map_or(0, |p| {
            p.half_extent
                .iter()
                .fold(8u64, |n, h| n.saturating_mul((*h).max(0) as u64))
        })
    });
    p.map(|v| i64::from(v) * UNIT)
}
pub(super) fn bearing(
    volume: &Volume,
    document: &BodyDocument,
    feet: &BTreeMap<PartId, [i64; 3]>,
    origin: [i64; 3],
    yaw: Yaw,
    bearing: &BTreeSet<PartId>,
) -> Result<Vec<[i64; 2]>> {
    bearing_rotated(
        volume,
        document,
        feet,
        origin,
        Rotation::at(yaw, 0),
        bearing,
    )
}
pub(super) fn bearing_rotated(
    volume: &Volume,
    document: &BodyDocument,
    feet: &BTreeMap<PartId, [i64; 3]>,
    origin: [i64; 3],
    rotation: Rotation,
    bearing: &BTreeSet<PartId>,
) -> Result<Vec<[i64; 2]>> {
    bearing_pads(
        volume,
        &super::narrow::pad_shapes(document, origin, rotation, feet)?,
        bearing,
    )
}
pub(super) fn bearing_pads(
    volume: &Volume,
    pads: &BTreeMap<PartId, super::narrow::Pad>,
    bearing: &BTreeSet<PartId>,
) -> Result<Vec<[i64; 2]>> {
    let mut points = Vec::new();
    for (id, pad) in pads {
        let [lo, hi] = pad.cells();
        if !bearing.contains(id) {
            continue;
        }
        if !(lo[0]..hi[0]).any(|x| {
            (lo[2]..hi[2])
                .any(|z| pad.columns(x, z) && contacts::supported(volume, [x, lo[1] - 1, z]))
        }) {
            return Err("paid foot has no dry terrain support".into());
        }
        for x in lo[0]..hi[0] {
            for z in lo[2]..hi[2] {
                if !pad.columns(x, z) || !contacts::supported(volume, [x, lo[1] - 1, z]) {
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
    check_rotated(
        volume,
        document,
        feet,
        origin,
        Rotation::at(yaw, 0),
        bearing_feet,
    )
}
pub(super) fn check_rotated(
    volume: &Volume,
    document: &BodyDocument,
    feet: &BTreeMap<PartId, [i64; 3]>,
    origin: [i64; 3],
    rotation: Rotation,
    bearing_feet: &BTreeSet<PartId>,
) -> Result<()> {
    let centre = centre_rotated(document, rotation)?;
    let polygon = bearing_rotated(volume, document, feet, origin, rotation, bearing_feet)?;
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

/// A planted support polygon is unchanged by the swinging foot or torso shift.
/// Its key keeps exact oriented pad geometry and the selected bearing feet.
pub(super) struct Bearing<'a> {
    volume: &'a Volume,
    capacity: usize,
    entries: BTreeMap<Vec<(PartId, narrow::Pad)>, Result<Vec<[i64; 2]>>>,
    order: std::collections::VecDeque<Vec<(PartId, narrow::Pad)>>,
    pub reads: usize,
    pub misses: usize,
}
impl<'a> Bearing<'a> {
    pub fn new(volume: &'a Volume, capacity: usize) -> Self {
        Self {
            volume,
            capacity,
            entries: BTreeMap::new(),
            order: Default::default(),
            reads: 0,
            misses: 0,
        }
    }
    pub fn hull(
        &mut self,
        pads: &BTreeMap<PartId, narrow::Pad>,
        bearing: &BTreeSet<PartId>,
    ) -> Result<Vec<[i64; 2]>> {
        self.reads += 1;
        if self.capacity == 0 {
            self.misses += 1;
            return bearing_pads(self.volume, pads, bearing);
        }
        let key: Vec<_> = pads
            .iter()
            .filter(|(id, _)| bearing.contains(id))
            .map(|(&id, &pad)| (id, pad))
            .collect();
        if let Some(found) = self.entries.get(&key) {
            return found.clone();
        }
        self.misses += 1;
        let result = bearing_pads(self.volume, pads, bearing);
        if self.capacity > 0 {
            if self.entries.len() >= self.capacity {
                self.entries.remove(&self.order.pop_front().unwrap());
            }
            self.order.push_back(key.clone());
            self.entries.insert(key, result.clone());
        }
        result
    }
}

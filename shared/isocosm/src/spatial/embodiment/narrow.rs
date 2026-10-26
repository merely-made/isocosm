// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Capsule-shaped paid links against actual terrain voxels, after broad boxes.
use super::*;
use isometer_core::{
    PartId,
    legs::{self, Capsule, Rotation, UNIT},
};
mod pads;
#[cfg(test)]
mod tests;
pub(super) use pads::Pad;

pub(super) struct Shapes {
    solids: BTreeMap<PartId, Vec<Solid>>,
    pub pads: BTreeMap<PartId, Pad>,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Solid {
    Link(Capsule),
    Pad(Pad),
}
impl Solid {
    fn hits(&self, at: [i64; 3]) -> bool {
        let cell = [at.map(|v| v * UNIT), at.map(|v| (v + 1) * UNIT)];
        match self {
            Self::Link(link) => link.hits(cell),
            Self::Pad(pad) => pad.hits(at),
        }
    }
}
fn point(p: [i64; 3], rotation: Rotation, origin: [i64; 3]) -> Result<[i64; 3]> {
    let p = rotation.rotate(p).ok_or("turn point overflow")?;
    Ok([0, 1, 2].map(|k| p[k] + origin[k]))
}
fn pad(bounds: [[i64; 3]; 2], rotation: Rotation, origin: [i64; 3]) -> Result<Pad> {
    Pad::new(bounds, rotation, origin).ok_or("turn pad overflow".into())
}
fn link(c: Capsule, yaw: Rotation, origin: [i64; 3]) -> Result<Solid> {
    Ok(Solid::Link(Capsule {
        a: point(c.a, yaw, origin)?,
        b: point(c.b, yaw, origin)?,
        radius: c.radius,
    }))
}

pub(super) fn pads(
    doc: &BodyDocument,
    origin: [i64; 3],
    yaw: Yaw,
    feet: &BTreeMap<PartId, [i64; 3]>,
) -> Result<BTreeMap<PartId, Box3>> {
    Ok(pad_shapes(doc, origin, Rotation::at(yaw, 0), feet)?
        .into_iter()
        .map(|(id, p)| (id, p.cells()))
        .collect())
}
pub(super) fn pad_shapes(
    doc: &BodyDocument,
    origin: [i64; 3],
    rotation: Rotation,
    feet: &BTreeMap<PartId, [i64; 3]>,
) -> Result<BTreeMap<PartId, Pad>> {
    legs::chains(doc)
        .into_iter()
        .map(|leg| {
            let skin = leg
                .skin(*feet.get(&leg.foot).ok_or("admitted foot absent")?)
                .ok_or("unreachable pad")?;
            Ok((leg.foot, pad(skin.pad, rotation, origin)?))
        })
        .collect()
}

pub(super) fn clear(
    volume: &Volume,
    doc: &BodyDocument,
    origin: [i64; 3],
    yaw: Yaw,
    feet: &BTreeMap<PartId, [i64; 3]>,
    occupied: &[Box3],
    blocked: &[Box3],
) -> Result<()> {
    clear_rotated(
        volume,
        doc,
        origin,
        Rotation::at(yaw, 0),
        feet,
        occupied,
        blocked,
    )
}
pub(super) fn clear_rotated(
    volume: &Volume,
    doc: &BodyDocument,
    origin: [i64; 3],
    yaw: Rotation,
    feet: &BTreeMap<PartId, [i64; 3]>,
    occupied: &[Box3],
    blocked: &[Box3],
) -> Result<()> {
    if intersects(occupied, blocked) {
        return Err("swing intersects another native body".into());
    }
    let chains = legs::chains(doc);
    let skins = chains
        .iter()
        .map(|leg| {
            leg.skin(*feet.get(&leg.foot).ok_or("admitted foot absent")?)
                .ok_or("unreachable paid links".into())
        })
        .collect::<Result<Vec<_>>>()?;
    let shapes = shapes(chains.iter().zip(&skins), origin, yaw)?;
    clear_prepared(
        volume,
        doc.living()
            .filter(|p| p.half_extent.iter().all(|h| *h > 0))
            .map(|p| p.id)
            .zip(occupied.iter().copied()),
        occupied,
        blocked,
        &shapes,
    )
}

pub(super) fn shapes<'a>(
    skins: impl Iterator<Item = (&'a legs::Leg, &'a legs::Skin)>,
    origin: [i64; 3],
    yaw: Rotation,
) -> Result<Shapes> {
    let mut shapes = BTreeMap::new();
    let mut pads = BTreeMap::new();
    for (leg, skin) in skins {
        let sole = pad(skin.pad, yaw, origin)?;
        pads.insert(leg.foot, sole);
        shapes.insert(leg.upper, vec![link(skin.upper, yaw, origin)?]);
        shapes.insert(leg.lower, vec![link(skin.lower, yaw, origin)?]);
        shapes.insert(
            leg.foot,
            vec![Solid::Pad(sole), link(skin.connector, yaw, origin)?],
        );
    }
    Ok(Shapes {
        solids: shapes,
        pads,
    })
}

pub(super) fn clear_prepared(
    volume: &Volume,
    parts: impl Iterator<Item = (PartId, Box3)>,
    occupied: &[Box3],
    blocked: &[Box3],
    shapes: &Shapes,
) -> Result<()> {
    if intersects(occupied, blocked) {
        return Err("swing intersects another native body".into());
    }
    check_parts(volume, parts, &shapes.solids)
}

pub(super) fn leg_clear(
    volume: &Volume,
    leg: &legs::Leg,
    sole: [i64; 3],
    origin: [i64; 3],
    yaw: Yaw,
) -> Result<()> {
    let skin = leg.skin(sole).ok_or("unreachable paid links")?;
    let rotation = Rotation::at(yaw, 0);
    let shapes = shapes(std::iter::once((leg, &skin)), origin, rotation)?;
    let occupied = [leg.upper, leg.lower, leg.foot]
        .into_iter()
        .zip(skin.occupied())
        .map(|(id, b)| Ok((id, super::swing::placed(b, yaw, origin)?)))
        .collect::<Result<Vec<_>>>()?;
    check_parts(volume, occupied.into_iter(), &shapes.solids)
}

fn check_parts(
    volume: &Volume,
    parts: impl Iterator<Item = (PartId, Box3)>,
    shapes: &BTreeMap<PartId, Vec<Solid>>,
) -> Result<()> {
    Clearance::new(volume, 0).check_parts(parts, shapes)
}

/// Terrain-only component readings. Other bodies are checked by each caller
/// before these results can be used; the borrowed volume cannot change.
pub(super) struct Clearance<'a> {
    volume: &'a Volume,
    capacity: usize,
    entries: BTreeMap<(PartId, Box3, Option<Vec<Solid>>), Result<()>>,
    order: std::collections::VecDeque<(PartId, Box3, Option<Vec<Solid>>)>,
    pub reads: usize,
    pub misses: usize,
}
impl<'a> Clearance<'a> {
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
    pub fn check(
        &mut self,
        parts: impl Iterator<Item = (PartId, Box3)>,
        shapes: &Shapes,
    ) -> Result<()> {
        self.check_parts(parts, &shapes.solids)
    }
    fn check_parts(
        &mut self,
        parts: impl Iterator<Item = (PartId, Box3)>,
        shapes: &BTreeMap<PartId, Vec<Solid>>,
    ) -> Result<()> {
        let mut work = 0u64;
        for (id, [lo, hi]) in parts {
            if !self.volume.holds(lo[0], lo[2]) || !self.volume.holds(hi[0] - 1, hi[2] - 1) {
                return Err("posed body leaves terrain window".into());
            }
            let count = (0..3)
                .try_fold(1u64, |n, k| n.checked_mul(lo[k].abs_diff(hi[k])))
                .ok_or("posed clearance overflow")?;
            work = work
                .checked_add(count)
                .filter(|n| *n <= 1_048_576)
                .ok_or("posed clearance budget exceeded")?;
            self.reads += 1;
            if self.capacity == 0 {
                self.misses += 1;
                check_part(self.volume, id, [lo, hi], shapes.get(&id))?;
                continue;
            }
            let key = (id, [lo, hi], shapes.get(&id).cloned());
            if let Some(result) = self.entries.get(&key) {
                result.clone()?;
                continue;
            }
            self.misses += 1;
            let result = check_part(self.volume, id, [lo, hi], shapes.get(&id));
            if self.capacity > 0 {
                if self.entries.len() >= self.capacity {
                    self.entries.remove(&self.order.pop_front().unwrap());
                }
                self.order.push_back(key.clone());
                self.entries.insert(key, result.clone());
            }
            result?;
        }
        Ok(())
    }
}
fn check_part(
    volume: &Volume,
    id: PartId,
    [lo, hi]: Box3,
    shapes: Option<&Vec<Solid>>,
) -> Result<()> {
    for x in lo[0]..hi[0] {
        for z in lo[2]..hi[2] {
            for y in lo[1]..hi[1] {
                let at = [x, y, z];
                if air(volume, at) {
                    continue;
                }
                if shapes.is_some_and(|parts| parts.iter().all(|p| !p.hits(at))) {
                    continue;
                }
                return Err(format!(
                    "posed part {} intersects terrain or water at {at:?}",
                    id.0
                ));
            }
        }
    }
    Ok(())
}

// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Capsule-shaped paid links against actual terrain voxels, after broad boxes.
use super::*;
use isometer_core::{
    PartId,
    legs::{self, Capsule, UNIT},
};

enum Solid {
    Link(Capsule),
    Pad([[i64; 3]; 2]),
}
impl Solid {
    fn hits(&self, at: [i64; 3]) -> bool {
        let cell = [at.map(|v| v * UNIT), at.map(|v| (v + 1) * UNIT)];
        match self {
            Self::Link(link) => link.hits(cell),
            Self::Pad([lo, hi]) => (0..3).all(|k| lo[k] < cell[1][k] && cell[0][k] < hi[k]),
        }
    }
}
fn point(p: [i64; 3], yaw: Yaw, origin: [i64; 3]) -> Result<[i64; 3]> {
    let p = super::super::birth::rotate(yaw, p)?;
    Ok([0, 1, 2].map(|k| p[k] + origin[k]))
}
fn pad(bounds: [[i64; 3]; 2], yaw: Yaw, origin: [i64; 3]) -> Result<[[i64; 3]; 2]> {
    let a = point(bounds[0], yaw, origin)?;
    let b = point(bounds[1], yaw, origin)?;
    Ok([
        [0, 1, 2].map(|k| a[k].min(b[k])),
        [0, 1, 2].map(|k| a[k].max(b[k])),
    ])
}
fn link(c: Capsule, yaw: Yaw, origin: [i64; 3]) -> Result<Solid> {
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
    legs::chains(doc)
        .into_iter()
        .map(|leg| {
            let skin = leg
                .skin(*feet.get(&leg.foot).ok_or("admitted foot absent")?)
                .ok_or("unreachable pad")?;
            let p = pad(skin.pad, yaw, origin)?;
            Ok((
                leg.foot,
                [
                    p[0].map(|v| v.div_euclid(UNIT)),
                    p[1].map(|v| -(-v).div_euclid(UNIT)),
                ],
            ))
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
    if intersects(occupied, blocked) {
        return Err("swing intersects another native body".into());
    }
    let mut shapes = BTreeMap::new();
    for leg in legs::chains(doc) {
        let skin = leg
            .skin(*feet.get(&leg.foot).ok_or("admitted foot absent")?)
            .ok_or("unreachable paid links")?;
        shapes.insert(leg.upper, vec![link(skin.upper, yaw, origin)?]);
        shapes.insert(leg.lower, vec![link(skin.lower, yaw, origin)?]);
        shapes.insert(
            leg.foot,
            vec![
                Solid::Pad(pad(skin.pad, yaw, origin)?),
                link(skin.connector, yaw, origin)?,
            ],
        );
    }
    check_parts(volume, doc
        .living()
        .filter(|p| p.half_extent.iter().all(|h| *h > 0))
        .zip(occupied).map(|(p,b)|(p.id,*b)), &shapes)
}

pub(super) fn leg_clear(volume:&Volume,leg:&legs::Leg,sole:[i64;3],origin:[i64;3],yaw:Yaw)->Result<()> {
    let skin=leg.skin(sole).ok_or("unreachable paid links")?;
    let shapes=BTreeMap::from([
        (leg.upper,vec![link(skin.upper,yaw,origin)?]),
        (leg.lower,vec![link(skin.lower,yaw,origin)?]),
        (leg.foot,vec![Solid::Pad(pad(skin.pad,yaw,origin)?),link(skin.connector,yaw,origin)?]),
    ]);
    let occupied=leg.occupied(sole).ok_or("unreachable paid links")?.into_iter()
        .map(|(id,b)|Ok((id,super::swing::placed(b,yaw,origin)?))).collect::<Result<Vec<_>>>()?;
    check_parts(volume,occupied.into_iter(),&shapes)
}

fn check_parts(volume:&Volume,parts:impl Iterator<Item=(PartId,Box3)>,
    shapes:&BTreeMap<PartId,Vec<Solid>>)->Result<()> {
    let mut work=0u64;
    for (id,[lo,hi]) in parts {
        if !volume.holds(lo[0], lo[2]) || !volume.holds(hi[0] - 1, hi[2] - 1) {
            return Err("posed body leaves terrain window".into());
        }
        let count = (0..3)
            .try_fold(1u64, |n, k| n.checked_mul(lo[k].abs_diff(hi[k])))
            .ok_or("posed clearance overflow")?;
        work = work
            .checked_add(count)
            .filter(|n| *n <= 1_048_576)
            .ok_or("posed clearance budget exceeded")?;
        for x in lo[0]..hi[0] {
            for z in lo[2]..hi[2] {
                for y in lo[1]..hi[1] {
                    let at = [x, y, z];
                    if air(volume, at) {
                        continue;
                    }
                    if shapes
                        .get(&id)
                        .is_some_and(|parts| parts.iter().all(|p| !p.hits(at)))
                    {
                        continue;
                    }
                    return Err(format!(
                        "posed part {} intersects terrain or water at {at:?}",
                        id.0
                    ));
                }
            }
        }
    }
    Ok(())
}

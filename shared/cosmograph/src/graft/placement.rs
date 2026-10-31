// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use isometer_core::{BodyDocument, Yaw};
use std::collections::{BTreeMap, BTreeSet};

fn pose(body: &BodyDocument, id: PartId) -> Result<([i64; 3], Yaw)> {
    let mut chain = Vec::new();
    let mut seen = BTreeSet::new();
    let mut cursor = id;
    loop {
        if !seen.insert(cursor) { return Err("graft placement contains a cycle".into()); }
        let part = body.part(cursor).filter(|p| p.id == cursor).ok_or("graft placement names absent geometry")?;
        match part.attachment {
            Some(at) => { chain.push(at); cursor = at.parent; },
            None if cursor == body.root => break,
            None => return Err("graft placement contains another root".into()),
        }
    }
    let mut position = [0i64; 3];
    let mut yaw = Yaw::Zero;
    for at in chain.into_iter().rev() {
        let offset = rotate(yaw, at.offset.map(i64::from));
        for axis in 0..3 { position[axis] = position[axis].checked_add(offset[axis]).ok_or("graft placement overflows")?; }
        yaw = yaw.compose(at.yaw);
    }
    Ok((position, yaw))
}

fn rotate(yaw: Yaw, v: [i64; 3]) -> [i64; 3] {
    let [x,y,z] = v;
    match yaw { Yaw::Zero => v, Yaw::Quarter => [z,y,-x],
        Yaw::Half => [-x,y,-z], Yaw::ThreeQuarter => [-z,y,x] }
}

fn bounds(body: &BodyDocument, id: PartId) -> Result<([i64; 3], [i64; 3])> {
    let part = body.part(id).ok_or("graft box is absent")?;
    let (position, yaw) = pose(body, id)?;
    let half = rotate(yaw, part.half_extent.map(|n| i64::from(n).abs())).map(i64::abs);
    let mut min = [0; 3]; let mut max = [0; 3];
    for axis in 0..3 {
        min[axis] = position[axis].checked_sub(half[axis]).ok_or("graft box overflows")?;
        max[axis] = position[axis].checked_add(half[axis]).ok_or("graft box overflows")?;
        if min[axis] < i64::from(i32::MIN) || max[axis] > i64::from(i32::MAX) {
            return Err("graft box is outside native voxel coordinates".into());
        }
    }
    Ok((min,max))
}

pub(super) fn check(actor: &Entity, donor: &Entity, carry: &Carry, mapping: &[(PartId,PartId)]) -> Result<()> {
    if !actor.lives(carry.attachment.parent) { return Err("graft mount parent is not living".into()); }
    let mut candidate = actor.body.as_ref().ok_or("graft recipient geometry is absent")?.clone();
    let source = donor.body.as_ref().ok_or("graft donor geometry is absent")?;
    let ids: BTreeMap<_,_> = mapping.iter().copied().collect();
    for (old,new) in mapping {
        let mut part = source.part(*old).ok_or("graft branch geometry is absent")?.clone();
        part.id = *new;
        part.attachment = if *old == carry.donor_root { Some(carry.attachment) } else {
            let mut at = part.attachment.ok_or("graft descendant has no parent")?;
            at.parent = *ids.get(&at.parent).ok_or("graft descendant is outside selected branch")?;
            Some(at)
        };
        if candidate.len() != new.0 as usize { return Err("graft mapping is not append-only".into()); }
        candidate.parts.push(part);
    }
    // Existing donor internal geometry is preserved. New branch boxes must
    // not overlap the recipient; touching is allowed as in growth::free.
    let existing: Vec<_> = actor.living().map(|(id,_)| bounds(&candidate,id)).collect::<Result<_>>()?;
    for (_,id) in mapping {
        let (a,b) = bounds(&candidate,*id)?;
        for (c,d) in &existing {
            if (0..3).all(|axis| a[axis] < d[axis] && c[axis] < b[axis]) {
                return Err("authored graft placement overlaps recipient tissue".into());
            }
        }
    }
    Ok(())
}

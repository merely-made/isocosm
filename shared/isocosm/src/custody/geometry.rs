// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::operations::Window;
use isometer_space::Atlas;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MountPose { pub at: [i64; 3], pub yaw: Yaw }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bounds { pub min: [i64; 3], pub max: [i64; 3] }

impl Bounds {
    pub fn overlaps(&self, other: &Self) -> bool {
        (0..3).all(|axis| self.min[axis] < other.max[axis] && other.min[axis] < self.max[axis])
    }
}

fn rotate(yaw: Yaw, value: [i64; 3]) -> Result<[i64; 3]> {
    let [x,y,z] = value;
    let neg = |n: i64| -> Result<i64> { n.checked_neg().ok_or_else(|| "item attachment rotation overflows".into()) };
    Ok(match yaw { Yaw::Zero => value, Yaw::Quarter => [z,y,neg(x)?],
        Yaw::Half => [neg(x)?,y,neg(z)?], Yaw::ThreeQuarter => [neg(z)?,y,x] })
}

fn plus(left: [i64; 3], right: [i64; 3]) -> Result<[i64; 3]> {
    let mut value = [0; 3];
    for axis in 0..3 { value[axis] = left[axis].checked_add(right[axis]).ok_or("item attachment position overflows")?; }
    Ok(value)
}

fn part_pose(body: &BodyDocument, id: PartId) -> Result<MountPose> {
    let mut cursor = id;
    let mut seen = BTreeSet::new();
    let mut chain = Vec::new();
    loop {
        if !seen.insert(cursor) { return Err("item attachment body contains a cycle".into()); }
        let part = body.part(cursor).ok_or("item attachment part is absent")?;
        match part.attachment {
            Some(attachment) => { chain.push(attachment); cursor = attachment.parent; },
            None if cursor == body.root => break,
            None => return Err("item attachment body contains another root".into()),
        }
    }
    let mut pose = MountPose { at: [0; 3], yaw: Yaw::Zero };
    for attachment in chain.into_iter().rev() {
        pose.at = plus(pose.at, rotate(pose.yaw, attachment.offset.map(i64::from))?)?;
        pose.yaw = pose.yaw.compose(attachment.yaw);
    }
    Ok(pose)
}

/// A game-side whole-item attachment uses native pivots without physiological grafting.
pub fn mount_pose(actor: &Entity, attachment: &Attachment) -> Result<MountPose> {
    if !actor.lives(attachment.parent) { return Err("item mount has no living native parent part".into()); }
    let mut pose = part_pose(actor.body.as_ref().ok_or("carrier body geometry is absent")?, attachment.parent)?;
    pose.at = plus(pose.at, rotate(pose.yaw, attachment.offset.map(i64::from))?)?;
    pose.yaw = pose.yaw.compose(attachment.yaw);
    Ok(pose)
}

pub fn item_bounds(entity: &Entity, root: MountPose) -> Result<Vec<Bounds>> {
    let body = entity.body.as_ref().ok_or("item geometry is absent")?;
    let mut boxes = Vec::new();
    for (id, _) in entity.living() {
        let part = body.part(id).ok_or("item physiology lacks corresponding native geometry")?;
        let local = part_pose(body, id)?;
        let centre = plus(root.at, rotate(root.yaw, local.at)?)?;
        let yaw = root.yaw.compose(local.yaw);
        let half = rotate(yaw, part.half_extent.map(i64::from))?.map(i64::abs);
        if half.contains(&0) { return Err("item geometry has an empty part".into()); }
        let mut min = [0; 3]; let mut max = [0; 3];
        for axis in 0..3 {
            min[axis] = centre[axis].checked_sub(half[axis]).ok_or("item bounds overflow")?;
            max[axis] = centre[axis].checked_add(half[axis]).ok_or("item bounds overflow")?;
        }
        boxes.push(Bounds { min, max });
    }
    if boxes.is_empty() { return Err("item has no actual living body geometry".into()); }
    Ok(boxes)
}

impl Simulation {
    pub fn item_clearance(&self, item: Id, site: Id, pose: MountPose, window: &Window) -> Result<Vec<Bounds>> {
        if window.depth < 0 { return Err("item window depth is negative".into()); }
        let entity = self.state.population.get(item).ok_or("physical item is absent")?;
        if entity.resident_site() != Some(site) { return Err("item geometry has no matching native residence".into()); }
        let boxes = item_bounds(entity, pose)?;
        let volume = self.volume(site, window.min, window.max, window.depth)?;
        for bounds in &boxes {
            if !volume.holds(bounds.min[0], bounds.min[2])
                || !volume.holds(bounds.max[0] - 1, bounds.max[2] - 1)
                || bounds.min[1] < volume.datum || bounds.max[1] > volume.ceiling() {
                return Err("whole item is outside its explicit native geometry window".into());
            }
            for z in bounds.min[2]..bounds.max[2] { for y in bounds.min[1]..bounds.max[1] {
                for x in bounds.min[0]..bounds.max[0] {
                    if volume.material([x,y,z]).is_none_or(|material|
                        material != volume.materials.air && material != volume.materials.water) {
                        return Err("whole item intersects actual native terrain".into());
                    }
                }
            }}
        }
        Ok(boxes)
    }
}

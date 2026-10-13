// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use isometer_space::places::Places;

pub(super) fn check(sim: &Simulation, r: &BirthRequest, g: &BirthGeometry) -> Result<PlaceId> {
    let parent = sim.state().population.get(r.parent).ok_or("birth parent absent")?;
    let child = sim.state().population.get(r.child).ok_or("birth child absent")?;
    let profile = sim.genesis().rules.birth_profiles.get(&g.profile).ok_or("native birth profile absent")?;
    let site = r.site.ok_or("travelling parent has no actual birth pose")?;
    let parentage = sim.state().relations.iter().any(|rel|
        rel.subject == r.child && rel.object == r.parent && rel.kind == "sim:parent");
    if !r.parent_alive || !parentage || parent.body.is_none() || child.body.is_none()
        || parent.resident_site() != Some(site) || child.resident_site() != Some(site)
        || child.journey != r.child_journey || child.journey.is_some() || child.born != r.tick
        || parent.body_revision != r.parent_revision || child.body_revision != r.child_revision
        || parent.arrived != r.parent_visit || child.arrived != r.child_visit
        || parent.patch != r.parent_patch || profile.process != r.process
        || sim.terrain_revision() != r.terrain_revision || sim.state().tick != r.tick
        || sim.genesis().rules.revision() != r.revision
    { return Err("biological birth geometry source changed or is unavailable".into()); }
    let volume = sim.volume(site, g.parent_window.min, g.parent_window.max, g.parent_window.depth)?;
    let places = Places::derive(&volume, g.parent_places)?;
    let patch = r.parent_patch.ok_or("parent has no admitted native place")?;
    for cell in [g.parent_pose.at, g.parent_eye] {
        if !geometry::clear(&volume, cell) || places.place_at(cell) != Some(patch) {
            return Err("parent pose is outside its current admitted clear patch".into());
        }
    }
    let (pose, eye) = birth_pose(g)?;
    let volume = sim.volume(site, g.window.min, g.window.max, g.window.depth)?;
    let places = Places::derive(&volume, profile.places)?;
    let patch = places.place_at(pose.at).ok_or("authored child pose has no realized patch")?;
    let document = child.body.as_ref().ok_or("child geometry absent")?;
    let bounds = document.aabb();
    if (0..3).any(|k| profile.clearance[0][k] > i64::from(bounds.min[k])
        || profile.clearance[1][k] < i64::from(bounds.max[k])) {
        return Err("authored birth clearance does not cover the current native child geometry".into());
    }
    let at = |local| -> Result<[i64; 3]> {
        let offset = birth::rotate(pose.yaw, local)?;
        let mut cell = pose.at;
        for k in 0..3 { cell[k] = cell[k].checked_add(offset[k]).ok_or("birth clearance overflow")?; }
        Ok(cell)
    };
    let fits = |cell| {
        volume.holds(cell[0], cell[2]) && cell[1] >= volume.datum && cell[1] < volume.ceiling()
            && volume.material(cell).is_some_and(|m| match profile.medium {
                BirthMedium::Air => m == volume.materials.air,
                BirthMedium::Water => m == volume.materials.water,
                BirthMedium::AirOrWater => m == volume.materials.air || m == volume.materials.water,
            })
    };
    let [lo, hi] = profile.clearance;
    for x in [lo[0], hi[0] - 1] { for y in [lo[1], hi[1] - 1] { for z in [lo[2], hi[2] - 1] {
        if !fits(at([x, y, z])?) { return Err("birth clearance leaves its declared volume/medium".into()); }
    } } }
    for x in lo[0]..hi[0] { for y in lo[1]..hi[1] { for z in lo[2]..hi[2] {
        if !fits(at([x, y, z])?) { return Err("birth clearance intersects actual terrain or another medium".into()); }
    } } }
    if let BirthSupport::Ground { stance_offset } = &profile.support {
        let foot = at(*stance_offset)?;
        let below = [foot[0], foot[1].checked_sub(1).ok_or("birth support overflow")?, foot[2]];
        if volume.material(foot) != Some(volume.materials.air)
            || !volume.material(below).is_some_and(|m| m != volume.materials.air && m != volume.materials.water)
        { return Err("authored ground support has no actual solid surface".into()); }
    }
    if !fits(pose.at) || !geometry::clear(&volume, eye) || places.place_at(eye) != Some(patch)
    { return Err("authored child pose/eye does not fit its declared clear placement".into()); }
    Ok(patch)
}

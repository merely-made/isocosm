// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Integer walking over the existing isometer places, with authored clearance.

use super::*;
use isometer_space::{volume::Volume, places::Places};

fn clear(volume: &Volume, cell: [i64; 3]) -> bool {
    volume.holds(cell[0], cell[2]) && cell[1] >= volume.datum && cell[1] < volume.ceiling()
        && volume.material(cell).is_some_and(|m| m == volume.materials.air || m == volume.materials.water)
}

fn stance(volume: &Volume, profile: &StepProfile, root: [i64; 3]) -> Option<[i64; 3]> {
    let mut cell = root;
    for k in 0..3 { cell[k] = cell[k].checked_add(profile.stance_offset[k])?; }
    let below = [cell[0], cell[1].checked_sub(1)?, cell[2]];
    let air = |at| volume.material(at) == Some(volume.materials.air);
    if !clear(volume, root) || !clear(volume, cell) || !air(cell)
        || !volume.material(below).is_some_and(|m| m != volume.materials.air && m != volume.materials.water)
        || !(0..profile.body.height).all(|y| cell[1].checked_add(i64::from(y))
            .is_some_and(|h| air([cell[0], h, cell[2]])))
    { return None; }
    Some(cell)
}

/// Existing one-column stances, filtered by the profile's actual passage clearance.
pub fn neighbours(volume: &Volume, places: &Places, profile: &StepProfile, at: [i64; 3])
    -> Vec<[i64; 3]> {
    let Some(foot) = stance(volume, profile, at) else { return Vec::new(); };
    let Some(from) = places.place_at(foot) else { return Vec::new(); };
    isometer_space::places::steps(volume, profile.places, foot).into_iter()
        .filter_map(|(next, rise)| {
            if rise > 0 && rise as u64 > u64::from(profile.body.climb) { return None; }
            let mut root = next;
            for k in 0..3 { root[k] = root[k].checked_sub(profile.stance_offset[k])?; }
            stance(volume, profile, root)?;
            let to = places.place_at(next)?;
            if from == to { return Some(root); }
            let key = if from < to { [from, to] } else { [to, from] };
            places.passages.get(&key).filter(|p| profile.body.fits(p).is_some()).map(|_| root)
        }).collect()
}

pub fn check_step(sim: &Simulation, request: &Request, geometry: &Geometry) -> Result<PlaceId> {
    let actor = sim.state().population.get(request.actor).ok_or("step actor absent")?;
    let profile = sim.genesis().rules.step_profiles.get(&request.profile).ok_or("step profile absent")?;
    if !actor.alive || actor.body.is_none() || actor.resident_site() != Some(request.site)
        || actor.patch != Some(request.patch) || actor.body_revision != request.body_revision
        || sim.state().tick != request.tick || sim.genesis().rules.revision() != request.revision
        || profile.process != request.process || sim.terrain_revision() != request.terrain_revision
        || geometry.path.is_empty() || geometry.path.len() as u128 > u128::from(request.cells) + 1
    { return Err("step source, profile, path or native binding changed".into()); }
    let volume = sim.volume(request.site, geometry.window.min, geometry.window.max, geometry.window.depth)?;
    let places = Places::derive(&volume, profile.places)?;
    if places.place_at(geometry.path[0].at) != Some(request.patch)
        || stance(&volume, profile, geometry.path[0].at).is_none() {
        return Err("step begins outside its admitted native patch".into());
    }
    for pair in geometry.path.windows(2) {
        if !neighbours(&volume, &places, profile, pair[0].at).contains(&pair[1].at) {
            return Err("step crosses an unavailable cell or authored clearance".into());
        }
    }
    let last = geometry.path.last().ok_or("empty step path")?;
    let patch = places.place_at(last.at).ok_or("step ends outside realized places")?;
    if !clear(&volume, geometry.eye) || places.place_at(geometry.eye) != Some(patch) {
        return Err("step eye is outside its realized clear patch".into());
    }
    Ok(patch)
}

pub(super) fn check_arrival(sim: &Simulation, request: &ArrivalRequest, geometry: &ArrivalGeometry)
    -> Result<PlaceId> {
    let actor = sim.state().population.get(request.actor).ok_or("arrival actor absent")?;
    if actor.body.is_none() || actor.resident_site() != Some(request.to) || actor.arrived != request.arrives
        || actor.body_revision != request.body_revision || sim.terrain_revision() != request.terrain_revision
    { return Err("arrival placement source changed".into()); }
    let volume = sim.volume(request.to, geometry.window.min, geometry.window.max, geometry.window.depth)?;
    let places = Places::derive(&volume, geometry.places)?;
    let patch = places.place_at(geometry.pose.at).ok_or("arrival pose has no realized patch")?;
    if !clear(&volume, geometry.pose.at) || !clear(&volume, geometry.eye)
        || places.place_at(geometry.eye) != Some(patch)
    { return Err("arrival pose or eye is outside its explicit clear patch".into()); }
    Ok(patch)
}

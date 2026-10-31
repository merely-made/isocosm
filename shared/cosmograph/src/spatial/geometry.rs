// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Integer walking over the existing isometer places, with authored clearance.

use super::*;
use isometer_space::{places::Places, volume::Volume};

pub(super) fn clear(volume: &Volume, cell: [i64; 3]) -> bool {
    volume.holds(cell[0], cell[2])
        && cell[1] >= volume.datum
        && cell[1] < volume.ceiling()
        && volume
            .material(cell)
            .is_some_and(|m| m == volume.materials.air || m == volume.materials.water)
}

pub(super) fn stance(
    volume: &Volume,
    offset: [i64; 3],
    height: u32,
    root: [i64; 3],
) -> Option<[i64; 3]> {
    let mut cell = root;
    for k in 0..3 {
        cell[k] = cell[k].checked_add(offset[k])?;
    }
    let below = [cell[0], cell[1].checked_sub(1)?, cell[2]];
    let air = |at| volume.material(at) == Some(volume.materials.air);
    if !clear(volume, root)
        || !clear(volume, cell)
        || !air(cell)
        || !volume
            .material(below)
            .is_some_and(|m| m != volume.materials.air && m != volume.materials.water)
        || !(0..height).all(|y| {
            cell[1]
                .checked_add(i64::from(y))
                .is_some_and(|h| air([cell[0], h, cell[2]]))
        })
    {
        return None;
    }
    Some(cell)
}

/// Existing one-column stances, filtered by the profile's actual passage clearance.
pub fn neighbours(
    volume: &Volume,
    places: &Places,
    profile: &StepProfile,
    at: [i64; 3],
) -> Vec<[i64; 3]> {
    let Some(foot) = stance(volume, profile.stance_offset, profile.body.height, at) else {
        return Vec::new();
    };
    let Some(from) = places.place_at(foot) else {
        return Vec::new();
    };
    isometer_space::places::steps(volume, profile.places, foot)
        .into_iter()
        .filter_map(|(next, rise)| {
            if rise > 0 && rise as u64 > u64::from(profile.body.climb) {
                return None;
            }
            let mut root = next;
            for k in 0..3 {
                root[k] = root[k].checked_sub(profile.stance_offset[k])?;
            }
            stance(volume, profile.stance_offset, profile.body.height, root)?;
            let to = places.place_at(next)?;
            if from == to {
                return Some(root);
            }
            let key = if from < to { [from, to] } else { [to, from] };
            places
                .passages
                .get(&key)
                .filter(|p| profile.body.fits(p).is_some())
                .map(|_| root)
        })
        .collect()
}

pub fn check_step(sim: &Simulation, request: &Request, geometry: &Geometry) -> Result<PlaceId> {
    let actor = sim
        .state()
        .population
        .get(request.actor)
        .ok_or("step actor absent")?;
    let profile = sim
        .genesis()
        .rules
        .step_profiles
        .get(&request.profile)
        .ok_or("step profile absent")?;
    if !actor.alive
        || actor.body.is_none()
        || actor.resident_site() != Some(request.site)
        || actor.patch != Some(request.patch)
        || actor.body_revision != request.body_revision
        || sim.state().tick != request.tick
        || sim.genesis().rules.revision() != request.revision
        || profile.process != request.process
        || sim.terrain_revision() != request.terrain_revision
        || geometry.path.is_empty()
        || geometry.path.len() as u128 > u128::from(request.cells) + 1
    {
        return Err("step source, profile, path or native binding changed".into());
    }
    let t = sim.genesis().rules.spatial_templates.as_ref();
    let source_window = t
        .filter(|t| t.version >= 5)
        .map(|t| embodiment::site_window(t, geometry.path[0].at))
        .unwrap_or_else(|| geometry.window.clone());
    let terrain = sim.site_geometry(request.site, &source_window, profile.places)?;
    let (volume, places) = (&terrain.volume, &terrain.places);
    let whole = embodiment::enabled(sim);
    let walking = embodiment::walking_rules(sim);
    if (!geometry.strides.is_empty() && geometry.strides.len() + 1 != geometry.path.len())
        || (walking.is_none() && geometry.strides.iter().any(Option::is_some))
    {
        return Err("step gait witnesses differ from its path or native policy".into());
    }
    if sim.foraging_enabled()
        && sim
            .state()
            .spatial_poses
            .get(&request.actor)
            .is_none_or(|at| {
                at.site != request.site
                    || at.arrived != actor.arrived
                    || at.pose != geometry.path[0]
            })
    {
        return Err("step begins away from its native resident pose".into());
    }
    if sim.foraging_enabled()
        && sim
            .genesis()
            .rules
            .spatial_templates
            .as_ref()
            .is_some_and(|t| {
                embodiment::site_window(t, geometry.path.last().unwrap().at) != geometry.window
            })
    {
        return Err("step differs from the native resident terrain window".into());
    }
    let document = actor.body.as_ref().ok_or("step body absent")?;
    if places.place_at(geometry.path[0].at) != Some(request.patch)
        || if whole {
            embodiment::check_member(sim, &volume, actor, &geometry.path[0]).is_err()
        } else {
            stance(
                &volume,
                profile.stance_offset,
                profile.body.height,
                geometry.path[0].at,
            )
            .is_none()
        }
    {
        return Err("step begins outside its admitted native patch".into());
    }
    let blocked: Vec<_> = sim
        .state()
        .spatial_poses
        .keys()
        .copied()
        .filter(|id| *id != request.actor)
        .filter_map(|id| sim.current_placed_boxes(id, request.site))
        .flatten()
        .collect();
    let mut checks = embodiment::StrideChecks::new(volume, document, &blocked)
        .with_balance(embodiment::balance_for(sim, actor)?);
    for (index, pair) in geometry.path.windows(2).enumerate() {
        if let Some(stride) = geometry.strides.get(index).and_then(Option::as_ref) {
            if pair[0].feet.is_empty()
                || (pair[0].yaw != pair[1].yaw
                    && !(walking.is_some_and(|r| r.turns) && stride.turn != 0))
            {
                return Err("gait witness has no matching admitted articulated step".into());
            }
        }
        if sim.foraging_enabled()
            && !sim.spatial_destination_clear(request.actor, request.site, &pair[1])
        {
            return Err("step destination intersects a current native resident".into());
        }
        let available = if whole {
            let local = embodiment::boxes(
                document,
                &Pose {
                    yaw: pair[0].yaw,
                    ..Pose::default()
                },
            )?;
            checks
                .neighbours(&places, profile, &local, &pair[0], walking)
                .contains(&pair[1])
        } else {
            neighbours(&volume, &places, profile, pair[0].at).contains(&pair[1].at)
        };
        if !available {
            return Err("step crosses an unavailable cell or authored clearance".into());
        }
        if embodiment::articulated(sim) && !pair[0].feet.is_empty() {
            checks.check_walk(
                &pair[0],
                &pair[1],
                walking,
                geometry.strides.get(index).and_then(Option::as_ref),
            )?;
        }
    }
    let last = geometry.path.last().ok_or("empty step path")?;
    let final_terrain = sim.site_geometry(request.site, &geometry.window, profile.places)?;
    let (volume, places) = (&final_terrain.volume, &final_terrain.places);
    let patch = places
        .place_at(last.at)
        .ok_or("step ends outside realized places")?;
    if !(if whole {
        embodiment::air(&volume, geometry.eye)
    } else {
        clear(&volume, geometry.eye)
    }) || places.place_at(geometry.eye) != Some(patch)
    {
        return Err("step eye is outside its realized clear patch".into());
    }
    Ok(patch)
}

pub(super) fn check_arrival(
    sim: &Simulation,
    request: &ArrivalRequest,
    geometry: &ArrivalGeometry,
) -> Result<PlaceId> {
    let actor = sim
        .state()
        .population
        .get(request.actor)
        .ok_or("arrival actor absent")?;
    if actor.body.is_none()
        || actor.resident_site() != Some(request.to)
        || actor.arrived != request.arrives
        || actor.body_revision != request.body_revision
        || sim.terrain_revision() != request.terrain_revision
    {
        return Err("arrival placement source changed".into());
    }
    let terrain = sim.site_geometry(request.to, &geometry.window, geometry.places)?;
    if let Some(t) = sim
        .genesis()
        .rules
        .spatial_templates
        .as_ref()
        .filter(|t| t.version >= 5)
        && geometry.window != embodiment::site_window(t, geometry.pose.at)
    {
        return Err("arrival window differs from its native site tile".into());
    }
    let (volume, places) = (&terrain.volume, &terrain.places);
    let patch = places
        .place_at(geometry.pose.at)
        .ok_or("arrival pose has no realized patch")?;
    let fits = if embodiment::enabled(sim) {
        embodiment::check_member(sim, &volume, actor, &geometry.pose)?;
        embodiment::air(&volume, geometry.eye)
    } else {
        clear(&volume, geometry.pose.at) && clear(&volume, geometry.eye)
    };
    if !fits || places.place_at(geometry.eye) != Some(patch) {
        return Err("arrival pose or eye is outside its explicit clear patch".into());
    }
    Ok(patch)
}

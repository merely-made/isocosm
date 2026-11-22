// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Resolve the local part of a voluntary border journey using the current body and terrain.
use cosmograph::{
    Simulation,
    schema::{Id, Journey},
    spatial::{ArrivalGeometry, Handback, Pose, embodiment},
};
use std::{
    cmp::Reverse,
    collections::{BTreeSet, BinaryHeap},
};

pub(super) fn walk(
    sim: &Simulation,
    id: Id,
    trip: Journey,
) -> Result<Option<(Handback, Vec<Pose>)>, String> {
    let Some(border) = trip.border else {
        return Ok(None);
    };
    let Some(held) = sim
        .state()
        .spatial_poses
        .get(&id)
        .filter(|r| r.site == trip.from)
    else {
        return Ok(None);
    };
    let entity = sim
        .state()
        .population
        .get(id)
        .ok_or("departing body absent")?;
    let Some(doc) = entity.body.as_ref() else {
        return Ok(None);
    };
    let rules = &sim.genesis().rules;
    let Some(t) = rules.spatial_templates.as_ref() else {
        return Ok(None);
    };
    let Some((key, policy)) = t.foraging.iter().find(|(key, _)| {
        key.strip_prefix("body:walk:")
            .is_some_and(|k| entity.traits.contains(k))
    }) else {
        return Ok(None);
    };
    let profile = &rules.step_profiles[key];
    let speed = rules.speed.ok_or("walking speed absent")?.of(doc);
    let allowance = u128::from(speed) * u128::from(rules.tick_microseconds()) / 1_000_000_000;
    let budget = (policy.nodes as u128).min(allowance) as usize;
    let mut occupied = Vec::new();
    for (&other, r) in &sim.state().spatial_poses {
        if other == id || r.site != held.site {
            continue;
        }
        if let Some(e) = sim.state().population.get(other)
            && let Some(body) = e.body.as_ref()
        {
            occupied.extend(embodiment::boxes(body, &r.pose)?);
        }
    }
    let axis = if border.side % 2 == 0 { 2 } else { 0 };
    let width = t.window.max[axis / 2] - t.window.min[axis / 2];
    let edge = if matches!(border.side, 0 | 3) {
        t.window.min[axis / 2]
    } else {
        t.window.max[axis / 2] - 1
    };
    let middle = t.window.min[(2 - axis) / 2] + width / 2;
    let distance = |p: &Pose| {
        let along = u128::from(p.at[axis].abs_diff(edge));
        let across = u128::from(p.at[2 - axis].abs_diff(middle));
        along * along + across * across
    };
    let mut nodes = vec![(held.pose.clone(), 0usize, 0usize)];
    let mut frontier = BinaryHeap::from([Reverse((distance(&held.pose), 0usize))]);
    let mut visited = BTreeSet::from([(held.pose.at, held.pose.yaw)]);
    let mut best = 0;
    let mut checked = 0;
    let mut last_refusal = None;
    // The body may have to step sideways before it makes progress toward the
    // exit. Retain parents so dead ends never appear in the actual walk.
    while let Some(Reverse((_, index))) = frontier.pop() {
        if checked >= policy.nodes as usize {
            break;
        }
        let (from, _, depth) = nodes[index].clone();
        if depth >= budget {
            continue;
        }
        let terrain =
            sim.site_geometry(trip.from, &embodiment::site_window(t, from.at), t.places)?;
        let mut candidates =
            embodiment::neighbours_for(sim, id, &terrain.volume, &terrain.places, profile, &from)?;
        candidates.sort_by_key(|p| (distance(p), p.at, p.yaw));
        for p in candidates {
            if checked >= policy.nodes as usize {
                break;
            }
            if visited.contains(&(p.at, p.yaw)) {
                continue;
            }
            checked += 1;
            if let Err(reason) =
                embodiment::step_for(sim, id, &terrain.volume, &from, &p, &occupied)
            {
                last_refusal = Some(reason);
                continue;
            }
            visited.insert((p.at, p.yaw));
            let cost = distance(&p);
            let next = nodes.len();
            if cost < distance(&nodes[best].0) {
                best = next;
            }
            nodes.push((p, index, depth + 1));
            frontier.push(Reverse((cost, next)));
        }
    }
    if best == 0 {
        eprintln!(
            "exit toward {}: no closer stance in {} checked strides ({} admitted); {:?}",
            trip.to,
            checked,
            nodes.len() - 1,
            last_refusal
        );
        return Ok(None);
    }
    let mut path = vec![];
    loop {
        path.push(nodes[best].0.clone());
        if best == 0 {
            break;
        }
        best = nodes[best].1;
    }
    path.reverse();
    let end = path.last().unwrap();
    let handback = Handback {
        actor: id,
        site: trip.from,
        body_revision: entity.body_revision,
        terrain_revision: sim.terrain_revision(),
        geometry: ArrivalGeometry {
            window: embodiment::site_window(t, end.at),
            eye: std::array::from_fn(|k| end.at[k] + t.eye[k]),
            pose: end.clone(),
            places: t.places,
        },
        contacts: Vec::new(),
    };
    Ok(Some((handback, path)))
}

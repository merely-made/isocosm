// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use isometer_space::places::Places;
use std::collections::VecDeque;

/// A bounded route to contact with the actual chosen food. The first step
/// uses the same native body support, passage and occupancy checks as its goal.
pub fn geometry(sim: &Simulation, request: &Request, window: &Window) -> Result<Geometry> {
    let f = sim
        .foraging_policy(&request.profile)
        .ok_or("foraging profile absent")?;
    let p = &sim.genesis.rules.processes[&request.process];
    if !sim.foraging_walk_matches(request.actor, request.target, p) {
        return Err("chosen food no longer serves a current need within search range".into());
    }
    let target = request.target.ok_or("foraging target absent")?;
    let before = &sim
        .spatial_pose_at_start(request.actor)
        .ok_or("foraging pose absent")?
        .pose;
    let document = sim
        .body_at_start(request.actor)
        .and_then(|e| e.body.as_ref())
        .ok_or("walking body absent")?;
    let food = sim
        .current_placed_boxes(target, request.site)
        .ok_or("chosen food pose absent")?;
    let profile = &sim.genesis.rules.step_profiles[&request.profile];
    let volume = sim.volume(request.site, window.min, window.max, window.depth)?;
    let places = Places::derive(&volume, profile.places)?;
    embodiment::check(&volume, document, before)?;
    let local = embodiment::boxes(
        document,
        &Pose {
            at: [0; 3],
            yaw: before.yaw,
        },
    )?;
    let mut grounding = embodiment::Grounding::new(&volume);
    let blocked: Vec<_> = sim
        .state
        .spatial_poses
        .keys()
        .copied()
        .filter(|id| *id != request.actor)
        .filter_map(|id| sim.current_placed_boxes(id, request.site))
        .flatten()
        .collect();
    let mut parents = BTreeMap::from([(before.at, None)]);
    let mut poses = BTreeMap::from([(before.at, before.clone())]);
    let mut queue = VecDeque::from([before.at]);
    let mut goal = None;
    while let Some(cell) = queue.pop_front() {
        let pose = &poses[&cell];
        if gap(&embodiment::translated(&local, pose.at)?, &food) <= f.reach {
            goal = Some(cell);
            break;
        }
        for next in
            embodiment::neighbours_from_boxes(&places, profile, &local, pose, &mut grounding)
        {
            if parents.contains_key(&next.at) {
                continue;
            }
            if parents.len() >= f.nodes {
                continue;
            }
            let boxes = embodiment::translated(&local, next.at)?;
            if embodiment::intersects(&boxes, &blocked) {
                continue;
            }
            parents.insert(next.at, Some(cell));
            queue.push_back(next.at);
            poses.insert(next.at, next);
        }
    }
    let mut cell = goal.ok_or("no supported route to chosen food within the saved node budget")?;
    while let Some(Some(parent)) = parents.get(&cell) {
        if *parent == before.at {
            break;
        }
        cell = *parent;
    }
    if cell == before.at {
        return Err("food is already within feeding reach".into());
    }
    let after = poses[&cell].clone();
    let eye_offset = sim
        .genesis
        .rules
        .spatial_templates
        .as_ref()
        .expect("foraging templates")
        .eye;
    let mut eye = after.at;
    for k in 0..3 {
        eye[k] = eye[k]
            .checked_add(eye_offset[k])
            .ok_or("foraging eye overflow")?;
    }
    Ok(Geometry {
        window: window.clone(),
        path: vec![before.clone(), after],
        eye,
    })
}

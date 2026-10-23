// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
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
    let terrain = sim.site_geometry(request.site, window, profile.places)?;
    let (volume, places) = (&terrain.volume, &terrain.places);
    embodiment::check_for(sim, &volume, document, before)?;
    let local = embodiment::boxes(
        document,
        &Pose {
            at: [0; 3],
            yaw: before.yaw,
            feet: Default::default(),
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
    let origin = (before.at, before.yaw);
    let mut parents = BTreeMap::from([(origin, None)]);
    let mut poses = BTreeMap::from([(origin, before.clone())]);
    let mut queue = VecDeque::from([origin]);
    let modern = embodiment::articulated(sim);
    let mut priority = BTreeSet::from([(
        gap(&embodiment::boxes(document, before)?, &food),
        0usize,
        origin,
    )]);
    let mut goal = None;
    while let Some(cell) = if modern {
        priority.pop_first().map(|(_, _, key)| key)
    } else {
        queue.pop_front()
    } {
        let pose = poses[&cell].clone();
        if gap(&embodiment::boxes(document, &pose)?, &food) <= f.reach {
            goal = Some(cell);
            break;
        }
        let neighbours = if !before.feet.is_empty() {
            embodiment::articulated_neighbours(places, profile, document, &pose, &mut grounding)
        } else {
            embodiment::neighbours_from_boxes(places, profile, &local, &pose, &mut grounding)
        };
        for next in neighbours {
            let key = (next.at, next.yaw);
            if parents.contains_key(&key) {
                continue;
            }
            if parents.len() >= f.nodes {
                continue;
            }
            let boxes = embodiment::boxes(document, &next)?;
            if embodiment::intersects(&boxes, &blocked) {
                continue;
            }
            if !before.feet.is_empty()
                && cell == origin
                && embodiment::sweep_clear(volume, document, &pose, &next, &blocked).is_err()
            {
                continue;
            }
            parents.insert(key, Some(cell));
            if modern {
                priority.insert((gap(&boxes, &food), parents.len(), key));
            } else {
                queue.push_back(key);
            }
            poses.insert(key, next);
        }
    }
    let mut cell = goal.ok_or("no supported route to chosen food within the saved node budget")?;
    while let Some(Some(parent)) = parents.get(&cell) {
        if *parent == origin {
            break;
        }
        cell = *parent;
    }
    if cell == origin {
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
        window: sim.genesis().rules.spatial_templates.as_ref()
            .filter(|t| t.version >= 5).map(|t| embodiment::site_window(t,after.at))
            .unwrap_or_else(|| window.clone()),
        path: vec![before.clone(), after],
        eye,
    })
}

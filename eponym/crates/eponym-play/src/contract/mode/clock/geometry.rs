// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;

pub(super) fn add(at: [i64; 3], offset: [i64; 3]) -> Result<[i64; 3], String> {
    let mut out = at;
    for k in 0..3 { out[k] = at[k].checked_add(offset[k]).ok_or("Eponym authored geometry overflows")?; }
    Ok(out)
}

pub(super) fn root(sim: &Simulation, binding: &PoseBinding) -> Result<[i64; 3], String> {
    let v = sim.volume(binding.site, binding.window.min, binding.window.max, binding.window.depth)?;
    let cell = binding.pose.cell().map_err(|e| format!("Eponym clock pose: {e:?}"))?;
    Ok(crate::walking::to_site(&v, cell))
}

pub(super) fn saved_root(binding: &PoseBinding, physical: &Physical) -> Result<[i64; 3], String> {
    add(physical.frame_origin, binding.pose.cell().map_err(|e| format!("Eponym clock pose: {e:?}"))?.map(i64::from))
}

fn clear(sim: &Simulation, binding: &PoseBinding, physical: &Physical) -> Result<PlaceId, String> {
    let v = sim.volume(binding.site, binding.window.min, binding.window.max, binding.window.depth)?;
    let places = Places::derive(&v, physical.places)?;
    let at = saved_root(binding, physical)?;
    let patch = places.place_at(at).ok_or("Eponym actual root has no realized patch")?;
    for cell in [at, physical.eye] {
        if !v.holds(cell[0], cell[2]) || cell[1] < v.datum || cell[1] >= v.ceiling()
            || places.place_at(cell) != Some(patch)
            || !v.material(cell).is_some_and(|m| m == v.materials.air || m == v.materials.water) {
            return Err("Eponym saved root/eye is outside its actual clear patch".into());
        }
    }
    Ok(patch)
}

pub(super) fn rebound(sim: &Simulation, binding: &PoseBinding, physical: &Physical) -> Result<(PoseBinding, Physical), String> {
    binding.pose.validate().map_err(|e| format!("Eponym clock pose: {e:?}"))?;
    let body = sim.state().population.get(binding.entity).ok_or("Eponym physical body is absent")?;
    if physical.subject != binding.subject || body.body.is_none()
        || body.resident_site() != Some(binding.site) || body.arrived != binding.resident_since
        || binding.places.unwrap_or_default() != physical.places {
        return Err("Eponym saved pose belongs to another native visit or authored places".into());
    }
    let mut next = binding.clone();
    next.patch = clear(sim, binding, physical)?;
    next.body_revision = body.body_revision; next.terrain_revision = sim.terrain_revision();
    let v = sim.volume(binding.site, binding.window.min, binding.window.max, binding.window.depth)?;
    let origin = crate::walking::to_site(&v, [0; 3]);
    let mut shift = [0; 3];
    for k in 0..3 { shift[k] = physical.frame_origin[k].checked_sub(origin[k]).ok_or("Eponym frame shift overflows")?; }
    next.pose = translate(binding.pose, shift)?;
    let mut held = physical.clone(); held.frame_origin = origin;
    Ok((next, held))
}

pub(super) fn current(sim: &Simulation, binding: &PoseBinding, physical: &Physical) -> Result<(), String> {
    let (next, held) = rebound(sim, binding, physical)?;
    let body = sim.state().population.get(binding.entity).ok_or("Eponym physical body is absent")?;
    if next != *binding || held != *physical || body.patch != Some(binding.patch) {
        return Err("Eponym clock physical binding differs from current native geometry".into());
    }
    Ok(())
}

fn held(mode: &ModeState, state: &State, sim: &Simulation, actor: Id)
    -> Result<(PoseBinding, Physical), String> {
    let old = mode.bindings.iter().find(|b| b.entity == actor).ok_or("chosen actor has no explicit Eponym subject")?;
    let physical = state.physical.get(&old.subject).ok_or("chosen actor has no authored eye/yaw/places")?;
    let (binding, physical) = rebound(sim, old, physical)?;
    current(sim, &binding, &physical)?;
    Ok((binding, physical))
}

pub(super) fn terrain(mode: &ModeState, state: &State, pending: &isocosm::operations::Pending)
    -> Result<isocosm::operations::Geometry, String> {
    let r = &pending.request;
    let (at, physical) = held(mode, state, &pending.sim, r.actor)?;
    let input = state.profile.terrain.get(&at.subject).and_then(|m| m.get(&r.process))
        .ok_or("chosen terrain process has no authored Eponym input")?;
    if input.target != r.target || at.site != r.site || at.patch != r.patch
        || at.body_revision != r.body_revision || at.terrain_revision != r.terrain_revision {
        return Err("terrain input differs from its chosen actor/target/native source".into());
    }
    let root = root(&pending.sim, &at)?;
    let shape = match &input.shape_from_root {
        Shape::Sphere { centre, radius } => Shape::Sphere { centre: add(root, *centre)?, radius: *radius },
        Shape::Box { min, max } => Shape::Box { min: add(root, *min)?, max: add(root, *max)? },
        Shape::Route { points, radius } => Shape::Route {
            points: points.iter().map(|p| add(root, *p)).collect::<Result<_, _>>()?, radius: *radius },
    };
    let op = match &r.change {
        isocosm::operations::Change::Carve => Op::Carve,
        isocosm::operations::Change::Deposit { material } => {
            let index = pending.sim.genesis().world.materials.iter().position(|m| &m.key == material)
                .ok_or("native deposit material is absent")?;
            Op::Fill(u8::try_from(index).map_err(|_| "native material ordinal overflow")?)
        },
    };
    let answer = isocosm::operations::Geometry { window: at.window, eye: physical.eye,
        focus: add(root, input.focus_from_root)?, edit: Edit { op, shape } };
    isocosm::operations::check_geometry(&pending.sim, r, &answer)?;
    Ok(answer)
}

pub(super) fn step(mode: &ModeState, state: &State, pending: &native::Pending) -> Result<native::Geometry, String> {
    let r = &pending.request;
    let (at, physical) = held(mode, state, &pending.sim, r.actor)?;
    let profile = pending.sim.genesis().rules.step_profiles.get(&r.profile).ok_or("native step profile absent")?;
    let input = state.profile.steps.get(&at.subject).and_then(|m| m.get(&r.profile))
        .ok_or("chosen step has no authored Eponym route")?;
    if input.target != r.target || physical.places != profile.places {
        return Err("step target/places differ from the authored native profile".into());
    }
    let root = root(&pending.sim, &at)?;
    let max = r.cells.checked_add(1).ok_or("native step budget overflow")?;
    let count = usize::try_from(max).unwrap_or(usize::MAX);
    let mut path = input.path_from_root.iter().take(count).map(|delta|
        Ok(native::Pose { at: add(root, *delta)?, yaw: input.yaw , feet: Default::default() }))
        .collect::<Result<Vec<_>, String>>()?;
    path.first_mut().ok_or("authored step path is empty")?.yaw = physical.yaw;
    let eye = add(path.last().ok_or("authored step path is empty")?.at, input.eye_from_root)?;
    let answer = native::Geometry { window: at.window, path, eye };
    native::check_step(&pending.sim, r, &answer)?;
    Ok(answer)
}

pub(super) fn translate(mut pose: MotionPose, delta: [i64; 3]) -> Result<MotionPose, String> {
    for k in 0..3 {
        let shift = delta[k].checked_mul(crate::MOTION_SCALE).ok_or("Eponym clock quanta overflow")?;
        pose.position[k] = pose.position[k].checked_add(shift).ok_or("Eponym clock position overflow")?;
        if k == 1 {
            pose.fall_origin_y = pose.fall_origin_y.map(|y| y.checked_add(shift)
                .ok_or("Eponym fall origin overflow")).transpose()?;
        }
    }
    pose.validate().map_err(|e| format!("Eponym translated clock pose: {e:?}"))?;
    Ok(pose)
}

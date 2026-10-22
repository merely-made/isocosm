// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::anatomy::NativeAnatomyRead;

fn slot<'a>(state: &'a State, r: &native::BirthRequest, sim: &Simulation)
    -> Result<(&'a BirthInput, &'a BirthSlot), String> {
    let mut matches = state.profile.births.iter().filter(|i| {
        sim.genesis().rules.birth_profiles.get(&i.profile).is_some_and(|p| p.process == r.process)
            && match &i.parent { BirthParent::Entity(id) => *id == r.parent,
                BirthParent::Lineage(key) => key == &r.parent_lineage }
    });
    let input = matches.next().ok_or("birth has no authored Eponym placement profile")?;
    if matches.next().is_some() { return Err("birth has ambiguous Eponym placement profiles".into()); }
    let ordinal = usize::try_from(r.ordinal).map_err(|_| "birth ordinal overflows")?;
    let slot = input.slots.get(ordinal).ok_or("birth ordinal has no complete authored Eponym slot")?;
    Ok((input, slot))
}

pub(super) fn resolve(mode: &ModeState, state: &State, actions: &ActionState, pending: &native::BirthPending)
    -> Result<native::BirthGeometry, String> {
    let r = &pending.request;
    if !r.parent_alive || r.site.is_none() || r.child_journey.is_some() {
        return Err("birth has no living-at-action-start resident parent pose".into());
    }
    let (input, slot) = slot(state, r, &pending.sim)?;
    if mode.bindings.iter().any(|b| b.subject == slot.subject || b.entity == r.child)
        || actions.profiles.iter().any(|p| p.subject == slot.subject) {
        return Err("authored birth slot reuses an accepted product/native identity".into());
    }
    let old = mode.bindings.iter().find(|b| b.entity == r.parent).ok_or("birth parent lacks its actual product binding")?;
    let physical = state.physical.get(&old.subject).ok_or("birth parent lacks its authored eye/yaw")?;
    let (parent, physical) = geometry::rebound(&pending.sim, old, physical)?;
    geometry::current(&pending.sim, &parent, &physical)?;
    if parent.site != r.site.ok_or("birth parent is travelling")? || parent.body_revision != r.parent_revision
        || parent.resident_since != r.parent_visit || parent.terrain_revision != r.terrain_revision {
        return Err("birth parent's actual pose differs from the native request".into());
    }
    let geometry = native::BirthGeometry { profile: input.profile.clone(), parent_pose: native::Pose {
        at: geometry::root(&pending.sim, &parent)?, yaw: physical.yaw , feet: Default::default() }, parent_eye: physical.eye,
        parent_window: parent.window, parent_places: physical.places,
        offset_from_parent: slot.offset_from_parent, yaw_from_parent: slot.yaw_from_parent,
        eye_from_child: slot.eye_from_child, window: slot.window.clone(), feet: Default::default() };
    let (pose, _) = native::birth_pose(&geometry)?;
    motion(&pending.sim, r.site.ok_or("birth parent is travelling")?, &slot.window, pose.at, &slot.pose)?;
    profile(&pending.sim, r.child, slot)?;
    Ok(geometry)
}

fn motion(sim: &Simulation, site: Id, window: &Window, at: [i64; 3], input: &BirthMotion)
    -> Result<MotionPose, String> {
    let v = sim.volume(site, window.min, window.max, window.depth)?;
    let origin = crate::walking::to_site(&v, [0; 3]);
    let mut position = [0; 3];
    for k in 0..3 {
        position[k] = at[k].checked_sub(origin[k]).and_then(|cell| cell.checked_mul(crate::MOTION_SCALE))
            .and_then(|quanta| quanta.checked_add(i64::from(input.fraction[k])))
            .ok_or("authored newborn fixed-point pose overflows")?;
    }
    let pose = MotionPose { position, velocity: input.velocity, grounded: input.grounded, step: input.step,
        fall_origin_y: input.fall_origin_from_root.map(|delta| position[1].checked_add(delta)
            .ok_or("authored newborn fall origin overflows")).transpose()? };
    pose.validate().map_err(|e| format!("authored newborn motion: {e:?}"))?;
    Ok(pose)
}

fn profile(sim: &Simulation, child: Id, slot: &BirthSlot) -> Result<SubjectProfile, String> {
    let child = sim.state().population.get(child).ok_or("accepted biological child is absent")?;
    let profile = SubjectProfile { subject: slot.subject, species: slot.species, revision: slot.revision,
        native_revision: child.body_revision, motion: slot.motion.clone(), combat: slot.combat.clone() };
    let record = NativeAnatomyRead::new(slot.subject, slot.revision, slot.species,
        child, &sim.genesis().rules)?.view();
    if let Some(policy) = &profile.motion {
        if (policy.rules.revision == crate::MOVEMENT_PROFILE_REVISION) != policy.profile.is_some() {
            return Err("newborn motion grammar and authored anatomy roles disagree".into());
        }
        if let Some(roles) = &policy.profile { roles.validate_view(record).map_err(|e| format!("newborn movement profile: {e:?}"))?; }
    }
    Ok(profile)
}

pub(super) fn publish(mode: &mut ModeState, state: &mut State, actions: &mut ActionState,
    sim: &Simulation, answers: &[native::BirthResolution], report: &native::BirthReport)
    -> Result<Vec<Change>, String> {
    let Ok(patch) = report.patch else { return Ok(Vec::new()); };
    let r = &report.request;
    let answer = answers.iter().find(|a| a.request == *r).ok_or("accepted birth answer absent")?;
    let geometry = answer.geometry.as_ref().map_err(|_| "accepted birth answer was refused")?;
    let (input, slot) = slot(state, r, sim)?;
    if report.profile.as_ref() != Some(&input.profile) || geometry.profile != input.profile
        || geometry.offset_from_parent != slot.offset_from_parent || geometry.yaw_from_parent != slot.yaw_from_parent
        || geometry.eye_from_child != slot.eye_from_child || geometry.window != slot.window {
        return Err("accepted birth differs from its authored Eponym slot".into());
    }
    if mode.bindings.iter().any(|b| b.subject == slot.subject || b.entity == r.child)
        || actions.profiles.iter().any(|p| p.subject == slot.subject) {
        return Err("accepted newborn attempts to reuse a product/native identity".into());
    }
    let parent = mode.bindings.iter().find(|b| b.entity == r.parent).ok_or("accepted birth parent pose absent")?;
    let physical = state.physical.get(&parent.subject).ok_or("accepted birth parent eye absent")?;
    if geometry.parent_pose.at != geometry::saved_root(parent, physical)? || geometry.parent_pose.yaw != physical.yaw
        || geometry.parent_eye != physical.eye || geometry.parent_window != parent.window
        || geometry.parent_places != physical.places || parent.resident_since != r.parent_visit {
        return Err("accepted birth does not use the saved current parent geometry".into());
    }
    let (root, eye) = native::birth_pose(geometry)?;
    let site = r.site.ok_or("accepted physical birth lacks a resident site")?;
    let pose = motion(sim, site, &slot.window, root.at, &slot.pose)?;
    // The exact request-time anatomy was validated by resolve(). Later native
    // injury may withdraw that profile; it must not undo a conserved birth.
    let authored = SubjectProfile { subject: slot.subject, species: slot.species, revision: slot.revision,
        native_revision: r.child_revision, motion: slot.motion.clone(), combat: slot.combat.clone() };
    let places = sim.genesis().rules.birth_profiles.get(&input.profile).ok_or("native birth profile absent")?.places;
    let after = PoseBinding { subject: slot.subject, entity: r.child, pose, site, patch, window: slot.window.clone(),
        body_revision: r.child_revision, terrain_revision: r.terrain_revision.clone(), resident_since: r.child_visit,
        places: Some(places) };
    let v = sim.volume(site, slot.window.min, slot.window.max, slot.window.depth)?;
    let physical = Physical { subject: slot.subject, yaw: root.yaw, eye, places,
        frame_origin: crate::walking::to_site(&v, [0; 3]) };
    let mut changes = Vec::new();
    publication::install(mode, state, Change { before: None, after, physical }, &mut changes)?;
    actions.profiles.push(authored);
    Ok(changes)
}

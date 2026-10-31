// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::{anatomy::NativeAnatomyRead, identity::BodyRevisionId};

pub(super) fn install(mode: &mut ModeState, state: &mut State, change: Change, out: &mut Vec<Change>)
    -> Result<(), String> {
    if change.after.subject != change.physical.subject { return Err("physical publication subject differs".into()); }
    match &change.before {
        Some(before) => {
            let held = mode.bindings.iter_mut().find(|b| b.subject == before.subject).ok_or("physical source subject absent")?;
            if *held != *before || change.after.entity != before.entity {
                return Err("physical publication source differs from saved binding".into());
            }
            *held = change.after.clone();
        },
        None => {
            if mode.bindings.iter().any(|b| b.subject == change.after.subject || b.entity == change.after.entity) {
                return Err("physical publication reuses an accepted subject/entity".into());
            }
            mode.bindings.push(change.after.clone()); mode.bindings.sort_by_key(|b| b.subject);
        },
    }
    state.physical.insert(change.after.subject, change.physical.clone()); out.push(change);
    Ok(())
}

fn step(mode: &mut ModeState, state: &mut State, answer: &native::Resolution) -> Result<Vec<Change>, String> {
    let geometry = answer.geometry.as_ref().map_err(|_| "accepted step answer was refused")?;
    let before = mode.bindings.iter().find(|b| b.entity == answer.request.actor).ok_or("accepted step source absent")?.clone();
    let physical = state.physical.get(&before.subject).ok_or("accepted step physical source absent")?;
    let first = geometry.path.first().ok_or("accepted step path is empty")?;
    let last = geometry.path.last().ok_or("accepted step path is empty")?;
    if first.at != geometry::saved_root(&before, physical)? || first.yaw != physical.yaw
        || geometry.window != before.window || answer.request.site != before.site {
        return Err("accepted step does not start at the saved full pose/frame".into());
    }
    let mut delta = [0; 3];
    for k in 0..3 { delta[k] = last.at[k].checked_sub(first.at[k]).ok_or("accepted step displacement overflows")?; }
    let mut after = before.clone(); after.pose = geometry::translate(before.pose, delta)?;
    after.body_revision = answer.request.body_revision; after.terrain_revision = answer.request.terrain_revision.clone();
    // Native admission owns the final patch; it is re-derived during refresh.
    let mut physical = physical.clone(); physical.yaw = last.yaw; physical.eye = geometry.eye;
    let mut changes = Vec::new();
    install(mode, state, Change { before: Some(before), after, physical }, &mut changes)?;
    Ok(changes)
}

pub(super) fn ordered(mode: &mut ModeState, state: &mut State, actions: &mut ActionState,
    sim: &Simulation, steps: &[native::Resolution], births: &[native::BirthResolution],
    reports: &[native::BirthReport], order: &[native::Publication]) -> Result<Vec<Change>, String> {
    let mut changes = Vec::new(); let mut used_steps = BTreeSet::new(); let mut used_births = BTreeSet::new();
    for item in order {
        match item {
            native::Publication::Step(action) => {
                if !used_steps.insert(action) { return Err("duplicate step publication".into()); }
                let input = steps.iter().find(|r| &r.request.action == action).ok_or("published step answer absent")?;
                changes.extend(step(mode, state, input)?);
            },
            native::Publication::Birth(child) => {
                if !used_births.insert(child) { return Err("duplicate birth publication".into()); }
                let report = reports.iter().find(|r| r.request.child == *child).ok_or("published birth report absent")?;
                changes.extend(birth::publish(mode, state, actions, sim, births, report)?);
            },
            // The sim's tier line (E5) and re-settling (D38) only matter for
            // a played subject; re-posing a saved binding is not built yet.
            native::Publication::Unplaced(id) | native::Publication::Drawn { actor: id, .. } => {
                if mode.bindings.iter().any(|b| b.entity == *id) {
                    return Err("tier or re-settle publication for a played subject is not supported".into());
                }
            },
        }
    }
    if used_births.len() != reports.len() { return Err("native birth report was omitted from publication order".into()); }
    Ok(changes)
}

/// Refresh only the accepted same-visit position. Obstructed, departed or
/// unsupported anatomy keeps its historical stamps and stays unavailable.
pub(super) fn refresh_inner(session: &mut Session, mode: &mut ModeState, state: &mut State,
    actions: &mut ActionState) -> Result<(Vec<Key>, Vec<Change>), String> {
    let first = session.entries.len(); let mut changes = Vec::new();
    for before in mode.bindings.clone() {
        let Some(physical) = state.physical.get(&before.subject) else { continue; };
        let Ok((after, physical)) = geometry::rebound(&session.sim, &before, physical) else { continue; };
        if session.sim.state().population.get(after.entity).is_some_and(|e| e.patch != Some(after.patch)) {
            session.command(Command::Patch { entity: after.entity, patch: Some(after.patch) })?;
        }
        geometry::current(&session.sim, &after, &physical)?;
        if before != after || state.physical.get(&before.subject) != Some(&physical) {
            install(mode, state, Change { before: Some(before), after, physical }, &mut changes)?;
        }
    }
    for profile in &mut actions.profiles {
        let Some(binding) = mode.bindings.iter().find(|b| b.subject == profile.subject) else { continue; };
        let Some(body) = session.sim.state().population.get(binding.entity) else { continue; };
        if profile.native_revision == body.body_revision || body.body.is_none() { continue; }
        let revision = BodyRevisionId(profile.revision.0.checked_add(1).ok_or("Eponym body revision overflows")?);
        let Ok(read) = NativeAnatomyRead::new(profile.subject, revision, profile.species,
            body, &session.sim.genesis().rules) else {
            continue; // Unsupported geometry withdraws this action profile.
        };
        let record = read.view();
        if profile.motion.as_ref().and_then(|p| p.profile.as_ref()).is_some_and(|roles|
            roles.project_view(record, profile.motion.as_ref().map_or(0, |m| m.rules.speed)).is_err()) {
            continue; // Existing authored role addresses are not silently replaced.
        }
        profile.revision = revision; profile.native_revision = body.body_revision;
    }
    Ok((session.entries[first..].iter().map(|e| e.id.clone()).collect(), changes))
}

/// Product callers invoke this inside their existing unpublished cut, after
/// an accepted action. Paid charge, policies and history are retained verbatim.
pub fn refresh(session: &mut Session, mode: &mut ModeState, actions: &mut ActionState) -> Result<(), String> {
    let Some(mut state) = mode.clock.take() else { return Ok(()); };
    let result = refresh_inner(session, mode, &mut state, actions);
    mode.clock = Some(state);
    result.map(|_| ())
}

/// An actual accepted solver displacement carries the authored eye with it.
pub fn motion(mode: &mut ModeState, before: &PoseBinding, after: &PoseBinding) -> Result<(), String> {
    let Some(state) = &mut mode.clock else { return Ok(()); };
    let Some(physical) = state.physical.get_mut(&before.subject) else { return Ok(()); };
    if before.subject != after.subject || before.entity != after.entity || before.site != after.site
        || before.window != after.window || before.resident_since != after.resident_since {
        return Err("Eponym motion cannot reinterpret the clock's physical frame".into());
    }
    let from = before.pose.cell().map_err(|e| format!("Eponym motion source: {e:?}"))?.map(i64::from);
    let to = after.pose.cell().map_err(|e| format!("Eponym motion result: {e:?}"))?.map(i64::from);
    let mut delta = [0; 3];
    for k in 0..3 { delta[k] = to[k].checked_sub(from[k]).ok_or("Eponym eye shift overflows")?; }
    physical.eye = geometry::add(physical.eye, delta)?;
    Ok(())
}

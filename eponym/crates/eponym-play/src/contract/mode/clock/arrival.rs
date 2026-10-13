// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;

pub(super) fn inputs(session: &Session, mode: &ModeState, state: &State)
    -> Result<Vec<native::ArrivalResolution>, String> {
    let end = session.sim.state().tick.checked_add(1).ok_or("Eponym clock overflows")?;
    let mut out = Vec::new();
    for input in &state.profile.arrivals {
        let Some(binding) = mode.bindings.iter().find(|b| b.subject == input.subject) else { continue; };
        let Some(actor) = session.sim.state().population.get(binding.entity) else { continue; };
        let Some(journey) = &actor.journey else { continue; };
        if journey.from != input.from || journey.to != input.to || journey.arrives > end
            || journey.arrives < session.sim.state().tick { continue; }
        input.pose.validate().map_err(|e| format!("Eponym arrival pose: {e:?}"))?;
        // Destination geometry is optional. An unavailable declared window
        // withdraws the picture, not the conserved native journey/clock.
        let Ok(v) = session.sim.volume(input.to, input.window.min, input.window.max, input.window.depth) else { continue; };
        out.push(native::ArrivalResolution {
            request: native::ArrivalRequest { actor: binding.entity, from: journey.from, to: journey.to,
                departed: journey.departed, arrives: journey.arrives, body_revision: actor.body_revision,
                terrain_revision: session.sim.terrain_revision() },
            geometry: native::ArrivalGeometry { window: input.window.clone(), pose: native::Pose {
                at: crate::walking::to_site(&v, input.pose.cell().map_err(|e| format!("Eponym arrival pose: {e:?}"))?),
                yaw: input.yaw }, eye: input.eye, places: input.places },
        });
    }
    Ok(out)
}

pub(super) fn publish(mode: &mut ModeState, state: &mut State, inputs: &[native::ArrivalResolution],
    reports: &[native::ArrivalReport]) -> Result<Vec<Change>, String> {
    let mut changes = Vec::new();
    for report in reports {
        let Ok(patch) = report.patch else { continue; };
        let answer = inputs.iter().find(|a| a.request.actor == report.actor).ok_or("accepted arrival answer absent")?;
        let before = mode.bindings.iter().find(|b| b.entity == report.actor).ok_or("accepted arrival subject absent")?.clone();
        let input = state.profile.arrivals.iter().find(|i| i.subject == before.subject
            && i.from == answer.request.from && i.to == answer.request.to).ok_or("accepted arrival authored input absent")?;
        if answer.geometry.window != input.window || answer.geometry.eye != input.eye
            || answer.geometry.pose.yaw != input.yaw || answer.geometry.places != input.places {
            return Err("arrival answer differs from the saved authored placement".into());
        }
        let after = PoseBinding { subject: before.subject, entity: before.entity, pose: input.pose,
            site: input.to, patch, window: input.window.clone(), body_revision: answer.request.body_revision,
            terrain_revision: answer.request.terrain_revision.clone(), resident_since: answer.request.arrives,
            places: Some(input.places) };
        let cell = input.pose.cell().map_err(|e| format!("Eponym arrival pose: {e:?}"))?.map(i64::from);
        let mut original_origin = [0; 3];
        for k in 0..3 { original_origin[k] = answer.geometry.pose.at[k].checked_sub(cell[k])
            .ok_or("Eponym arrival frame overflows")?; }
        let physical = Physical { subject: before.subject, yaw: input.yaw, eye: input.eye, places: input.places,
            frame_origin: original_origin };
        publication::install(mode, state, Change { before: Some(before), after, physical }, &mut changes)?;
    }
    Ok(changes)
}

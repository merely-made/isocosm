// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The handle on a lineage that has no critter body (cases 1216, 1217, 1221,
//! 1223, 1224, 1226) as the saved loop carries it: what a holder reads, the
//! answers that hold, steer and turn, and what befell the held lineages in a
//! round the loop runs itself. The sim's body-less round runs no epoch
//! boundary and knows nothing of the round's ending, so a holder with no
//! living body takes the watcher's round, and these events follow it.

use super::*;
use crate::Refusal;
use cosmograph::{Command, Session, directing::{WORLD, interim::Happening, readings},
    schema::Id, simulation::Simulation, spread::{self, Identity}};
use std::collections::BTreeSet;

/// A holder with no living body: it holds a lineage, and what it plays, if
/// anything, is dead (case 1217). Only such a participant is admitted
/// without a body.
pub(crate) fn bodiless(session: &Session, participant: Id) -> bool {
    let sim = &session.sim;
    !sim.holds(participant).is_empty() && sim.played(participant).len() <= 1
        && !sim.plays(participant).is_some_and(|id| sim.state().population.get(id).is_some_and(|e| e.alive))
}

/// What `participant` holds, then what it could hold and does not.
pub(super) fn reading(sim: &Simulation, participant: Id) -> (Vec<HeldLine>, Vec<String>) {
    let held = sim.holds(participant);
    let state = sim.state();
    let lines = held.iter().filter_map(|key| {
        if sim.is_surface(key) { return readings::held(sim, key).map(HeldLine::Surface); }
        state.lineages.get(key).filter(|line| line.kingdom == WORLD).map(|_| HeldLine::World {
            lineage: key.clone(), shift: state.turn,
            bound: sim.genesis().rules.climate.as_ref().and_then(|climate| climate.turn) })
    }).collect();
    let holdable = readings::holdable(sim).into_iter().filter(|key| !held.contains(key)).collect();
    (lines, holdable)
}

/// The sim's command for a holding answer, issued for `participant`; `None`
/// for any other answer. The sim's refusal is the loop's error.
pub(super) fn command(participant: Id, answer: &LoopAnswer) -> Option<Command> {
    Some(match answer {
        LoopAnswer::Hold { lineage } => Command::Hold { participant, lineage: lineage.clone() },
        LoopAnswer::Unhold { lineage } => Command::Unhold { participant, lineage: lineage.clone() },
        LoopAnswer::Steer { lineage, on, off } => Command::Steer { participant, lineage: lineage.clone(),
            on: on.clone(), off: off.clone() },
        LoopAnswer::Turn { lineage, shift } => Command::Turn { participant, lineage: lineage.clone(), shift: *shift },
        _ => return None,
    })
}

/// Whether an answer is one of the four of the handle.
pub(super) fn is_handle(answer: &LoopAnswer) -> bool {
    matches!(answer, LoopAnswer::Hold { .. } | LoopAnswer::Unhold { .. }
        | LoopAnswer::Steer { .. } | LoopAnswer::Turn { .. })
}

/// Issues a handle answer on `session`; the loop's phase, question and
/// round are the caller's to keep.
pub(super) fn apply(session: &mut Session, participant: Id, answer: &LoopAnswer) -> Result<(), Refusal> {
    let command = command(participant, answer).ok_or(Refusal::Unasked)?;
    session.command(command).map(drop).map_err(Refusal::Sim)
}

/// What befell the lineages `held` between `before` and `after`: a surface's
/// reaches, losses, brims, meetings and end, then what the world raised if one
/// is world-class (cases 1224, 1226). The sim's own round reports the same for
/// a holder it runs; this is that report for a round the loop ran.
pub(super) fn happenings(before: &Simulation, after: &Simulation, held: &[String]) -> Vec<Happening> {
    let ended = |sim: &Simulation, key: &str| readings::held(sim, key).is_some_and(|line| line.ended);
    let surfaces: BTreeSet<&String> = held.iter().filter(|key| after.is_surface(key)).collect();
    let mut out: Vec<Happening> = spread::happened(&after.genesis().rules, before.state(), after.state()).into_iter()
        .filter(|event| surfaces.contains(&identity(event).key()))
        .map(Happening::Surfaced).collect();
    for lineage in &surfaces {
        if ended(after, lineage) && !ended(before, lineage) {
            out.push(Happening::LineageEnded { lineage: (*lineage).clone() });
        }
    }
    let world = held.iter().any(|key| after.state().lineages.get(key).is_some_and(|line| line.kingdom == WORLD));
    if world { out.extend(readings::raised(before, after).into_iter().map(Happening::Raised)); }
    out
}

fn identity(event: &spread::Surfaced) -> &Identity {
    match event {
        spread::Surfaced::Reached { identity, .. } | spread::Surfaced::Left { identity, .. }
            | spread::Surfaced::Brimmed { identity, .. } | spread::Surfaced::HostDied { identity, .. }
            | spread::Surfaced::Met { identity, .. } => identity,
    }
}

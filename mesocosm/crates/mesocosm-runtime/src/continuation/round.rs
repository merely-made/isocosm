// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use std::cell::RefCell;
use super::*;
use crate::{Refusal, succession};
use cosmograph::{Session, schema::Id, directing::interim::{BorrowedInterim, Happening},
    lineage::{boundary, revise}};

pub fn advance_loop(session: &mut Session, game: &mut Game, participant: Id)
    -> Result<String, Refusal> {
    if game.continuation.as_ref().is_some_and(|saved| !saved.queued.is_empty()) {
        return Err(Refusal::Unbuilt("saved owned queue requires its original owned driver".into()));
    }
    advance_with_flows(session, game, participant).map(|out| out.0)
}

pub(crate) fn advance_with_flows(session: &mut Session, game: &mut Game, participant: Id)
    -> Result<(String, Vec<cosmograph::flows::Flow>, Option<VerifiedQuestion>), Refusal> {
    let mut geometry = crate::spatial::GeometryCache::default();
    advance_with_flows_cached(session, game, participant, &mut geometry)
}

pub(crate) fn advance_with_flows_cached(session: &mut Session, game: &mut Game, participant: Id,
    geometry: &mut crate::spatial::GeometryCache)
    -> Result<(String, Vec<cosmograph::flows::Flow>, Option<VerifiedQuestion>), Refusal> {
    let _span = crate::profile::Span::new("round.borrowed");
    game.validate(session).map_err(Refusal::Sim)?;
    let mut saved = game.continuation.clone()
        .ok_or_else(|| Refusal::Unbuilt("configure the explicit played loop before stepping".into()))?;
    if saved.participant != participant { return Err(Refusal::NotPlayed); }
    if let Some(fault) = &saved.fault { return Err(Refusal::Sim(fault.clone())); }
    if matches!(saved.phase, LoopPhase::Held | LoopPhase::LineageEnded | LoopPhase::WorldEnded) {
        return Ok(("held for an exact loop answer".into(), Vec::new(), None));
    }
    if saved.phase == LoopPhase::NewWorldRequested {
        return Ok(("new world requested; save and use the explicit founding caller".into(), Vec::new(), None));
    }
    let mut candidate = crate::profile::measure("round.clone_session", || session.clone());
    let start = session.entries.len();
    let before_native = super::prefix(session);
    let before_tick = session.sim.state().tick;
    let driver = RefCell::new(physical(game));
    let settings = saved.settings;
    let mut world_end = saved.world_end.clone();
    saved.ending.wait = settings.rounds == Rounds::Waiting;
    let played_lineage = session.sim.plays(participant)
        .and_then(|id| session.sim.state().population.get(id)).map(|e| e.lineage.clone());
    if let (true, LoopPhase::Running, Some(lineage), None) =
        (settings.condition, saved.phase, &played_lineage, saved.ending.par) {
        let span = candidate.sim.genesis().rules.epoch_ticks;
        saved.ending.par = Some(scoring::par(&candidate, game, lineage, span).map_err(Refusal::Sim)?);
    }
    let ending_before = saved.ending;
    let mut ending = saved.ending;
    let mut advance = |session: &mut Session| driver.borrow_mut().advance_clock_with_geometry(session, geometry)
        .map(|out| out.flows);
    let mut scorer = scoring::Scorer::default();
    let mut score = |session: &Session, lineage: &str, choice: &boundary::Candidate, ticks|
        scorer.score(session, &driver.borrow(), lineage, choice, ticks);
    // A watcher's round, or a holder's with no living body to play: the sim's
    // body-less round runs no epoch boundary and keeps no ending, so both run
    // the clock and the ending here, and a holder's lineages report after.
    let holds = candidate.sim.holds(participant);
    let bodiless = saved.phase == LoopPhase::Watching
        || (saved.phase == LoopPhase::Running && super::holding::bodiless(&candidate, participant));
    let held_before = (!holds.is_empty()).then(|| candidate.sim.clone());
    let (happenings, flows) = if bodiless {
        let before = candidate.sim.state().tick;
        let epoch = candidate.sim.genesis().rules.epoch_ticks;
        if epoch == 0 { return Err(Refusal::Sim("watch epoch length is zero".into())); }
        let mut flows = Vec::new();
        for _ in 0..settings.pace.round {
            flows.extend(advance(&mut candidate).map_err(Refusal::Sim)?);
        }
        let now = candidate.sim.state().tick;
        let mut out = Vec::new();
        if world_end.is_none() && candidate.sim.global_collapse().map_err(Refusal::Sim)?
            .is_some_and(|reading| reading.collapsed) {
            let entry = candidate.entries.last().ok_or_else(|| Refusal::Sim("world end lacks its accepted clock entry".into()))?;
            world_end = Some(WorldEndSource { entry: entry.id.clone(), tick: now });
            out.push(Happening::WorldEnded { tick: now });
        }
        let due = ending.due(before, now, epoch, 0);
        if ending.span_passed(before, now, epoch) { out.push(Happening::SpanPassed { tick: now }); }
        if !ending.wait && now / epoch > before / epoch && ending.extend && !due { ending.extend = false; }
        if due {
            ending.begin(now);
            let turns = boundary::adapt_using(&mut candidate, None, &revise::revisions,
                settings.pace.scoring, &mut score).map_err(Refusal::Sim)?;
            out.push(Happening::Boundary { tick: now, turns });
        }
        if let Some(was) = &held_before {
            out.extend(super::holding::happenings(was, &candidate.sim, &holds));
        }
        (out, flows)
    } else {
        let (out, flows, spent) = crate::profile::measure("round.native_work", || {
            let mut interim = BorrowedInterim::new(&mut candidate, participant, settings.mode,
                settings.pace, settings.on_collapse).map_err(Refusal::Sim)?;
            interim.ending = ending;
            let (out, flows) = interim.round(&revise::revisions, &mut advance, &mut score)
                .map_err(Refusal::Sim)?;
            Ok::<_, Refusal>((out, flows, interim.ending))
        })?;
        ending = spent;
        (out, flows)
    };
    drop(advance);
    drop(score);
    let mut next = driver.into_inner();
    next.refresh_with(&mut candidate, geometry).map_err(Refusal::Sim)?;
    saved.windows.absorb(&candidate.sim, &happenings, &flows);
    // A holder's ended world is watched, as a watcher's is.
    if bodiless && world_end.is_some() { saved.phase = LoopPhase::Watching; }
    if !bodiless {
        let world_ended = happenings.iter().any(|h| matches!(h, Happening::WorldEnded { .. }));
        if world_ended && world_end.is_none() {
            let entry = candidate.entries.last().ok_or_else(|| Refusal::Sim("world end lacks its accepted clock entry".into()))?;
            world_end = Some(WorldEndSource { entry: entry.id.clone(), tick: candidate.sim.state().tick });
        }
        // Continuous: a timed boundary opens the played line's review only after a call.
        let quiet = settings.rounds == Rounds::Continuous && !ending_before.call && !ending_before.forfeit
            && !happenings.iter().any(|h| matches!(h, Happening::ConditionMet { .. }));
        let opens: Vec<_> = happenings.iter().filter(|h|
            !(quiet && matches!(h, Happening::Boundary { .. }))).cloned().collect();
        saved.checkpoint = if world_ended { None } else {
            succession::opened_for(&candidate, participant, &opens)
        };
        saved.ended_lineage = happenings.iter().find_map(|h| match h {
            // A held surface's end is its holder's news, not the played line's.
            Happening::LineageEnded { lineage } if !candidate.sim.is_surface(lineage) => Some(lineage.clone()),
            _ => None,
        });
        saved.phase = if world_ended { LoopPhase::WorldEnded } else if saved.ended_lineage.is_some() {
            LoopPhase::LineageEnded
        } else if saved.checkpoint.is_some() { LoopPhase::Held } else { LoopPhase::Running };
        saved.review = if saved.checkpoint.as_ref().is_some_and(|c|
            matches!(&c.occasion, succession::Occasion::Epoch(_))) {
            let epoch = candidate.sim.genesis().rules.epoch_ticks;
            let draft = scoring::draft(settings.mode, ending_before, before_tick, epoch);
            Some(scoring::review(&candidate, &next, participant,
                settings.pace.scoring, draft).map_err(Refusal::Sim)?)
        } else { None };
        saved.proposed = super::restore::proposals(&saved, &candidate).map_err(Refusal::Sim)?;
    }
    saved.happenings = happenings;
    // `since` only means something while waiting; keeping it zero leaves other saves as they were.
    if !ending.wait { ending.since = 0; }
    saved.ending = ending;
    saved.world_end = world_end;
    saved.round = Some(RoundSource {
        first_entry: candidate.entries.get(start).ok_or_else(||
            Refusal::Sim("loop clock produced no native entry".into()))?.id.clone(),
        from: before_tick, native: before_native,
        played: session.sim.plays(participant), ending: ending_before,
    });
    saved.seal(&candidate, &next);
    next.continuation = Some(saved);
    next.version = next.current_version();
    next.validate(&candidate).map_err(Refusal::Sim)?;
    let verified = VerifiedQuestion::after_validation(&next);
    crate::profile::measure("round.assign", || { *session = candidate; *game = next; });
    Ok((format!("advanced {} loop ticks", settings.pace.round), flows, verified))
}

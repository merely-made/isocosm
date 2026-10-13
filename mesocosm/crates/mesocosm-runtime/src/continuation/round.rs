// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use std::cell::RefCell;
use super::*;
use crate::{Refusal, succession};
use isocosm::{Session, schema::Id, directing::interim::{BorrowedInterim, Happening},
    lineage::{boundary, revise}};

pub fn advance_loop(session: &mut Session, game: &mut Game, participant: Id)
    -> Result<String, Refusal> {
    if game.continuation.as_ref().is_some_and(|saved| !saved.queued.is_empty()) {
        return Err(Refusal::Unbuilt("saved owned queue requires its original owned driver".into()));
    }
    advance_with_flows(session, game, participant).map(|out| out.0)
}

pub(crate) fn advance_with_flows(session: &mut Session, game: &mut Game, participant: Id)
    -> Result<(String, Vec<isocosm::flows::Flow>), Refusal> {
    game.validate(session).map_err(Refusal::Sim)?;
    let mut saved = game.continuation.clone()
        .ok_or_else(|| Refusal::Unbuilt("configure the explicit played loop before stepping".into()))?;
    if saved.participant != participant { return Err(Refusal::NotPlayed); }
    if let Some(fault) = &saved.fault { return Err(Refusal::Sim(fault.clone())); }
    if matches!(saved.phase, LoopPhase::Held | LoopPhase::LineageEnded) {
        return Ok(("held for an exact loop answer".into(), Vec::new()));
    }
    if saved.phase == LoopPhase::NewWorldRequested {
        return Ok(("new world requested; save and use the explicit founding caller".into(), Vec::new()));
    }
    let mut candidate = session.clone();
    let start = session.entries.len();
    let before_native = super::prefix(session);
    let before_tick = session.sim.state().tick;
    let driver = RefCell::new(physical(game));
    let settings = saved.settings;
    let mut advance = |session: &mut Session| driver.borrow_mut().advance_clock(session)
        .map(|out| out.flows);
    let mut score = |session: &Session, lineage: &str, choice: &boundary::Candidate, ticks|
        scoring::score(session, &driver.borrow(), lineage, choice, ticks);
    let (happenings, flows) = if saved.phase == LoopPhase::Watching {
        let before = candidate.sim.state().tick;
        let epoch = candidate.sim.genesis().rules.epoch_ticks;
        if epoch == 0 { return Err(Refusal::Sim("watch epoch length is zero".into())); }
        let mut flows = Vec::new();
        for _ in 0..settings.pace.round {
            flows.extend(advance(&mut candidate).map_err(Refusal::Sim)?);
        }
        let now = candidate.sim.state().tick;
        let mut out = Vec::new();
        if now / epoch > before / epoch {
            let turns = boundary::adapt_using(&mut candidate, None, &revise::revisions,
                settings.pace.scoring, &mut score).map_err(Refusal::Sim)?;
            out.push(Happening::Boundary { tick: now, turns });
        }
        (out, flows)
    } else {
        BorrowedInterim::new(&mut candidate, participant, settings.mode,
            settings.pace, settings.on_collapse).map_err(Refusal::Sim)?
            .round(&revise::revisions, &mut advance, &mut score).map_err(Refusal::Sim)?
    };
    drop(advance);
    drop(score);
    let mut next = driver.into_inner();
    next.refresh(&mut candidate).map_err(Refusal::Sim)?;
    saved.windows.absorb(&candidate.sim, &happenings, &flows);
    if saved.phase != LoopPhase::Watching {
        saved.checkpoint = succession::opened_for(&candidate, participant, &happenings);
        saved.ended_lineage = happenings.iter().find_map(|h| match h {
            Happening::LineageEnded { lineage } => Some(lineage.clone()), _ => None,
        });
        saved.phase = if saved.ended_lineage.is_some() {
            LoopPhase::LineageEnded
        } else if saved.checkpoint.is_some() { LoopPhase::Held } else { LoopPhase::Running };
        saved.review = if saved.checkpoint.as_ref().is_some_and(|c|
            matches!(&c.occasion, succession::Occasion::Epoch(_))) {
            Some(scoring::review(&candidate, &next, participant,
                settings.pace.scoring).map_err(Refusal::Sim)?)
        } else { None };
        saved.proposed = super::restore::proposals(&saved, &candidate).map_err(Refusal::Sim)?;
    }
    saved.happenings = happenings;
    saved.round = Some(RoundSource {
        first_entry: candidate.entries.get(start).ok_or_else(||
            Refusal::Sim("loop clock produced no native entry".into()))?.id.clone(),
        from: before_tick, native: before_native,
        played: session.sim.plays(participant),
    });
    saved.seal(&candidate, &next);
    next.continuation = Some(saved);
    next.version = next.current_version();
    next.validate(&candidate).map_err(Refusal::Sim)?;
    *session = candidate;
    *game = next;
    Ok((format!("advanced {} loop ticks", settings.pace.round), flows))
}

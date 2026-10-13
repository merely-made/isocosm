// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::{Refusal, FlowWindows, succession::Occasion};
use isocosm::{Session, Command, directing::readings::{self, Mode, View}};
use isocosm_overlay::{OrdinaryAccess, Seat, mesocosm::{BirthAnswer, CheckpointAnswerKind}};

fn access(session: &Session, access: OrdinaryAccess) -> Result<(), Refusal> {
    if access.seat != Seat::Player || access.subject.is_none()
        || session.sim.plays(access.participant.0) != access.subject.map(|id| id.0) {
        return Err(Refusal::NotPlayed);
    }
    Ok(())
}

fn source(session: &Session, game: &Game, access: OrdinaryAccess, input: &LoopInput)
    -> Result<(), Refusal> {
    self::access(session, access)?;
    game.validate(session).map_err(Refusal::Sim)?;
    if input.kind != KIND || input.version != VERSION || input.participant != access.participant
        || input.tick.0 != session.sim.state().tick || input.expected != cut(session, game) {
        return Err(Refusal::Sim("loop input names another native/product/question cut".into()));
    }
    Ok(())
}

pub fn configure_loop(session: &mut Session, game: &mut Game,
    access: OrdinaryAccess, input: &LoopInput) -> Result<String, Refusal> {
    source(session, game, access, input)?;
    let LoopAction::Configure(settings) = &input.action else { return Err(Refusal::Unasked); };
    if game.continuation.is_some() {
        return Err(Refusal::Sim("saved loop already configured; a hold cannot be overwritten".into()));
    }
    if settings.pace.round == 0 || settings.pace.scoring == 0 {
        return Err(Refusal::Sim("supply positive round and scoring ticks".into()));
    }
    let played = access.subject.ok_or(Refusal::NotPlayed)?;
    if !session.sim.state().population.get(played.0).is_some_and(|e| e.alive) {
        return Err(Refusal::NotPlayed);
    }
    let mut saved = Continuation { version: VERSION, participant: access.participant.0,
        settings: *settings, native: String::new(), native_entries: 0, native_tick: 0, physical: String::new(), phase: LoopPhase::Running,
        checkpoint: None, review: None, proposed: Vec::new(), happenings: Vec::new(),
        windows: FlowWindows::new(), expression: None, ended_lineage: None, world_end: None,
        queued: Vec::new(), round: None, origin: None, fault: None };
    saved.seal(session, game);
    game.continuation = Some(saved);
    game.version = game.current_version();
    Ok("played loop configured".into())
}

pub(crate) fn visible(session: &Session, game: &Game, access: OrdinaryAccess) -> Result<View, Refusal> {
    let observer = access.subject.ok_or(Refusal::NotPlayed)?.0;
    // These global lifecycle controls reveal no bodies or inputs and need no
    // fabricated eye when an otherwise living observer's binding is unavailable.
    if game.continuation.as_ref().is_some_and(|saved| saved.world_end.is_some()) {
        return Ok(View::default());
    }
    let native = readings::view(&session.sim, observer, Mode::Survival);
    if !game.has_sight_source()
        || !session.sim.state().population.get(observer).is_some_and(|e| e.alive) {
        return Ok(native);
    }
    match game.sight(&session.sim, observer, &native).map_err(Refusal::Sim)? {
        Some((site, sight)) => site.refine(&session.sim, Mode::Survival, &sight).map_err(Refusal::Sim),
        None => Err(Refusal::Sim("marked answer observer has no actual sight source".into())),
    }
}

pub fn answer_loop(session: &mut Session, game: &mut Game,
    access: OrdinaryAccess, input: &LoopInput) -> Result<String, Refusal> {
    source(session, game, access, input)?;
    let LoopAction::Answer(answer) = &input.action else { return Err(Refusal::Unasked); };
    let view = visible(session, game, access)?;
    let offered = read_loop(session, game, access, &view).map_err(Refusal::Sim)?;
    let mut saved = game.continuation.clone().ok_or(Refusal::Unasked)?;
    if let Some(fault) = &saved.fault { return Err(Refusal::Sim(fault.clone())); }
    super::restore::validate_round(&saved, session).map_err(Refusal::Sim)?;
    if super::restore::proposals(&saved, session).map_err(Refusal::Sim)? != saved.proposed {
        return Err(Refusal::Sim("saved expression differs from its actual authored proposal".into()));
    }
    let mut candidate = session.clone();
    match answer {
        LoopAnswer::Checkpoint(answer) => {
            let personal_parent = matches!((&offered.personal, answer.kind),
                (Some(PersonalQuestion::Birth { parent }), CheckpointAnswerKind::Birth(BirthAnswer::KeepParent))
                    if *parent == answer.entity.0);
            let checkpoint = offered.checkpoint.as_ref().or_else(||
                personal_parent.then_some(saved.checkpoint.as_ref()).flatten()).ok_or(Refusal::Unasked)?;
            if saved.phase != LoopPhase::Held { return Err(Refusal::Unasked); }
            let mut taken = None;
            match (&checkpoint.occasion, answer.kind) {
                (Occasion::Birth(birth), CheckpointAnswerKind::Birth(kind))
                    if answer.entity.0 == birth.parent => {
                    if kind == BirthAnswer::TakeOffspring { taken = Some(birth.child); }
                },
                (Occasion::Loss(loss), CheckpointAnswerKind::Death(death))
                    if answer.entity.0 == loss.critter && checkpoint.heirs.contains(&death.next.0) =>
                    taken = Some(death.next.0),
                (Occasion::Epoch(_), CheckpointAnswerKind::EpochReview(revision))
                    if Some(answer.entity.0) == candidate.sim.plays(saved.participant) => {
                    let index = usize::try_from(revision.revision.0).map_err(|_| Refusal::Unasked)?;
                    let review = saved.review.as_ref().ok_or(Refusal::Unasked)?;
                    let offer = review.offers.get(index).ok_or(Refusal::Unasked)?;
                    if let Some(why) = &offer.why_not { return Err(Refusal::Sim(why.clone())); }
                    for command in &offer.commands { candidate.command(command.clone()).map_err(Refusal::Sim)?; }
                    if let Some(proposal) = saved.proposed.iter().find(|p| p.offer == index && p.cells.is_ok()) {
                        for command in &proposal.commands { candidate.command(command.clone()).map_err(Refusal::Sim)?; }
                    }
                },
                _ => return Err(Refusal::Unasked),
            }
            if let Some(critter) = taken {
                candidate.command(Command::Take { participant: saved.participant, critter }).map_err(Refusal::Sim)?;
            }
            saved.phase = LoopPhase::Running;
        },
        LoopAnswer::OtherLineage(critter) => {
            if !matches!(saved.phase, LoopPhase::LineageEnded | LoopPhase::Watching)
                || saved.world_end.is_some() || !offered.lineages.contains(critter) { return Err(Refusal::Unasked); }
            candidate.command(Command::Take { participant: saved.participant, critter: critter.0 }).map_err(Refusal::Sim)?;
            saved.ended_lineage = None;
            saved.phase = LoopPhase::Running;
        },
        LoopAnswer::Watch if matches!(saved.phase, LoopPhase::LineageEnded | LoopPhase::NewWorldRequested | LoopPhase::WorldEnded) => saved.phase = LoopPhase::Watching,
        LoopAnswer::NewWorld if matches!(saved.phase, LoopPhase::LineageEnded | LoopPhase::Watching | LoopPhase::WorldEnded) =>
            saved.phase = LoopPhase::NewWorldRequested,
        _ => return Err(Refusal::Unasked),
    }
    saved.checkpoint = None; saved.review = None; saved.proposed.clear();
    let mut next = physical(game);
    next.refresh(&mut candidate).map_err(Refusal::Sim)?;
    saved.seal(&candidate, &next);
    next.continuation = Some(saved); next.version = next.current_version();
    next.validate(&candidate).map_err(Refusal::Sim)?;
    *session = candidate; *game = next;
    Ok("answered exact saved loop question".into())
}

pub fn submit_loop(session: &mut Session, game: &mut Game,
    access: OrdinaryAccess, input: &LoopInput) -> Result<String, Refusal> {
    source(session, game, access, input)?;
    match &input.action {
        LoopAction::Configure(_) => configure_loop(session, game, access, input),
        LoopAction::Step => advance_loop(session, game, access.participant.0),
        LoopAction::Answer(_) => answer_loop(session, game, access, input),
    }
}

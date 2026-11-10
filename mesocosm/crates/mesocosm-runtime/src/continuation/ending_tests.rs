// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A forfeit end to end (case 1212). A lineage can end here only by a dev
//! kill between rounds, which the historical replay of a held loss rightly
//! refuses, so the answer is taken on the process-local receipt the live
//! driver holds for a question it has just verified.

use super::*;
use cosmograph::{digest, directing::readings::Mode};

use cosmograph::{Command, Execution, Founding, directing::{Played, interim::{Interim, Happening, OnCollapse, Pace, Start}}};
use cosmograph_overlay::{EntityHandle, OrdinaryAccess, ParticipantHandle, Seat, Tick};
use crate::{Refusal, spatial::Input};

const EPOCH: u64 = 8;
const PACE: Pace = Pace { round: 1, scoring: 1 };

#[test]
fn a_forfeit_moves_a_lost_lineage_to_watching_and_the_next_step_ends_the_epoch() {
    let mut genesis = Founding { seed: 5, sites: 4, population: 48, cohort_size: 4, lineages: 3, ecology: true,
        played: Some(Played { lineage: 1, region_sites: 2 }), ..Default::default() }.generate().unwrap();
    genesis.rules.epoch_ticks = EPOCH;
    let i = Interim::found(genesis, Start::default(), Mode::Survival, PACE, Execution::Grouped).unwrap();
    let (mut session, participant, mut game) = (i.session, i.participant, Game::default());
    let access = |s: &Session| OrdinaryAccess { participant: ParticipantHandle(participant), seat: Seat::Player,
        subject: s.sim.plays(participant).map(EntityHandle) };
    let input = |s: &Session, g: &Game, action| LoopInput { kind: KIND.into(), version: VERSION,
        participant: ParticipantHandle(participant), tick: Tick(s.sim.state().tick), expected: cut(s, g), action };
    let send = |s: &mut Session, g: &mut Game, action| {
        let (_, next) = crate::borrowed::submit_game(s, access(s), 0, &Input::Loop(input(s, g, action)), g.clone())?;
        *g = next;
        Ok::<_, Refusal>(())
    };
    let settings = LoopSettings { mode: Mode::Survival, pace: PACE, on_collapse: OnCollapse::Stay,
        rounds: Rounds::Capped, condition: false };
    send(&mut session, &mut game, LoopAction::Configure(settings)).unwrap();
    while session.sim.state().tick % EPOCH != 2 { send(&mut session, &mut game, LoopAction::Step).unwrap(); }
    assert!(matches!(send(&mut session, &mut game, LoopAction::Answer(LoopAnswer::Forfeit)), Err(Refusal::Unasked)),
        "a player with a living member cannot forfeit");

    // The lineage ends: every member dies (a group a member at a time), and the loop is resealed to that cut.
    let played = session.sim.plays(participant).unwrap();
    let lineage = session.sim.state().population.get(played).unwrap().lineage.clone();
    while let Some(entity) = session.sim.state().population.groups.iter()
        .find(|(_, g)| g.entity.alive && g.entity.lineage == lineage).map(|(id, _)| *id) {
        session.command(Command::Kill { entity }).unwrap();
    }
    let saved = game.continuation.as_mut().unwrap();
    saved.native = digest(&(&session.branch, session.sim.genesis(), &session.entries, session.sim.state().tick));
    saved.native_entries = session.entries.len() as u64;
    saved.native_tick = session.sim.state().tick;
    send(&mut session, &mut game, LoopAction::Step).unwrap();
    assert_eq!(game.continuation.as_ref().unwrap().phase, LoopPhase::LineageEnded);
    assert!(!session.sim.global_collapse().unwrap().is_some_and(|reading| reading.collapsed));

    let before = session.sim.state().tick;
    let proof = VerifiedQuestion::after_validation(&game).unwrap();
    let forfeit = input(&session, &game, LoopAction::Answer(LoopAnswer::Forfeit));
    let seat = access(&session);
    admission::answer_verified(&mut session, &mut game, seat, &forfeit, Some(&proof)).unwrap();
    let saved = game.continuation.as_ref().unwrap();
    assert_eq!((saved.phase, saved.ending.forfeit), (LoopPhase::Watching, true));
    assert!(matches!(send(&mut session, &mut game, LoopAction::Answer(LoopAnswer::Forfeit)), Err(Refusal::Unasked)));
    send(&mut session, &mut game, LoopAction::Step).unwrap();
    let saved = game.continuation.as_ref().unwrap();
    assert!(saved.happenings.iter().any(|h| matches!(h, Happening::Boundary { .. })),
        "the boundary fires whatever the span");
    assert_eq!(session.sim.state().tick / EPOCH, before / EPOCH, "without crossing an epoch");
    assert!(!saved.ending.forfeit, "and the next epoch begins");
}

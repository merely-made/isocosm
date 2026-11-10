// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A holder whose lineage has ended (cases 1216, 1217, 1224, 1226): it holds
//! the microbe, watches, and is admitted with a dead body or none. A lineage
//! can end here only by a dev kill, so the held question is answered on the
//! receipt for it, as the forfeit's test does.

use super::*;
use super::ending_tests::{lost_in, take};
use crate::{Refusal, microbe, spatial::Input};
use cosmograph::{Command, Session, digest, directing::interim::Happening};
use cosmograph_overlay::{EntityHandle, OrdinaryAccess, ParticipantHandle, Seat, Tick};

type World = (Session, u64, Game, VerifiedQuestion);

/// A lost world with the microbe declared and seeded.
fn lost_micro() -> World {
    (1..60).find_map(|seed| lost_in(seed, |g| microbe::declare(g).unwrap(), |s| {
        let site = *s.sim.state().sites.iter().find(|(_, site)| site.accounts.contains_key(microbe::ACCOUNT)).unwrap().0;
        for command in microbe::seeding(site) { s.command(command).unwrap(); }
    })).expect("a seed that loses its line in a hosting world")
}

fn send(session: &mut Session, p: u64, game: &mut Game, action: LoopAction) -> Result<String, Refusal> {
    let access = OrdinaryAccess { participant: ParticipantHandle(p), seat: Seat::Player,
        subject: session.sim.plays(p).map(EntityHandle) };
    let input = LoopInput { kind: KIND.into(), version: VERSION, participant: ParticipantHandle(p),
        tick: Tick(session.sim.state().tick), expected: cut(session, game), action };
    let (out, next) = crate::borrowed::submit_game(session, access, 0, &Input::Loop(input), game.clone())?;
    *game = next;
    Ok(out)
}

fn micro() -> String { microbe::KEY.into() }

/// Holds the microbe on the lost question's receipt and takes up watching.
fn watching() -> (Session, u64, Game) {
    let (mut session, p, mut game, proof) = lost_micro();
    take(&mut session, p, &mut game, &proof, LoopAnswer::Hold { lineage: micro() }).unwrap();
    let saved = game.continuation.as_ref().unwrap();
    assert_eq!(saved.phase, LoopPhase::LineageEnded, "holding leaves the question as it stood");
    assert!(saved.checkpoint.is_some());
    let proof = VerifiedQuestion::after_validation(&game).unwrap();
    take(&mut session, p, &mut game, &proof, LoopAnswer::Watch).unwrap();
    (session, p, game)
}

/// Puts the dead body down and reseals the loop to that cut.
fn leave(session: &mut Session, p: u64, game: &mut Game) {
    let body = session.sim.plays(p).unwrap();
    session.command(Command::Leave { participant: p, critter: body }).unwrap();
    let saved = game.continuation.as_mut().unwrap();
    saved.native = digest(&(&session.branch, session.sim.genesis(), &session.entries, session.sim.state().tick));
    saved.native_entries = session.entries.len() as u64;
    saved.native_tick = session.sim.state().tick;
}

#[test]
fn a_holder_with_a_dead_body_watches_and_hears_what_befalls_its_microbe() {
    let (mut session, p, mut game) = watching();
    let tick = session.sim.state().tick;
    let mut surfaced = 0;
    for _ in 0..40 {
        send(&mut session, p, &mut game, LoopAction::Step).unwrap();
        surfaced += game.continuation.as_ref().unwrap().happenings.iter()
            .filter(|h| matches!(h, Happening::Surfaced(_))).count();
    }
    assert!(session.sim.state().tick > tick, "the rounds advance the clock");
    assert!(surfaced > 0, "the microbe reached a new host or site");
    let access = OrdinaryAccess { participant: ParticipantHandle(p), seat: Seat::Player,
        subject: session.sim.plays(p).map(EntityHandle) };
    let view = cosmograph::directing::readings::View::default();
    let read = read_loop(&session, &game, access, &view).unwrap();
    assert!(matches!(read.held.as_slice(), [HeldLine::Surface(line)] if line.lineage == microbe::KEY));
    // A forfeit ends the round as a watcher's does: the boundary fires at once.
    send(&mut session, p, &mut game, LoopAction::Answer(LoopAnswer::Forfeit)).unwrap();
    send(&mut session, p, &mut game, LoopAction::Step).unwrap();
    assert!(game.continuation.as_ref().unwrap().happenings.iter().any(|h| matches!(h, Happening::Boundary { .. })));
}

#[test]
fn a_holder_with_no_body_is_admitted_and_one_that_holds_nothing_is_not() {
    let (mut session, p, mut game) = watching();
    leave(&mut session, p, &mut game);
    assert_eq!(session.sim.plays(p), None);
    send(&mut session, p, &mut game, LoopAction::Step).unwrap();
    // A holder is still refused what needs a body.
    for answer in [LoopAnswer::OtherLineage(EntityHandle(1)), LoopAnswer::Start { lineage: "lineage:2".into(), site: 0 }] {
        assert!(matches!(send(&mut session, p, &mut game, LoopAction::Answer(answer)), Err(Refusal::NotPlayed)));
    }
    send(&mut session, p, &mut game, LoopAction::Answer(LoopAnswer::Unhold { lineage: micro() })).unwrap();
    // Nothing held and nobody played: refused, even for the step.
    assert!(matches!(send(&mut session, p, &mut game, LoopAction::Step), Err(Refusal::NotPlayed)));
    assert!(matches!(send(&mut session, p, &mut game, LoopAction::Answer(LoopAnswer::Call)), Err(Refusal::NotPlayed)));
}

#[test]
fn a_reload_of_a_holder_with_no_body_continues_to_the_same_hash() {
    let (mut session, p, mut game) = watching();
    leave(&mut session, p, &mut game);
    for _ in 0..3 { send(&mut session, p, &mut game, LoopAction::Step).unwrap(); }
    let mut loaded = Session::load(session.save(), cosmograph::Execution::Grouped).unwrap();
    let mut kept: Game = serde_json::from_slice(&serde_json::to_vec(&game).unwrap()).unwrap();
    kept.validate_replay(&loaded).expect("the saved holder restores");
    for _ in 0..10 {
        send(&mut session, p, &mut game, LoopAction::Step).unwrap();
        send(&mut loaded, p, &mut kept, LoopAction::Step).unwrap();
    }
    assert_eq!(session.sim.state_hash(), loaded.sim.state_hash());
    let replayed = Session::load(session.save(), cosmograph::Execution::Grouped).unwrap();
    assert_eq!(replayed.sim.state_hash(), session.sim.state_hash(), "the session log replays to the same hash");
}

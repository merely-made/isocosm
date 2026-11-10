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

/// A world whose played lineage has just ended by a dev kill, the loop resealed
/// and stepped to `LineageEnded`; the receipt for that verified question.
fn lost(seed: u64) -> Option<(Session, u64, Game, VerifiedQuestion)> {
    let mut genesis = Founding { seed, sites: 4, population: 48, cohort_size: 4, lineages: 3, ecology: true,
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
    let played = session.sim.plays(participant).unwrap();
    let lineage = session.sim.state().population.get(played).unwrap().lineage.clone();
    // Only the played lineage is deliberative here, so a body beside the
    // player's splits off to found a line of its own, which outlives it.
    let place = session.sim.state().population.get(played).unwrap().place;
    let kin = session.sim.state().population.groups.iter()
        .find(|(first, g)| g.entity.alive && g.entity.lineage == lineage && g.entity.place == place
            && g.entity.journey.is_none() && **first != played)
        .map(|(first, _)| *first)?;
    session.command(Command::Speciate { founder: kin, name: "split".into() }).unwrap();
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
    let proof = VerifiedQuestion::after_validation(&game).unwrap();
    Some((session, participant, game, proof))
}

fn reading(session: &Session, participant: u64, game: &Game) -> LoopReading {
    let access = OrdinaryAccess { participant: ParticipantHandle(participant), seat: Seat::Player,
        subject: session.sim.plays(participant).map(EntityHandle) };
    let view = cosmograph::directing::readings::view(&session.sim, access.subject.unwrap().0, Mode::Survival);
    read_loop(session, game, access, &view).unwrap()
}

/// The first world, by seed, in which the player who lost can see another
/// lineage's body (they see only where their body fell) and `wanted` holds
/// of the starts that sight offers.
fn lost_where(wanted: impl Fn(&[LoopStart]) -> bool) -> Option<(Session, u64, Game, VerifiedQuestion)> {
    (1..60).filter_map(lost).find(|(s, p, g, _)| wanted(&reading(s, *p, g).starts))
}

/// Answers `answer` on the receipt for the held question.
fn take(session: &mut Session, participant: u64, game: &mut Game, proof: &VerifiedQuestion, answer: LoopAnswer)
    -> Result<String, Refusal> {
    let access = OrdinaryAccess { participant: ParticipantHandle(participant), seat: Seat::Player,
        subject: session.sim.plays(participant).map(EntityHandle) };
    let input = LoopInput { kind: KIND.into(), version: VERSION, participant: ParticipantHandle(participant),
        tick: Tick(session.sim.state().tick), expected: cut(session, game), action: LoopAction::Answer(answer) };
    admission::answer_verified(session, game, access, &input, Some(proof))
}

#[test]
fn a_lost_lineage_is_offered_the_starts_of_the_others_and_a_default_reading_has_none() {
    let (session, participant, game, _) = lost_where(|starts| !starts.is_empty()).expect("a seed whose loser sees another lineage");
    let read = reading(&session, participant, &game);
    assert!(!read.lineages.is_empty() && !read.starts.is_empty(), "another lineage is offered starts");
    let ended = game.continuation.as_ref().unwrap().ended_lineage.clone().unwrap();
    assert!(read.starts.iter().all(|s| s.lineage != ended), "never the lineage that ended");
    let order: Vec<_> = read.starts.iter().map(|s| s.lineage.clone()).collect();
    assert!(order.windows(2).all(|w| w[0] <= w[1]), "in lineage-key order");
    let wire = serde_json::to_string(&read).unwrap();
    assert!(wire.contains("\"starts\""));
    assert_eq!(serde_json::from_str::<LoopReading>(&wire).unwrap(), read);
    let quiet = LoopReading { starts: Vec::new(), ..read };
    assert!(!serde_json::to_string(&quiet).unwrap().contains("\"starts\""), "no key when empty");
}

/// Starts by `journey`, in a world whose loser is offered both kinds.
fn both_kinds() -> (Session, u64, Game, VerifiedQuestion) {
    lost_where(|starts| starts.iter().any(|s| s.journey) && starts.iter().any(|s| !s.journey))
        .expect("a seed offering a range and a fringe start")
}

fn commands(session: &Session, since: usize) -> Vec<Command> {
    session.entries[since..].iter().map(|e| e.command.clone()).collect()
}

#[test]
fn a_range_start_takes_the_body_at_its_site_and_resumes_the_round() {
    let (mut session, participant, mut game, proof) = both_kinds();
    let at = reading(&session, participant, &game).starts.into_iter().find(|s| !s.journey).unwrap();
    assert_eq!(session.sim.state().population.get(at.critter.0).unwrap().place, at.site);
    let since = session.entries.len();
    take(&mut session, participant, &mut game, &proof,
        LoopAnswer::Start { lineage: at.lineage.clone(), site: at.site }).unwrap();
    assert_eq!(session.sim.plays(participant), Some(at.critter.0));
    assert!(!commands(&session, since).iter().any(|c| matches!(c, Command::Nudge { .. })), "no journey");
    let saved = game.continuation.as_ref().unwrap();
    assert_eq!((saved.phase, saved.ended_lineage.clone()), (LoopPhase::Running, None));
}

#[test]
fn a_fringe_start_takes_the_nearest_body_and_nudges_it_toward_the_site() {
    use cosmograph::directing::{Aim, Toward};
    let (mut session, participant, mut game, proof) = both_kinds();
    let at = reading(&session, participant, &game).starts.into_iter().find(|s| s.journey).unwrap();
    let since = session.entries.len();
    take(&mut session, participant, &mut game, &proof,
        LoopAnswer::Start { lineage: at.lineage.clone(), site: at.site }).unwrap();
    assert_eq!(session.sim.plays(participant), Some(at.critter.0));
    let sent = commands(&session, since);
    assert!(matches!(sent.as_slice(), [Command::Take { critter: a, .. }, Command::Nudge { critter: b, aim: Aim::Attend,
        toward: Toward::Site(site), act: None, .. }] if *a == at.critter.0 && *b == at.critter.0 && *site == at.site),
        "{sent:?}");
    assert_eq!(game.continuation.as_ref().unwrap().phase, LoopPhase::Running);
}

#[test]
fn a_start_that_was_not_offered_is_refused() {
    let (mut session, participant, mut game, proof) = both_kinds();
    let read = reading(&session, participant, &game);
    let offered = read.starts[0].clone();
    let free = (0..64).find(|site| read.starts.iter().all(|s| s.site != *site)).unwrap();
    let before = session.sim.state_hash();
    for (lineage, site) in [(offered.lineage.clone(), free), ("lineage:none".into(), offered.site)] {
        let refused = take(&mut session, participant, &mut game, &proof, LoopAnswer::Start { lineage, site });
        assert!(matches!(refused, Err(Refusal::Unasked)), "{refused:?}");
    }
    assert_eq!((session.sim.state_hash(), game.continuation.as_ref().unwrap().phase), (before, LoopPhase::LineageEnded));
}

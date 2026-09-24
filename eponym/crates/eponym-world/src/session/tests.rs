// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::{ItemLocation, Name, WorldConfig};

const FIRST: SubjectId = SubjectId(10);
const SECOND: SubjectId = SubjectId(11);
const THIRD: SubjectId = SubjectId(12);

fn game() -> GameState {
    let mut game = GameState::new(World::generate(7, WorldConfig::default()).unwrap());
    let at = game
        .items()
        .all()
        .find_map(|item| match item.location {
            ItemLocation::At(at) => Some(at),
            _ => None,
        })
        .unwrap();
    for (subject, seed, name) in [
        (FIRST, 1, "First"),
        (SECOND, 2, "Second"),
        (THIRD, 3, "Third"),
    ] {
        let tick = game.next_tick();
        game.apply(GameIntent::Generate {
            tick,
            subject,
            body_seed: seed,
            at,
        })
        .unwrap();
        let tick = game.next_tick();
        game.apply(GameIntent::Name {
            tick,
            subject,
            name: Name::new(name).unwrap(),
        })
        .unwrap();
    }
    game
}

fn kill(session: &mut Session) {
    let tick = session.game().next_tick();
    session
        .apply_game(GameIntent::Fall {
            tick,
            subject: session.control().played(),
            distance: 20,
        })
        .unwrap();
}

fn rechecksum(mut save: SessionSave) -> SessionSave {
    let world = World::restore_record(save.game.world.clone()).unwrap();
    let mut game = GameState::new(world);
    for intent in &save.game.intents {
        game.apply(intent.clone()).unwrap();
    }
    save.game.expected_hash = game.state_hash().unwrap();
    save.expected_hash = snapshot::encode(&(save.game.expected_hash, save.control.as_slice()))
        .map(|bytes| hash_bytes(&bytes))
        .unwrap();
    save
}

#[test]
fn control_gates_actions_without_copying_other_lives() {
    let game = game();
    let before = game.bodies().get(SECOND).unwrap().clone();
    let mut session = Session::begin(game, FIRST).unwrap();
    let tick = session.game().next_tick();
    assert_eq!(
        session.apply_game(GameIntent::Wait {
            tick,
            subject: SECOND
        }),
        Err(SessionError::NotControlled {
            expected: FIRST,
            actual: SECOND
        })
    );
    assert_eq!(session.game().bodies().get(SECOND).unwrap(), &before);
}

#[test]
fn begin_rejects_missing_unnamed_and_dead_subjects() {
    let state = game();
    assert_eq!(
        Session::begin(state.clone(), SubjectId(99)),
        Err(SessionError::Ineligible {
            subject: SubjectId(99)
        })
    );
    let at = state.movement().position(FIRST).unwrap();
    let mut unnamed = state.clone();
    unnamed
        .apply(GameIntent::Generate {
            tick: unnamed.next_tick(),
            subject: SubjectId(99),
            body_seed: 99,
            at,
        })
        .unwrap();
    assert!(matches!(
        Session::begin(unnamed, SubjectId(99)),
        Err(SessionError::Ineligible { .. })
    ));
    let mut dead = state;
    dead.apply(GameIntent::Fall {
        tick: dead.next_tick(),
        subject: FIRST,
        distance: 20,
    })
    .unwrap();
    assert!(matches!(
        Session::begin(dead, FIRST),
        Err(SessionError::Ineligible { .. })
    ));
}

#[test]
fn death_succeeds_to_an_existing_life_and_replays() {
    let mut session = Session::begin(game(), FIRST).unwrap();
    let at = session.game().movement().position(FIRST).unwrap();
    let item = session.game().items().at(at).next().unwrap().id;
    let tick = session.game().next_tick();
    session
        .apply_game(GameIntent::Take {
            tick,
            subject: FIRST,
            item,
        })
        .unwrap();
    kill(&mut session);
    session.succeed_existing(SECOND).unwrap();
    assert_eq!(session.control().played(), SECOND);
    assert!(!session.game().bodies().get(FIRST).unwrap().alive());
    assert_eq!(
        session.game().items().get(item).unwrap().location,
        ItemLocation::Carried(FIRST)
    );
    let restored = Session::restore(&session.save().unwrap()).unwrap();
    assert_eq!(restored, session);
    assert_eq!(
        restored.game().items().get(item).unwrap().location,
        ItemLocation::Carried(FIRST)
    );
}

#[test]
fn historical_cuts_allow_a_later_dead_successor() {
    let mut session = Session::begin(game(), FIRST).unwrap();
    kill(&mut session);
    session.succeed_existing(SECOND).unwrap();
    kill(&mut session);
    session.succeed_existing(THIRD).unwrap();
    assert_eq!(session.control().played(), THIRD);
    assert_eq!(Session::restore(&session.save().unwrap()).unwrap(), session);
}

#[test]
fn malformed_control_history_is_rejected_even_with_a_recomputed_hash() {
    let session = Session::begin(game(), FIRST).unwrap();
    let mut save = session.save_record().unwrap();
    save.control.push(ControlIntent::Succeed {
        to: SECOND,
        at: Tick(6),
    });
    let save = rechecksum(save);
    assert!(matches!(
        Session::restore_record(save),
        Err(SessionError::HomeStillAlive(FIRST))
    ));
}

#[test]
fn forged_post_begin_foreign_action_is_rejected_with_a_valid_checksum() {
    let session = Session::begin(game(), FIRST).unwrap();
    let mut save = session.save_record().unwrap();
    save.game.intents.push(GameIntent::Wait {
        tick: Tick(save.game.intents.len() as u64),
        subject: SECOND,
    });
    let save = rechecksum(save);
    assert!(matches!(
        Session::restore_record(save),
        Err(SessionError::NotControlled {
            expected: FIRST,
            actual: SECOND
        })
    ));
}

#[test]
fn corrupt_or_version_diverged_candidates_leave_a_live_session_unchanged() {
    let session = Session::begin(game(), FIRST).unwrap();
    let before = session.clone();
    assert_eq!(Session::restore(&[0, 1, 2]), Err(SessionError::Decode));
    let mut save = session.save_record().unwrap();
    save.version = 9;
    assert!(matches!(
        Session::restore_record(save),
        Err(SessionError::VersionDiverged { .. })
    ));
    assert_eq!(session, before);
}

#[test]
fn dead_controlled_subject_can_be_saved_while_awaiting_successor() {
    let mut session = Session::begin(game(), FIRST).unwrap();
    kill(&mut session);
    let restored = Session::restore(&session.save().unwrap()).unwrap();
    assert_eq!(restored, session);
    assert!(!restored.game().bodies().get(FIRST).unwrap().alive());
    assert_eq!(restored.control().played(), FIRST);
}

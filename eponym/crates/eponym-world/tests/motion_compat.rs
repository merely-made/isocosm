// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use mesocosm_core::snapshot;
use eponym_identity::{BodyRevisionId, SubjectId, Tick};
use eponym_world::timed_action::{TimedActionSave, TimedActionSession};
use eponym_world::{
    GAME_STATE_VERSION, GameError, GameIntent, GameState, ItemLocation, MotionInput, MotionRules,
    Name, Session, SessionError, World, WorldConfig,
};

const SUBJECT: SubjectId = SubjectId(91);

fn motion(tick: Tick) -> GameIntent {
    GameIntent::AdvanceMotion {
        tick,
        subject: SUBJECT,
        revision: BodyRevisionId(0),
        step: 1,
        input: MotionInput::default(),
        rules: MotionRules::default(),
    }
}

fn session() -> Session {
    let mut game = GameState::new(World::generate(17, WorldConfig::default()).unwrap());
    let at = game
        .items()
        .all()
        .find_map(|item| matches!(item.location, ItemLocation::At(_)).then_some(item.location))
        .and_then(|location| match location {
            ItemLocation::At(at) => Some(at),
            _ => None,
        })
        .unwrap();
    game.apply(GameIntent::Generate {
        tick: game.next_tick(),
        subject: SUBJECT,
        body_seed: 1,
        at,
    })
    .unwrap();
    game.apply(GameIntent::Name {
        tick: game.next_tick(),
        subject: SUBJECT,
        name: Name::new("Motion archive").unwrap(),
    })
    .unwrap();
    Session::begin(game, SUBJECT).unwrap()
}

#[test]
fn v4_native_timed_action_archive_restores_and_rewrites_as_v5() {
    let bytes = include_bytes!("fixtures/timed-action-v1-game-v4.save");
    let saved: TimedActionSave = snapshot::decode(bytes).unwrap();
    assert_eq!(saved.session.game.version, 4);
    let restored = TimedActionSession::restore(bytes).unwrap();
    let rewritten: TimedActionSave = snapshot::decode(&restored.save().unwrap()).unwrap();
    assert_eq!(rewritten.session.game.version, GAME_STATE_VERSION);
    assert_eq!(
        TimedActionSession::restore(&restored.save().unwrap()).unwrap(),
        restored
    );
}

#[test]
fn v3_and_v4_archives_cannot_disguise_continuous_motion() {
    for version in [3, 4] {
        let game = GameState::new(World::generate(19, WorldConfig::default()).unwrap());
        let mut game_save = game.save_record().unwrap();
        game_save.version = version;
        game_save.intents.push(motion(Tick(0)));
        assert_eq!(
            GameState::restore_record(game_save),
            Err(GameError::LegacyMotionIntent)
        );

        let session = session();
        let mut session_save = session.save_record().unwrap();
        session_save.game.version = version;
        session_save
            .game
            .intents
            .push(motion(Tick(session_save.game.intents.len() as u64)));
        assert_eq!(
            Session::restore_record(session_save),
            Err(SessionError::Game(GameError::LegacyMotionIntent))
        );
    }
}

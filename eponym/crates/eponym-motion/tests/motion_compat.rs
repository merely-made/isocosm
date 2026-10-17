// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use eponym_play::identity::{BodyRevisionId, SubjectId, Tick};
use eponym_play::{
    GameError, GameIntent, GameState, ItemLocation, MotionInput, MotionRules,
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
    let mut game = GameState::new(World::generate(17, WorldConfig::default()).unwrap())
        .with_motion_solver(eponym_motion::SOLVER);
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
fn v3_and_v4_archives_cannot_disguise_continuous_motion() {
    for version in [3, 4] {
        let game = GameState::new(World::generate(19, WorldConfig::default()).unwrap())
            .with_motion_solver(eponym_motion::SOLVER);
        let mut game_save = game.save_record().unwrap();
        game_save.version = version;
        game_save.intents.push(motion(Tick(0)));
        assert_eq!(
            GameState::restore_record_solving(game_save, eponym_motion::SOLVER),
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
            Session::restore_record_solving(session_save, eponym_motion::SOLVER),
            Err(SessionError::Game(GameError::LegacyMotionIntent))
        );
    }
}

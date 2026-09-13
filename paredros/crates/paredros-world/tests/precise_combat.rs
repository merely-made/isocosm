// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use mesocosm_core::{BodyDocument, SpeciesId, VolumeRef};
use paredros_identity::{BodyRevisionId, SubjectId};
use paredros_world::timed_action::{Direction, LimbBinding, StrikeReceipt};
use paredros_world::*;

const ACTOR: SubjectId = SubjectId(31);
const TARGET: SubjectId = SubjectId(32);

fn setup() -> GameState {
    let mut game = GameState::new(World::generate(7, WorldConfig::default()).unwrap());
    let at = game
        .items()
        .all()
        .find_map(|item| match item.location {
            ItemLocation::At(at) if item.kind == ItemKind::Dressing => Some(at),
            _ => None,
        })
        .unwrap();
    for (subject, dimensions) in [(ACTOR, [1; 3]), (TARGET, [1, 3, 1])] {
        game.apply(GameIntent::Generate {
            tick: game.next_tick(),
            subject,
            body_seed: 1,
            at,
        })
        .unwrap();
        game.apply(GameIntent::Name {
            tick: game.next_tick(),
            subject,
            name: Name::new("Body").unwrap(),
        })
        .unwrap();
        game.apply(GameIntent::AdmitAnatomy {
            tick: game.next_tick(),
            subject,
            revision: BodyRevisionId(0),
            document: Box::new(BodyDocument::new(
                SpeciesId(1),
                VolumeRef::from_tag(1),
                100,
                dimensions,
            )),
        })
        .unwrap();
    }
    for _ in 0..5 {
        game.apply(GameIntent::Move {
            tick: game.next_tick(),
            subject: TARGET,
            toward: [at[0] + 5, at[1], at[2]],
        })
        .unwrap();
    }
    assert_eq!(game.movement().position(TARGET).unwrap()[0], at[0] + 5);
    game
}
fn fire(game: &mut GameState, revision: u32) -> StrikeOutcome {
    let events = game
        .apply(GameIntent::ResolveVolley {
            tick: game.next_tick(),
            actor: ACTOR,
            target: TARGET,
            strikes: vec![StrikeReceipt {
                binding: LimbBinding {
                    part: mesocosm_core::PartId(0),
                    revision: BodyRevisionId(0),
                },
                direction: Direction::Right,
                charge: 4,
            }],
            rules: CombatRules {
                revision,
                max_reach: 3,
                ..CombatRules::default()
            },
        })
        .unwrap();
    events
        .into_iter()
        .find_map(|event| match event {
            GameEvent::VolleyResolved { strikes, .. } => Some(strikes[0].outcome),
            _ => None,
        })
        .unwrap()
}
fn move_actor(game: &mut GameState) {
    game.apply(GameIntent::AdvanceMotion {
        tick: game.next_tick(),
        subject: ACTOR,
        revision: BodyRevisionId(0),
        step: 1,
        input: MotionInput {
            move_x: 32767,
            move_z: 0,
        },
        rules: MotionRules::default(),
    })
    .unwrap();
}
#[test]
fn fractional_motion_changes_contact_without_crossing_a_cell() {
    let initial = setup();
    assert_eq!(
        fire(&mut initial.clone(), 2),
        StrikeOutcome::Miss,
        "face touching is not overlap"
    );
    let mut moved = initial.clone();
    move_actor(&mut moved);
    assert_eq!(
        initial.movement().position(ACTOR),
        moved.movement().position(ACTOR)
    );
    assert!(moved.pose(ACTOR).unwrap().position[0] > initial.pose(ACTOR).unwrap().position[0]);
    assert_eq!(
        fire(&mut moved.clone(), 1),
        StrikeOutcome::Miss,
        "recorded revision1 remains cell-based"
    );
    let mut both = moved.clone();
    both.apply(GameIntent::AdvanceMotion {
        tick: both.next_tick(),
        subject: TARGET,
        revision: BodyRevisionId(0),
        step: 1,
        input: MotionInput {
            move_x: 32767,
            move_z: 0,
        },
        rules: MotionRules::default(),
    })
    .unwrap();
    assert_eq!(
        fire(&mut both, 2),
        StrikeOutcome::Miss,
        "equal fractional target movement restores the touching boundary"
    );
    let before = moved.bodies().get(TARGET).unwrap().vitality;
    assert!(matches!(
        fire(&mut moved, 2),
        StrikeOutcome::Hit {
            quality: 0,
            harm: 12,
            ..
        }
    ));
    assert_eq!(moved.bodies().get(TARGET).unwrap().vitality, before - 12);
    assert_eq!(
        moved.current_anatomy(TARGET).unwrap().revision,
        BodyRevisionId(1)
    );
}
#[test]
fn saved_fractional_contact_continues_exactly() {
    let mut straight = setup();
    move_actor(&mut straight);
    let mut resumed = GameState::restore(&straight.save().unwrap()).unwrap();
    assert_eq!(fire(&mut straight, 2), fire(&mut resumed, 2));
    assert_eq!(straight, resumed);
    assert_eq!(
        GameState::restore(&resumed.save().unwrap()).unwrap(),
        straight
    );
}

#[test]
fn real_v5_motion_archive_retains_its_legacy_combat_history() {
    use paredros_world::timed_action::{TimedActionSave, TimedActionSession};
    let bytes = include_bytes!("fixtures/timed-action-v1-game-v5.save");
    let save: TimedActionSave = mesocosm_core::snapshot::decode(bytes).unwrap();
    assert_eq!(save.session.game.version, 5);
    assert!(save.session.game.intents.iter().any(|intent| matches!(
        intent,
        GameIntent::ResolveVolley {
            rules: CombatRules { revision: 1, .. },
            ..
        }
    )));
    let mut restored = TimedActionSession::restore(bytes).unwrap();
    assert_eq!(restored.save().unwrap(), bytes.as_slice());
    let subject = restored.session().control().played();
    let game = restored.session().game();
    let step = game.pose(subject).unwrap().step + 1;
    restored
        .apply_game_batch(&[GameIntent::AdvanceMotion {
            tick: game.next_tick(),
            subject,
            revision: game.bodies().get(subject).unwrap().revision,
            step,
            input: MotionInput::default(),
            rules: MotionRules::default(),
        }])
        .unwrap();
    assert_eq!(
        TimedActionSession::restore(&restored.save().unwrap()).unwrap(),
        restored
    );
}

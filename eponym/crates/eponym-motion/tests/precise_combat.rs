// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use eponym_play::identity::{BodyRevisionId, SubjectId};
use eponym_play::timed_action::{Direction, LimbBinding, StrikeReceipt};
use eponym_play::*;
use isocosm::lineage::SpeciesId;

const ACTOR: SubjectId = SubjectId(31);
const TARGET: SubjectId = SubjectId(32);

fn setup() -> GameState {
    let mut game = GameState::new(World::generate_native_fixture(7, WorldConfig::default(), NativeFixtureProfile::PreciseCombatV1).unwrap())
        .with_motion_solver(eponym_motion::SOLVER);
    let at = game
        .items()
        .all()
        .find_map(|item| match item.location {
            ItemLocation::At(at) if item.kind == ItemKind::Dressing => Some(at),
            _ => None,
        })
        .unwrap();
    for (subject, recipe) in [(ACTOR, NativeFixtureRecipe::ProbeActorV1), (TARGET, NativeFixtureRecipe::ProbeTargetV1)] {
        game.apply(GameIntent::GenerateNativeRecipe {
            tick: game.next_tick(),
            subject,
            body_seed: 1,
            at,
            recipe,
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
            document: Box::new(game.native_anatomy(subject, SpeciesId(1)).unwrap()),
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
                    part: isometer_core::PartId(0),
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
    let mut resumed =
        GameState::restore_solving(&straight.save().unwrap(), eponym_motion::SOLVER).unwrap();
    assert_eq!(fire(&mut straight, 2), fire(&mut resumed, 2));
    assert_eq!(straight, resumed);
    assert_eq!(
        GameState::restore_solving(&resumed.save().unwrap(), eponym_motion::SOLVER).unwrap(),
        straight
    );
}

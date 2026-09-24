// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use eponym_identity::{BodyRevisionId, SubjectId};
use eponym_world::{
    GameIntent, GameState, ItemLocation, MotionInput, MotionPose, MotionRules, Name, World,
    WorldConfig, fixtures::three_lives::wetland_body,
};

const SUBJECT: SubjectId = SubjectId(1);

#[test]
fn malformed_standalone_pose_receipts_cannot_corrupt_movement() {
    let state = game();
    let mut movement = state.movement().clone();
    let mut pose = movement.pose(SUBJECT).unwrap();
    pose.step = 1;
    let mut invalid = Vec::new();
    let mut wrong = pose;
    wrong.position[0] = i64::MAX;
    invalid.push(wrong);
    let mut wrong = pose;
    wrong.fall_origin_y = Some(pose.position[1]);
    invalid.push(wrong);
    let mut wrong = pose;
    wrong.grounded = false;
    invalid.push(wrong);
    let mut wrong = pose;
    wrong.step = 8;
    invalid.push(wrong);
    for pose in invalid {
        let before = movement.clone();
        assert!(
            movement
                .apply(
                    state.world(),
                    eponym_world::MovementIntent::ContactPose {
                        tick: movement.next_tick(),
                        subject: SUBJECT,
                        pose,
                    }
                )
                .is_err()
        );
        assert_eq!(movement, before);
    }
}

fn game() -> GameState {
    let world = World::generate(7, WorldConfig::default()).unwrap();
    let mut game = GameState::new(world);
    let at = game
        .items()
        .all()
        .find_map(|item| match item.location {
            ItemLocation::At(at)
                if game
                    .world()
                    .ground()
                    .stands(at, mesocosm_core::places::WALKER_HEIGHT) =>
            {
                Some(at)
            },
            _ => None,
        })
        .unwrap();
    game.apply(GameIntent::Generate {
        tick: game.next_tick(),
        subject: SUBJECT,
        body_seed: 7,
        at,
    })
    .unwrap();
    game.apply(GameIntent::Name {
        tick: game.next_tick(),
        subject: SUBJECT,
        name: Name::new("Mover").unwrap(),
    })
    .unwrap();
    game.apply(GameIntent::AdmitAnatomy {
        tick: game.next_tick(),
        subject: SUBJECT,
        revision: BodyRevisionId(0),
        document: Box::new(wetland_body()),
    })
    .unwrap();
    game
}

fn advance(game: &mut GameState, step: u64, input: MotionInput) {
    game.apply(GameIntent::AdvanceMotion {
        tick: game.next_tick(),
        subject: SUBJECT,
        revision: BodyRevisionId(0),
        step,
        input,
        rules: MotionRules::default(),
    })
    .unwrap();
}

fn input(x: i16, z: i16) -> MotionInput {
    MotionInput {
        move_x: x,
        move_z: z,
    }
}

#[test]
fn repeated_small_motion_retains_fraction_and_projects_to_the_logical_cell() {
    let mut state = game();
    let origin = state.movement().position(SUBJECT).unwrap();
    let origin_pose = state.movement().pose(SUBJECT).unwrap();
    advance(&mut state, 1, input(32767, 0));
    let first = state.movement().pose(SUBJECT).unwrap();
    let displacement = first.position[0] - origin_pose.position[0];
    assert!(displacement > 0 && displacement < 65_536);
    assert_eq!(first.cell().unwrap(), origin);
    assert_eq!(
        first.cell().unwrap(),
        state.movement().position(SUBJECT).unwrap()
    );

    advance(&mut state, 2, input(32767, 0));
    advance(&mut state, 3, input(32767, 0));
    let final_pose = state.movement().pose(SUBJECT).unwrap();
    assert!(final_pose.position[0] > first.position[0]);
    assert_eq!(
        final_pose.cell().unwrap(),
        state.movement().position(SUBJECT).unwrap()
    );
    assert_eq!(first.cell().unwrap(), origin);
}

#[test]
fn motion_save_restore_and_continuation_are_exact() {
    let mut straight = game();
    advance(&mut straight, 1, input(32767, 0));
    advance(&mut straight, 2, input(0, 32767));
    let mut resumed = GameState::restore(&straight.save().unwrap()).unwrap();
    assert_eq!(resumed, straight);

    advance(&mut straight, 3, input(-32767, 0));
    advance(&mut resumed, 3, input(-32767, 0));
    assert_eq!(resumed, straight);
    assert_eq!(
        resumed.movement().pose(SUBJECT),
        straight.movement().pose(SUBJECT)
    );
}

#[test]
fn stale_revision_duplicate_step_and_bad_input_or_rules_are_atomic() {
    let mut state = game();
    advance(&mut state, 1, input(32767, 0));

    let before = state.clone();
    assert!(
        state
            .apply(GameIntent::AdvanceMotion {
                tick: state.next_tick(),
                subject: SUBJECT,
                revision: BodyRevisionId(9),
                step: 2,
                input: input(32767, 0),
                rules: MotionRules::default(),
            })
            .is_err()
    );
    assert_eq!(state, before);

    let before = state.clone();
    assert!(
        state
            .apply(GameIntent::AdvanceMotion {
                tick: state.next_tick(),
                subject: SUBJECT,
                revision: BodyRevisionId(0),
                step: 1,
                input: input(32767, 0),
                rules: MotionRules::default(),
            })
            .is_err()
    );
    assert_eq!(state, before);

    for bad in [(-32768, 0), (0, -32768)] {
        let before = state.clone();
        assert!(
            state
                .apply(GameIntent::AdvanceMotion {
                    tick: state.next_tick(),
                    subject: SUBJECT,
                    revision: BodyRevisionId(0),
                    step: 2,
                    input: input(bad.0, bad.1),
                    rules: MotionRules::default(),
                })
                .is_err()
        );
        assert_eq!(state, before);
    }

    let before = state.clone();
    let mut bad_rules = MotionRules::default();
    bad_rules.speed = 0;
    assert!(
        state
            .apply(GameIntent::AdvanceMotion {
                tick: state.next_tick(),
                subject: SUBJECT,
                revision: BodyRevisionId(0),
                step: 2,
                input: input(32767, 0),
                rules: bad_rules,
            })
            .is_err()
    );
    assert_eq!(state, before);
}

#[test]
fn legacy_integer_move_cannot_reset_an_active_fractional_pose() {
    let mut state = game();
    advance(&mut state, 1, input(32767, 0));
    let before = state.clone();
    let at = state.movement().position(SUBJECT).unwrap();
    assert!(
        state
            .apply(GameIntent::Move {
                tick: state.next_tick(),
                subject: SUBJECT,
                toward: at,
            })
            .is_err()
    );
    assert_eq!(state, before);
    assert_ne!(state.movement().pose(SUBJECT).unwrap().position, [0; 3]);
}

#[allow(dead_code)]
fn _pose_type_is_publicly_constructible() {
    let _ = MotionPose {
        position: [0; 3],
        velocity: [0; 3],
        grounded: true,
        step: 0,
        fall_origin_y: None,
    };
}

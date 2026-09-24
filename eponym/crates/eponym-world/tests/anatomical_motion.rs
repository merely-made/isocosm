// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use mesocosm_core::PartId;
use eponym_identity::{BodyRevisionId, SubjectId};
use eponym_world::{
    GameError, GameIntent, GameState, MotionEnvelope, MotionInput, MotionRules, MovementProfile,
    Name, SupportBand, World, WorldConfig, fixtures::three_lives::wetland_body,
};

const SUBJECT: SubjectId = SubjectId(73);
const ROOT: PartId = PartId(0);
const LIMB_A: PartId = PartId(1);
const LIMB_B: PartId = PartId(2);

fn state() -> GameState {
    let mut state = GameState::new(World::generate(7, WorldConfig::default()).unwrap());
    let at = state
        .items()
        .all()
        .find_map(|item| match item.location {
            eponym_world::ItemLocation::At(at)
                if state
                    .world()
                    .ground()
                    .stands(at, mesocosm_core::places::WALKER_HEIGHT) =>
            {
                Some(at)
            },
            _ => None,
        })
        .unwrap();
    state
        .apply(GameIntent::Generate {
            tick: state.next_tick(),
            subject: SUBJECT,
            body_seed: 7,
            at,
        })
        .unwrap();
    state
        .apply(GameIntent::Name {
            tick: state.next_tick(),
            subject: SUBJECT,
            name: Name::new("Shape tester").unwrap(),
        })
        .unwrap();
    state
        .apply(GameIntent::AdmitAnatomy {
            tick: state.next_tick(),
            subject: SUBJECT,
            revision: BodyRevisionId(0),
            document: Box::new(wetland_body()),
        })
        .unwrap();
    state
}

fn profile(revision: u64, supports: Vec<PartId>) -> MovementProfile {
    MovementProfile {
        revision: eponym_world::MOVEMENT_PROFILE_REVISION,
        source_revision: BodyRevisionId(revision),
        envelope: MotionEnvelope {
            anchor: ROOT,
            half_width: MotionRules::default().half_width,
            height: MotionRules::default().height,
        },
        supports,
        support_band: SupportBand {
            min_y: -2,
            max_y: 2,
        },
    }
}

fn configure(state: &mut GameState, revision: u64, supports: Vec<PartId>) {
    state
        .apply(GameIntent::ConfigureMovementProfile {
            tick: state.next_tick(),
            subject: SUBJECT,
            revision: BodyRevisionId(revision),
            profile: profile(revision, supports),
        })
        .unwrap();
}

fn advance(state: &mut GameState, step: u64, input: MotionInput) -> Result<(), GameError> {
    let revision = state.bodies().get(SUBJECT).unwrap().revision;
    let mut rules = MotionRules::default();
    rules.revision = 2;
    state
        .apply(GameIntent::AdvanceMotion {
            tick: state.next_tick(),
            subject: SUBJECT,
            revision,
            step,
            input,
            rules,
        })
        .map(|_| ())
}

#[test]
fn support_removal_changes_declared_projection_and_disallows_input() {
    let mut state = state();
    configure(&mut state, 0, vec![LIMB_A]);
    advance(
        &mut state,
        1,
        MotionInput {
            move_x: 32767,
            move_z: 0,
        },
    )
    .unwrap();

    state
        .apply(GameIntent::Fall {
            tick: state.next_tick(),
            subject: SUBJECT,
            distance: 5,
        })
        .unwrap();
    state
        .apply(GameIntent::ReconcileAnatomy {
            tick: state.next_tick(),
            subject: SUBJECT,
            from_revision: BodyRevisionId(0),
            revision: BodyRevisionId(1),
            severed_parts: vec![LIMB_A],
        })
        .unwrap();
    let before = state.clone();
    assert!(
        advance(
            &mut state,
            2,
            MotionInput {
                move_x: 32767,
                move_z: 0,
            },
        )
        .is_err()
    );
    assert_eq!(state, before);
    advance(&mut state, 2, MotionInput::default()).unwrap();
}

#[test]
fn losing_one_of_two_supports_halves_speed_and_survives_resume() {
    let mut one = state();
    let mut two = state();
    configure(&mut one, 0, vec![LIMB_A, LIMB_B]);
    configure(&mut two, 0, vec![LIMB_A, LIMB_B]);
    injure(&mut one, vec![LIMB_B]);
    injure(&mut two, vec![]);
    let one_projection = one.movement_projection(SUBJECT).unwrap().unwrap();
    let two_projection = two.movement_projection(SUBJECT).unwrap().unwrap();
    assert_eq!(one_projection.speed * 2, two_projection.speed);
    assert_eq!(one_projection.active_supports, vec![LIMB_A]);
    let input = MotionInput {
        move_x: 32767,
        move_z: 0,
    };
    advance(&mut one, 1, input).unwrap();
    advance(&mut two, 1, input).unwrap();
    assert!(two.pose(SUBJECT).unwrap().position[0] > one.pose(SUBJECT).unwrap().position[0]);
    let mut resumed = GameState::restore(&one.save().unwrap()).unwrap();
    advance(&mut one, 2, input).unwrap();
    advance(&mut resumed, 2, input).unwrap();
    assert_eq!(resumed, one);
}

fn injure(state: &mut GameState, severed_parts: Vec<PartId>) {
    state
        .apply(GameIntent::Fall {
            tick: state.next_tick(),
            subject: SUBJECT,
            distance: 5,
        })
        .unwrap();
    state
        .apply(GameIntent::ReconcileAnatomy {
            tick: state.next_tick(),
            subject: SUBJECT,
            from_revision: BodyRevisionId(0),
            revision: BodyRevisionId(1),
            severed_parts,
        })
        .unwrap();
}

#[test]
fn unrelated_extended_limb_does_not_enlarge_collision_envelope() {
    let mut with_arm = state();
    let mut without_arm = with_arm.clone();
    configure(&mut with_arm, 0, vec![LIMB_A, LIMB_B]);
    configure(&mut without_arm, 0, vec![LIMB_A, LIMB_B]);
    for (world, severed_parts) in [(&mut with_arm, vec![]), (&mut without_arm, vec![PartId(3)])] {
        world
            .apply(GameIntent::Fall {
                tick: world.next_tick(),
                subject: SUBJECT,
                distance: 5,
            })
            .unwrap();
        world
            .apply(GameIntent::ReconcileAnatomy {
                tick: world.next_tick(),
                subject: SUBJECT,
                from_revision: BodyRevisionId(0),
                revision: BodyRevisionId(1),
                severed_parts,
            })
            .unwrap();
    }
    let input = MotionInput {
        move_x: 32767,
        move_z: 0,
    };
    advance(&mut with_arm, 1, input).unwrap();
    advance(&mut without_arm, 1, input).unwrap();
    assert_eq!(with_arm.pose(SUBJECT), without_arm.pose(SUBJECT));
    assert_eq!(
        with_arm.movement_projection(SUBJECT).unwrap(),
        without_arm.movement_projection(SUBJECT).unwrap()
    );
}

#[test]
fn configured_shape_survives_save_resume_and_continued_motion() {
    let mut straight = state();
    configure(&mut straight, 0, vec![LIMB_A]);
    advance(
        &mut straight,
        1,
        MotionInput {
            move_x: 32767,
            move_z: 0,
        },
    )
    .unwrap();
    let mut resumed = GameState::restore(&straight.save().unwrap()).unwrap();
    assert_eq!(resumed, straight);
    advance(&mut straight, 2, MotionInput::default()).unwrap();
    advance(&mut resumed, 2, MotionInput::default()).unwrap();
    assert_eq!(resumed, straight);
}

#[test]
fn stale_or_invalid_profiles_and_disguised_old_archives_reject() {
    let mut state = state();
    let before = state.clone();
    let mut invalid = profile(0, vec![PartId(99)]);
    invalid.envelope.anchor = PartId(99);
    assert!(
        state
            .apply(GameIntent::ConfigureMovementProfile {
                tick: state.next_tick(),
                subject: SUBJECT,
                revision: BodyRevisionId(0),
                profile: invalid,
            })
            .is_err()
    );
    assert_eq!(state, before);

    configure(&mut state, 0, vec![LIMB_A]);
    injure(&mut state, vec![LIMB_A]);
    configure(&mut state, 1, vec![LIMB_B]);
    let before = state.clone();
    assert!(
        state
            .apply(GameIntent::ConfigureMovementProfile {
                tick: state.next_tick(),
                subject: SUBJECT,
                revision: BodyRevisionId(0),
                profile: profile(0, vec![LIMB_A]),
            })
            .is_err()
    );
    assert_eq!(state, before);

    advance(&mut state, 1, MotionInput::default()).unwrap();
    let resumed = GameState::restore(&state.save().unwrap()).unwrap();
    assert_eq!(resumed, state);

    let mut save = state.save_record().unwrap();
    save.version = 5;
    assert!(GameState::restore_record(save).is_err());
}

#[test]
fn configured_band_and_requested_rule_limits_remain_authoritative() {
    let mut state = state();
    let mut selected = profile(0, vec![LIMB_A, LIMB_B]);
    selected.support_band = SupportBand {
        min_y: 10,
        max_y: 12,
    };
    state
        .apply(GameIntent::ConfigureMovementProfile {
            tick: state.next_tick(),
            subject: SUBJECT,
            revision: BodyRevisionId(0),
            profile: selected,
        })
        .unwrap();
    assert_eq!(
        state.movement_projection(SUBJECT).unwrap().unwrap().speed,
        0
    );
    let before = state.clone();
    let invalid = MotionRules {
        revision: 2,
        speed: -1,
        ..MotionRules::default()
    };
    assert!(
        state
            .apply(GameIntent::AdvanceMotion {
                tick: state.next_tick(),
                subject: SUBJECT,
                revision: BodyRevisionId(0),
                step: 1,
                input: MotionInput::default(),
                rules: invalid,
            })
            .is_err()
    );
    assert_eq!(state, before);
    advance(&mut state, 1, MotionInput::default()).unwrap();
}

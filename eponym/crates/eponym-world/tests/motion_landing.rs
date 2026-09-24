// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use eponym_identity::{BodyRevisionId, SubjectId};
use eponym_world::{
    GameEvent, GameIntent, GameState, MotionInput, MotionRules, Name, World, WorldConfig,
    fixtures::three_lives::wetland_body,
};

const SUBJECT: SubjectId = SubjectId(1);

fn ledge(world: &World) -> ([i32; 3], MotionInput, i32) {
    let ground = world.ground();
    for x in -63..=63 {
        for z in -63..=63 {
            let Some(high) = ground.surface(x, z) else {
                continue;
            };
            for (dx, dz, input) in [
                (
                    1,
                    0,
                    MotionInput {
                        move_x: 32767,
                        move_z: 0,
                    },
                ),
                (
                    -1,
                    0,
                    MotionInput {
                        move_x: -32767,
                        move_z: 0,
                    },
                ),
                (
                    0,
                    1,
                    MotionInput {
                        move_x: 0,
                        move_z: 32767,
                    },
                ),
                (
                    0,
                    -1,
                    MotionInput {
                        move_x: 0,
                        move_z: -32767,
                    },
                ),
            ] {
                let Some(low) = ground.surface(x + dx, z + dz) else {
                    continue;
                };
                let distance = high - low;
                let start = [x, high + 1, z];
                if distance > 4
                    && ground.stands(start, mesocosm_core::places::WALKER_HEIGHT)
                    && ground.stands(
                        [x + dx, low + 1, z + dz],
                        mesocosm_core::places::WALKER_HEIGHT,
                    )
                {
                    return (start, input, distance);
                }
            }
        }
    }
    panic!("seeded world has no adjacent ledge greater than SAFE_FALL");
}

fn game() -> (GameState, MotionInput, i32) {
    let mut world = World::generate(7, WorldConfig::default()).unwrap();
    let (at, input, drop) = ledge(&world);
    // Deepen the lower column through the real recorded terrain-edit API.
    // The natural relief alone sits at the safe-fall boundary.
    let lower = [
        at[0] + i32::from(input.move_x.signum()),
        at[1] - 1 - drop,
        at[2] + i32::from(input.move_z.signum()),
    ];
    world
        .apply(eponym_world::WorldIntent::Carve {
            tick: eponym_identity::Tick(0),
            by: SUBJECT,
            centre: [lower[0], lower[1] - 1, lower[2]],
            radius: 2,
        })
        .unwrap();
    let mut game = GameState::new(world);
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
        name: Name::new("Lander").unwrap(),
    })
    .unwrap();
    game.apply(GameIntent::AdmitAnatomy {
        tick: game.next_tick(),
        subject: SUBJECT,
        revision: BodyRevisionId(0),
        document: Box::new(wetland_body()),
    })
    .unwrap();
    (game, input, drop)
}

fn advance(game: &mut GameState, step: u64, input: MotionInput) -> Vec<GameEvent> {
    game.apply(GameIntent::AdvanceMotion {
        tick: game.next_tick(),
        subject: SUBJECT,
        revision: game.bodies().get(SUBJECT).unwrap().revision,
        step,
        input,
        rules: MotionRules::default(),
    })
    .unwrap()
}

fn input_for(step: u64, direction: MotionInput) -> MotionInput {
    if step <= 16 {
        direction
    } else {
        MotionInput::default()
    }
}

#[test]
fn ledge_landing_admits_one_fall_consequence_and_current_anatomy() {
    let (mut state, input, _drop) = game();
    let mut landed = Vec::new();
    for step in 1..=240 {
        let events = advance(&mut state, step, input_for(step, input));
        if events
            .iter()
            .any(|event| matches!(event, GameEvent::Injured { .. }))
        {
            landed = events;
            break;
        }
    }
    assert!(!landed.is_empty(), "ledge should produce a landing injury");
    assert!(
        landed
            .iter()
            .any(|event| matches!(event, GameEvent::Injured {
        distance, harm, revision: BodyRevisionId(1), ..
    } if *distance > 4 && *harm > 0))
    );
    assert_eq!(
        landed
            .iter()
            .filter(|event| matches!(event, GameEvent::Injured { .. }))
            .count(),
        1
    );
    assert_eq!(
        state.bodies().get(SUBJECT).unwrap().revision,
        BodyRevisionId(1)
    );
    assert_eq!(
        state.current_anatomy(SUBJECT).unwrap().revision,
        BodyRevisionId(1)
    );
    assert!(state.movement().pose(SUBJECT).unwrap().grounded);
    let next = state.movement().pose(SUBJECT).unwrap().step + 1;
    for step in next..next + 10 {
        assert!(
            !advance(&mut state, step, MotionInput::default())
                .iter()
                .any(|event| matches!(event, GameEvent::Injured { .. }))
        );
    }
    assert_eq!(
        state.bodies().get(SUBJECT).unwrap().revision,
        BodyRevisionId(1)
    );
    assert_eq!(
        state.movement().pose(SUBJECT).unwrap().cell().unwrap(),
        state.movement().position(SUBJECT).unwrap()
    );
}

#[test]
fn midair_save_restore_lands_with_identical_consequence() {
    let (mut straight, input, _) = game();
    let mut resumed = None;
    let mut resume_step = 0;
    for step in 1..=240 {
        advance(&mut straight, step, input_for(step, input));
        if straight
            .movement()
            .pose(SUBJECT)
            .unwrap()
            .fall_origin_y
            .is_some()
        {
            resumed = Some(GameState::restore(&straight.save().unwrap()).unwrap());
            resume_step = step;
            break;
        }
    }
    let mut resumed = resumed.expect("ledge must produce a midair pose");
    for step in (resume_step + 1)..=240 {
        if straight.movement().pose(SUBJECT).unwrap().grounded {
            break;
        }
        let a = advance(&mut straight, step, input_for(step, input));
        let b = advance(&mut resumed, step, input_for(step, input));
        assert_eq!(a, b);
    }
    assert_eq!(resumed, straight);
    assert_eq!(
        straight
            .events()
            .iter()
            .filter(|event| matches!(event, GameEvent::Injured { .. }))
            .count(),
        1
    );
}

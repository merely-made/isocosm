// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use eponym_play::identity::{BodyRevisionId, SubjectId};
use eponym_play::{
    GameEvent, GameIntent, GameState, MotionInput, MotionRules, Name, World, WorldConfig,
};
use isocosm::{anatomy, history::Command};

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
                    && ground.stands(start, eponym_play::walking::WALKER_HEIGHT)
                    && ground.stands(
                        [x + dx, low + 1, z + dz],
                        eponym_play::walking::WALKER_HEIGHT,
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
        .apply(eponym_play::WorldIntent::Carve {
            tick: eponym_play::identity::Tick(0),
            by: SUBJECT,
            centre: [lower[0], lower[1] - 1, lower[2]],
            radius: 2,
        })
        .unwrap();
    let mut game = GameState::new(world).with_motion_solver(eponym_motion::SOLVER);
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
        document: Box::new(game.native_anatomy(SUBJECT, isocosm::lineage::SpeciesId(71)).unwrap()),
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
    let entity = state.entity(SUBJECT).unwrap();
    let mut landed = Vec::new();
    let mut prior = None;
    for step in 1..=240 {
        let sim = state.world().sim();
        let body = sim.state().population.get(entity).unwrap().clone();
        let matter = sim.matter();
        let site = body.resident_site().unwrap();
        let tissue_at_site = sim.state().sites[&site].accounts
            .get(eponym_play::founding::TISSUE).copied().unwrap_or(0);
        let entries = state.world().session().entries.len();
        let revision = state.bodies().get(SUBJECT).unwrap().revision;
        let events = advance(&mut state, step, input_for(step, input));
        if events
            .iter()
            .any(|event| matches!(event, GameEvent::Injured { .. }))
        {
            landed = events;
            prior = Some((body, matter, tissue_at_site, entries, revision));
            break;
        }
    }
    assert!(!landed.is_empty(), "ledge should produce a landing injury");
    let (before, matter, tissue_at_site, entries, before_revision) = prior.unwrap();
    let sim = state.world().sim();
    let native = sim.state().population.get(entity).unwrap();
    let severed: Vec<_> = native.body.as_ref().unwrap().parts.iter()
        .filter(|part| part.severed && !before.body.as_ref().unwrap().part(part.id).unwrap().severed)
        .map(|part| part.id).collect();
    let revision = BodyRevisionId(before_revision.0 + u64::from(!severed.is_empty()));
    let (distance, harm, injured_revision) = landed.iter().find_map(|event| match event {
        GameEvent::Injured { distance, harm, revision, .. } => Some((*distance, *harm, *revision)),
        _ => None,
    }).unwrap();
    assert!(distance > eponym_play::SAFE_FALL && harm > 0);
    assert_eq!(injured_revision, revision);
    assert_eq!(
        landed
            .iter()
            .filter(|event| matches!(event, GameEvent::Injured { .. }))
            .count(),
        1
    );
    assert_eq!(
        state.bodies().get(SUBJECT).unwrap().revision,
        revision
    );
    let admitted = state.current_anatomy(SUBJECT).unwrap();
    assert_eq!(admitted.revision, revision);
    assert_eq!(native.body_revision, before.body_revision + u64::from(!severed.is_empty()));
    assert_eq!(admitted.native_source.as_ref().unwrap().body_revision, native.body_revision);
    assert_eq!(admitted.document.doc(), native.body.as_ref().unwrap());
    let current = state.native_anatomy(SUBJECT, admitted.document.species()).unwrap();
    assert_eq!(current.doc(), admitted.document.doc());
    assert_eq!(current.total_mass_mg(), u64::from(state.bodies().get(SUBJECT).unwrap().profile.mass_mg));

    let wounds: Vec<_> = state.world().session().entries[entries..].iter()
        .filter_map(|entry| match entry.command {
            Command::Wound { entity: wounded, part, cells } if wounded == entity => Some((part, cells)),
            _ => None,
        }).collect();
    assert_eq!(wounds.len(), 1, "one landing must lower to one native Wound");
    assert_eq!(wounds[0].0, None, "the native draw owns whole-body fall injury");
    let living: u64 = before.living().map(|(id, part)|
        u64::from(anatomy::living_cells(before.extent(id), part))).sum();
    assert_eq!(u64::from(wounds[0].1), (living * u64::from(harm)).div_ceil(100).max(1));
    let lost: usize = native.parts.iter().map(|(id, part)| {
        let old = &before.parts[id];
        assert!(old.lost.iter().all(|cell| part.lost.contains(cell)));
        assert!(part.lost.iter().all(|cell| part.losses.contains_key(cell)));
        part.lost.len() - old.lost.len()
    }).sum();
    assert!(lost > 0, "the accepted native Wound must lose actual cells");
    assert_eq!(sim.matter(), matter, "native injury must conserve matter");
    let rules = &sim.genesis().rules;
    let tissue_before = anatomy::held(&before, rules, eponym_play::founding::TISSUE);
    let tissue_after = anatomy::held(native, rules, eponym_play::founding::TISSUE);
    assert!(tissue_after < tissue_before);
    let site = native.resident_site().unwrap();
    assert_eq!(sim.state().sites[&site].accounts.get(eponym_play::founding::TISSUE).copied().unwrap_or(0),
        tissue_at_site + tissue_before - tissue_after, "lost tissue must remain in the native site ledger");
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
        revision
    );
    assert_eq!(state.world().session().entries.iter().filter(|entry|
        matches!(entry.command, Command::Wound { entity: wounded, .. } if wounded == entity)).count(), 1);
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
            resumed = Some(
                GameState::restore_solving(&straight.save().unwrap(), eponym_motion::SOLVER)
                    .unwrap(),
            );
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

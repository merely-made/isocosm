// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use isocosm::{Command, Execution, Session, spatial as native};
use mesocosm_runtime::{
    Founded, Runtime,
    spatial::{Admission, Game},
};
#[path = "foraging_play/fixtures.rs"]
mod fixtures;
use fixtures::*;

#[test]
fn capacity_appetite_reads_the_body_and_keeps_other_computed_needs_excluded() {
    use isocosm::rules::{Binding, Expr, Query, Reading};
    let mut g = ordinary(16_003_851_315_348_448_882).genesis;
    native::embodiment::declare(&mut g).unwrap();
    let index = g.rules.mind.as_ref().unwrap().needs.iter()
        .position(|n| matches!(n.query, Query::Computed(_))).unwrap();
    let need = g.rules.mind.as_ref().unwrap().needs[index].clone();
    let key = isocosm::needs::account(&need.query).unwrap().clone();
    let actor = g.population.groups.values().map(|g| &*g.entity)
        .find(|e| e.traits.contains("life:consumer") && need.traits.is_subset(&e.traits)).unwrap();
    let want = isocosm::needs::want(&g.rules, actor, &key).unwrap();
    assert_eq!(want.deficit(), isocosm::anatomy::room(actor, &g.rules, &key));
    for (who, account) in [(Binding::Target, key), (Binding::Actor, "missing:account".into())] {
        g.rules.mind.as_mut().unwrap().needs[index].query = Query::Computed(Expr::AtLeast(
            Box::new(Expr::Read(Reading::Room { who, key: account })),
            Box::new(Expr::Const(1)),
        ));
        assert!(g.validate().is_err());
    }
    g.rules.mind.as_mut().unwrap().needs[index].query = Query::Computed(Expr::Const(1));
    assert!(g.validate().is_err());
}

#[test]
fn available_food_can_wait_for_its_native_feeding_cadence() {
    let mut f = at_food();
    let site = f.session.sim.state().population.get(f.actor).unwrap().resident_site();
    assert_eq!(f.distance(), 0);
    assert_eq!(f.session.sim.genesis().rules.processes[&f.walk].period, Some(1));
    assert_eq!(f.session.sim.genesis().rules.processes[&f.feed].period, Some(3));
    f.tick();
    assert_eq!(f.session.sim.state().population.get(f.actor).unwrap().resident_site(), site);
    assert_eq!(f.steps(), 0);
    f.game.validate_replay(&f.session).unwrap();
}

fn act(f: &mut Fixture, target: u64) -> isocosm::simulation::Receipt {
    let result = f
        .session
        .command(Command::Act {
            actor: f.actor,
            target: Some(target),
            process: f.feed.clone(),
            cause: None,
        })
        .unwrap();
    serde_json::from_str(&result).unwrap()
}

#[test]
fn hungry_native_choice_approaches_actual_food_and_meals_require_contact() {
    let mut f = control(true, 256, true);
    let before = f.distance();
    let food = f.food;
    assert!(
        !act(&mut f, food).accepted(),
        "site-grain meals must not reach remote food"
    );
    for _ in 0..12 {
        f.tick();
        if f.distance() == 0 {
            break;
        }
    }
    assert!(
        f.steps() > 0,
        "ordinary native choice must select a food-directed step"
    );
    assert!(
        f.distance() < before,
        "walking must approach the chosen food"
    );
    assert_eq!(
        f.distance(),
        0,
        "short supported route reaches food contact"
    );
    for id in [f.actor, f.food] {
        let e = f.session.sim.state().population.get(id).unwrap();
        println!(
            "contact id={id}, alive={}, site={:?}, patch={:?}, visit={}, holdings={:?}",
            e.alive,
            e.resident_site(),
            e.patch,
            e.arrived,
            isocosm::anatomy::books(e)
        );
    }
    let meal = act(&mut f, food);
    assert!(
        meal.accepted(),
        "the same native meal is available at contact: {meal:?}"
    );
    for (&id, placement) in &f.game.members {
        let native = &f.session.sim.state().spatial_poses[&id];
        assert_eq!(native.pose.at, placement.pose.at);
        assert_eq!(native.pose.yaw, placement.pose.yaw);
    }
    f.game.validate_replay(&f.session).unwrap();
    let loaded = Session::load_json(
        &serde_json::to_vec(&f.session.save()).unwrap(),
        Execution::Grouped,
    )
    .unwrap();
    assert_eq!(loaded.sim.state_hash(), f.session.sim.state_hash());
    assert_eq!(
        loaded.sim.state().spatial_poses,
        f.session.sim.state().spatial_poses
    );
    let mut tampered = f.game.clone();
    tampered.members.get_mut(&f.actor).unwrap().pose.at[0] += 1;
    assert!(tampered.validate_replay(&loaded).is_err());
}

#[test]
fn satisfied_or_ineligible_food_does_not_cause_seeking() {
    for (hungry, edible) in [(false, true), (true, false)] {
        let mut f = control(hungry, 256, edible);
        let before = f.game.members[&f.actor].pose.clone();
        for _ in 0..3 {
            f.tick();
        }
        assert_eq!(f.steps(), 0, "hungry={hungry}, edible={edible}");
        assert_eq!(f.game.members[&f.actor].pose, before);
        f.game.validate_replay(&f.session).unwrap();
    }
}

#[test]
fn a_saved_initial_projection_cannot_move_the_native_root() {
    let mut f = control(true, 256, true);
    for at in [
        f.game.members.get_mut(&f.actor).unwrap(),
        f.game
            .driver
            .as_mut()
            .unwrap()
            .initial
            .get_mut(&f.actor)
            .unwrap(),
    ] {
        at.pose.at[0] += 1;
        at.eye[0] += 1;
    }
    assert!(f.game.validate_replay(&f.session).is_err());
}

#[test]
fn bounded_routes_refuse_without_moving_or_teleporting() {
    let mut f = control(true, 1, true);
    let before = f.game.members[&f.actor].pose.clone();
    for _ in 0..3 {
        f.tick();
    }
    assert_eq!(f.steps(), 0);
    assert_eq!(f.game.members[&f.actor].pose, before);
    let refused = f.session.entries.iter().any(|entry| match &entry.command {
        Command::AdvanceBirthSpatial { steps, .. } => steps.iter().any(|step| {
            step.request.process == f.walk
                && step
                    .geometry
                    .as_ref()
                    .is_err_and(|why| why.contains("route"))
        }),
        _ => false,
    });
    assert!(
        refused,
        "native choice must actually try the bounded food route"
    );
    f.game.validate_replay(&f.session).unwrap();
}

#[test]
fn pending_and_stale_publication_cannot_change_native_poses() {
    let mut f = control(true, 256, true);
    for _ in 0..2 {
        f.tick();
    }
    let source = f.session.sim.state_hash();
    let poses = f.session.sim.state().spatial_poses.clone();
    let native::Preparation::Step(pending) = f
        .session
        .prepare_birth_spatial(1, &[], &[], &[], &[])
        .unwrap()
    else {
        panic!("ordinary choice should request a food step");
    };
    assert_eq!(f.session.sim.state_hash(), source);
    assert_eq!(f.session.sim.state().spatial_poses, poses);
    let geometry = native::foraging::geometry(
        &pending.sim,
        &pending.request,
        &f.game.members[&f.actor].window,
    )
    .unwrap();
    let answer = native::Resolution {
        request: pending.request,
        geometry: Ok(geometry),
    };
    let native::Preparation::Ready(prepared) = f
        .session
        .prepare_birth_spatial(1, &[], &[answer], &[], &[])
        .unwrap()
    else {
        panic!("resolved food route should prepare");
    };
    assert_eq!(f.session.sim.state().spatial_poses, poses);
    f.session.command(Command::Inspect(f.actor)).unwrap();
    assert!(prepared.publish(&mut f.session).is_err());
    assert_eq!(f.session.sim.state().spatial_poses, poses);
}

#[test]
fn a_terrain_wall_blocks_food_walking_without_invalidating_the_source_pose() {
    let mut f = control(true, 256, true);
    let member = &f.game.members[&f.actor];
    let document = f
        .session
        .sim
        .state()
        .population
        .get(f.actor)
        .unwrap()
        .body
        .as_ref()
        .unwrap();
    let boxes = native::embodiment::boxes(
        document,
        &f.session.sim.state().spatial_poses[&f.actor].pose,
    )
    .unwrap();
    let right = boxes.iter().map(|b| b[1][0]).max().unwrap();
    let bottom = boxes.iter().map(|b| b[0][1]).min().unwrap();
    let top = boxes.iter().map(|b| b[1][1]).max().unwrap() + 16;
    let at = member.pose.clone();
    f.session
        .command(Command::Edit {
            site: member.site.0,
            by: None,
            edit: isometer_space::Edit {
                op: isometer_space::Op::Fill(2),
                shape: isometer_space::Shape::Box {
                    min: [right + 1, bottom, member.window.min[1]],
                    max: [right + 2, top, member.window.max[1]],
                },
            },
        })
        .unwrap();
    f.game.refresh(&mut f.session).unwrap();
    f.game.members[&f.actor].realized(&f.session.sim).unwrap();
    for _ in 0..3 {
        f.tick();
    }
    assert_eq!(f.steps(), 0);
    assert_eq!(f.game.members[&f.actor].pose, at);
    f.game.validate_replay(&f.session).unwrap();
}

#[test]
fn unplaced_members_of_a_food_cohort_do_not_inherit_its_reach() {
    let mut f = control(true, 256, true);
    let unplaced = f.food + 1;
    assert!(f.session.sim.state().population.get(unplaced).is_some());
    assert!(!f.session.sim.state().spatial_poses.contains_key(&unplaced));
    for _ in 0..12 {
        f.tick();
        if f.distance() == 0 {
            break;
        }
    }
    assert_eq!(f.distance(), 0);
    assert!(!act(&mut f, unplaced).accepted());
}

#[test]
fn an_ordinary_a1_play_world_walks_without_forced_choices_and_restores() {
    let mut observed = Vec::new();
    for seed in (16_003_851_315_348_448_882..16_003_851_315_348_448_898).chain([7, 9]) {
        let Ok(setup) = mesocosm_runtime::runtime::ecology::Setup::generated(
            ordinary(seed),
            Execution::Individuals,
        ) else {
            continue;
        };
        let mut runtime = Runtime::bench_placed(setup.founded, 2).unwrap();
        if runtime
            .critter()
            .is_none_or(|id| !runtime.spatial().members.contains_key(&id))
        {
            continue;
        }
        for _ in 0..6 {
            runtime.step(1);
            if runtime.tick() % 3 == 0 {
                let r = runtime.session();
                let needs = runtime
                    .critter()
                    .and_then(|id| r.sim.state().population.get(id))
                    .map(|e| isocosm::needs::wants(&r.sim.genesis().rules, e));
                let refusals: Vec<_> = r
                    .entries
                    .last()
                    .into_iter()
                    .filter_map(|e| match &e.command {
                        Command::AdvanceBirthSpatial { steps, .. } => Some(
                            steps
                                .iter()
                                .filter_map(|s| s.geometry.as_ref().err())
                                .collect::<Vec<_>>(),
                        ),
                        _ => None,
                    })
                    .flatten()
                    .collect();
                println!(
                    "ordinary seed {seed}, tick {}, needs {needs:?}, route refusals {refusals:?}",
                    runtime.tick()
                );
            }
            let walking = runtime
                .spatial()
                .driver
                .as_ref()
                .unwrap()
                .records
                .iter()
                .flat_map(|r| &r.changes)
                .any(|c| matches!(c.admission, mesocosm_runtime::spatial::Admission::Step { .. }));
            if walking {
                let saved = runtime.save_world().unwrap();
                let restored = Runtime::resume_world(saved.clone(), 2, None).unwrap();
                assert_eq!(restored.state_hash(), runtime.state_hash());
                assert_eq!(restored.spatial(), runtime.spatial());
                assert_eq!(restored.save_world().unwrap(), saved);
                println!(
                    "ordinary seed {seed}, tick {}, accepted walking",
                    runtime.tick()
                );
                return;
            }
            if runtime.fault().is_some()
                || runtime.review().is_some()
                || runtime.checkpoint().is_some()
            {
                break;
            }
        }
        let needs = runtime.critter().and_then(|id| {
            runtime
                .sim()
                .state()
                .population
                .get(id)
                .map(|e| isocosm::needs::wants(&runtime.sim().genesis().rules, e))
        });
        let refused: Vec<_> = runtime
            .session()
            .entries
            .iter()
            .filter_map(|entry| match &entry.command {
                Command::AdvanceBirthSpatial { steps, .. } => Some(
                    steps
                        .iter()
                        .filter_map(|s| s.geometry.as_ref().err().cloned())
                        .collect::<Vec<_>>(),
                ),
                _ => None,
            })
            .flatten()
            .collect();
        observed.push((
            seed,
            runtime.tick(),
            runtime.fault().map(str::to_owned),
            needs,
            refused,
        ));
    }
    panic!("ordinary A1 play did not select walking: {observed:?}");
}

#[test]
fn actual_native_children_publish_their_own_resident_pose() {
    let mut f = control(true, 256, true);
    for _ in 0..18 {
        f.tick();
    }
    let children: Vec<_> = f.game.driver.as_ref().unwrap().records.iter()
        .flat_map(|r| &r.changes)
        .filter(|c| matches!(c.admission, Admission::Birth { .. })).collect();
    assert!(!children.is_empty(), "native reproduction must actually place a child");
    for child in children {
        let at = &f.session.sim.state().spatial_poses[&child.actor];
        let placed = &f.game.members[&child.actor];
        assert_eq!(at.site, placed.site.0);
        assert_eq!(at.pose.at, placed.pose.at);
        assert_eq!(at.pose.yaw, placed.pose.yaw);
        assert_eq!(Some(at.arrived), placed.resident_since);
    }
    f.game.validate_replay(&f.session).unwrap();
}

// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
use cosmograph::{Command, Execution, Session, simulation::Genesis, spatial as native};
use mesocosm_runtime::{
    Founded,
    spatial::{Admission, Game},
};
#[path = "foraging_play/fixtures.rs"]
mod fixtures;
use fixtures::*;

fn rebuild(mut f: Fixture, change: impl FnOnce(&mut Genesis)) -> Fixture {
    let mut g = f.session.sim.genesis().clone();
    change(&mut g);
    f.session = Session::new(g, Execution::Grouped).unwrap();
    f.game = Game::default();
    f.game.found_generated(&mut f.session).unwrap();
    f
}

fn blind(g: &mut Genesis, actor: u64) {
    let e = std::sync::Arc::make_mut(&mut g.population.groups.get_mut(&actor).unwrap().entity);
    let extents: std::collections::BTreeMap<_, _> =
        e.living().map(|(id, _)| (id, e.extent(id))).collect();
    for (id, p) in e.parts_mut() {
        p.functions.remove("function:sense");
        p.cells.remove("function:sense");
        p.tracts.clear();
        cosmograph::mosaic::sync(p, extents[id]);
    }
}

fn detected(f: &Fixture) -> Option<native::foraging::FoodDetection> {
    f.session.sim.food_detection(f.actor, f.food, &f.walk)
}

#[test]
fn terrain_hides_food_from_choice_and_removing_it_reveals_the_same_meal() {
    let mut f = control(true, 256, true);
    assert!(matches!(
        detected(&f),
        Some(native::foraging::FoodDetection::Sight(_))
    ));
    let at = f.game.members[&f.actor].clone();
    let materials = f
        .session
        .sim
        .site_geometry(at.site.0, &at.window, at.places.unwrap())
        .unwrap()
        .volume
        .materials;
    let boxes = |id| {
        native::embodiment::boxes(
            f.session
                .sim
                .state()
                .population
                .get(id)
                .unwrap()
                .body
                .as_ref()
                .unwrap(),
            &f.session.sim.state().spatial_poses[&id].pose,
        )
        .unwrap()
    };
    let a = boxes(f.actor);
    let b = boxes(f.food);
    let x = a.iter().map(|b| b[1][0]).max().unwrap() + 1;
    let bottom = a.iter().chain(&b).map(|b| b[0][1]).min().unwrap();
    let top = a.iter().chain(&b).map(|b| b[1][1]).max().unwrap() + 32;
    let wall = isometer_space::Shape::Box {
        min: [x, bottom, at.window.min[1]],
        max: [x + 1, top, at.window.max[1]],
    };
    let edit = |op| Command::Edit {
        site: at.site.0,
        by: None,
        edit: isometer_space::Edit {
            op,
            shape: wall.clone(),
        },
    };
    f.session
        .command(edit(isometer_space::Op::Fill(materials.rock)))
        .unwrap();
    f.game.refresh(&mut f.session).unwrap();
    assert_eq!(
        detected(&f),
        None,
        "nearby stock must not be sensed through a wall"
    );
    let first = f.session.entries.len();
    for _ in 0..3 {
        f.tick();
    }
    assert_eq!(f.steps(), 0);
    assert!(
        !f.session.entries[first..].iter().any(|e| match &e.command {
            Command::AdvanceBirthSpatial { steps, .. } => steps
                .iter()
                .any(|s| s.request.actor == f.actor && s.request.process == f.walk),
            _ => false,
        }),
        "hidden food must not even request a route"
    );
    f.session.command(edit(isometer_space::Op::Carve)).unwrap();
    f.game.refresh(&mut f.session).unwrap();
    assert!(
        detected(&f).is_some(),
        "edited terrain must invalidate the cached obstruction"
    );
    for _ in 0..3 {
        f.tick();
    }
    assert!(f.steps() > 0, "ordinary choice must seek the revealed meal");
    f.game.validate_replay(&f.session).unwrap();
    let restored = Session::load_json(
        &serde_json::to_vec(&f.session.save()).unwrap(),
        Execution::Grouped,
    )
    .unwrap();
    assert_eq!(restored.sim.state_hash(), f.session.sim.state_hash());
}

#[test]
fn absent_senses_and_short_ranges_refuse_remote_food_but_contact_remains() {
    let f = control(true, 256, true);
    let actor = f.actor;
    let mut f = rebuild(f, |g| blind(g, actor));
    assert_eq!(detected(&f), None);
    for _ in 0..3 {
        f.tick();
    }
    assert_eq!(f.steps(), 0);
    let f = control(true, 256, true);
    let walk = f.walk.clone();
    let mut f = rebuild(f, |g| {
        g.rules
            .spatial_templates
            .as_mut()
            .unwrap()
            .foraging
            .get_mut(&walk)
            .unwrap()
            .search = 1
    });
    assert_eq!(detected(&f), None);
    for _ in 0..3 {
        f.tick();
    }
    assert_eq!(f.steps(), 0);
    let f = at_food();
    let actor = f.actor;
    let mut f = rebuild(f, |g| blind(g, actor));
    assert_eq!(f.distance(), 0);
    assert_eq!(detected(&f), Some(native::foraging::FoodDetection::Contact));
    let outcome = f
        .session
        .command(Command::Act {
            actor: f.actor,
            target: Some(f.food),
            process: f.feed.clone(),
            cause: None,
        })
        .unwrap();
    let meal: cosmograph::simulation::Receipt = serde_json::from_str(&outcome).unwrap();
    assert!(
        meal.accepted(),
        "blind contact feeding still uses the native meal: {meal:?}"
    );
}

#[test]
fn detection_and_authored_picture_sight_cross_local_window_boundaries() {
    let f = control(true, 256, true);
    let actor = f.actor;
    let mut f = rebuild(f, |g| {
        let t = g.rules.spatial_templates.as_mut().unwrap();
        let at = t
            .stands
            .values()
            .flatten()
            .find(|s| s.actor == Some(actor))
            .unwrap()
            .at;
        let delta = 127 - at[0];
        t.version = 5;
        t.window.min = [0; 2];
        t.window.max = [g.world.footprint.unwrap().side as i64; 2];
        for s in t.stands.values_mut().flatten() {
            s.at[0] += delta;
        }
        t.crossings.clear();
    });
    assert_ne!(
        f.game.members[&f.actor].window,
        f.game.members[&f.food].window
    );
    assert!(detected(&f).is_some());
    f.game
        .set_sight(
            &f.session,
            &mesocosm_runtime::spatial::SightConfig {
                version: 1,
                ranges: [(f.actor, 128)].into(),
            },
        )
        .unwrap();
    let view = cosmograph::directing::readings::view(
        &f.session.sim,
        f.actor,
        cosmograph::directing::readings::Mode::Creative,
    );
    let (site, sight) = f
        .game
        .sight(&f.session.sim, f.actor, &view)
        .unwrap()
        .unwrap();
    assert!(site.seen(&f.session.sim, &sight).unwrap().contains(&f.food));
    for _ in 0..4 {
        f.tick();
    }
    assert!(f.steps() > 0);
    assert!(
        site.seen(&f.session.sim, &sight).is_err(),
        "cached terrain cannot grant stale state access"
    );
    f.game.validate_replay(&f.session).unwrap();
}

#[test]
fn unplaced_cohort_members_do_not_acquire_the_representatives_visibility() {
    let f = control(true, 256, true);
    assert!(detected(&f).is_some());
    assert!(f.session.sim.state().population.get(f.food + 1).is_some());
    assert_eq!(
        f.session.sim.food_detection(f.actor, f.food + 1, &f.walk),
        None
    );
}

#[test]
fn sensory_reallocation_changes_detection_while_reusing_identical_terrain() {
    let mut f = control(true, 256, true);
    assert!(detected(&f).is_some());
    let at = f.game.members[&f.actor].clone();
    let before = f
        .session
        .sim
        .site_geometry(at.site.0, &at.window, at.places.unwrap())
        .unwrap();
    let e = f.session.sim.state().population.get(f.actor).unwrap();
    let changes: Vec<_> = e
        .living()
        .filter(|(_, p)| p.cells.contains_key("function:sense"))
        .map(|(id, p)| {
            (
                id,
                p.tracts
                    .iter()
                    .filter(|t| t.function != "function:sense")
                    .map(|t| (t.function.clone(), t.cells.clone()))
                    .collect(),
            )
        })
        .collect();
    assert!(!changes.is_empty());
    for (part, tracts) in changes {
        f.session
            .command(Command::Express {
                entity: f.actor,
                part,
                tracts,
            })
            .unwrap();
    }
    f.game.refresh(&mut f.session).unwrap();
    let after = f
        .session
        .sim
        .site_geometry(at.site.0, &at.window, at.places.unwrap())
        .unwrap();
    assert!(std::sync::Arc::ptr_eq(&before, &after));
    assert_eq!(
        detected(&f),
        None,
        "a terrain cache hit must not cache biological answers"
    );
    for _ in 0..3 {
        f.tick();
    }
    assert_eq!(f.steps(), 0);
    f.game.validate_replay(&f.session).unwrap();
}

#[test]
fn the_sight_witness_moves_with_rotated_sensory_anatomy() {
    let f = control(true, 256, true);
    let actor = f.actor;
    let pose = &f.session.sim.state().spatial_poses[&actor].pose;
    let e = f.session.sim.state().population.get(actor).unwrap();
    let bounds = native::embodiment::part_boxes(e.body.as_ref().unwrap(), pose).unwrap();
    let sensor = e
        .living()
        .filter(|(id, p)| *id != e.body.as_ref().unwrap().root && p.cells.values().any(|n| *n > 1))
        .max_by_key(|(id, _)| bounds.get(id).map(|[lo, hi]| lo[0] + hi[0]))
        .unwrap()
        .0;
    let f = rebuild(f, |g| {
        blind(g, actor);
        let e = std::sync::Arc::make_mut(&mut g.population.groups.get_mut(&actor).unwrap().entity);
        let extent = e.extent(sensor);
        let p = e.part_mut(&sensor).unwrap();
        let source = p.cells.iter().find(|(_, n)| **n > 1).unwrap().0.clone();
        *p.cells.get_mut(&source).unwrap() -= 1;
        p.cells.insert("function:sense".into(), 1);
        p.functions.insert("function:sense".into());
        cosmograph::mosaic::sync(p, extent);
    });
    let Some(native::foraging::FoodDetection::Sight(before)) = detected(&f) else {
        panic!("visible food");
    };
    let food = f.food;
    let f = rebuild(f, |g| {
        let t = g.rules.spatial_templates.as_mut().unwrap();
        let origin = t
            .stands
            .values()
            .flatten()
            .find(|s| s.actor == Some(actor))
            .unwrap()
            .at;
        t.version = 5;
        t.window.min = [0; 2];
        t.window.max = [g.world.footprint.unwrap().side as i64; 2];
        for s in t
            .stands
            .values_mut()
            .flatten()
            .filter(|s| s.actor == Some(actor) || s.actor == Some(food))
        {
            s.at[0] = 2 * origin[0] - s.at[0];
            s.at[2] = 2 * origin[2] - s.at[2];
            s.yaw = s.yaw.compose(isometer_core::Yaw::Half);
        }
        t.crossings.clear();
    });
    let Some(native::foraging::FoodDetection::Sight(after)) = detected(&f) else {
        panic!("visible food after turning");
    };
    assert_ne!(before.eye, after.eye);
    let e = f.session.sim.state().population.get(actor).unwrap();
    let at = &f.session.sim.state().spatial_poses[&actor].pose;
    let bounds = native::embodiment::part_boxes(e.body.as_ref().unwrap(), at).unwrap();
    assert!(
        e.living()
            .filter(|(_, p)| p.cells.contains_key("function:sense"))
            .any(|(id, _)| bounds.get(&id).is_some_and(
                |[lo, hi]| (0..3).all(|k| lo[k] <= after.eye[k] && after.eye[k] < hi[k])
            ))
    );
}

#[test]
fn old_rule_grammar_omits_sight_and_keeps_its_original_sensorless_search() {
    let f = control(true, 256, true);
    let (actor, walk) = (f.actor, f.walk.clone());
    let mut f = rebuild(f, |g| {
        blind(g, actor);
        g.rules
            .spatial_templates
            .as_mut()
            .unwrap()
            .foraging
            .get_mut(&walk)
            .unwrap()
            .sight = false;
    });
    let policy = &f
        .session
        .sim
        .genesis()
        .rules
        .spatial_templates
        .as_ref()
        .unwrap()
        .foraging[&f.walk];
    let wire = serde_json::to_value(policy).unwrap();
    assert!(wire.get("sight").is_none());
    let decoded: native::foraging::Foraging = serde_json::from_value(wire).unwrap();
    assert!(!decoded.sight);
    for _ in 0..3 {
        f.tick();
    }
    assert!(f.steps() > 0);
    f.game.validate_replay(&f.session).unwrap();
}

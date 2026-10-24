// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
use isocosm::{Command, Execution, Session, simulation::Genesis, spatial as native};
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

fn wall(f: &Fixture, op: isometer_space::Op) -> Command {
    let at = &f.game.members[&f.actor];
    let sim = &f.session.sim;
    let boxes = native::embodiment::boxes(
        sim.state()
            .population
            .get(f.actor)
            .unwrap()
            .body
            .as_ref()
            .unwrap(),
        &at.pose,
    )
    .unwrap();
    let right = boxes.iter().map(|b| b[1][0]).max().unwrap();
    Command::Edit {
        site: at.site.0,
        by: None,
        edit: isometer_space::Edit {
            op,
            shape: isometer_space::Shape::Box {
                min: [
                    right + 1,
                    boxes.iter().map(|b| b[0][1]).min().unwrap(),
                    at.window.min[1],
                ],
                max: [
                    right + 2,
                    boxes.iter().map(|b| b[1][1]).max().unwrap() + 16,
                    at.window.max[1],
                ],
            },
        },
    }
}

fn without_sight(f: Fixture) -> Fixture {
    let walk = f.walk.clone();
    // Isolate physical reachability from the independently tested sensory gate.
    rebuild(f, |g| {
        g.rules
            .spatial_templates
            .as_mut()
            .unwrap()
            .foraging
            .get_mut(&walk)
            .unwrap()
            .sight = false
    })
}

#[test]
fn blocked_nearest_meal_yields_to_a_reachable_alternative() {
    let f = without_sight(control(true, 256, true));
    let (actor, food) = (f.actor, f.food);
    let mut f = rebuild(f, |g| {
        let t = g.rules.spatial_templates.as_ref().unwrap();
        let (&site, stands) = t
            .stands
            .iter()
            .find(|(_, s)| s.iter().any(|s| s.actor == Some(actor)))
            .unwrap();
        let stand = stands.iter().find(|s| s.actor == Some(actor)).unwrap();
        let before = native::Pose {
            at: stand.at,
            yaw: stand.yaw,
            feet: stand.feet.clone(),
        };
        let occupied = native::embodiment::boxes(
            g.population.get(actor).unwrap().body.as_ref().unwrap(),
            &before,
        )
        .unwrap();
        let left = occupied.iter().map(|b| b[0][0]).min().unwrap();
        let terrain = isocosm::Simulation::new(g.clone(), Execution::Grouped)
            .unwrap()
            .site_volume(site, &t.window)
            .unwrap();
        let doc = g.population.get(food).unwrap().body.as_ref().unwrap();
        let mut grounding = native::embodiment::Grounding::new(&terrain);
        let pose = (0..96i64)
            .find_map(|offset| {
                let pose = grounding
                    .at(doc, before.at[0] - offset, before.at[2], before.yaw)
                    .ok()?;
                let boxes = native::embodiment::boxes(doc, &pose).ok()?;
                (boxes.iter().map(|b| b[1][0]).max()? <= left - 6
                    && native::foraging::gap(&occupied, &boxes) == 6)
                    .then_some(pose)
            })
            .unwrap();
        let t = g.rules.spatial_templates.as_mut().unwrap();
        let stands = t.stands.get_mut(&site).unwrap();
        stands.retain(|s| s.actor == Some(actor) || s.actor == Some(food));
        stands.push(native::Stand {
            actor: Some(food + 1),
            at: pose.at,
            yaw: pose.yaw,
            feet: pose.feet,
        });
        t.crossings.clear();
    });
    assert_eq!(f.distance(), 4);
    f.session
        .command(wall(&f, isometer_space::Op::Fill(2)))
        .unwrap();
    f.game.refresh(&mut f.session).unwrap();
    let report = f.session.sim.food_route(actor, &f.walk).unwrap();
    let route = report.route.unwrap();
    assert_eq!(route.target, food + 1);
    assert!(route.path.len() > 1 && report.nodes <= 256);
    assert_complete(&f, &route);
    f.tick();
    assert!(f.steps() > 0, "ordinary choice must use the reachable meal");
    assert!(f.session.entries.iter().any(|e| match &e.command {
        Command::AdvanceBirthSpatial { steps, .. } =>
            steps.iter().any(|s| s.request.actor == actor
                && s.request.target == Some(food + 1)
                && s.geometry.is_ok()),
        _ => false,
    }));
    f.game.validate_replay(&f.session).unwrap();
}

fn assert_complete(f: &Fixture, route: &native::foraging::FoodRoute) {
    let sim = &f.session.sim;
    let at = &f.game.members[&f.actor];
    let profile = &sim.genesis().rules.step_profiles[&f.walk];
    let terrain = sim
        .site_geometry(at.site.0, &at.window, profile.places)
        .unwrap();
    let document = sim
        .state()
        .population
        .get(f.actor)
        .unwrap()
        .body
        .as_ref()
        .unwrap();
    let blocked: Vec<_> = sim
        .state()
        .spatial_poses
        .iter()
        .filter(|(id, p)| **id != f.actor && p.site == at.site.0)
        .flat_map(|(id, p)| {
            native::embodiment::boxes(
                sim.state()
                    .population
                    .get(*id)
                    .unwrap()
                    .body
                    .as_ref()
                    .unwrap(),
                &p.pose,
            )
            .unwrap()
        })
        .collect();
    let walking=native::embodiment::walking_rules(sim);
    for (index,pair) in route.path.windows(2).enumerate() {
        assert!(
            native::embodiment::neighbours_with_walking(
                &terrain.volume,
                &terrain.places,
                profile,
                document,
                &pair[0],walking
            )
            .contains(&pair[1])
        );
        native::embodiment::check_walk(&terrain.volume, document, &pair[0], &pair[1], walking,
            route.strides.get(index).and_then(Option::as_ref), &blocked)
            .unwrap();
    }
    let target = &sim.state().spatial_poses[&route.target].pose;
    let food = native::embodiment::boxes(
        sim.state()
            .population
            .get(route.target)
            .unwrap()
            .body
            .as_ref()
            .unwrap(),
        target,
    )
    .unwrap();
    assert_eq!(
        native::foraging::gap(
            &native::embodiment::boxes(document, route.path.last().unwrap()).unwrap(),
            &food
        ),
        0
    );
}

#[test]
fn edited_fork_cannot_reuse_another_worlds_clear_route() {
    let original = without_sight(control(true, 256, true));
    let route = original
        .session
        .sim
        .food_route(original.actor, &original.walk)
        .unwrap();
    assert_complete(&original, route.route.as_ref().unwrap());
    let mut blocked = Fixture {
        session: original.session.clone(),
        game: original.game.clone(),
        actor: original.actor,
        food: original.food,
        walk: original.walk.clone(),
        feed: original.feed.clone(),
    };
    blocked
        .session
        .command(wall(&blocked, isometer_space::Op::Fill(2)))
        .unwrap();
    blocked.game.refresh(&mut blocked.session).unwrap();
    assert!(
        blocked
            .session
            .sim
            .food_route(blocked.actor, &blocked.walk)
            .unwrap()
            .route
            .is_none()
    );
    assert_eq!(
        original
            .session
            .sim
            .food_route(original.actor, &original.walk)
            .unwrap(),
        route
    );
    blocked
        .session
        .command(wall(&blocked, isometer_space::Op::Carve))
        .unwrap();
    blocked.game.refresh(&mut blocked.session).unwrap();
    let restored = blocked
        .session
        .sim
        .food_route(blocked.actor, &blocked.walk)
        .unwrap();
    assert_complete(&blocked, restored.route.as_ref().unwrap());
}

#[test]
fn unplaced_cohort_siblings_cannot_supply_a_route_goal() {
    let f = control(true, 256, true);
    assert!(
        !f.session
            .sim
            .state()
            .spatial_poses
            .contains_key(&(f.food + 1))
    );
    let route = f.session.sim.food_route(f.actor, &f.walk).unwrap();
    assert_eq!(route.route.as_ref().unwrap().target, f.food);
    assert_complete(&f, route.route.as_ref().unwrap());
}

#[test]
fn changing_a_residents_body_invalidates_a_cached_route_without_changing_terrain() {
    let mut f = control(true, 256, true);
    let at = f.game.members[&f.actor].clone();
    let terrain = f
        .session
        .sim
        .site_geometry(at.site.0, &at.window, at.places.unwrap())
        .unwrap();
    let route = f
        .session
        .sim
        .food_route(f.actor, &f.walk)
        .unwrap()
        .route
        .unwrap();
    assert_complete(&f, &route);
    // Admitted geometry can change independently of terrain. A broad new food
    // part now occupies the old approach, which cannot reuse its old witness.
    let mut body = f
        .session
        .sim
        .state()
        .population
        .get(f.food)
        .unwrap()
        .body
        .clone()
        .unwrap();
    body.parts[0].half_extent[0] += 8;
    f.session
        .command(Command::Embody {
            entity: f.food,
            body,
        })
        .unwrap();
    let same = f
        .session
        .sim
        .site_geometry(at.site.0, &at.window, at.places.unwrap())
        .unwrap();
    assert!(std::sync::Arc::ptr_eq(&terrain, &same));
    let now = f.session.sim.food_route(f.actor, &f.walk).unwrap();
    assert_ne!(now.route.as_ref(), Some(&route));
    if let Some(route) = &now.route {
        assert_complete(&f, route);
    }
}

#[test]
fn recorded_rules_without_reachability_still_make_the_original_attempt() {
    let f = control(true, 1, true);
    let walk = f.walk.clone();
    let mut f = rebuild(f, |g| {
        g.rules
            .spatial_templates
            .as_mut()
            .unwrap()
            .foraging
            .get_mut(&walk)
            .unwrap()
            .reachable = false
    });
    let wire = serde_json::to_value(f.session.sim.foraging_policy(&walk).unwrap()).unwrap();
    assert!(wire.get("reachable").is_none());
    let decoded: native::foraging::Foraging = serde_json::from_value(wire).unwrap();
    assert!(!decoded.reachable);
    f.tick();
    assert!(f.session.entries.iter().any(|entry| match &entry.command {
        Command::AdvanceBirthSpatial { steps, .. } =>
            steps.iter().any(|step| step.request.actor == f.actor
                && step.request.process == f.walk
                && step.geometry.is_err()),
        _ => false,
    }));
    assert_eq!(f.steps(), 0);
    f.game.validate_replay(&f.session).unwrap();
}

#[test]
fn another_residents_geometry_cannot_reuse_a_clear_route_from_a_shared_cache() {
    let f = control(true, 256, true);
    let (actor, food) = (f.actor, f.food);
    let mut blocker = 0;
    let original = rebuild(f, |g| {
        let entity = g.population.get(actor).unwrap().clone();
        let site = entity.resident_site().unwrap();
        let t = g.rules.spatial_templates.as_ref().unwrap();
        let terrain = isocosm::Simulation::new(g.clone(), Execution::Grouped)
            .unwrap()
            .site_volume(site, &t.window)
            .unwrap();
        let stands: Vec<_> = t.stands[&site]
            .iter()
            .filter(|s| s.actor == Some(actor) || s.actor == Some(food))
            .cloned()
            .collect();
        let occupied: Vec<_> = stands
            .iter()
            .flat_map(|s| {
                native::embodiment::boxes(
                    g.population
                        .get(s.actor.unwrap())
                        .unwrap()
                        .body
                        .as_ref()
                        .unwrap(),
                    &native::Pose {
                        at: s.at,
                        yaw: s.yaw,
                        feet: s.feet.clone(),
                    },
                )
                .unwrap()
            })
            .collect();
        let pose = native::embodiment::Grounding::new(&terrain)
            .with_articulation(true)
            .find(
                entity.body.as_ref().unwrap(),
                [t.window.min[0] + 24, 0, t.window.min[1] + 24],
                Default::default(),
                24,
                2,
                256,
                &occupied,
            )
            .unwrap();
        blocker = g.population.insert(entity, 1).unwrap();
        let t = g.rules.spatial_templates.as_mut().unwrap();
        t.stands.insert(
            site,
            stands
                .into_iter()
                .chain([native::Stand {
                    actor: Some(blocker),
                    at: pose.at,
                    yaw: pose.yaw,
                    feet: pose.feet,
                }])
                .collect(),
        );
        t.crossings.clear();
    });
    let at = &original.game.members[&actor];
    assert!(
        !original
            .session
            .sim
            .placement_food(actor, at.site.0)
            .contains_key(&blocker)
    );
    let clear = original
        .session
        .sim
        .food_route(actor, &original.walk)
        .unwrap();
    assert_complete(&original, clear.route.as_ref().unwrap());
    let mut changed = original.session.clone();
    let mut body = changed
        .sim
        .state()
        .population
        .get(blocker)
        .unwrap()
        .body
        .clone()
        .unwrap();
    // A new non-food body part enters the approach without changing the actor,
    // its meal, its senses or the terrain. The clone shares the warmed cache.
    body.attach(
        isometer_core::VolumeRef::from_tag(1),
        [160, 1, 160],
        isometer_core::Attachment {
            parent: body.root,
            offset: [0; 3],
            yaw: Default::default(),
        },
        None,
    )
    .unwrap();
    let terrain = original
        .session
        .sim
        .site_geometry(at.site.0, &at.window, at.places.unwrap())
        .unwrap();
    let seen = changed.sim.food_detection(actor, food, &original.walk);
    changed
        .command(Command::Embody {
            entity: blocker,
            body,
        })
        .unwrap();
    assert!(std::sync::Arc::ptr_eq(
        &terrain,
        &changed
            .sim
            .site_geometry(at.site.0, &at.window, at.places.unwrap())
            .unwrap()
    ));
    assert_eq!(
        changed.sim.food_detection(actor, food, &original.walk),
        seen
    );
    assert!(
        changed
            .sim
            .food_route(actor, &original.walk)
            .unwrap()
            .route
            .is_none()
    );
    assert_eq!(
        original
            .session
            .sim
            .food_route(actor, &original.walk)
            .unwrap(),
        clear
    );
}

// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Ordinary unchosen regression, without changing needs, terrain or anatomy.
use isocosm::spatial::{Pose, embodiment as body};
use std::collections::{BTreeMap, BTreeSet};
use isometer_space::places::Places;
use mesocosm_runtime::{Founded, Runtime};

fn ordinary(seed: u64, proportions: bool) -> Runtime {
    let mut founded = Founded::generated(seed, 60).unwrap();
    let mut domain = founded.genesis.founding.clone().unwrap();
    domain.ecology_rates = isocosm::ecology::Rates::A1;
    let b = domain.bodies.as_mut().unwrap();
    b.roster = true;
    b.founding = Some("SpacedRoster".into());
    b.proportions = proportions;
    founded.genesis = domain.generate().unwrap();
    // Every lineage decides each tick: these tests ask how a starter walks,
    // not how often it decides (D19 paces played worlds).
    for l in founded.genesis.lineages.values_mut() {
        l.decides = None;
    }
    founded.genesis.rules.epoch_ticks = 60;
    let sun = founded.genesis.rules.sun.clone().unwrap_or_default();
    founded.genesis.rules.sun = Some(isocosm::light::SunRules {
        start_hour: 7,
        ..sun
    });
    Runtime::bench_placed(founded, 2).unwrap()
}

#[test]
fn unchosen_hungry_starter_can_take_a_supported_food_step() {
    let mut runtime = ordinary(175_745_444_879_347_014, false);
    {
        let sim = runtime.sim();
        let game = runtime.spatial();
        let id = runtime.critter().unwrap();
        let at = &game.members[&id];
        let doc = sim
            .state()
            .population
            .get(id)
            .unwrap()
            .body
            .as_ref()
            .unwrap();
        let profile = &sim.genesis().rules.step_profiles["body:walk:ability:cycle-1"];
        let t = sim.genesis().rules.spatial_templates.as_ref().unwrap();
        let v = sim
            .volume(at.site.0, t.window.min, t.window.max, t.window.depth)
            .unwrap();
        let places = Places::derive(&v, profile.places).unwrap();
        let blocked: Vec<_> = game
            .members
            .iter()
            .filter(|(i, p)| **i != id && p.site == at.site)
            .flat_map(|(i, p)| {
                body::boxes(
                    sim.state()
                        .population
                        .get(*i)
                        .unwrap()
                        .body
                        .as_ref()
                        .unwrap(),
                    &p.pose,
                )
                .unwrap()
            })
            .collect();
        let swings: Vec<_> = body::neighbours(&v, &places, profile, doc, &at.pose)
            .into_iter()
            .map(|next| body::sweep_clear(&v, doc, &at.pose, &next, &blocked))
            .collect();
        assert!(
            swings.iter().any(Result::is_ok),
            "no supported swing: {swings:?}"
        );
    }
    runtime.step(1);
    let game = runtime.spatial();
    let records = &game.driver.as_ref().unwrap().records;
    assert!(
        records.iter().flat_map(|r| &r.changes).any(|c| matches!(
            c.admission,
            mesocosm_runtime::spatial::Admission::Step { .. }
        )),
        "hungry starter should take an admitted food step"
    );
    runtime.step(7);
    let sim = runtime.sim();
    let game = runtime.spatial();
    let id = runtime.critter().unwrap();
    let e = sim.state().population.get(id).unwrap();
    assert!(e.alive && e.body.is_some());
    // It later chooses native travel. The old site's historical pose must
    // withdraw on departure, rather than masquerading as another failed gait.
    assert_ne!(e.resident_site(), Some(game.members[&id].site.0));
    assert!(game.members[&id].realized(&sim).is_err());
}

#[test]
fn varied_unchosen_starter_walks_on_ordinary_needs_and_keeps_its_feet() {
    // First physically food-eligible seed from the B2 clock draw. Selection
    // did not inspect walking; the headed receipt shows its real swing frames.
    let mut runtime = ordinary(15_301_219_037_052_331_983, true);
    let walked = |runtime: &Runtime| runtime.spatial().driver.as_ref().unwrap().records.iter().flat_map(|r|&r.changes)
        .filter(|c|matches!(c.admission, mesocosm_runtime::spatial::Admission::Step { .. })).count();
    // Its feet are checked at the step it walks. It may depart in that same
    // tick (a frame fills over several advances, D40); its pose is realized
    // exactly while it is resident, and a departed pose withdraws (as the
    // hungry starter's test asks).
    for _ in 0..2 {
        runtime.step(1);
        if walked(&runtime) > 0 {
            break;
        }
    }
    assert!(walked(&runtime) > 0, "ordinary needs must admit a physical food step");
    let game = runtime.spatial();
    let id = runtime.critter().unwrap();
    assert_eq!(game.members[&id].pose.feet.len(), 6);
    let sim = runtime.sim();
    let resident = sim.state().population.get(id).unwrap().resident_site() == Some(game.members[&id].site.0);
    assert_eq!(game.members[&id].realized(&sim).is_ok(), resident);
}

#[test]
fn a_drawn_member_whose_food_is_crowded_takes_the_ordinary_search() {
    // D39: where no stance near its food is free, a drawn member takes the
    // search of a member with no food placed, and its food counts as crowded.
    let runtime = ordinary(175_745_444_879_347_014, false);
    let sim = runtime.sim();
    let site = sim.state().population.get(runtime.critter().unwrap()).unwrap().resident_site().unwrap();
    let t = sim.genesis().rules.spatial_templates.clone().unwrap();
    let mut placed: BTreeMap<u64, Pose> = sim.state().spatial_poses.iter()
        .filter(|(_, r)| r.site == site).map(|(id, r)| (*id, r.pose.clone())).collect();
    let far: Vec<u64> = sim.state().population.groups.iter()
        .filter(|(_, g)| g.entity.alive && g.entity.body.is_some() && g.entity.resident_site() == Some(site))
        .flat_map(|(&first, g)| first..first + g.count)
        .filter(|id| !placed.contains_key(id))
        .collect();
    // Members stand near their food by the arrival policy until it is full.
    for id in far {
        let food: Vec<u64> = sim.placement_food(id, site).into_keys().filter(|f| placed.contains_key(f)).collect();
        if let Ok(pose) = body::place_within(&sim, &t, id, site, &t.window, &placed) {
            placed.insert(id, pose);
            continue;
        }
        if food.is_empty() {
            continue;
        }
        let mut crowded = BTreeSet::new();
        let drawn = body::draw_within(&sim, &t, id, site, &t.window, &placed, &mut crowded);
        assert_eq!(crowded, BTreeSet::from([food]), "its food is found crowded");
        let skipped = body::draw_within(&sim, &t, id, site, &t.window, &placed, &mut crowded);
        assert_eq!(skipped.as_ref().ok(), drawn.as_ref().ok(), "crowded food goes straight to the same search");
        if drawn.is_ok() {
            return;
        }
    }
    panic!("no member found its food crowded and a stance elsewhere");
}

// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Ordinary unchosen regression, without changing needs, terrain or anatomy.
use isocosm::spatial::embodiment as body;
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
    runtime.step(2);
    let game = runtime.spatial();
    let walks = game.driver.as_ref().unwrap().records.iter().flat_map(|r|&r.changes)
        .filter(|c|matches!(c.admission, mesocosm_runtime::spatial::Admission::Step { .. })).count();
    assert!(walks > 0, "ordinary needs must admit a physical food step");
    let id = runtime.critter().unwrap();
    assert_eq!(game.members[&id].pose.feet.len(), 6);
    assert!(game.members[&id].realized(&runtime.sim()).is_ok());
}

// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Ordinary unchosen regression, without changing needs, terrain or anatomy.
use isocosm::spatial::embodiment as body;
use isometer_space::places::Places;
use mesocosm_runtime::{Founded, Runtime};

#[test]
fn unchosen_hungry_starter_can_take_a_supported_food_step() {
    let mut founded = Founded::generated(175_745_444_879_347_014, 60).unwrap();
    let mut domain = founded.genesis.founding.clone().unwrap();
    domain.ecology_rates = isocosm::ecology::Rates::A1;
    let b = domain.bodies.as_mut().unwrap();
    b.roster = true;
    b.founding = Some("SpacedRoster".into());
    founded.genesis = domain.generate().unwrap();
    founded.genesis.rules.epoch_ticks = 60;
    let sun = founded.genesis.rules.sun.clone().unwrap_or_default();
    founded.genesis.rules.sun = Some(isocosm::light::SunRules {
        start_hour: 7,
        ..sun
    });
    let mut runtime = Runtime::bench_placed(founded, 2).unwrap();
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

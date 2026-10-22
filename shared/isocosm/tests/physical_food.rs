// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Regression from the first unchosen foot-placement capture: a valid walker
//! was founded 98 cells from food, beyond its native 32-cell search.
use isocosm::{
    Execution, Simulation,
    spatial::{Pose, embodiment as body, foraging},
};

#[test]
fn scattered_founding_keeps_declared_consumers_near_native_edible_food_or_refuses() {
    let domain: isocosm::Founding =
        serde_json::from_str(include_str!("fixtures/physical_food_founding.json")).unwrap();
    let mut genesis = domain.generate().unwrap();
    body::declare(&mut genesis).unwrap();
    let sim = Simulation::new(genesis, Execution::Grouped).unwrap();
    let t = sim.genesis().rules.spatial_templates.as_ref().unwrap();
    for (&site, stands) in &t.stands {
        let volume = sim
            .volume(site, t.window.min, t.window.max, t.window.depth)
            .unwrap();
        for stand in stands {
            let id = stand.actor.unwrap();
            let eligible = sim.placement_food(id, site);
            if eligible.is_empty() {
                continue;
            }
            let doc = sim
                .state()
                .population
                .get(id)
                .unwrap()
                .body
                .as_ref()
                .unwrap();
            let pose = Pose {
                at: stand.at,
                yaw: stand.yaw,
                feet: stand.feet.clone(),
            };
            body::check_for(&sim, &volume, doc, &pose).unwrap();
            let actor = body::boxes(doc, &pose).unwrap();
            assert!(
                stands.iter().any(|food| {
                    let Some(&range) = eligible.get(&food.actor.unwrap()) else {
                        return false;
                    };
                    let doc = sim
                        .state()
                        .population
                        .get(food.actor.unwrap())
                        .unwrap()
                        .body
                        .as_ref()
                        .unwrap();
                    let food = body::boxes(
                        doc,
                        &Pose {
                            at: food.at,
                            yaw: food.yaw,
                            feet: food.feet.clone(),
                        },
                    )
                    .unwrap();
                    foraging::gap(&actor, &food) <= range
                }),
                "declared consumer {id} must have an actual edible placed meal within search"
            );
        }
    }
}

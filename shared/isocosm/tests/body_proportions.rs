// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use isocosm::{Founding, bodied::{Bodies, CAPITAL}, development::{develop, soma}};
use std::collections::BTreeSet;

fn founding(seed: u64) -> Founding {
    Founding { seed, lineages: 3, population: 24, ecology: true,
        bodies: Some(Bodies { founding: Some("SpacedRoster".into()),
            proportions: true, ..Bodies::default() }), ..Founding::default() }
}

#[test]
fn native_shapes_vary_and_keep_paid_feet_and_life_history() {
    let mut producers = BTreeSet::new();
    let mut consumers = BTreeSet::new();
    for seed in 1..=20 {
        let f = founding(seed);
        let g = f.generate().unwrap();
        g.validate().unwrap();
        assert_eq!(serde_json::to_value(&g).unwrap(), serde_json::to_value(f.generate().unwrap()).unwrap());
        for (name, lineage) in &g.lineages {
            let Some(d) = &lineage.development else { continue; };
            let recipe = serde_json::to_string(&d.recipe).unwrap();
            if lineage.kingdom == "kingdom:flora" { producers.insert(recipe); }
            else {
                consumers.insert(recipe);
                let legs: Vec<_> = d.recipe.tagmata.iter().filter(|t|!t.chain.is_empty()).collect();
                assert!(!legs.is_empty(), "seed {seed}, {name}");
                for t in legs {
                    assert_eq!(t.segments, 1, "fixed leg stations");
                    assert_eq!(t.variance, Some(0));
                    assert_eq!(t.segment, if lineage.traits.contains(CAPITAL) { "kind:block" } else { "kind:broad" });
                    assert_eq!(t.chain.iter().map(|l|l.kind.as_str()).collect::<Vec<_>>(),
                        ["kind:leg", "kind:upright_leg", "kind:foot"]);
                }
            }
            // The saved lineage develops new bodies using the same paid
            // geometry, without a host-only scale or a second shape catalogue.
            let d2 = serde_json::from_str(&serde_json::to_string(d).unwrap()).unwrap();
            for offspring_seed in [1, 7] {
                let s = soma(&g.rules, &d.recipe, offspring_seed);
                assert_eq!(develop(&g.rules, d, &s).unwrap(), develop(&g.rules, &d2, &s).unwrap());
            }
        }
    }
    assert!(producers.len() >= 5, "{producers:?}");
    assert!(consumers.len() >= 5, "{consumers:?}");
}

#[test]
fn absent_proportions_preserves_the_saved_founding_and_recipe() {
    let mut value = serde_json::to_value(founding(3)).unwrap();
    value["bodies"].as_object_mut().unwrap().remove("proportions");
    let old: Founding = serde_json::from_value(value.clone()).unwrap();
    assert!(!old.bodies.as_ref().unwrap().proportions);
    assert_eq!(serde_json::to_value(&old).unwrap(), value);
    let g = old.generate().unwrap();
    let d = g.lineages["lineage:1"].development.as_ref().unwrap();
    assert_eq!(d.recipe.tagmata[6].segments, 5, "the historical long browser is unchanged");
}

#[test]
fn shorter_paid_bodies_can_stand_and_swing_without_changing_terrain() {
    use isocosm::{Execution, Simulation, map::{Grid, Layout}, spatial::embodiment::{Grounding, sweep_clear}};
    use isometer_core::Yaw;
    let terrain = Founding { seed: 7, sites: 4, population: 8,
        map: Some(Layout::Grid(Grid { shape: "shape:plane".into(), width: 2, height: 2,
            side: 768, elevation: [16,16], relief: [0,0], sea_per_mille: 0 })),
        ..Founding::default() }.generate().unwrap();
    let sim = Simulation::new(terrain, Execution::Individuals).unwrap();
    let volume = sim.volume(0, [350;2], [418;2], 16).unwrap();
    let mut grounding = Grounding::new(&volume).with_articulation(true);
    for seed in 1..=5 {
        let g = founding(seed).generate().unwrap();
        let d = g.lineages["lineage:1"].development.as_ref().unwrap();
        let mut s = soma(&g.rules, &d.recipe, seed);
        s.absent.clear();
        let body = develop(&g.rules, d, &s).unwrap();
        for yaw in [Yaw::Zero, Yaw::Quarter] {
            let from = grounding.at(&body.doc, 384, 384, yaw).unwrap();
            let to = grounding.at(&body.doc, 385, 384, yaw).unwrap();
            sweep_clear(&volume, &body.doc, &from, &to, &[])
                .unwrap_or_else(|why|panic!("seed {seed}, {yaw:?}: {why}"));
        }
    }
}

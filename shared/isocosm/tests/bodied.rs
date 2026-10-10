// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Checkpoint 8, step 8g: bodies for worlds that ask for them (rulings 511,
//! 512, 525, 531 and 547 to 550). Flora draws as Mesocosm's producers and
//! fauna as its consumers, myco and micro staying bodiless; every recipe
//! reproduces in one cell of its root; life histories and recipes are drawn
//! within the founding's bounds; the authored bodies are presets; founders
//! develop with their matter in their parts; and the world runs alike
//! grouped and individually.

use isocosm::{
    Execution, Founding, Simulation, anatomy,
    bodied::{self, Bodies},
    rules::Facing,
    simulation::Genesis,
};
use std::collections::BTreeSet;

fn founding(seed: u64, bodies: Bodies) -> Founding {
    Founding {
        seed,
        lineages: 4,
        bodies: Some(bodies),
        ..Founding::default()
    }
}

fn world(seed: u64) -> Genesis {
    founding(seed, Bodies::default()).generate().unwrap()
}

fn development(g: &Genesis, lineage: &str) -> Option<isocosm::rules::Development> {
    g.lineages[lineage].development.clone()
}

#[test]
fn flora_and_fauna_draw_recipes_that_reproduce_and_myco_and_micro_wait() {
    let g = world(3);
    let kingdom = |l: &str| g.lineages[l].kingdom.clone();
    for l in ["lineage:0", "lineage:1", "lineage:2", "lineage:3"] {
        let d = development(&g, l);
        match kingdom(l).as_str() {
            "kingdom:flora" | "kingdom:fauna" => {
                let d = d.expect("a recipe");
                let root = &d.recipe.tagmata[0].segment;
                assert!(root.ends_with("-reproducing"), "{l}: {root}");
                assert_eq!(g.rules.kinds[root].cells["function:reproduce"], 1);
                assert!(d.recipe.kinds().is_subset(&d.lexicon));
            },
            _ => assert!(d.is_none(), "{l} waits for territories and surfaces"),
        }
    }
    let flora = development(&g, "lineage:0").unwrap();
    assert!(
        flora
            .recipe
            .tagmata
            .iter()
            .any(|t| t.bears.as_deref() == Some("kind:frond"))
    );
    assert!(flora.recipe.tagmata[0].bears.is_none(), "a bare head");
    let fauna = development(&g, "lineage:1").unwrap();
    assert_eq!(
        fauna.recipe.tagmata[0].socket,
        Facing::Below,
        "a mouth under the head"
    );
    // Founders are developed, their own matter in their parts.
    let mut founders = 0;
    for c in g.population.groups.values() {
        let e = &c.entity;
        if development(&g, &e.lineage).is_none() {
            continue;
        }
        founders += 1;
        assert!(
            e.parts
                .keys()
                .all(|id| e.situs(*id).is_some() && e.bodied(*id))
        );
        let own = e
            .accounts
            .keys()
            .any(|k| k.starts_with(&format!("matter:{}", &e.lineage[8..])));
        assert!(!own, "its own matter sits in its parts");
        assert!(anatomy::books(e).keys().any(|k| k.starts_with("matter:")));
    }
    assert!(founders > 0);
}

#[test]
fn a_bodied_world_runs_alike_grouped_and_individually() {
    for seed in [3, 4] {
        let g = world(seed);
        let mut one = Simulation::new(g.clone(), Execution::Individuals).unwrap();
        let mut all = Simulation::new(g, Execution::Grouped).unwrap();
        let before = one.matter();
        for tick in 1..=6 {
            one.advance(1).unwrap();
            all.advance(1).unwrap();
            assert_eq!(
                one.state_hash(),
                all.state_hash(),
                "seed {seed}, tick {tick}"
            );
        }
        assert_eq!(all.matter(), before, "seed {seed}");
    }
}

#[test]
fn draws_stay_within_the_bounds_and_cover_them() {
    let (mut variance, mut absence, mut clutch) =
        (BTreeSet::new(), BTreeSet::new(), BTreeSet::new());
    let mut traits: BTreeSet<String> = BTreeSet::new();
    let mut lacking: BTreeSet<String> = BTreeSet::new();
    for seed in 0..60 {
        let g = world(seed);
        for l in ["lineage:0", "lineage:1"] {
            let d = development(&g, l).unwrap();
            variance.insert(d.recipe.variance);
            absence.insert(d.recipe.absence);
            clutch.insert(d.clutch);
            let carried = &g.lineages[l].traits;
            assert!(
                [bodied::BROOD, bodied::EGG, bodied::BUD]
                    .iter()
                    .any(|s| carried.contains(*s)),
                "seed {seed}, {l}: no strategy"
            );
            for t in bodied::TRAITS {
                if carried.contains(t) {
                    traits.insert(t.into());
                } else {
                    lacking.insert(t.into());
                }
            }
        }
    }
    assert_eq!(variance, BTreeSet::from([1, 2]));
    assert_eq!(absence, BTreeSet::from([[1, 12]]));
    assert_eq!(clutch, BTreeSet::from([1, 2, 3, 4]));
    let all: BTreeSet<String> = bodied::TRAITS.map(String::from).into();
    assert_eq!(traits, all, "every trait drawn somewhere");
    assert_eq!(lacking, all, "and lacked somewhere");
    // The control: widened bounds draw past Mesocosm's.
    let wide = Bodies {
        variance: [0, 3],
        clutch: [1, 12],
        ..Bodies::default()
    };
    let mut seen = BTreeSet::new();
    for seed in 0..60 {
        let g = founding(seed, wide.clone()).generate().unwrap();
        seen.insert(development(&g, "lineage:0").unwrap().recipe.variance);
    }
    assert!(seen.contains(&0) && seen.contains(&3), "{seen:?}");
}

#[test]
fn the_same_founding_founds_the_same_world_and_seeds_vary_it() {
    assert_eq!(world(5), world(5));
    let recipes: BTreeSet<String> = (0..10)
        .map(|s| format!("{:?}", development(&world(s), "lineage:0").unwrap().recipe))
        .collect();
    assert!(recipes.len() > 1);
}

#[test]
fn the_authored_bodies_are_presets() {
    let presets = Bodies {
        roster: true,
        ..Bodies::default()
    };
    let g = founding(3, presets).generate().unwrap();
    let authored = bodied::roster();
    let producer = &authored.iter().find(|a| a.1).unwrap().2;
    let consumer = &authored.iter().find(|a| !a.1).unwrap().2;
    for (l, of) in [("lineage:0", producer), ("lineage:1", consumer)] {
        let d = development(&g, l).unwrap();
        assert_eq!(d.recipe.tagmata.len(), of.tagmata.len(), "{l}");
        assert_eq!(d.recipe.tagmata[1..], of.tagmata[1..], "{l}");
        assert_eq!(
            d.recipe.tagmata[0].segment,
            format!("{}-reproducing", of.tagmata[0].segment)
        );
    }
    // Each authored body is a valid recipe over the default kinds.
    let kinds = bodied::default_kinds();
    for (name, _, r) in &authored {
        assert!(r.kinds().iter().all(|k| kinds.contains_key(k)), "{name}");
    }
}

// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Pinned before the growth grammar (lineage-rounds LR6): these bodies must
//! not change. Each of Mesocosm's eight authored bodies, and each developed
//! lineage of generated founding worlds for seeds 1 to 5, develops at its
//! recipe's own segment counts and at drawn somas for seeds 1, 2 and 3; one
//! digest per body, and one over them all.

use cosmograph::{
    Founding, bodied,
    bodied::Bodies,
    development::{Soma, develop, soma},
    rules::{Development, Kingdoms, Policy, Recipe, Rules},
};

const PINNED: &str = "a22558b3e3b8ecf898987b9ddd78ffc828db809f159ef163d3bb27c31655b30a";

fn authored(recipe: Recipe) -> Development {
    Development {
        lexicon: recipe.kinds(),
        recipe,
        policy: Policy::default(),
        domain: 0,
        clutch: 1,
        anamorphic: false,
        tracts: vec![],
        systems: Default::default(),
        comfort: 0,
    }
}

/// Digests of `d`'s bodies at its recipe's counts and at drawn somas.
fn bodies(out: &mut Vec<(String, String)>, label: &str, rules: &Rules, d: &Development) {
    let counts = Soma {
        segments: d.recipe.tagmata.iter().map(|t| t.segments).collect(),
        absent: vec![],
        seed: 0,
    };
    let mut somas = vec![("recipe".to_string(), counts)];
    for seed in 1..=3 {
        somas.push((format!("soma {seed}"), soma(rules, &d.recipe, seed)));
    }
    for (how, s) in somas {
        let body = develop(rules, d, &s).unwrap_or_else(|why| panic!("{label} {how}: {why}"));
        out.push((format!("{label} {how}"), cosmograph::digest(&(&body.doc, &body.parts))));
    }
}

/// Each body's label and digest, in a fixed order, with kingdom weights set
/// or not: weights change prices, never bodies (case 1232).
fn digests(weighted: bool) -> Vec<(String, String)> {
    let kingdoms = || weighted.then(Kingdoms::default);
    let mut all = vec![];
    let presets = Bodies {
        roster: true,
        ..Bodies::default()
    };
    let mut rules = Founding {
        seed: 3,
        lineages: 4,
        bodies: Some(presets),
        ..Founding::default()
    }
    .generate()
    .unwrap()
    .rules;
    rules.kingdoms = kingdoms();
    for (k, t) in bodied::default_kinds() {
        rules.kinds.entry(k).or_insert(t);
    }
    for (name, _, recipe) in bodied::roster() {
        bodies(&mut all, &format!("authored {name}"), &rules, &authored(recipe));
    }
    for seed in 1..=5 {
        let g = Founding {
            seed,
            lineages: 4,
            bodies: Some(Bodies::default()),
            ..Founding::default()
        }
        .generate()
        .unwrap();
        let mut world = g.rules.clone();
        world.kingdoms = kingdoms();
        for (key, l) in &g.lineages {
            if let Some(d) = &l.development {
                bodies(&mut all, &format!("world {seed} {key}"), &world, d);
            }
        }
    }
    assert!(all.len() >= 8 * 4 + 5 * 4, "{} bodies", all.len());
    all
}

#[test]
fn developed_bodies_are_pinned() {
    for weighted in [false, true] {
        pinned(&digests(weighted));
    }
}

fn pinned(all: &[(String, String)]) {
    let overall = cosmograph::digest(&all.iter().map(|b| &b.1).collect::<Vec<_>>());
    if overall != PINNED {
        for (label, digest) in all {
            eprintln!("{label}: {digest}");
        }
        eprintln!("overall: {overall}");
    }
    assert_eq!(overall, PINNED, "a developed body changed");
}

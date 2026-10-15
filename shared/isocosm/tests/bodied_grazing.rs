// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Ruling 905: a generated ecology's consumers graze. A drawn jaw admits only
//! fauna (832), and a generated ecology's only fauna is the consumer lineage
//! itself, so a jawed consumer feeds on its own kind (A1 findings, 2026-10-14).
//! `Bodies::grazing` keeps the limbs and makes the mouth intake; it is the
//! default for new worlds, and a saved founding that never named it keeps its
//! meaning and its digest.

use isocosm::{Founding, bodied::Bodies};
use std::collections::BTreeSet;

fn diet(seed: u64, bodies: Bodies) -> BTreeSet<String> {
    let g = Founding {
        seed,
        lineages: 3,
        ecology: true,
        bodies: Some(bodies),
        ..Founding::default()
    }
    .generate()
    .unwrap();
    let feeding = serde_json::to_value(&g.rules.processes["ecology:feed-1"].port_feeding).unwrap();
    feeding["sources"]["Only"]
        .as_array()
        .expect("a bodied consumer's meals are limited to its port's sources")
        .iter()
        .map(|v| v.as_str().unwrap().to_owned())
        .collect()
}

fn producers() -> BTreeSet<String> {
    BTreeSet::from(["lineage:0".to_owned()])
}

#[test]
fn by_default_a_bodied_consumer_always_eats_the_producers() {
    for seed in 0..40 {
        assert_eq!(diet(seed, Bodies::default()), producers(), "seed {seed}");
    }
}

#[test]
fn without_grazing_a_drawn_jaw_makes_a_consumer_eat_its_own_lineage() {
    let own = BTreeSet::from(["lineage:1".to_owned()]);
    let off = Bodies { grazing: false, ..Bodies::default() };
    let diets: Vec<_> = (0..40).map(|seed| diet(seed, off.clone())).collect();
    assert!(diets.iter().any(|d| *d == own), "the defect: some draws take a jaw");
    assert!(diets.iter().any(|d| *d == producers()), "and some take an intake mouth");
    assert!(diets.iter().all(|d| *d == own || *d == producers()));
}

#[test]
fn grazing_is_on_by_default_and_an_old_founding_keeps_its_meaning() {
    assert!(Bodies::default().grazing);
    let new = serde_json::to_value(Bodies::default()).unwrap();
    assert_eq!(new["grazing"], true);
    // A founding saved before 905 does not name it: it reads as off and is
    // written back without it, so its digest is the one it was saved under.
    let old: Bodies = serde_json::from_str(r#"{"variance":[1,2],"absence":[12,12],"clutch":[1,4]}"#).unwrap();
    assert!(!old.grazing);
    assert!(serde_json::to_value(&old).unwrap().get("grazing").is_none());
}

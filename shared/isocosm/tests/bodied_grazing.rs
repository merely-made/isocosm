// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A1's diet finding: a drawn jaw admits only fauna (832), and a generated
//! ecology's only fauna is the consumer lineage itself, so a jawed consumer
//! feeds on its own kind. `Bodies::grazing` keeps the limbs and makes the
//! mouth intake. It is off by default and then changes no digest.

use isocosm::{Founding, bodied::Bodies};
use std::collections::BTreeSet;

fn diet(seed: u64, grazing: bool) -> BTreeSet<String> {
    let g = Founding {
        seed,
        lineages: 3,
        ecology: true,
        bodies: Some(Bodies { grazing, ..Bodies::default() }),
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

#[test]
fn a_grazing_consumer_always_eats_the_producers() {
    for seed in 0..40 {
        assert_eq!(diet(seed, true), BTreeSet::from(["lineage:0".to_owned()]), "seed {seed}");
    }
}

#[test]
fn without_the_flag_a_drawn_jaw_makes_a_consumer_eat_its_own_lineage() {
    let own = BTreeSet::from(["lineage:1".to_owned()]);
    let producers = BTreeSet::from(["lineage:0".to_owned()]);
    let diets: Vec<_> = (0..40).map(|seed| diet(seed, false)).collect();
    assert!(diets.iter().any(|d| *d == own), "the defect: some draws take a jaw");
    assert!(diets.iter().any(|d| *d == producers), "and some take an intake mouth");
    assert!(diets.iter().all(|d| *d == own || *d == producers));
}

#[test]
fn the_flag_is_off_by_default_and_unwritten_when_off() {
    assert!(!Bodies::default().grazing);
    let json = serde_json::to_value(Bodies::default()).unwrap();
    assert!(json.get("grazing").is_none(), "{json}");
    let on = serde_json::to_value(Bodies { grazing: true, ..Bodies::default() }).unwrap();
    assert_eq!(on["grazing"], true);
}

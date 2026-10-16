// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Rulings 906 and 907. The generator's rates stay the stock rates, and the
//! A1 preset is named, never a default. Individual lifespans (906): from age
//! 60 each member of a lineage can die at 2.5 percent a tick, never past the
//! lineage's drawn lifespan, which is the ceiling; the hazard is declared only
//! where the ceiling is above maturity, so stock worlds do not carry it.

use isocosm::{
    Founding,
    ecology::Rates,
    rules::*,
    simulation::Genesis,
};

fn world(rates: Rates, lineages: u32) -> Genesis {
    Founding {
        seed: 5,
        sites: 3,
        population: 60,
        cohort_size: 2,
        lineages,
        ecology: true,
        ecology_rates: rates,
        ..Founding::default()
    }
    .generate()
    .unwrap()
}

fn age_of(p: &Process) -> u64 {
    p.requires
        .iter()
        .find_map(|q| if let Query::Age { at_least } = q { Some(*at_least) } else { None })
        .expect("an age requirement")
}

fn gate_of(p: &Process) -> u64 {
    p.requires
        .iter()
        .find_map(|q| if let Query::Account { at_least, .. } = q { Some(*at_least) } else { None })
        .expect("a birth gate")
}

fn meal_of(p: &Process) -> u64 {
    match &p.effects[..] {
        [Effect::Eat { amount: Amount::Fixed(n), .. }] => *n,
        other => panic!("a single meal, found {other:?}"),
    }
}

#[test]
fn the_defaults_are_the_stock_rates_and_unwritten() {
    assert_eq!(Rates::default(), Rates::Stock);
    let json = serde_json::to_value(Founding::default()).unwrap();
    assert!(json.get("ecology_rates").is_none(), "a stock founding hashes as before");
    let g = world(Rates::Stock, 3);
    let p = &g.rules.processes;
    for i in 0..3 {
        assert_eq!(gate_of(&p[&format!("ecology:birth-{i}")]), 30, "lineage {i}");
        assert_eq!(p[&format!("ecology:birth-{i}")].period, Some(7));
        assert_eq!(p[&format!("ecology:upkeep-{i}")].period, Some(5));
        let life = age_of(&p[&format!("ecology:age-{i}")]);
        assert!((20..=60).contains(&life), "lineage {i} lives {life}");
        assert!(!p.contains_key(&format!("ecology:hazard-{i}")), "no hazard under maturity");
    }
    assert_eq!(meal_of(&p["ecology:feed-1"]), 1);
}

#[test]
fn the_a1_preset_is_named_and_sets_exactly_the_candidates_rates() {
    let json = serde_json::to_value(Founding { ecology_rates: Rates::A1, ..Founding::default() }).unwrap();
    assert_eq!(json["ecology_rates"], "a1");
    let g = world(Rates::A1, 3);
    let p = &g.rules.processes;
    assert_eq!(gate_of(&p["ecology:birth-0"]), 12);
    assert_eq!(gate_of(&p["ecology:birth-1"]), 9, "consumers");
    assert_eq!(gate_of(&p["ecology:birth-2"]), 12);
    assert_eq!(p["ecology:birth-1"].period, Some(14));
    assert_eq!(p["ecology:birth-0"].period, Some(7));
    assert_eq!(p["ecology:upkeep-1"].period, Some(10));
    assert_eq!(p["ecology:upkeep-0"].period, Some(5));
    assert_eq!(meal_of(&p["ecology:feed-1"]), 3);
    assert_eq!(meal_of(&p["ecology:feed-2"]), 1, "decomposers keep one");
}

#[test]
fn an_individual_can_die_from_maturity_and_never_outlives_its_lineages_ceiling() {
    let g = world(Rates::A1, 3);
    let p = &g.rules.processes;
    for i in 0..3 {
        let ceiling = age_of(&p[&format!("ecology:age-{i}")]);
        assert!((100..=140).contains(&ceiling), "lineage {i}: {ceiling}");
        let hazard = &p[&format!("ecology:hazard-{i}")];
        assert_eq!(age_of(hazard), 60, "from maturity");
        let risk = hazard.risk.as_ref().expect("a risk");
        assert_eq!(risk.per_million, 25_000);
        assert_eq!(risk.effects, vec![Effect::Death]);
        assert!(hazard.effects.is_empty(), "the ordinary outcome does nothing");
        assert_eq!(hazard.priority, p[&format!("ecology:age-{i}")].priority + 1);
    }
}

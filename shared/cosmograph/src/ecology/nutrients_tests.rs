// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Nutrient axes as a founding rule (D39 model 1): unset it changes nothing,
//! set it round trips and adds exactly the accounts, sorting and retuned
//! fixing the module doc names, bad values and worlds are named, and a line's
//! need is a fixed proportion that differs by line.

use super::*;
use crate::{bodied::Bodies, directing::Played};

pub(super) fn founding(rule: Option<Nutrients>) -> Founding {
    let grid = crate::map::Grid { width: 3, height: 2, ..crate::map::Grid::drawn(5) };
    Founding {
        seed: 5,
        population: 256,
        sites: grid.width * grid.height,
        map: Some(crate::map::Layout::Grid(grid)),
        ecology: true,
        ecology_rates: Rates::A1,
        bodies: Some(Bodies::default()),
        played: Some(Played { lineage: 1, region_sites: 2 }),
        lineages: 4,
        nutrients: rule,
        ..Default::default()
    }
}

#[test]
fn unset_founds_and_hashes_as_before() {
    let plain = founding(None);
    let json = serde_json::to_string(&plain).unwrap();
    assert!(!json.contains("nutrients"), "no mention of the rule");
    let bare: Founding = serde_json::from_str(&json).unwrap();
    let (a, b) = (plain.generate().unwrap(), bare.generate().unwrap());
    assert_eq!(crate::digest(&a), crate::digest(&b));
    assert!(a.rules.nutrients.is_none());
    assert!(!serde_json::to_string(&a.rules).unwrap().contains("nutrients"));
    assert!(!a.rules.processes.keys().any(|k| k.starts_with("nutrients:")));
    assert!(!a.rules.accounts.contains_key("world:soil-1"));
    assert!(!a.rules.conditions.iter().any(|k| k.starts_with("nutrients:")));
    assert!(a.sites.values().all(|s| s.accounts.keys().all(|k| !k.starts_with("world:soil-"))));
    // A producer's act is the founded one: two soil, to two tissue.
    let produce = &a.rules.processes["ecology:produce-0"];
    assert!(produce.effects.iter().any(|e| matches!(e, Effect::Transfer { account, amount, .. } if account == "world:soil" && amount == &Amount::Fixed(2))));
}

#[test]
fn it_round_trips_and_adds_the_nutrients_the_sorting_and_nothing_else() {
    let f = founding(Some(Nutrients::default()));
    let json = serde_json::to_string(&f).unwrap();
    assert_eq!(serde_json::from_str::<Founding>(&json).unwrap(), f);
    let (plain, g) = (founding(None).generate().unwrap(), f.generate().unwrap());
    assert_eq!(g.rules.nutrients, Some(Nutrients::default()));
    // One further nutrient, sorted in three chunks; and a tuned draw of each nutrient for the three lines (two producers, a decomposer) that could bear the organ.
    assert_eq!(g.rules.processes.len(), plain.rules.processes.len() + 3 + 6);
    assert!(g.rules.processes.contains_key("nutrients:tuned-2-1") && !g.rules.processes.contains_key("nutrients:tuned-1-0"), "no grazer draws");
    for chunk in [64, 8, 1] {
        assert!(g.rules.processes.contains_key(&format!("nutrients:sort-1-{chunk}")));
    }
    assert!(g.rules.accounts.contains_key("world:soil-1") && g.rules.conditions.contains("nutrients:owed-1"));
    for (id, site) in &g.sites {
        // The soil is as it was founded and the other nutrient is a share of it.
        assert_eq!(site.accounts["world:soil"], plain.sites[id].accounts["world:soil"]);
        let (soil, other) = (site.accounts["world:soil"], site.accounts["world:soil-1"]);
        assert!(other * 100 >= soil * 30 && other * 100 <= soil * 170 + 99, "{other} of {soil}");
    }
    let held: std::collections::BTreeSet<u64> = g.sites.values().map(|s| s.accounts["world:soil-1"] * 1000 / s.accounts["world:soil"]).collect();
    assert!(held.len() > 1, "sites differ in what they hold");
    assert_eq!(g.population, plain.population, "the rule gives no body anything");
    assert!(g.rules.validate().is_ok());
    // Four lines: two producers (0 and 3), which need the nutrients two to one and one to two.
    let takes = |line: u32, account: &str| {
        let p = &g.rules.processes[&format!("ecology:produce-{line}")];
        p.effects.iter().find_map(|e| match e {
            Effect::Transfer { account: a, amount, from: Binding::Place, .. } if a == account => amount.resolved().ok(),
            _ => None,
        })
    };
    assert_eq!([takes(0, "world:soil"), takes(0, "world:soil-1")], [Some(2), Some(1)]);
    assert_eq!([takes(3, "world:soil"), takes(3, "world:soil-1")], [Some(1), Some(2)]);
    assert_ne!(crate::digest(&g), crate::digest(&plain));
}

#[test]
fn the_need_is_a_fixed_proportion_that_differs_by_line() {
    let n = Nutrients::default();
    assert_eq!([n.need(0, 0), n.need(0, 1)], [2, 1]);
    assert_eq!([n.need(3, 0), n.need(3, 1)], [1, 2]);
    assert_eq!(n.yields(0), 3);
    let flat = Nutrients { tilt: 0, ..n };
    assert_eq!([flat.need(0, 0), flat.need(3, 1)], [1, 1]);
    let sole = Nutrients { base: 0, tilt: 2, ..n };
    assert_eq!([sole.need(0, 0), sole.need(0, 1), sole.need(3, 0), sole.need(3, 1)], [2, 0, 0, 2]);
    let wide = Nutrients { axes: 3, ..n };
    assert_eq!((0..3).map(|a| wide.need(4, a)).collect::<Vec<_>>(), [1, 2, 1], "line 4 leads on 4 % 3");
    assert_eq!(n.held(1000, 0), 300);
    assert_eq!(n.held(1000, 140), 1000 * (30 + 140 % 141) / 100);
}

#[test]
fn bad_values_and_worlds_are_refused_by_name() {
    let bad = |n: Nutrients| founding(Some(n)).generate().unwrap_err();
    assert!(bad(Nutrients { axes: 1, ..Nutrients::default() }).contains("axes"));
    assert!(bad(Nutrients { axes: 5, ..Nutrients::default() }).contains("axes"));
    assert!(bad(Nutrients { base: 0, tilt: 0, ..Nutrients::default() }).contains("nothing"));
    assert!(bad(Nutrients { spread: [0, 100], ..Nutrients::default() }).contains("spread"));
    assert!(bad(Nutrients { spread: [90, 80], ..Nutrients::default() }).contains("spread"));
    assert!(bad(Nutrients { spread: [10, 1001], ..Nutrients::default() }).contains("spread"));
    assert!(bad(Nutrients { uptake: 0, ..Nutrients::default() }).contains("uptake"));
    assert!(bad(Nutrients { period: 0, ..Nutrients::default() }).contains("period"));
    let world_time = Founding { ecology_rates: Rates::WorldTime, ..founding(Some(Nutrients::default())) };
    assert!(world_time.generate().unwrap_err().contains("world time"));
    let bare = Founding { bodies: None, ..founding(Some(Nutrients::default())) };
    assert!(bare.generate().unwrap_err().contains("bodied"));
    assert!(Founding { ecology: false, ..founding(Some(Nutrients::default())) }.generate().is_err());
}

#[test]
fn sensing_plants_need_their_soil_share_to_cover_the_first_nutrient_only() {
    let sensing = |rule| Founding { bodies: Some(Bodies { sensing: true, ..Bodies::default() }), ..founding(rule) };
    let plain = sensing(None).generate().unwrap();
    let g = sensing(Some(Nutrients::default())).generate().unwrap();
    let share = |g: &crate::simulation::Genesis, line: u32| {
        g.rules.processes[&format!("ecology:produce-{line}")].requires.iter().find_map(|q| match q {
            Query::Sensed { reading, at_least } if reading == "sense:soil-share" => Some(*at_least),
            _ => None,
        })
    };
    assert_eq!([share(&plain, 0), share(&plain, 3)], [Some(2), Some(2)]);
    assert_eq!([share(&g, 0), share(&g, 3)], [Some(2), Some(1)]);
}

// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::tests::grazer;
use super::*;
use crate::rules::{Recipe, default_systems};
use std::collections::{BTreeMap, BTreeSet};

fn recipe(riff: [u32; 2], vary: [u32; 2]) -> Recipe {
    Recipe {
        tagmata: vec![],
        variance: 0,
        absence: [0, 1],
        riff,
        vary,
    }
}

fn rules() -> crate::rules::Rules {
    let mut r = crate::probe::BodyFounding::default()
        .generate()
        .expect("a probe world")
        .genesis
        .rules;
    r.systems = default_systems();
    r
}

fn sited(mut e: Entity) -> Entity {
    for (id, p) in e.parts.iter_mut() {
        p.situs = Some([0, *id as u8, 0]);
    }
    e
}

#[test]
fn a_founded_body_carries_the_defaults_it_realizes() {
    let (r, e) = (rules(), grazer());
    assert_eq!(founded(&e, &r), e.systems);
}

#[test]
fn a_developed_gland_brings_its_system_and_a_varied_cell_does_not() {
    let r = rules();
    let mut e = grazer();
    let lump = e.parts.get_mut(&0).unwrap();
    lump.cells.insert("function:fix".into(), 1);
    lump.functions.insert("function:fix".into());
    let before = e.clone();
    *e.parts
        .get_mut(&0)
        .unwrap()
        .cells
        .get_mut("function:store")
        .unwrap() += 1;
    take_up(&mut e, &r, &before);
    assert!(
        !e.systems.contains_key("system:photosynthetic"),
        "latent fix"
    );
    let before = e.clone();
    let lump = e.parts.get_mut(&0).unwrap();
    lump.cells.insert("function:secrete".into(), 1);
    lump.functions.insert("function:secrete".into());
    take_up(&mut e, &r, &before);
    assert!(e.systems.contains_key("system:glandular"));
    assert!(!e.systems.contains_key("system:photosynthetic"));
}

#[test]
fn varied_cells_pass_down_where_their_parts_lie() {
    let e = sited(grazer());
    let varied = vec![
        Varied {
            situs: [0, 1, 0],
            from: "function:contract".into(),
            to: "function:fix".into(),
        },
        // A part this body never develops takes its cell with it.
        Varied {
            situs: [0, 9, 0],
            from: "function:contract".into(),
            to: "function:fix".into(),
        },
    ];
    let mut parts = e.parts.clone();
    inherit(&mut parts, &varied);
    assert_eq!(parts[&1].cells["function:contract"], 1);
    assert_eq!(parts[&1].cells["function:fix"], 1);
    assert_eq!(parts[&2], e.parts[&2]);
    let mut limb = e.parts[&1].clone();
    regrow(&mut limb, &varied);
    assert_eq!(limb, parts[&1]);
}

#[test]
fn with_no_odds_nothing_varies_or_riffs() {
    let r = rules();
    for seed in 0..200 {
        let mut e = sited(grazer());
        let before = e.clone();
        assert_eq!(vary(&mut e, &r, &recipe([0, 1], [0, 1]), seed), None);
        assert_eq!(riff(&mut e, &recipe([0, 1], [0, 1]), seed), None);
        assert_eq!(e, before);
    }
}

#[test]
fn a_varied_cell_takes_any_grown_function_but_its_own() {
    let r = rules();
    let mut taken = BTreeSet::new();
    for seed in 0..2000 {
        let mut e = sited(grazer());
        let cells = |e: &Entity| -> u32 { e.parts.values().flat_map(|p| p.cells.values()).sum() };
        let before = cells(&e);
        let v = vary(&mut e, &r, &recipe([0, 1], [1, 1]), seed).expect("odds of one");
        assert_ne!(v.from, v.to);
        assert_eq!(cells(&e), before, "a cell moves; none is made");
        assert_eq!(e.varied, vec![v.clone()]);
        taken.insert(v.to);
    }
    let grown: BTreeSet<Key> = r
        .functions
        .iter()
        .filter(|(_, f)| f.seeding == crate::rules::Seeding::Grown)
        .map(|(k, _)| k.clone())
        .collect();
    assert_eq!(taken, grown);
    assert!(!taken.contains("function:secrete"));
}

#[test]
fn a_riff_swaps_or_adds_a_function_the_body_expresses() {
    let mut kinds = BTreeMap::new();
    for seed in 0..2000 {
        let mut e = grazer();
        let before = e.systems.clone();
        let Some(r) = riff(&mut e, &recipe([1, 1], [0, 1]), seed) else {
            continue;
        };
        let expressed: BTreeSet<&Key> = e.parts.values().flat_map(|p| &p.functions).collect();
        assert!(expressed.contains(&r.added));
        assert!(!before[&r.system].routes(&r.added, r.role));
        assert!(e.systems[&r.system].routes(&r.added, r.role));
        assert!(realizes(&e, &e.systems[&r.system]));
        *kinds.entry(r.swapped.is_some()).or_insert(0) += 1;
        let changed = e.systems.iter().filter(|(k, s)| before[*k] != **s).count();
        assert_eq!(changed, 1);
    }
    let (swaps, adds) = (kinds[&true], kinds[&false]);
    assert!(swaps > 700 && adds > 700, "{swaps} swaps, {adds} adds");
}

#[test]
fn a_riff_on_a_dormant_system_is_kept_only_where_it_wakes() {
    let mut e = grazer();
    // Limbless, its muscular system realizes nothing: dormant (583).
    e.parts.retain(|id, _| *id == 0 || *id == 3);
    e.systems.retain(|k, _| k == "system:muscular");
    assert!(!realizes(&e, &e.systems["system:muscular"]));
    for seed in 0..500 {
        let mut child = e.clone();
        let r = riff(&mut child, &recipe([1, 1], [0, 1]), seed);
        // A riff keeps only what the child realizes; adding contract or
        // swapping the effect for one it expresses can wake it.
        match r {
            Some(_) => assert!(realizes(&child, &child.systems["system:muscular"])),
            None => assert_eq!(child.systems, e.systems),
        }
    }
}

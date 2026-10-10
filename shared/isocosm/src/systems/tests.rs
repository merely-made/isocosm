// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::geometry::{Body, Frame};
use crate::rules::{Carriage, Rules, default_systems};
use std::collections::BTreeMap;

pub(super) fn rules(per_cell: u64) -> Rules {
    let mut r = crate::probe::BodyFounding::default()
        .generate()
        .expect("a probe world")
        .genesis
        .rules;
    r.carriage = Some(Carriage { per_cell });
    r
}

pub(super) fn part(
    parent: Option<u32>,
    half_extent: [i32; 3],
    cells: &[(&str, u32)],
) -> (Frame, Part) {
    let cells: BTreeMap<Key, u32> = cells
        .iter()
        .map(|(f, n)| (format!("function:{f}"), *n))
        .collect();
    let frame = Frame {
        parent: parent.map(PartId),
        half_extent,
        ..Default::default()
    };
    let part = Part {
        functions: cells.keys().cloned().collect(),
        cells,
        ..Default::default()
    };
    (frame, part)
}

/// A grazer: a lump that takes in, stores and reproduces, a limb on it, a
/// second limb on the first, and an eye.
pub(super) fn grazer() -> Entity {
    let mut e = crate::probe::BodyFounding::default()
        .generate()
        .expect("a probe world")
        .genesis
        .population
        .groups
        .into_values()
        .find(|c| c.entity.lineage == "lineage:1")
        .expect("a grazer")
        .entity;
    let body = Body::of([
        part(
            None,
            [2, 2, 2],
            &[("intake", 4), ("store", 2), ("reproduce", 2)],
        ),
        part(Some(0), [3, 1, 1], &[("contract", 2)]),
        part(Some(1), [3, 1, 1], &[("contract", 2)]),
        part(Some(0), [1, 1, 1], &[("sense", 1)]),
    ]);
    e.embody(body.expect("a grazer's body"));
    e.systems = default_systems()
        .into_iter()
        .filter(|(_, s)| realizes(&e, s))
        .collect();
    e
}

fn asks(each: &[(u32, u64)]) -> BTreeMap<PartId, u64> {
    each.iter().map(|(id, n)| (PartId(*id), *n)).collect()
}

#[test]
fn a_grazer_carries_the_systems_its_parts_realize() {
    let e = grazer();
    let carried: Vec<&str> = e.systems.keys().map(String::as_str).collect();
    let expected = [
        "system:digestive",
        "system:muscular",
        "system:nervous",
        "system:reproductive",
    ];
    assert_eq!(carried, expected);
}

#[test]
fn every_living_part_names_no_function() {
    let e = grazer();
    assert!(routes(&e, "function:intake", Role::Source));
    assert!(routes(&e, "function:contract", Role::Effect));
    // The muscular system's sources are every living part, the lump with
    // its intake cells among them, and still it routes no intake.
    assert!(!routes(&e, "function:intake", Role::Effect));
    assert!(!routes(&e, "function:fix", Role::Source));
}

#[test]
fn a_latent_cell_becomes_an_ability_only_once_a_system_routes_it() {
    let mut e = grazer();
    let lump = e.parts.get_mut(&PartId(0)).unwrap();
    *lump.cells.get_mut("function:intake").unwrap() -= 1;
    lump.cells.insert("function:fix".into(), 1);
    lump.functions.insert("function:fix".into());
    assert!(!routes(&e, "function:fix", Role::Source), "latent");
    let gut = e.systems.get_mut("system:digestive").unwrap();
    gut.sources.insert(Fill::Function("function:fix".into()));
    assert!(routes(&e, "function:fix", Role::Source), "riffed");
    assert!(routes(&e, "function:intake", Role::Source));
}

#[test]
fn an_intact_body_carries_all_it_asks() {
    let (r, e) = (rules(7), grazer());
    let c = carry(
        &e,
        &r,
        ("function:intake", Role::Source),
        &asks(&[(0, 10), (1, 6), (2, 6), (3, 2)]),
        None,
    )
    .unwrap();
    assert_eq!(c.total, 24);
    assert_eq!(c.parts, asks(&[(0, 10), (1, 6), (2, 6), (3, 2)]));
}

#[test]
fn a_cut_route_carries_nothing_beyond_it() {
    let (r, mut e) = (rules(7), grazer());
    e.body.as_mut().unwrap().parts[1].severed = true;
    let c = carry(
        &e,
        &r,
        ("function:intake", Role::Source),
        &asks(&[(0, 10), (2, 6), (3, 2)]),
        None,
    )
    .unwrap();
    assert_eq!(c.parts, asks(&[(0, 10), (2, 0), (3, 2)]));
}

#[test]
fn parts_share_the_trunk_that_carries_to_them() {
    // Each limb carries 2 cells' worth, 2 a tick: the second limb's 6 must
    // pass the first, so the two take 2 between them.
    let (r, e) = (rules(1), grazer());
    let c = carry(
        &e,
        &r,
        ("function:intake", Role::Source),
        &asks(&[(1, 6), (2, 6)]),
        None,
    )
    .unwrap();
    assert_eq!(c.total, 2);
}

#[test]
fn nothing_is_carried_where_the_world_sets_no_carriage() {
    let (mut r, e) = (rules(7), grazer());
    r.carriage = None;
    let c = carry(
        &e,
        &r,
        ("function:intake", Role::Source),
        &asks(&[(0, 10)]),
        None,
    )
    .unwrap();
    assert_eq!(c.total, 0);
}

#[test]
fn nothing_is_carried_through_a_system_the_body_does_not_carry() {
    let (r, e) = (rules(7), grazer());
    let c = carry(
        &e,
        &r,
        ("function:fix", Role::Source),
        &asks(&[(0, 10)]),
        None,
    )
    .unwrap();
    assert_eq!(c, Carried::default());
}

#[test]
fn a_conduct_cell_carries_by_its_cross_section() {
    let r = rules(10);
    // A lump's smallest face is 25 voxels, the reference segment's: a
    // conduct cell there carries as any cell.
    let lump = part(None, [2, 2, 2], &[("conduct", 2), ("store", 2)]);
    assert_eq!(capacity(lump.0.half_extent, &lump.1, &r), 40);
    // A rod's is 9: its conduct cells carry 9/25 of a cell's.
    let rod = part(None, [3, 1, 1], &[("conduct", 2)]);
    assert_eq!(capacity(rod.0.half_extent, &rod.1, &r), 2 * 10 * 9 / 25);
}

#[test]
fn the_parts_bitten_are_the_glands_effects() {
    let mut e = grazer();
    let lump = e.parts.get_mut(&PartId(0)).unwrap();
    lump.cells.insert("function:secrete".into(), 1);
    lump.functions.insert("function:secrete".into());
    let gland = default_systems()["system:glandular"].clone();
    assert!(realizes(&e, &gland));
    e.systems.insert("system:glandular".into(), gland);
    let r = rules(7);
    let to = |bitten| {
        carry(
            &e,
            &r,
            ("function:secrete", Role::Source),
            &asks(&[(1, 3), (2, 3)]),
            bitten,
        )
        .unwrap()
        .parts
    };
    assert_eq!(to(Some(PartId(2))), asks(&[(2, 3)]));
    assert_eq!(to(None), BTreeMap::new());
}

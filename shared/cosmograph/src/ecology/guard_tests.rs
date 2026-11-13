// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Guard as a founding rule: unset it changes nothing, set it round trips,
//! bad values are named, rent rides beside upkeep, and the readings (armour
//! by the strength rule, spines, glands) say what the rule says.

use super::*;
use crate::{
    bodied::Bodies,
    directing::Played,
    process::{IntakePort, NisKind, ProcessId, Registry, Strength, Tolerance},
    schema::Entity,
};

pub(super) fn founding(guard: Option<Guard>) -> Founding {
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
        web: Web::Diets,
        lineages: 5,
        guard,
        ..Default::default()
    }
}

#[test]
fn unset_founds_and_hashes_as_before() {
    let plain = founding(None);
    let json = serde_json::to_string(&plain).unwrap();
    assert!(!json.contains("guard"), "no mention of the rule");
    let bare: Founding = serde_json::from_str(&json).unwrap();
    let (a, b) = (plain.generate().unwrap(), bare.generate().unwrap());
    assert_eq!(crate::digest(&a), crate::digest(&b));
    assert!(a.rules.guard.is_none());
    assert!(!serde_json::to_string(&a.rules).unwrap().contains("guard"));
    assert!(!a.rules.processes.keys().any(|k| k.starts_with("guard:")));
    // No part's port declares a strength, and none serializes one.
    assert!(a.population.groups.values().flat_map(|g| g.entity.parts.values()).all(|p| p.port.is_none_or(|p| p.strength().is_none())));
    assert!(!serde_json::to_string(&IntakePort::live(NisKind::Producer)).unwrap().contains("strength"));
}

#[test]
fn it_round_trips_and_adds_rent_beside_upkeep_and_nothing_else_to_a_founded_body() {
    let f = founding(Some(Guard::default()));
    let json = serde_json::to_string(&f).unwrap();
    assert_eq!(serde_json::from_str::<Founding>(&json).unwrap(), f);
    let (plain, g) = (founding(None).generate().unwrap(), f.generate().unwrap());
    assert_eq!(g.rules.guard, Some(Guard::default()));
    for i in 0..5 {
        let (rent, upkeep) = (&g.rules.processes[&format!("guard:rent-{i}")], &g.rules.processes[&format!("ecology:upkeep-{i}")]);
        assert_eq!((rent.period, rent.priority), (upkeep.period, upkeep.priority), "rent falls due with upkeep");
    }
    assert_eq!(g.rules.processes.len(), plain.rules.processes.len() + 5);
    // The founded bodies are the same bodies: the rule gives nothing to them.
    assert_eq!(g.population, plain.population);
    assert!(g.rules.validate().is_ok());
}

#[test]
fn bad_values_and_worlds_are_refused_by_name() {
    let bad = |guard: Guard| founding(Some(guard)).generate().unwrap_err();
    assert!(bad(Guard { bite: 11, ..Guard::default() }).contains("nought to ten"));
    assert!(bad(Guard { gland: 11, ..Guard::default() }).contains("nought to ten"));
    assert!(bad(Guard { spine_harm: 0, ..Guard::default() }).contains("spine_harm"));
    assert!(bad(Guard { dose: 0, ..Guard::default() }).contains("dose"));
    assert!(bad(Guard { toxin_cap: 0, ..Guard::default() }).contains("toxin_cap"));
    assert!(bad(Guard { rent_cells: 0, ..Guard::default() }).contains("rent_cells"));
    let world_time = Founding { ecology_rates: Rates::WorldTime, ..founding(Some(Guard::default())) };
    assert!(world_time.generate().unwrap_err().contains("world time"));
    let bare = Founding { bodies: None, ..founding(Some(Guard::default())) };
    assert!(bare.generate().unwrap_err().contains("bodied"));
    let no_ecology = Founding { ecology: false, ..founding(Some(Guard::default())) };
    assert!(no_ecology.generate().is_err());
}

/// A lineage 0 body whose first part holds `cells` of `function` (a
/// function of the native registry, by name) under a port of `strength`.
pub(super) fn organ(function: &str, cells: u32, strength: Strength) -> (Entity, crate::schema::PartId) {
    let g = founding(None).generate().unwrap();
    let mut e = g.population.groups.values().find(|g| g.entity.lineage == "lineage:0").unwrap().entity.as_ref().clone();
    let id = *e.parts.keys().next().unwrap();
    let held = Registry::native().get(&ProcessId::new("mesocosm", function)).unwrap().reference();
    let cells = cells.min(crate::anatomy::capacity(e.extent(id)));
    let part = e.part_mut(&id).unwrap();
    part.cells = [(format!("function:{function}"), cells)].into();
    part.functions = [format!("function:{function}")].into();
    part.port = Some(IntakePort::none().with_strength(strength).supported_by(held));
    e.lay_out();
    (e, id)
}

const CRUSH: fn(u8) -> Strength = |crush| Strength { crush, ..Strength::NONE };
const PIERCE: fn(u8) -> Strength = |pierce| Strength { pierce, ..Strength::NONE };

#[test]
fn armour_resists_by_the_strength_rule_against_the_mouths_stronger_attack() {
    let rule = Guard::default();
    let (e, id) = organ("support", u32::MAX, Strength { crush: 6, pierce: 5, ..Strength::NONE });
    // A founded mouth (crush 2) meets crush armour 6: four tenths resisted.
    assert_eq!(rule.resisted(&e, id, rule.attack(IntakePort::none())), 4);
    // A jaw as strong as the armour, or stronger, is not resisted.
    assert_eq!(rule.resisted(&e, id, CRUSH(6)), 0);
    assert_eq!(rule.resisted(&e, id, CRUSH(9)), 0);
    // A piercing mouth is met by the pierce armour, not the crush: the counter is by kind.
    assert_eq!(rule.resisted(&e, id, PIERCE(3)), 2);
    assert_eq!(rule.resisted(&e, id, PIERCE(5)), 0);
    // Its stronger attack is the one that bites.
    assert_eq!(rule.resisted(&e, id, Strength { crush: 2, pierce: 3, ..Strength::NONE }), 2);
    // Armour is capped at nine tenths: none is proof against a bite.
    let (strong, id) = organ("support", u32::MAX, Strength { crush: 10, ..Strength::NONE });
    assert_eq!(rule.resisted(&strong, id, CRUSH(0)), 9);
}

#[test]
fn a_wounded_part_has_less_armour_and_a_mouth_is_no_armour() {
    let rule = Guard::default();
    let (whole, id) = organ("support", u32::MAX, Strength { crush: 8, ..Strength::NONE });
    let capacity = crate::anatomy::capacity(whole.extent(id));
    assert_eq!(rule.resisted(&whole, id, CRUSH(2)), 6);
    let (half, id) = organ("support", capacity / 2, Strength { crush: 8, ..Strength::NONE });
    assert_eq!(rule.resisted(&half, id, CRUSH(2)), 2, "half the tissue, half the armour");
    // A mouth's strength is its attack: biting a jaw is not biting armour.
    let (mut jaw, id) = organ("intake", u32::MAX, CRUSH(8));
    let port = jaw.parts[&id].port.unwrap().with_live(NisKind::Consumer);
    jaw.part_mut(&id).unwrap().port = Some(port);
    assert_eq!(rule.resisted(&jaw, id, CRUSH(2)), 0);
    // Armour whose tissue is gone is dormant.
    let (mut bare, id) = organ("support", u32::MAX, CRUSH(8));
    bare.part_mut(&id).unwrap().cells.clear();
    assert_eq!(rule.resisted(&bare, id, CRUSH(2)), 0);
}

#[test]
fn spines_wound_unless_tolerated_or_matched() {
    let rule = Guard::default();
    let (e, id) = organ("support", 3, Strength { spine: 4, ..Strength::NONE });
    let none = IntakePort::none();
    assert_eq!(rule.spined(&e, id, CRUSH(2), none), rule.spine_harm);
    assert_eq!(rule.spined(&e, id, CRUSH(9), none), rule.spine_harm, "crushing is no way round a spine");
    assert_eq!(rule.spined(&e, id, CRUSH(2), none.tolerating(Tolerance::Spines)), 0, "a tolerant body is spared");
    assert_eq!(rule.spined(&e, id, PIERCE(4), none), 0, "a stylet works between them");
    assert_eq!(rule.spined(&e, id, PIERCE(3), none), rule.spine_harm);
    let (bare, id) = organ("support", 3, Strength::NONE);
    assert_eq!(rule.spined(&bare, id, CRUSH(2), none), 0);
}

#[test]
fn a_gland_charges_its_eater_by_the_tissue_and_is_cleared_by_tolerance() {
    let rule = Guard::default();
    let none = IntakePort::none();
    let (e, _) = organ("secrete", 8, Strength { toxin: 5, ..Strength::NONE });
    assert_eq!(rule.toxic(&e, none), 3, "forty over a dose of sixteen, rounded up");
    assert_eq!(rule.toxic(&e, none.tolerating(Tolerance::Toxin)), 0);
    let (small, _) = organ("secrete", 2, Strength { toxin: 5, ..Strength::NONE });
    assert_eq!(rule.toxic(&small, none), 1, "any gland costs a cell");
    let (huge, _) = organ("secrete", 8, Strength { toxin: 10, ..Strength::NONE });
    assert_eq!(rule.toxic(&huge, none), 4, "capped");
    assert_eq!(Guard { toxin_cap: 2, ..rule }.toxic(&huge, none), 2);
    // Secreting cells that declare no strength use the rule's.
    let (mut plain, id) = organ("secrete", 8, Strength::NONE);
    plain.part_mut(&id).unwrap().port = None;
    assert_eq!(rule.toxic(&plain, none), 1);
    assert_eq!(Guard { gland: 6, ..rule }.toxic(&plain, none), 3);
    // No secreting tissue, no toxin.
    let (bare, _) = organ("support", 8, Strength { toxin: 9, ..Strength::NONE });
    assert_eq!(rule.toxic(&bare, none), 0);
}

#[test]
fn a_mouths_attack_is_its_own_or_the_founded_crush() {
    let rule = Guard::default();
    assert_eq!(rule.attack(IntakePort::live(NisKind::Producer)), Strength { crush: rule.bite, ..Strength::NONE });
    let jaw = IntakePort::live(NisKind::Consumer).with_strength(Strength { crush: 6, ..Strength::NONE });
    assert_eq!(rule.attack(jaw).crush, 6);
    let stylet = IntakePort::live(NisKind::Producer).with_strength(Strength { pierce: 5, ..Strength::NONE });
    assert_eq!(rule.attack(stylet), Strength { pierce: 5, ..Strength::NONE });
}

#[test]
fn withholding_takes_tenths_off_every_account_whole_units_by_the_draw() {
    let ledger = || crate::schema::Ledger::from([("matter:0-0".into(), 30), ("matter:1-0".into(), 10)]);
    let mut bite = ledger();
    assert_eq!(Guard::withhold(&mut bite, 0, 7), 0, "no resistance, no change");
    assert_eq!(bite, ledger());
    assert_eq!(Guard::withhold(&mut bite, 4, 0), 16);
    assert_eq!(bite, crate::schema::Ledger::from([("matter:0-0".into(), 18), ("matter:1-0".into(), 6)]));
    // The draw rounds: a unit against four tenths is taken when the draw is 4 or more of ten, so six in ten.
    let taken = |luck| {
        let mut one = crate::schema::Ledger::from([("matter:0-0".into(), 1)]);
        Guard::withhold(&mut one, 4, luck);
        one.values().sum::<u64>()
    };
    assert_eq!((0..10).map(taken).sum::<u64>(), 6);
    assert_eq!(taken(13), taken(3), "only the draw's last digit counts");
    // The expectation is the tenths' worth, to the unit, over the ten draws.
    let total: u64 = (0..10).map(|luck| { let mut b = ledger(); Guard::withhold(&mut b, 3, luck); b.values().sum::<u64>() }).sum();
    assert_eq!(total, 10 * 40 * 7 / 10);
    // An emptied account is not left in the bite.
    let mut none = crate::schema::Ledger::from([("matter:0-0".into(), 1)]);
    assert_eq!(Guard::withhold(&mut none, 9, 0), 1);
    assert!(none.is_empty());
}

#[test]
fn tolerance_bits_widen_to_hold_the_guards_two() {
    let port = IntakePort::none().tolerating(Tolerance::Spines).tolerating(Tolerance::Toxin);
    assert!(port.tolerates(Tolerance::Spines) && port.tolerates(Tolerance::Toxin) && !port.tolerates(Tolerance::ToughPlant));
    assert_eq!(IntakePort::none().with_tolerance_bits(port.tolerance_bits()), port);
}

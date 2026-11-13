// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The guard rule at the meal: a grazer biting an armoured part takes less
//! and what it does not take stays in the plant; one biting a spined part is
//! wounded unless it tolerates or matches the spines; one eating a plant
//! with glands is harmed by the tissue and spared by tolerance; the plant
//! pays rent; the same meals with the rule unset are as before; the budget
//! stays closed. The played A1 world founds none of these parts, so lineage
//! 0's plants are given them, with all their matter in the one part, and
//! lineage 3 (their grazer) shares their site.

use super::guard_tests::founding;
use super::*;
use crate::{
    Execution, Session,
    flows::{Holder, MadeBy},
    process::{IntakePort, ProcessId, Registry, Strength, Tolerance},
    schema::{Entity, PartId},
    simulation::Genesis,
};

const SPINED: Strength = Strength { spine: 4, ..Strength::NONE };

/// Lineage 0's every body with its matter all in its first part, and that
/// part's tissue given by `organ` (a function and cells under a port).
fn plants(mut g: Genesis, function: &str, cells: u32, strength: Strength) -> Genesis {
    let held = Registry::native().get(&ProcessId::new("mesocosm", function)).unwrap().reference();
    for group in g.population.groups.values_mut().filter(|g| g.entity.lineage == "lineage:0") {
        let e = group.entity_mut();
        let id = *e.parts.keys().next().unwrap();
        let mut all = crate::schema::Ledger::new();
        for part in e.parts.values_mut() {
            for (k, n) in std::mem::take(&mut std::sync::Arc::make_mut(part).matter) {
                *all.entry(k).or_default() += n;
            }
        }
        let cells = cells.min(crate::anatomy::capacity(e.extent(id)));
        let part = e.part_mut(&id).unwrap();
        part.matter = all;
        part.cells = [(format!("function:{function}"), cells)].into();
        part.functions = [format!("function:{function}")].into();
        part.port = Some(IntakePort::none().with_strength(strength).supported_by(held));
        e.lay_out();
    }
    g
}

/// Everything of lineages 0 and 3 at one site, so the grazer meets its food.
fn together(mut g: Genesis) -> Genesis {
    let site = *g.sites.keys().next().unwrap();
    for group in g.population.groups.values_mut().filter(|g| ["lineage:0", "lineage:3"].contains(&g.entity.lineage.as_str())) {
        group.entity_mut().place = site;
    }
    g
}

/// Lineage 3's mouths changed by `port`.
fn mouth(mut g: Genesis, port: impl Fn(IntakePort) -> IntakePort) -> Genesis {
    for group in g.population.groups.values_mut().filter(|g| g.entity.lineage == "lineage:3") {
        let e: &mut Entity = group.entity_mut();
        let ports: Vec<(PartId, IntakePort)> = e.parts.iter().filter_map(|(id, p)| p.port.map(|p| (*id, p))).collect();
        for (id, old) in ports {
            e.part_mut(&id).unwrap().port = Some(port(old));
        }
    }
    g
}

fn world(guard: Option<Guard>, function: &str, cells: u32, strength: Strength) -> Genesis {
    together(plants(founding(guard).generate().unwrap(), function, cells, strength))
}

#[derive(Default)]
struct Seen {
    /// What each bite of lineage 0 took, per leg.
    legs: Vec<u64>,
    /// Matter lineage 3's meals took from lineage 0.
    taken: u64,
    /// Harm events by ward.
    wards: Vec<crate::flows::Harm>,
    /// Matter the plants paid as rent.
    rent: u64,
    closed: bool,
}

fn run(g: Genesis, ticks: u64) -> (Seen, Session) {
    let mut s = Session::new(g, Execution::Individuals).unwrap();
    let total = s.sim.budget().total();
    let mut seen = Seen { closed: true, ..Seen::default() };
    for _ in 0..ticks {
        let r = s.advance_tick_with_flows().unwrap();
        for f in &r.flows {
            let by = |name: &str| f.made_by == MadeBy::Process(name.into());
            if by("ecology:feed-3") && matches!(f.to.0, Holder::Entity(_) | Holder::Part(..)) && f.from_kind.as_ref().is_some_and(|k| k.lineage == "lineage:0") {
                seen.legs.push(f.amount);
                seen.taken += f.amount * f.count;
            }
            if by("guard:rent-0") && !f.internal() {
                seen.rent += f.amount * f.count;
            }
        }
        seen.wards.extend(s.sim.harms().iter().filter(|h| h.made_by == "ecology:feed-3" && h.ward.is_some()).cloned());
        seen.closed &= s.sim.budget().total() == total;
    }
    (seen, s)
}

fn of(seen: &Seen, ward: Ward) -> Vec<&crate::flows::Harm> {
    seen.wards.iter().filter(|h| h.ward == Some(ward)).collect()
}

#[test]
fn a_bite_on_armour_takes_less_than_on_a_bare_part_and_the_rest_stays() {
    let rule = Some(Guard::default());
    let (bare, _) = run(world(rule, "fix", u32::MAX, Strength::NONE), 30);
    let (armoured, s) = run(world(rule, "support", u32::MAX, Strength { crush: 6, pierce: 5, ..Strength::NONE }), 30);
    assert!(!bare.legs.is_empty() && !armoured.legs.is_empty(), "both were eaten");
    // A founded mouth bites at crush 2 against armour 6: four tenths resisted, so about six bites in ten are taken.
    let withheld = of(&armoured, Ward::Armour);
    let held: u64 = withheld.iter().map(|h| h.withheld).sum();
    assert!(!withheld.is_empty() && withheld.iter().all(|h| h.cells == 0 && h.withheld > 0));
    assert!(armoured.taken < bare.taken, "armoured {} against bare {}", armoured.taken, bare.taken);
    let share = armoured.taken * 100 / (armoured.taken + held);
    assert!((45..=75).contains(&share), "{share} percent of the bites were taken (six in ten expected)");
    assert!(bare.wards.is_empty(), "a bare part has no ward to record");
    // The matter not taken is still the prey's, and no matter is made or lost.
    assert!(armoured.closed && bare.closed);
    let plant: u64 = s.sim.state().population.groups.values().filter(|g| g.entity.lineage == "lineage:0" && g.entity.alive)
        .map(|g| crate::anatomy::books(&g.entity).get("matter:0-0").copied().unwrap_or(0)).sum();
    assert!(plant > 0);
}

#[test]
fn a_jaw_as_strong_as_the_armour_is_not_resisted() {
    let rule = Some(Guard::default());
    let strong = |g| mouth(g, |p| p.with_strength(Strength { crush: 6, ..Strength::NONE }));
    let g = |function, strength| strong(world(rule, function, u32::MAX, strength));
    let (bare, _) = run(g("fix", Strength::NONE), 30);
    let (armoured, _) = run(g("support", Strength { crush: 6, ..Strength::NONE }), 30);
    assert_eq!(armoured.legs.iter().max(), bare.legs.iter().max(), "the counter-defence takes the whole bite");
    assert!(of(&armoured, Ward::Armour).is_empty());
}

#[test]
fn a_spined_part_wounds_the_eater_unless_tolerated_or_matched_and_the_meal_still_happens() {
    let rule = Some(Guard::default());
    let (spined, s) = run(world(rule, "support", 3, SPINED), 30);
    let wounds = of(&spined, Ward::Spines);
    assert!(spined.taken > 0, "the meal still moves");
    assert!(!wounds.is_empty() && wounds.iter().all(|h| h.cells == 2 && h.lineage == "lineage:0"), "and the eater pays for it");
    assert!(spined.closed);
    let hurt = s.sim.state().population.groups.values().filter(|g| g.entity.lineage == "lineage:3").any(|g| crate::harm::wound_share(&g.entity) > 0 || !g.entity.alive);
    assert!(hurt, "the wound is on the eater");
    // The matching tolerance, and the matching attack, spare it.
    let (tolerant, _) = run(mouth(world(rule, "support", 3, SPINED), |p| p.tolerating(Tolerance::Spines)), 30);
    assert!(tolerant.taken > 0 && of(&tolerant, Ward::Spines).is_empty());
    let (stylet, _) = run(mouth(world(rule, "support", 3, SPINED), |p| p.with_strength(Strength { pierce: 4, ..Strength::NONE })), 30);
    assert!(stylet.taken > 0 && of(&stylet, Ward::Spines).is_empty());
    // A part with no spines costs nothing.
    let (bare, _) = run(world(rule, "support", 3, Strength::NONE), 30);
    assert!(bare.taken > 0 && bare.wards.is_empty());
}

#[test]
fn a_gland_harms_its_eater_by_the_tissue_and_a_tolerant_eater_is_spared() {
    let rule = Some(Guard::default());
    let glands = |cells| world(rule, "secrete", cells, Strength { toxin: 5, ..Strength::NONE });
    let (many, _) = run(glands(u32::MAX), 30);
    let (few, _) = run(glands(2), 30);
    let (cells_many, cells_few) = (of(&many, Ward::Toxin), of(&few, Ward::Toxin));
    assert!(many.taken > 0 && few.taken > 0, "the meal still moves");
    assert!(!cells_many.is_empty() && !cells_few.is_empty());
    assert!(cells_few.iter().all(|h| h.cells == 1), "two cells of strength five over a dose of sixteen is one cell");
    let most = cells_many.iter().map(|h| h.cells).max().unwrap();
    assert!(most > 1 && most <= 4, "more secreting tissue, more harm, and capped: {most}");
    assert!(many.closed && few.closed);
    let (spared, _) = run(mouth(glands(u32::MAX), |p| p.tolerating(Tolerance::Toxin)), 30);
    assert!(spared.taken > 0 && of(&spared, Ward::Toxin).is_empty());
}

#[test]
fn the_secreter_pays_standing_rent_and_a_bare_plant_pays_none() {
    let rule = Some(Guard::default());
    let (glands, _) = run(world(rule, "secrete", u32::MAX, Strength { toxin: 5, ..Strength::NONE }), 40);
    let (bare, _) = run(world(rule, "fix", u32::MAX, Strength::NONE), 40);
    assert!(glands.rent > 0, "a body with secreting cells pays rent");
    assert_eq!(bare.rent, 0, "a body with none pays none");
    assert!(glands.closed && bare.closed, "the rent goes to the soil");
}

#[test]
fn with_the_rule_unset_the_same_parts_change_no_meal() {
    let mut worlds = vec![];
    for (function, strength) in [("fix", Strength::NONE), ("support", Strength { crush: 6, pierce: 5, spine: 4, toxin: 5 }), ("secrete", Strength { toxin: 5, ..Strength::NONE })] {
        let (seen, _) = run(world(None, function, u32::MAX, strength), 30);
        assert!(seen.wards.is_empty() && seen.rent == 0 && seen.closed, "{function}");
        worlds.push(seen.legs);
    }
    assert_eq!(worlds[0], worlds[1], "armour, spines and glands read nothing where the rule is unset");
    assert_eq!(worlds[0], worlds[2]);
}

#[test]
fn the_rule_is_the_same_in_both_executions() {
    let hash = |mode| {
        let strength = Strength { crush: 6, pierce: 5, spine: 4, toxin: 5 };
        let mut s = Session::new(world(Some(Guard::default()), "support", u32::MAX, strength), mode).unwrap();
        s.advance(30).unwrap();
        s.sim.state_hash()
    };
    assert_eq!(hash(Execution::Individuals), hash(Execution::Grouped));
}

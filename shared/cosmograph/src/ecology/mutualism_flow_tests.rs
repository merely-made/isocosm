// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The trade at work: both goods move, each side pays only while the other
//! does, a taker that does not give stops receiving, a body with no partner
//! pays nothing, the budget stays closed and both executions agree. The
//! founded world gives no body an exchange organ, so lineage 0 (producers)
//! and lineage 2 (decomposers) are given one, and meet at one site.

use super::mutualism_tests::founding;
use super::*;
use crate::{
    Execution, Session,
    flows::MadeBy,
    process::{IntakePort, ProcessId, Registry, Trade},
    schema::Entity,
    simulation::Genesis,
};

/// `line`'s every body, its first part with a living function given a port
/// that does `trades` (keeping any port it has).
pub(super) fn equip(mut g: Genesis, line: &str, trades: &[Trade]) -> Genesis {
    for group in g.population.groups.values_mut().filter(|g| g.entity.lineage == line) {
        let e: &mut Entity = group.entity_mut();
        let id = *e.parts.iter().find(|(_, p)| p.cells.is_empty() || p.cells.values().any(|n| *n > 1)).map(|(id, _)| id).unwrap();
        let held = Registry::native().get(&ProcessId::new("mesocosm", "adhesion")).unwrap().reference();
        let part = e.part_mut(&id).unwrap();
        // One cell of its largest function becomes the organ's.
        if let Some(biggest) = part.cells.iter().max_by_key(|(_, n)| **n).map(|(k, _)| k.clone()) {
            *part.cells.get_mut(&biggest).unwrap() -= 1;
        }
        *part.cells.entry("function:adhesion".into()).or_default() += 1;
        part.functions.insert("function:adhesion".into());
        let port = part.port.unwrap_or_else(|| IntakePort::none().supported_by(held));
        part.port = Some(trades.iter().fold(port, |p, t| p.trading(*t)));
        e.lay_out();
    }
    g
}

pub(super) fn together(mut g: Genesis) -> Genesis {
    let site = *g.sites.keys().next().unwrap();
    for group in g.population.groups.values_mut().filter(|g| ["lineage:0", "lineage:2"].contains(&g.entity.lineage.as_str())) {
        group.entity_mut().place = site;
    }
    g
}

pub(super) const BOTH: [Trade; 2] = [Trade::Gives, Trade::Takes];

fn world(rule: Option<Mutualism>, plants: &[Trade], fungi: &[Trade]) -> Genesis {
    together(equip(equip(founding(rule).generate().unwrap(), "lineage:0", plants), "lineage:2", fungi))
}

#[derive(Default)]
struct Seen {
    /// The sugar the plants paid.
    sugar: u64,
    /// Turns of each, by tick.
    nutrient_ticks: Vec<u64>,
    sugar_ticks: Vec<u64>,
    fee: u64,
    closed: bool,
}

fn run(g: Genesis, ticks: u64) -> Seen {
    let mut s = Session::new(g, Execution::Individuals).unwrap();
    let total = s.sim.budget().total();
    let mut seen = Seen { closed: true, ..Seen::default() };
    for t in 0..ticks {
        let r = s.advance_tick_with_flows().unwrap();
        for f in &r.flows {
            let by = |name: &str| f.made_by == MadeBy::Process(name.into());
            if by("mutualism:nutrient-2-0") && !f.internal() {
                seen.fee += f.amount * f.count;
            }
            if by("mutualism:nutrient-2-0") && !seen.nutrient_ticks.contains(&t) {
                seen.nutrient_ticks.push(t);
            }
            if by("mutualism:sugar-0-2") && !seen.sugar_ticks.contains(&t) {
                seen.sugar_ticks.push(t);
                seen.sugar += f.amount * f.count;
            }
        }
        seen.closed &= s.sim.budget().total() == total;
    }
    seen
}

#[test]
fn both_goods_move_and_the_budget_stays_closed() {
    let seen = run(world(Some(Mutualism::default()), &BOTH, &BOTH), 120);
    assert!(!seen.nutrient_ticks.is_empty(), "the decomposers carried soil");
    assert!(!seen.sugar_ticks.is_empty(), "the plants paid in tissue");
    assert!(seen.sugar > 0 && seen.closed);
}

#[test]
fn a_partner_that_takes_and_does_not_give_stops_receiving() {
    let honest = run(world(Some(Mutualism::default()), &BOTH, &BOTH), 120);
    let cheat = run(world(Some(Mutualism::default()), &[Trade::Takes], &BOTH), 120);
    assert!(cheat.closed);
    assert!(cheat.sugar_ticks.is_empty(), "a cheater gives nothing");
    // A gift of credit a site: the decomposers carry once at each site the plants reach, then wait for
    // the answer that never comes. The honest partner is served again and again.
    assert!(cheat.nutrient_ticks.len() <= 6 && cheat.nutrient_ticks.iter().all(|t| *t < 60), "{:?}", cheat.nutrient_ticks);
    assert!(honest.nutrient_ticks.len() > 2 * cheat.nutrient_ticks.len(), "{} against {}", honest.nutrient_ticks.len(), cheat.nutrient_ticks.len());
}

#[test]
fn a_decomposer_that_does_not_give_leaves_the_plants_unpaid_and_unserved() {
    let seen = run(world(Some(Mutualism::default()), &BOTH, &[Trade::Takes]), 120);
    assert!(seen.nutrient_ticks.is_empty() && seen.sugar_ticks.is_empty(), "no giver, no turn at all");
    assert!(seen.closed);
}

#[test]
fn no_partner_means_no_flow() {
    // Organs on the plants alone, then on both but the lines apart: nothing is paid or received.
    let alone = run(world(Some(Mutualism::default()), &BOTH, &[]), 120);
    assert!(alone.nutrient_ticks.is_empty() && alone.sugar_ticks.is_empty() && alone.fee == 0);
    let sites = founding(Some(Mutualism::default())).generate().unwrap();
    let mut apart = equip(equip(sites, "lineage:0", &BOTH), "lineage:2", &BOTH);
    let mut ids = apart.sites.keys().copied();
    let (a, b) = (ids.next().unwrap(), ids.next().unwrap());
    for group in apart.population.groups.values_mut() {
        match group.entity.lineage.as_str() {
            "lineage:0" => group.entity_mut().place = a,
            "lineage:2" => group.entity_mut().place = b,
            _ => {},
        }
    }
    // Soon enough that no body has wandered to the other's site.
    let seen = run(apart, 12);
    assert!(seen.nutrient_ticks.is_empty() && seen.sugar_ticks.is_empty() && seen.fee == 0, "out of contact, no trade");
    assert!(seen.closed);
}

#[test]
fn with_the_rule_unset_an_organ_changes_no_meal() {
    let with = |organs: bool| {
        let g = if organs { world(None, &BOTH, &BOTH) } else { together(founding(None).generate().unwrap()) };
        let mut s = Session::new(g, Execution::Individuals).unwrap();
        s.advance(40).unwrap();
        s.sim.state().population.groups.values().filter(|g| g.entity.alive).map(|g| (g.entity.lineage.clone(), g.count)).collect::<std::collections::BTreeMap<_, u64>>().len()
    };
    assert_eq!(with(true), with(false));
}

#[test]
fn the_rule_is_the_same_in_both_executions() {
    let hash = |mode| {
        let mut s = Session::new(world(Some(Mutualism::default()), &BOTH, &BOTH), mode).unwrap();
        s.advance(60).unwrap();
        s.sim.state_hash()
    };
    assert_eq!(hash(Execution::Individuals), hash(Execution::Grouped));
}

/// `equip`, for the goods at `gives` and `takes` of the world's table.
fn equip_goods(g: Genesis, line: &str, gives: &[&str], takes: &[&str]) -> Genesis {
    let table = g.rules.repertoire.as_ref().unwrap().goods.clone();
    let at = |key: &str| table.iter().position(|good| good.key == key).unwrap();
    let mut g = equip(g, line, &[]);
    for group in g.population.groups.values_mut().filter(|g| g.entity.lineage == line) {
        let e: &mut Entity = group.entity_mut();
        let id = *e.parts.iter().find(|(_, p)| p.port.is_some_and(|port| port.support().is_some() && !port.is_mouth())).map(|(id, _)| id).unwrap();
        let part = e.part_mut(&id).unwrap();
        let port = gives.iter().fold(part.port.unwrap(), |p, key| p.emitting(at(key)));
        part.port = Some(takes.iter().fold(port, |p, key| p.admitting(at(key))));
    }
    g
}

#[test]
fn bodies_trade_wherever_what_one_gives_the_other_takes() {
    use crate::repertoire::Repertoire;

    let f = Founding {
        repertoire: Some(Repertoire::default()),
        nutrients: Some(Nutrients::default()),
        ..founding(Some(Mutualism::default()))
    };
    let mut g = f.generate().unwrap();
    let site = *g.sites.keys().next().unwrap();
    for group in g.population.groups.values_mut() {
        group.entity_mut().place = site;
    }
    // One act per lineage, open to every other, and no pair is founded.
    assert!(g.rules.processes["mutualism:exchange-0"].target.as_ref().unwrap().among.is_empty());
    assert!(!g.rules.processes.contains_key("mutualism:nutrient-2-0"));
    // The plant pays in nectar and takes what the ground's scarce nutrient
    // brings: neither is a good the old pair knew.
    g = equip_goods(g, "lineage:0", &["nectar"], &["soil:scarce"]);

    // The ordinary grazer, line 1, can supply it if its organs fit.
    let sees_payment = |gives: &[&str], takes: &[&str]| {
        let mut s = Session::new(equip_goods(g.clone(), "lineage:1", gives, takes), Execution::Grouped).unwrap();
        let total = s.sim.budget().total();
        let (mut payment, mut drawn, mut kept) = (false, false, false);
        for _ in 0..12 {
            for flow in s.advance_tick_with_flows().unwrap().flows {
                if flow.by_process("mutualism:exchange-0") {
                    if flow.from.1 == IN_PASSAGE && !flow.internal() {
                        assert_eq!(flow.to_kind.as_ref().unwrap().lineage, "lineage:1");
                        payment = true;
                    }
                    drawn |= flow.from.1 == "world:soil-1" && flow.to.0.body().is_some();
                }
                kept |= flow.by_process("mutualism:assimilate-1");
            }
        }
        assert_eq!(s.sim.budget().total(), total);
        (payment, drawn, kept)
    };
    // It takes the nectar but gives nothing wanted; it gives but takes sugar, not nectar; it fits.
    assert_eq!(sees_payment(&[], &["nectar"]), (false, false, false));
    assert_eq!(sees_payment(&["soil:scarce"], &["sugar"]), (false, false, false));
    assert_eq!(sees_payment(&["soil:scarce"], &["nectar"]), (true, true, true));
}

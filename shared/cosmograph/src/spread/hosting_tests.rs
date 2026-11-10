// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! What a surface does where it lives (cases 1223, 1225, 1226). The
//! fixture's bodies are lineage 0 flora: host 12 stands on site 0, hosts 3
//! and 9 on site 2, and body 0 is the world's.

use super::tests::fixture;
use super::*;
use crate::{
    Execution, Session,
    directing::interim::regard::dealings,
    ecology::world_time::DETRITUS,
    flows::{Flow, Holder},
    rules::{AccountKind, Hosting},
    schema::*,
    simulation::Genesis,
};
use std::collections::BTreeSet;

const GENES: [&str; 4] = ["gene:rot", "gene:feed", "gene:leak", "gene:enter"];
const STRAIN: &str = "lineage:strain:test";
const TISSUE: &str = "matter:0-0";
const RESERVE: &str = "matter:0-1";

fn strain() -> Identity { Identity::Surface(STRAIN.into()) }

/// The fixture with a strain carrying `genes`, and the hosting rule set at a
/// period of two ticks where `hosting`.
fn world(genes: &[&str], hosting: bool) -> Genesis {
    let (mut g, ..) = fixture();
    g.rules.traits.extend(GENES.map(String::from));
    g.rules.accounts.insert(DETRITUS.into(), AccountKind::Matter {
        lineage: "world:ground".into(), reserve: false, provision: false });
    g.rules.accounts.insert(RESERVE.into(), AccountKind::Matter {
        lineage: "lineage:0".into(), reserve: true, provision: false });
    let mut line = g.lineages["lineage:0"].clone();
    line.parent = Some("lineage:0".into());
    line.traits = genes.iter().map(|k| (*k).into()).collect();
    g.lineages.insert(STRAIN.into(), line);
    g.spreads.identities.insert(strain());
    if hosting { g.rules.hosting = Some(Hosting { period: 2, ..Hosting::default() }); }
    g
}

fn ledger_of(g: &mut Genesis, at: Substrate) -> &mut Ledger {
    match at {
        Substrate::Site(id) => &mut g.sites.get_mut(&id).unwrap().accounts,
        Substrate::Host(id) => &mut g.population.lift(id).unwrap().accounts,
        Substrate::Place(_) => unreachable!("not planted here"),
    }
}

fn put(g: &mut Genesis, at: Substrate, key: &str, amount: u64) {
    ledger_of(g, at).insert(key.into(), amount);
}

fn read(s: &Session, at: Substrate, key: &str) -> u64 {
    super::stock::ledger(s.sim.state(), at).unwrap().get(key).copied().unwrap_or(0)
}

fn start(g: Genesis) -> Session { Session::new(g, Execution::Individuals).unwrap() }

/// Everything at a site's ledger and a host's, for comparing across a turn.
fn ledgers(s: &Session) -> Vec<Ledger> {
    let state = s.sim.state();
    state.sites.values().map(|site| site.accounts.clone())
        .chain([3, 9, 12].map(|id| state.population.get(id).unwrap().accounts.clone())).collect()
}

/// A turn at tick 2, its moves handed over.
fn turn_flows(s: &mut Session) -> Vec<Flow> {
    s.advance(1).unwrap();
    s.advance_tick_with_flows().unwrap().flows
}

fn replay(session: &Session) {
    let json = serde_json::to_vec(&session.save()).unwrap();
    for mode in [Execution::Individuals, Execution::Grouped] {
        let loaded = Session::load_json(&json, mode).unwrap();
        assert_eq!(loaded.sim.state_hash(), session.sim.state_hash());
        assert_eq!(loaded.sim.matter(), session.sim.matter());
        assert_eq!(loaded.entries, session.entries);
    }
}

#[test]
fn an_unset_rule_makes_no_turn_and_leaves_the_genesis_and_hash_as_before() {
    let mut g = world(&GENES, false);
    put(&mut g, Substrate::Site(0), STRAIN, 40);
    put(&mut g, Substrate::Site(0), DETRITUS, 9);
    put(&mut g, Substrate::Host(12), STRAIN, 5);
    assert!(g.rules.hosting.is_none());
    assert!(!serde_json::to_string(&g.rules).unwrap().contains("hosting"));
    let mut set = g.rules.clone();
    set.hosting = Some(Hosting::default());
    assert_ne!(set.revision(), g.rules.revision());
    let mut s = start(g.clone());
    assert_eq!(*s.sim.genesis(), g, "an unset rule leaves the genesis as it was");
    let before = ledgers(&s);
    s.advance(2).unwrap();
    assert_eq!(ledgers(&s), before);
    s.advance(10).unwrap();
    let mut plain = start(g);
    plain.advance(12).unwrap();
    assert_eq!(plain.sim.state_hash(), s.sim.state_hash());
    assert_eq!(read(&s, Substrate::Site(0), STRAIN), 40);
    assert_eq!(read(&s, Substrate::Host(12), STRAIN), 5);
}

#[test]
fn the_rule_validates_its_period_shares_and_genes() {
    let g = world(&GENES, true);
    let rules = &g.rules;
    assert!(Hosting::default().validate(rules).is_ok());
    let bad = |edit: &dyn Fn(&mut Hosting)| {
        let mut h = Hosting::default();
        edit(&mut h);
        h.validate(rules).is_err()
    };
    assert!(bad(&|h| h.period = 0));
    assert!(bad(&|h| h.take = [1, 0]));
    assert!(bad(&|h| h.give = [3, 2]));
    assert!(bad(&|h| h.send = [2, 1]));
    assert!(bad(&|h| h.rot = "gene:absent".into()));
    let mut unset = world(&GENES, true);
    unset.rules.traits.remove("gene:leak");
    assert!(unset.rules.validate().is_err(), "a world that sets the rule declares the genes");
}

#[test]
fn rot_takes_its_share_from_detritus_then_soil_and_never_more_than_is_there() {
    let mut g = world(&["gene:rot"], true);
    put(&mut g, Substrate::Site(0), STRAIN, 40);
    put(&mut g, Substrate::Site(0), DETRITUS, 3);
    let mut s = start(g);
    let total = s.sim.matter();
    s.advance(1).unwrap();
    assert_eq!(read(&s, Substrate::Site(0), STRAIN), 40, "no turn before the period");
    s.advance(1).unwrap();
    // A tenth of 40 is 4: the 3 detritus, then 1 soil.
    assert_eq!(read(&s, Substrate::Site(0), STRAIN), 44);
    assert_eq!(read(&s, Substrate::Site(0), DETRITUS), 0);
    assert_eq!(read(&s, Substrate::Site(0), "world:soil"), 285);
    assert_eq!(s.sim.matter(), total);
    // Where little is there, little is taken.
    let mut g = world(&["gene:rot"], true);
    put(&mut g, Substrate::Site(0), STRAIN, 40);
    put(&mut g, Substrate::Site(0), "world:soil", 2);
    let mut s = start(g);
    s.advance(2).unwrap();
    assert_eq!(read(&s, Substrate::Site(0), STRAIN), 42);
    assert_eq!(read(&s, Substrate::Site(0), "world:soil"), 0);
}

#[test]
fn feed_takes_from_a_living_hosts_reserve_then_tissue() {
    let mut g = world(&["gene:feed"], true);
    put(&mut g, Substrate::Host(12), STRAIN, 40);
    put(&mut g, Substrate::Host(12), RESERVE, 1);
    let mut s = start(g);
    let total = s.sim.matter();
    s.advance(2).unwrap();
    // Four taken: the 1 reserve, then 3 of the 10 tissue.
    assert_eq!(read(&s, Substrate::Host(12), STRAIN), 44);
    assert_eq!(read(&s, Substrate::Host(12), RESERVE), 0);
    assert_eq!(read(&s, Substrate::Host(12), TISSUE), 7);
    assert_eq!(s.sim.matter(), total);
    assert!(s.sim.state().population.get(12).unwrap().alive, "emptying is the starving process's to end");
}

#[test]
fn leak_gives_to_the_hosts_reserve() {
    let mut g = world(&["gene:leak"], true);
    put(&mut g, Substrate::Host(12), STRAIN, 40);
    let mut s = start(g);
    let total = s.sim.matter();
    s.advance(2).unwrap();
    // A twentieth of 40 is 2.
    assert_eq!(read(&s, Substrate::Host(12), STRAIN), 38);
    assert_eq!(read(&s, Substrate::Host(12), RESERVE), 2);
    assert_eq!(read(&s, Substrate::Host(12), TISSUE), 10);
    assert_eq!(s.sim.matter(), total);
}

#[test]
fn enter_sends_into_each_resident_body_in_id_order_skipping_kin_participants_and_the_world() {
    let mut g = world(&["gene:enter"], true);
    put(&mut g, Substrate::Site(2), STRAIN, 40);
    let mut s = start(g);
    let total = s.sim.matter();
    s.advance(2).unwrap();
    // Body 2 is the world's; 3 then 9 take a tenth of what is left each.
    assert_eq!(read(&s, Substrate::Host(3), STRAIN), 4);
    assert_eq!(read(&s, Substrate::Host(9), STRAIN), 3);
    assert_eq!(read(&s, Substrate::Host(2), STRAIN), 0);
    assert_eq!(read(&s, Substrate::Site(2), STRAIN), 33);
    assert_eq!(read(&s, Substrate::Host(4), STRAIN), 0, "a cohort enters as its first member");
    assert_eq!(s.sim.matter(), total);
    // Not its own lineage: lineage 0 itself finds no host among its own.
    let mut g = world(&["gene:enter"], true);
    g.lineages.get_mut("lineage:0").unwrap().traits.insert("gene:enter".into());
    g.spreads.identities.insert(Identity::Surface("lineage:0".into()));
    put(&mut g, Substrate::Site(2), "lineage:0", 40);
    let mut s = start(g);
    s.advance(2).unwrap();
    assert_eq!(read(&s, Substrate::Site(2), "lineage:0"), 40);
    // Nor a participant.
    let mut g = world(&["gene:enter"], true);
    put(&mut g, Substrate::Site(2), STRAIN, 40);
    g.population.lift(3).unwrap().kingdom = crate::directing::PARTICIPANT.into();
    let mut s = start(g);
    s.advance(2).unwrap();
    assert_eq!(read(&s, Substrate::Host(3), STRAIN), 0);
    assert_eq!(read(&s, Substrate::Host(4), STRAIN), 4, "the rest of its cohort is not a participant");
    assert_eq!(read(&s, Substrate::Host(9), STRAIN), 3);
}

#[test]
fn a_strain_with_no_gene_does_nothing_but_is_released() {
    let mut g = world(&[], true);
    put(&mut g, Substrate::Site(0), STRAIN, 40);
    put(&mut g, Substrate::Site(2), STRAIN, 40);
    put(&mut g, Substrate::Host(12), STRAIN, 8);
    let mut s = start(g);
    let before = ledgers(&s);
    s.advance(2).unwrap();
    assert_eq!(ledgers(&s), before);
    // The host dies; at the next turn its surface goes to its site.
    s.sim.state.population.lift(12).unwrap().alive = false;
    let total = s.sim.matter();
    s.advance(1).unwrap();
    assert_eq!(read(&s, Substrate::Host(12), STRAIN), 8, "not before the next turn");
    s.advance(1).unwrap();
    assert_eq!(read(&s, Substrate::Host(12), STRAIN), 0);
    assert_eq!(read(&s, Substrate::Site(0), STRAIN), 48);
    assert_eq!(s.sim.matter(), total);
}

#[test]
fn a_share_that_rounds_to_nought_is_one_and_nothing_held_is_nothing() {
    let mut g = world(&["gene:rot"], true);
    put(&mut g, Substrate::Site(0), STRAIN, 5);
    let mut s = start(g);
    s.advance(2).unwrap();
    assert_eq!(read(&s, Substrate::Site(0), STRAIN), 6, "a tenth of 5 is nought, so 1");
    let mut g = world(&["gene:rot", "gene:enter"], true);
    put(&mut g, Substrate::Site(2), STRAIN, 1);
    let mut s = start(g);
    s.advance(2).unwrap();
    // 1 rotted in makes 2; 1 is sent to body 3, and 1 of what is left to body 9.
    assert_eq!([3, 9].map(|id| read(&s, Substrate::Host(id), STRAIN)), [1, 1]);
    assert_eq!(read(&s, Substrate::Site(2), STRAIN), 0);
    let g = world(&GENES, true);
    let mut s = start(g);
    let before = ledgers(&s);
    s.advance(2).unwrap();
    assert_eq!(ledgers(&s), before, "a surface holding nothing does nothing");
    assert_eq!(super::hosting::share(0, [1, 10]), 0);
}

#[test]
fn matter_is_conserved_across_turns_of_every_kind() {
    let mut g = world(&GENES, true);
    put(&mut g, Substrate::Site(0), STRAIN, 40);
    put(&mut g, Substrate::Site(0), DETRITUS, 6);
    put(&mut g, Substrate::Site(2), STRAIN, 60);
    put(&mut g, Substrate::Host(12), STRAIN, 30);
    put(&mut g, Substrate::Host(12), RESERVE, 3);
    let mut s = start(g);
    let total = s.sim.matter();
    for _ in 0..8 {
        s.advance(2).unwrap();
        assert_eq!(s.sim.matter(), total);
    }
    assert!(read(&s, Substrate::Site(0), STRAIN) != 40);
    assert!(read(&s, Substrate::Host(3), STRAIN) > 0);
}

fn named<'a>(flows: &'a [Flow], by: &str) -> Vec<&'a Flow> {
    flows.iter().filter(|f| f.by_process(&format!("hosting:{by}"))).collect()
}

#[test]
fn feeding_and_leaking_are_flows_between_the_lineages_and_regard_reads_them() {
    let mut g = world(&["gene:feed"], true);
    put(&mut g, Substrate::Host(12), STRAIN, 40);
    let mut s = start(g);
    let flows = turn_flows(&mut s);
    let fed = named(&flows, "feed");
    assert_eq!(fed.len(), 1);
    assert_eq!(fed[0].from, (Holder::Entity(12), TISSUE.into()));
    assert_eq!(fed[0].to, (Holder::Entity(12), STRAIN.into()));
    assert_eq!((fed[0].from_kind.as_ref().unwrap().lineage.as_str(), fed[0].to_kind.as_ref().unwrap().lineage.as_str()),
        ("lineage:0", STRAIN));
    // The played host line is what was fed on: matter it gave to the strain.
    assert_eq!(dealings(&flows, "lineage:0")[STRAIN], (4, 0));
    let mut g = world(&["gene:leak"], true);
    put(&mut g, Substrate::Host(12), STRAIN, 40);
    let mut s = start(g);
    let flows = turn_flows(&mut s);
    let leaked = named(&flows, "leak");
    assert_eq!(leaked.len(), 1);
    assert_eq!(leaked[0].from, (Holder::Entity(12), STRAIN.into()));
    assert_eq!(leaked[0].to, (Holder::Entity(12), RESERVE.into()));
    assert_eq!(leaked[0].from_kind.as_ref().unwrap().lineage, STRAIN);
    assert_eq!(leaked[0].to_kind.as_ref().unwrap().lineage, "lineage:0");
    // The played host line took what the strain gave.
    assert_eq!(dealings(&flows, "lineage:0")[STRAIN], (0, 2));
    // Rot, entering and release stay between a surface and a site or itself.
    let mut g = world(&["gene:rot", "gene:enter"], true);
    put(&mut g, Substrate::Site(0), STRAIN, 40);
    put(&mut g, Substrate::Site(2), STRAIN, 40);
    let mut s = start(g);
    let flows = turn_flows(&mut s);
    assert_eq!(named(&flows, "rot").len(), 2);
    assert_eq!(named(&flows, "enter").len(), 3);
    assert!(dealings(&flows, "lineage:0").is_empty());
}

#[test]
fn a_surface_grown_past_its_capacity_spills_along_a_route() {
    let mut g = world(&["gene:rot", "function:fix"], true);
    let gene = Expr::Read(Reading::Kept { name: "capacity:gene".into() });
    g.rules.spread_capacity = Some(CapacityRules { key: "capacity:base".into(),
        trait_inputs: [("capacity:gene".to_string(), "function:fix".to_string())].into(),
        mechanic: Expr::Mul(vec![gene, Expr::Const(40)]), layers: vec![] });
    put(&mut g, Substrate::Site(0), STRAIN, 40);
    let mut s = start(g);
    let total = s.sim.matter();
    s.advance(2).unwrap();
    // 40 grew to 44 and 4 spilled, shared over the two routes.
    assert_eq!(read(&s, Substrate::Site(0), STRAIN) + s.sim.state().spreads.transit.iter()
        .filter(|t| t.identity == strain()).map(|t| t.amount).sum::<u64>(), 44);
    assert!(s.sim.state().spreads.transit.iter().any(|t| t.identity == strain()));
    assert_eq!(s.sim.matter(), total);
}

fn wiring() -> (Genesis, Rules) {
    let g = world(&["gene:enter"], true);
    let rules = g.rules.clone();
    (g, rules)
}

use crate::rules::{Expr, Reading, Rules};

#[test]
fn happened_reports_an_entry_and_a_release() {
    let (mut g, rules) = wiring();
    put(&mut g, Substrate::Site(2), STRAIN, 40);
    let mut s = start(g);
    let before = s.sim.state().clone();
    s.advance(2).unwrap();
    let found = happened(&rules, &before, s.sim.state());
    assert!(found.contains(&Surfaced::Reached { identity: strain(), at: Substrate::Host(3) }), "{found:?}");
    // A dead host's matter is released to its site.
    let mut g = world(&[], true);
    put(&mut g, Substrate::Host(12), STRAIN, 8);
    let mut s = start(g);
    s.sim.state.population.lift(12).unwrap().alive = false;
    let before = s.sim.state().clone();
    s.advance(2).unwrap();
    let found = happened(&rules, &before, s.sim.state());
    assert!(found.contains(&Surfaced::Left { identity: strain(), at: Substrate::Host(12) }), "{found:?}");
    assert!(found.contains(&Surfaced::Reached { identity: strain(), at: Substrate::Site(0) }), "{found:?}");
    assert!(!found.iter().any(|f| matches!(f, Surfaced::HostDied { .. })), "it died before the turn");
}

#[test]
fn a_session_with_the_rule_set_replays_to_the_same_hash_under_both_executions() {
    let mut g = world(&GENES, true);
    put(&mut g, Substrate::Site(0), STRAIN, 40);
    put(&mut g, Substrate::Site(0), DETRITUS, 6);
    put(&mut g, Substrate::Site(2), STRAIN, 60);
    put(&mut g, Substrate::Host(12), STRAIN, 30);
    let mut hashes = Vec::new();
    for mode in [Execution::Individuals, Execution::Grouped] {
        let mut s = Session::new(g.clone(), mode).unwrap();
        s.advance(5).unwrap();
        replay(&s);
        s.advance(3).unwrap();
        s.advance(1).unwrap();
        s.advance(11).unwrap();
        replay(&s);
        assert!(read(&s, Substrate::Host(3), STRAIN) > 0);
        hashes.push(s.sim.state_hash());
    }
    assert_eq!(hashes[0], hashes[1]);
    let mut once = Session::new(g, Execution::Individuals).unwrap();
    once.advance(20).unwrap();
    assert_eq!(once.sim.state_hash(), hashes[0], "however the advance is cut");
}

#[test]
fn switching_a_gene_off_stops_its_turn_from_the_next_period() {
    let mut g = world(&["gene:rot"], true);
    put(&mut g, Substrate::Site(0), STRAIN, 40);
    let mut s = start(g);
    s.advance(2).unwrap();
    assert_eq!(read(&s, Substrate::Site(0), STRAIN), 44);
    s.command(crate::Command::Spread(Command::Switch { lineage: STRAIN.into(),
        on: BTreeSet::new(), off: BTreeSet::from(["gene:rot".into()]) })).unwrap();
    s.advance(4).unwrap();
    assert_eq!(read(&s, Substrate::Site(0), STRAIN), 44);
    s.command(crate::Command::Spread(Command::Switch { lineage: STRAIN.into(),
        on: BTreeSet::from(["gene:rot".into()]), off: BTreeSet::new() })).unwrap();
    s.advance(2).unwrap();
    assert!(read(&s, Substrate::Site(0), STRAIN) > 44);
}

// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Holding, steering a surface and turning a cycle (cases 1216, 1217, 1221,
//! 1223, 1226). The world is a small flora ecology: lineage 0 is the critters
//! (taken, never held), the strain is a surface cut from it, and the world's
//! own lineage is world-class.

use crate::{
    Execution, Founding, Session,
    ecology::world_time::DETRITUS,
    history::Command,
    rules::{AccountKind, Climate, Hosting},
    schema::*,
    simulation::Genesis,
    spread::{self, Identity, Substrate},
};
use std::collections::BTreeSet;

pub(crate) const STRAIN: &str = "lineage:strain:held";
pub(crate) const WORLD_LINE: &str = "world:ground";
pub(crate) const GENES: [&str; 4] = ["gene:rot", "gene:feed", "gene:leak", "gene:enter"];
const RESERVE: &str = "matter:0-1";

pub(crate) fn strain() -> Identity { Identity::Surface(STRAIN.into()) }

pub(crate) fn keys(of: &[&str]) -> BTreeSet<Key> { of.iter().map(|k| (*k).into()).collect() }

/// The world with a strain on site 0 and in host 12, a climate whose cycles
/// may be turned by `turn` degrees, and the hosting rule where `hosting`.
pub(crate) fn genesis(genes: &[&str], hosting: Option<Hosting>, turn: Option<i32>) -> Genesis {
    let mut g = Founding { seed: 17, sites: 3, population: 12, lineages: 1,
        cohort_size: 3, ecology: true, ..Default::default() }.generate().unwrap();
    g.rules.processes.clear();
    g.rules.epoch_ticks = 2;
    g.rules.shapes = crate::rules::default_shapes();
    g.rules.functions = crate::rules::default_functions();
    g.rules.traits.extend(["function:fix".into()]);
    g.rules.traits.extend(GENES.map(String::from));
    g.rules.accounts.insert(DETRITUS.into(), AccountKind::Matter {
        lineage: WORLD_LINE.into(), reserve: false, provision: false });
    g.rules.accounts.insert(RESERVE.into(), AccountKind::Matter {
        lineage: "lineage:0".into(), reserve: true, provision: false });
    let mut line = g.lineages["lineage:0"].clone();
    line.parent = Some("lineage:0".into());
    line.traits = keys(genes);
    g.lineages.insert(STRAIN.into(), line);
    g.spreads.identities = BTreeSet::from([strain()]);
    for site in g.sites.values_mut() { site.routes.clear(); }
    g.sites.get_mut(&0).unwrap().accounts.insert(STRAIN.into(), 40);
    g.population.lift(12).unwrap().accounts.insert(STRAIN.into(), 5);
    g.rules.hosting = hosting;
    g.rules.climate = Some(Climate { turn, ..Climate::default() });
    g
}

pub(crate) fn hosting() -> Option<Hosting> { Some(Hosting { period: 2, ..Hosting::default() }) }

pub(crate) fn start(g: Genesis) -> Session { Session::new(g, Execution::Individuals).unwrap() }

/// A joined participant of `s`.
pub(crate) fn join(s: &mut Session) -> Id {
    s.command(Command::Join).unwrap().strip_prefix("participant:").unwrap().parse().unwrap()
}

pub(crate) fn hold(s: &mut Session, participant: Id, lineage: &str) -> crate::Result<String> {
    s.command(Command::Hold { participant, lineage: lineage.into() })
}

fn steer(s: &mut Session, participant: Id, on: &[&str], off: &[&str]) -> crate::Result<String> {
    s.command(Command::Steer { participant, lineage: STRAIN.into(), on: keys(on), off: keys(off) })
}

fn turn(s: &mut Session, participant: Id, shift: i32) -> crate::Result<String> {
    s.command(Command::Turn { participant, lineage: WORLD_LINE.into(), shift })
}

fn read(s: &Session, at: Substrate, key: &str) -> u64 {
    spread::held(s.sim.state(), at, &Identity::Surface(key.into())).unwrap_or(0)
}

#[test]
fn a_world_where_nobody_holds_and_no_shift_is_set_hashes_as_before() {
    let g = genesis(&GENES, None, Some(5));
    let json = serde_json::to_string(&g).unwrap();
    assert!(!json.contains("\"held\"") && !json.contains("\"turn\":null"), "defaults serialize to nothing");
    let plain = genesis(&GENES, None, None);
    assert!(!serde_json::to_string(&plain).unwrap().contains("\"turn\""));
    let mut a = start(plain);
    join(&mut a);
    let state = serde_json::to_string(a.sim.state()).unwrap();
    assert!(!state.contains("\"held\"") && !state.contains("\"turn\""));
    // Holding then letting go, and a turn of nought, leave nothing behind.
    let mut b = start(genesis(&GENES, None, Some(5)));
    let q = join(&mut b);
    let clean = b.sim.state_hash();
    hold(&mut b, q, WORLD_LINE).unwrap();
    assert_ne!(b.sim.state_hash(), clean, "a hold is witnessed in the lineage");
    turn(&mut b, q, 3).unwrap();
    turn(&mut b, q, 0).unwrap();
    b.command(Command::Unhold { participant: q, lineage: WORLD_LINE.into() }).unwrap();
    assert_eq!(b.sim.state_hash(), clean);
}

#[test]
fn a_surface_and_a_world_lineage_can_be_held_by_two_and_released() {
    let mut s = start(genesis(&GENES, None, Some(5)));
    let (a, b) = (join(&mut s), join(&mut s));
    assert_eq!(hold(&mut s, a, STRAIN).unwrap(), "held");
    hold(&mut s, a, WORLD_LINE).unwrap();
    hold(&mut s, b, STRAIN).unwrap();
    assert_eq!(s.sim.holds(a), vec![STRAIN.to_string(), WORLD_LINE.to_string()]);
    assert_eq!(s.sim.holders(STRAIN), vec![a, b], "a lineage may be shared (case 1221)");
    assert!(hold(&mut s, a, STRAIN).is_err(), "not twice");
    s.command(Command::Unhold { participant: a, lineage: STRAIN.into() }).unwrap();
    assert_eq!(s.sim.holders(STRAIN), vec![b]);
    assert!(s.command(Command::Unhold { participant: a, lineage: STRAIN.into() }).is_err());
}

#[test]
fn a_critter_lineage_an_unknown_participant_or_lineage_cannot_be_held() {
    let mut s = start(genesis(&GENES, None, Some(5)));
    let p = join(&mut s);
    let why = hold(&mut s, p, "lineage:0").unwrap_err();
    assert!(why.contains("taken, not held"), "{why}");
    assert!(hold(&mut s, p, "lineage:nowhere").is_err());
    assert!(hold(&mut s, 99_999, STRAIN).is_err());
    assert!(hold(&mut s, 0, STRAIN).is_err(), "a world body is no participant");
    assert!(s.sim.holds(p).is_empty());
}

#[test]
fn steering_is_refused_for_a_non_holder_and_switches_as_the_authored_switch_does() {
    let mut s = start(genesis(&["gene:rot"], None, None));
    let (p, other) = (join(&mut s), join(&mut s));
    assert!(steer(&mut s, p, &["gene:leak"], &[]).is_err(), "not held yet");
    hold(&mut s, p, STRAIN).unwrap();
    assert!(steer(&mut s, other, &["gene:leak"], &[]).is_err(), "another's hold is not mine");
    let mut twin = start(genesis(&["gene:rot"], None, None));
    join(&mut twin);
    join(&mut twin);
    let revision = s.sim.state().lineages[STRAIN].revision;
    steer(&mut s, p, &["gene:leak"], &["gene:rot"]).unwrap();
    twin.command(Command::Spread(spread::Command::Switch {
        lineage: STRAIN.into(), on: keys(&["gene:leak"]), off: keys(&["gene:rot"]) })).unwrap();
    let (ours, theirs) = (&s.sim.state().lineages[STRAIN], &twin.sim.state().lineages[STRAIN]);
    assert_eq!(ours.traits, theirs.traits);
    assert_eq!(ours.revision, theirs.revision);
    assert_eq!(ours.revision, revision + 1);
    assert_eq!(s.sim.state().spreads, twin.sim.state().spreads);
    assert_eq!(s.sim.matter(), twin.sim.matter());
    assert!(steer(&mut s, p, &["gene:leak"], &[]).is_err(), "a switch that changes nothing is refused");
    assert!(steer(&mut s, p, &["gene:unknown"], &[]).is_err());
}

#[test]
fn a_steered_rot_gene_makes_the_surface_take_dead_matter_on_its_next_turn_until_switched_off() {
    let mut g = genesis(&[], hosting(), None);
    g.sites.get_mut(&0).unwrap().accounts.insert(DETRITUS.into(), 30);
    let mut s = start(g);
    let p = join(&mut s);
    hold(&mut s, p, STRAIN).unwrap();
    s.advance(2).unwrap();
    assert_eq!(read(&s, Substrate::Site(0), STRAIN), 40, "no rot gene, no taking");
    steer(&mut s, p, &["gene:rot"], &[]).unwrap();
    let total = s.sim.matter();
    s.advance(2).unwrap();
    assert_eq!(read(&s, Substrate::Site(0), STRAIN), 44, "a tenth of 40 taken");
    assert_eq!(s.sim.matter(), total);
    steer(&mut s, p, &[], &["gene:rot"]).unwrap();
    s.advance(4).unwrap();
    assert_eq!(read(&s, Substrate::Site(0), STRAIN), 44, "switched off, it takes no more");
}

#[test]
fn a_turn_needs_a_holder_a_bound_and_stays_within_it() {
    let mut s = start(genesis(&GENES, None, Some(5)));
    let p = join(&mut s);
    assert!(turn(&mut s, p, 1).is_err(), "no holder");
    hold(&mut s, p, STRAIN).unwrap();
    assert!(turn(&mut s, p, 1).is_err(), "a surface does not turn cycles");
    hold(&mut s, p, WORLD_LINE).unwrap();
    assert!(turn(&mut s, p, 6).is_err() && turn(&mut s, p, -6).is_err(), "beyond the bound");
    let site = |s: &Session| s.sim.temperature(1).unwrap();
    let before = site(&s);
    turn(&mut s, p, 5).unwrap();
    assert_eq!(site(&s), before + 5);
    turn(&mut s, p, -5).unwrap();
    assert_eq!(site(&s), before - 5);
    let mut unbound = start(genesis(&GENES, None, None));
    let q = join(&mut unbound);
    hold(&mut unbound, q, WORLD_LINE).unwrap();
    assert!(turn(&mut unbound, q, 1).is_err(), "no bound, no turning");
    let mut dry = start({ let mut g = genesis(&GENES, None, None); g.rules.climate = None; g });
    let r = join(&mut dry);
    hold(&mut dry, r, WORLD_LINE).unwrap();
    assert!(turn(&mut dry, r, 0).is_err(), "no climate");
}

#[test]
fn a_turn_moves_the_strain_read_from_temperature() {
    let mut s = start(genesis(&GENES, None, Some(5)));
    let p = join(&mut s);
    hold(&mut s, p, WORLD_LINE).unwrap();
    let sim = &s.sim;
    let (state, rules) = (sim.state(), &sim.genesis().rules);
    let body = state.population.get(12).unwrap();
    let at = (body.place, &state.sites[&body.place]);
    let owed = |turn: i64| (0..20).map(|t| crate::climate::owed_shifted(rules, 17,
        &state.lineages, body, at, t, turn)).sum::<u64>();
    assert_eq!(owed(0), (0..20).map(|t| crate::climate::owed(rules, 17, &state.lineages, body, at, t)).sum::<u64>());
    // The flat equator sits at the band's top edge: five degrees hotter is five outside it.
    assert!(owed(5) > owed(0), "{} against {}", owed(5), owed(0));
    turn(&mut s, p, 5).unwrap();
    assert_eq!(s.sim.state().turn, 5);
}

#[test]
fn a_session_of_holds_steers_turns_and_rounds_replays_and_reloads_to_one_hash() {
    let mut g = genesis(&[], hosting(), Some(5));
    g.sites.get_mut(&0).unwrap().accounts.insert(DETRITUS.into(), 30);
    let mut s = start(g);
    let p = join(&mut s);
    hold(&mut s, p, STRAIN).unwrap();
    hold(&mut s, p, WORLD_LINE).unwrap();
    steer(&mut s, p, &["gene:rot"], &[]).unwrap();
    s.advance(3).unwrap();
    turn(&mut s, p, -4).unwrap();
    let json = serde_json::to_vec(&s.save()).unwrap();
    let mid = Session::load_json(&json, Execution::Individuals).unwrap();
    assert_eq!(mid.sim.state_hash(), s.sim.state_hash());
    s.advance(4).unwrap();
    steer(&mut s, p, &["gene:leak"], &["gene:rot"]).unwrap();
    s.advance(2).unwrap();
    s.command(Command::Unhold { participant: p, lineage: STRAIN.into() }).unwrap();
    let json = serde_json::to_vec(&s.save()).unwrap();
    for mode in [Execution::Individuals, Execution::Grouped] {
        let loaded = Session::load_json(&json, mode).unwrap();
        assert_eq!(loaded.sim.state_hash(), s.sim.state_hash());
        assert_eq!(loaded.sim.state().turn, -4);
        assert_eq!(loaded.entries, s.entries);
    }
    // The saved middle carries on to the same place.
    let mut carried = mid;
    carried.advance(4).unwrap();
    steer(&mut carried, p, &["gene:leak"], &["gene:rot"]).unwrap();
    carried.advance(2).unwrap();
    carried.command(Command::Unhold { participant: p, lineage: STRAIN.into() }).unwrap();
    assert_eq!(carried.sim.state_hash(), s.sim.state_hash());
}

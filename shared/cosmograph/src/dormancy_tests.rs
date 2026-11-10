// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Dormancy (cases 1192 to 1197): the rule unset changes nothing; a seed
//! bank keeps a lean lineage alive beside a control without one; a
//! propagule is skipped, uncounted and never offered; its upkeep, death and
//! waking follow their stated rates; and a run replays.

use crate::{
    Execution, Founding, Session, Simulation,
    bodied::Bodies,
    directing::readings,
    ecology::{Rates, Web},
    rules::{Dormancy, Query},
    schema::{Id, Method},
    simulation::Genesis,
};
use std::collections::BTreeSet;

fn founding(seed: u64, dormancy: Option<Dormancy>) -> Genesis {
    let grid = crate::map::Grid { width: 2, height: 2, ..crate::map::Grid::drawn(seed) };
    Founding {
        seed, sites: 4, population: 60, cohort_size: 1, lineages: 5, ecology: true,
        ecology_rates: Rates::WorldTime, experimental_food_contract: true, web: Web::Diets,
        bodies: Some(Bodies::default()), map: Some(crate::map::Layout::Grid(grid)),
        dormancy, ..Founding::default()
    }.generate().unwrap()
}

/// A rule with the seed share fixed and a short season, in minutes a tick.
fn rule(share: u64, season: u64) -> Dormancy {
    Dormancy { seed: [share, share], season, ..Dormancy::default() }
}

/// Keeps only the processes `keep` names (and the sun), so what is left is all that acts.
fn only(g: &mut Genesis, keep: &[&str]) {
    g.rules.processes.retain(|id, _| id == "ecology:sunlight" || keep.iter().any(|k| id.starts_with(k)));
}

/// The producer lineage's birth at `period`, asking `held` of its body.
fn eager(g: &mut Genesis, period: u64, held: u64) {
    let birth = g.rules.processes.get_mut("ecology:birth-0").unwrap();
    birth.period = Some(period);
    for q in &mut birth.requires {
        if let crate::rules::Query::Account { at_least, .. } = q { *at_least = held; }
    }
}

fn producers(sim: &Simulation) -> Vec<Id> {
    sim.state().population.groups.iter()
        .filter(|(_, g)| g.entity.alive && g.entity.lineage == "lineage:0" && !sim.genesis().rules.dormant(&g.entity))
        .map(|(f, _)| *f).collect()
}

/// A propagule of the first producer, `count` of them holding `matter`
/// each, taken from its site's soil so no matter is made.
fn lay(sim: &mut Simulation, matter: u64, count: u64) -> Id {
    let parent = *producers(sim).first().unwrap();
    let rules = sim.genesis().rules.clone();
    let d = rules.dormancy.clone().unwrap();
    let mut e = sim.state().population.get(parent).unwrap().clone();
    e.accounts.clear();
    for part in e.parts_mut().map(|(_, p)| p).filter(|p| !p.matter.is_empty()) { part.matter.clear(); }
    e.born = sim.state().tick;
    e.arrived = e.born;
    e.traits.insert(d.key);
    let key = "matter:0-0";
    if crate::anatomy::root_store::allocation(&e, &rules, key).is_some() {
        crate::anatomy::root_store::bank(&mut e, &rules, key, matter).unwrap().unwrap();
    } else {
        crate::anatomy::give(&mut e, &rules, key, matter).unwrap().unwrap();
    }
    let site = e.resident_site().unwrap();
    *sim.state.sites.get_mut(&site).unwrap().accounts.get_mut("world:soil").unwrap() -= matter * count;
    sim.state.population.insert(e, count).unwrap()
}

fn matter_of(sim: &Simulation, id: Id) -> u128 {
    let e = sim.state().population.get(id).unwrap();
    crate::meaning::mass(&crate::anatomy::books(e), &sim.genesis().rules)
}

fn soil(sim: &Simulation, site: Id) -> u64 {
    sim.state().sites[&site].accounts["world:soil"]
}

fn kill(sim: &mut Simulation, ids: &[Id]) {
    for id in ids { sim.state.population.lift(*id).unwrap().alive = false; }
}

#[test]
fn unset_the_founding_and_its_rules_are_what_they_were() {
    let (a, b) = (founding(5, None), founding(5, None));
    assert_eq!(a, b);
    assert!(a.rules.dormancy.is_none());
    let json = serde_json::to_string(&a).unwrap();
    assert!(!json.contains("dormancy") && !json.contains("stage:dormant") && !json.contains("ecology:wake"));
    assert!(!a.rules.processes.keys().any(|k| k.contains("dormant") || k.contains("wake")));
    let birth = &a.rules.processes["ecology:birth-0"];
    assert!(matches!(&birth.effects[0], crate::rules::Effect::Birth { provision, .. } if provision["matter:0-0"] == 10));
    let hash = |g: &Genesis| {
        let mut s = Simulation::new(g.clone(), Execution::Individuals).unwrap();
        s.advance(500).unwrap();
        s.state_hash()
    };
    assert_eq!(hash(&a), hash(&b));
    // The rule is the only difference between the two worlds.
    let on = founding(5, Some(Dormancy::default()));
    assert_ne!(hash(&a), hash(&on));
}

/// Short-period births of a producer lineage in a small world: dormancy on
/// or off, the young come as propagules or as seedlings.
fn lean(dormancy: Option<Dormancy>) -> Simulation {
    let mut g = founding(5, dormancy);
    eager(&mut g, 40, 20);
    Simulation::new(g, Execution::Individuals).unwrap()
}

#[test]
fn a_seed_bank_outlasts_a_lean_interval_that_ends_the_lineage_without_one() {
    let mut ends = Vec::new();
    for dormancy in [None, Some(rule(10, 60))] {
        let on = dormancy.is_some();
        let mut sim = lean(dormancy);
        let before = sim.matter();
        sim.advance(300).unwrap();
        assert_eq!(sim.matter(), before);
        let kept = readings::propagules(&sim).get("lineage:0").copied().unwrap_or_default();
        assert_eq!(kept.count > 0, on, "only the world with dormancy holds a seed bank");
        let held = sim.matter();
        // The lean interval: every adult is lost, then the world runs on.
        let adults = producers(&sim);
        assert!(!adults.is_empty());
        kill(&mut sim, &adults);
        sim.advance(1_500).unwrap();
        assert_eq!(sim.matter(), held, "the interval makes and loses no matter");
        ends.push((producers(&sim).len(), readings::propagules(&sim).contains_key("lineage:0")));
    }
    assert_eq!(ends[0].0, 0, "without dormancy the lineage is gone");
    assert!(ends[1].0 > 0, "with it the lineage has living adults again");
}

#[test]
fn a_propagule_is_skipped_uncounted_and_never_offered() {
    let mut g = founding(5, Some(rule(10, 1_000_000)));
    eager(&mut g, 10, 1);
    let mut sim = Simulation::new(g, Execution::Individuals).unwrap();
    let parent = producers(&sim)[0];
    let site = sim.state().population.get(parent).unwrap().resident_site().unwrap();
    let plants: BTreeSet<_> = ["lineage:0".to_string()].into();
    let (crowd, members) = (sim.sensed(parent, "sense:crowding"), sim.members_at(site, &plants));
    let seed = lay(&mut sim, 10, 5);
    assert_eq!((sim.sensed(parent, "sense:crowding"), sim.members_at(site, &plants)), (crowd, members));
    let now = sim.state().tick;
    sim.advance(3_000).unwrap();
    // Not grown, not seeding: no process but upkeep (far off) and waking (far off) visits it.
    assert_eq!(matter_of(&sim, seed), 10);
    assert!(sim.state().relations.iter().all(|r| !(r.subject == seed || r.object == seed)), "no birth from a propagule");
    assert_eq!(sim.state().population.get(seed).unwrap().born, now);
    assert!(sim.state().population.get(seed).unwrap().alive);
    // Awake, the same body grows and seeds like its parent.
    let mut awake = lean(None);
    let parent = producers(&awake)[0];
    awake.advance(400).unwrap();
    assert!(awake.state().relations.iter().any(|r| r.subject == parent && r.kind == "sim:child"));
}

#[test]
fn a_propagule_is_no_extinction_no_heir_no_start_and_reports_apart() {
    let build = |lay_seed: bool| {
        let mut sim = Simulation::new(founding(5, Some(rule(10, 1_000_000))), Execution::Individuals).unwrap();
        let adults = producers(&sim);
        let site = sim.state().population.get(adults[0]).unwrap().resident_site().unwrap();
        let seed = lay_seed.then(|| lay(&mut sim, 10, 3));
        if let Some(seed) = seed {
            // Playable in every way but being dormant.
            sim.state.population.lift(seed).unwrap().method = Method::Deliberative;
        }
        (sim, adults, seed, site)
    };
    let (before, ..) = build(true);
    let (mut after, adults, seed, site) = build(true);
    let seed = seed.unwrap();
    kill(&mut after, &adults);
    assert!(!readings::raised(&before, &after).iter().any(|r| matches!(r, readings::Raised::Extinct { lineage } if lineage == "lineage:0")),
        "a lineage holding only propagules is not extinct");
    let apart = readings::propagules(&after);
    assert_eq!(apart["lineage:0"].count, 3);
    assert_eq!(apart["lineage:0"].matter, 30);
    assert!(readings::heirs(&after, adults[0]).is_empty());
    assert!(readings::starts(&after, "lineage:0").is_empty());
    assert!(readings::start_at(&after, "lineage:0", site).is_err());
    // Awake, it would be offered: the dormant trait alone withholds it.
    let key = after.genesis().rules.dormancy.clone().unwrap().key;
    after.state.population.lift(seed).unwrap().traits.remove(&key);
    assert_eq!(readings::heirs(&after, adults[0]), vec![seed]);
    assert_eq!(readings::starts(&after, "lineage:0").len(), 1);
    // A control with no seed bank does read as extinct.
    let (before, ..) = build(false);
    let (mut after, adults, ..) = build(false);
    kill(&mut after, &adults);
    assert!(readings::raised(&before, &after).iter().any(|r| matches!(r, readings::Raised::Extinct { lineage } if lineage == "lineage:0")));
}

#[test]
fn dormant_upkeep_is_its_fraction_of_the_active_rate_and_a_drained_seed_dies_to_the_site() {
    let mut g = founding(5, Some(rule(10, 1_000_000)));
    only(&mut g, &["ecology:dormant-"]);
    let active = g.rules.processes.get("ecology:dormant-upkeep-0").unwrap().period.unwrap();
    let rules_active = founding(5, None).rules.processes["ecology:upkeep-0"].period.unwrap();
    assert_eq!(active, rules_active * 10, "a tenth of the active rate");
    let mut sim = Simulation::new(g, Execution::Individuals).unwrap();
    let (total, parent) = (sim.matter(), producers(&sim)[0]);
    let site = sim.state().population.get(parent).unwrap().resident_site().unwrap();
    let seed = lay(&mut sim, 3, 1);
    let at = soil(&sim, site);
    sim.advance(active * 2 + 1).unwrap();
    assert_eq!(matter_of(&sim, seed), 1, "two periods, two matter");
    assert_eq!(soil(&sim, site), at + 2);
    sim.advance(active + 5).unwrap();
    let e = sim.state().population.get(seed).unwrap();
    assert!(!e.alive, "a propagule whose matter runs out dies");
    assert_eq!(matter_of(&sim, seed), 0);
    assert_eq!(soil(&sim, site), at + 3, "all its matter is in the site");
    assert_eq!(sim.matter(), total);
}

/// A world where only waking acts, with `n` propagules of `share` laid at one
/// site, the light there set to `light` for each plant it holds.
fn waking(share: u64, n: u64, light: u64, mode: Execution) -> (Simulation, Vec<Id>) {
    let mut g = founding(5, Some(rule(share, 100)));
    only(&mut g, &["ecology:wake-"]);
    g.rules.processes.remove("ecology:sunlight");
    let mut sim = Simulation::new(g, mode).unwrap();
    let parent = producers(&sim)[0];
    let site = sim.state().population.get(parent).unwrap().resident_site().unwrap();
    let ids: Vec<Id> = (0..n).map(|_| lay(&mut sim, share, 1)).collect();
    let crowding = sim.sensed(parent, "sense:crowding").unwrap();
    sim.state.sites.get_mut(&site).unwrap().accounts.insert("sim:energy".into(), light * crowding);
    (sim, ids)
}

fn awake(sim: &Simulation, ids: &[Id]) -> Vec<Id> {
    let key = sim.genesis().rules.dormancy.clone().unwrap().key;
    ids.iter().copied().filter(|id| !sim.state().population.get(*id).unwrap().traits.contains(&key)).collect()
}

#[test]
fn about_the_wake_share_wakes_a_season_and_which_ones_replays() {
    let n = 200;
    let mut runs = Vec::new();
    for mode in [Execution::Individuals, Execution::Grouped, Execution::Individuals] {
        let (mut sim, ids) = waking(10, n, 4_000, mode);
        sim.advance(99).unwrap();
        assert!(awake(&sim, &ids).is_empty(), "nothing wakes before the season");
        sim.advance(2).unwrap();
        runs.push(awake(&sim, &ids));
    }
    let woke = runs[0].len() as u64;
    assert!((n / 10..=n * 2 / 5).contains(&woke), "{woke} of {n} woke, about a quarter");
    assert_eq!(runs[0], runs[1], "the same under both executions");
    assert_eq!(runs[0], runs[2], "the same on a replayed run");
}

#[test]
fn a_heavier_seed_wakes_at_a_light_share_a_lighter_one_lacks() {
    let threshold = |share| Dormancy::waking(3_000, share);
    assert_eq!((threshold(5), threshold(10), threshold(20)), (6_000, 3_000, 1_500));
    for share in [5, 20] {
        let g = founding(5, Some(rule(share, 100)));
        let need = g.rules.processes["ecology:wake-0"].requires.iter().find_map(|q| match q {
            Query::Sensed { at_least, .. } => Some(*at_least), _ => None }).unwrap();
        assert_eq!(need, threshold(share), "the waking process reads the lineage's own share");
    }
    let (mut light, ids) = waking(5, 100, 3_000, Execution::Individuals);
    light.advance(101).unwrap();
    assert!(awake(&light, &ids).is_empty(), "a light seed lacks 3,000");
    let (mut heavy, ids) = waking(20, 100, 3_000, Execution::Individuals);
    heavy.advance(101).unwrap();
    assert!(!awake(&heavy, &ids).is_empty(), "a heavy seed wakes at it");
}

#[test]
fn a_heavier_seed_makes_fewer_births_from_the_same_root() {
    let births = |share| {
        let mut g = founding(5, Some(rule(share, 1_000_000)));
        only(&mut g, &["ecology:birth-0"]);
        g.rules.processes.remove("ecology:sunlight");
        eager(&mut g, 10, share);
        let mut sim = Simulation::new(g, Execution::Individuals).unwrap();
        let parent = producers(&sim)[0];
        sim.advance(400).unwrap();
        let held = matter_of(&sim, parent);
        let n = sim.state().relations.iter().filter(|r| r.subject == parent && r.kind == "sim:child").count();
        (n, held)
    };
    let ((light, left_light), (heavy, left_heavy)) = (births(5), births(20));
    // A seed is filled from the root's store of 20, which is spent at its share.
    assert_eq!((light, heavy), (4, 1), "cheap seeds against robust ones");
    assert_eq!(left_light, left_heavy, "the same matter is spent");
}

#[test]
fn a_run_with_the_rule_set_replays_to_the_same_hash_under_both_executions() {
    let mut g = founding(5, Some(rule(10, 60)));
    eager(&mut g, 40, 20);
    for mode in [Execution::Individuals, Execution::Grouped] {
        let mut s = Session::new(g.clone(), mode).unwrap();
        s.advance(250).unwrap();
        assert!(!readings::propagules(&s.sim).is_empty());
        s.advance(450).unwrap();
        let json = serde_json::to_vec(&s.save()).unwrap();
        let mut loaded = Session::load_json(&json, mode).unwrap();
        assert_eq!(loaded.sim.state_hash(), s.sim.state_hash());
        s.advance(400).unwrap();
        loaded.advance(400).unwrap();
        assert_eq!(loaded.sim.state_hash(), s.sim.state_hash());
        assert_eq!(loaded.sim.matter(), s.sim.matter());
    }
}

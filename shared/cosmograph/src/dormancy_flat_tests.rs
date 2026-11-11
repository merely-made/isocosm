// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Dormancy in the flat rate sets (cases 1192 to 1197, A1 and stock bodied
//! worlds): the rule unset changes nothing; a seed bank returns a lineage a
//! lean interval ended; a propagule is skipped, uncounted and never offered;
//! its upkeep and death follow their stated rates; waking is a keyed draw
//! gated on crowding, heavier seeds waking at a greater count; a run replays.

use crate::{
    Execution, Founding, Session, Simulation,
    bodied::Bodies,
    directing::{Played, readings},
    ecology::Rates,
    rules::{Dormancy, Query},
    schema::{Id, Method},
    simulation::Genesis,
};
use std::collections::BTreeSet;

/// Mesocosm's seed-9 world under A1, bodied and played, with the rule `rule`.
fn founding(rates: Rates, rule: Option<Dormancy>) -> Founding {
    let grid = crate::map::Grid { width: 3, height: 2, ..crate::map::Grid::drawn(9) };
    Founding {
        seed: 9, sites: 6, population: 256, map: Some(crate::map::Layout::Grid(grid)), ecology: true,
        ecology_rates: rates,
        bodies: Some(Bodies { hazard: crate::harm::hazard::RATE, wound: crate::harm::hazard::CELLS, ..Bodies::default() }),
        played: Some(Played { lineage: 1, region_sites: 2 }),
        dormancy: rule, ..Founding::default()
    }
}

fn world(rule: Option<Dormancy>) -> Genesis {
    founding(Rates::A1, rule).generate().unwrap()
}

fn rule(share: u64, season: u64) -> Dormancy {
    Dormancy { seed: [share, share], season_ticks: Some(season), ..Dormancy::flat() }
}

/// Only the processes `keep` names act (and the sun).
fn only(g: &mut Genesis, keep: &[&str]) {
    g.rules.processes.retain(|id, _| id == "ecology:sunlight" || keep.iter().any(|k| id.starts_with(k)));
}

/// Every awake member of `lineage`, by identity.
fn awake(sim: &Simulation, lineage: &str) -> Vec<Id> {
    sim.state().population.groups.iter()
        .filter(|(_, g)| g.entity.alive && g.entity.lineage == lineage && !sim.genesis().rules.dormant(&g.entity))
        .flat_map(|(f, g)| *f..*f + g.count).collect()
}

fn living(sim: &Simulation, lineage: &str) -> u64 {
    sim.state().population.groups.values()
        .filter(|g| g.entity.alive && g.entity.lineage == lineage && !sim.genesis().rules.dormant(&g.entity))
        .map(|g| g.count).sum()
}

/// A propagule of `lineage:0`'s first adult: `count` of them holding `matter`
/// each, taken from the site's soil so no matter is made.
fn lay(sim: &mut Simulation, matter: u64, count: u64) -> Id {
    let parent = awake(sim, "lineage:0")[0];
    lay_of(sim, parent, None, matter, count)
}

/// The same, of `parent`, at the place of `beside` where it is named.
fn lay_of(sim: &mut Simulation, parent: Id, beside: Option<Id>, matter: u64, count: u64) -> Id {
    let key = format!("matter:{}-0", sim.state().population.get(parent).unwrap().lineage.trim_start_matches("lineage:"));
    let rules = sim.genesis().rules.clone();
    let dormant = rules.dormancy.clone().unwrap().key;
    let mut e = sim.state().population.get(parent).unwrap().clone();
    e.accounts.clear();
    for part in e.parts_mut().map(|(_, p)| p).filter(|p| !p.matter.is_empty()) { part.matter.clear(); }
    e.born = sim.state().tick;
    e.arrived = e.born;
    e.traits.insert(dormant);
    if let Some(next) = beside {
        let place = sim.state().population.get(next).unwrap().place.clone();
        e.place = place;
        e.patch = None;
    }
    crate::anatomy::give(&mut e, &rules, &key, matter).unwrap().unwrap();
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

fn site_of(sim: &Simulation, id: Id) -> Id {
    sim.state().population.get(id).unwrap().resident_site().unwrap()
}

/// Ids of lineage `i`'s awake adults.
fn adults(sim: &Simulation, lineages: &[&str]) -> Vec<Id> {
    lineages.iter().flat_map(|l| awake(sim, l)).collect()
}

const PLANTS: [&str; 2] = ["lineage:0", "lineage:3"];

#[test]
fn unset_the_world_never_mentions_dormancy_and_its_rates_are_the_flat_ones() {
    let (a, b) = (world(None), world(None));
    assert_eq!(a, b);
    let json = serde_json::to_string(&a).unwrap();
    for word in ["dormancy", "stage:dormant", "ecology:wake", "dormant-", "season_ticks"] {
        assert!(!json.contains(word), "an unset world mentions {word}");
    }
    let birth = &a.rules.processes["ecology:birth-0"];
    assert!(matches!(&birth.effects[0], crate::rules::Effect::Birth { provision, .. } if provision["matter:0-0"] == 8));
    assert!(birth.requires.iter().any(|q| matches!(q, Query::Account { at_least: 12, .. })));
    let hash = |g: &Genesis| {
        let mut s = Simulation::new(g.clone(), Execution::Individuals).unwrap();
        s.advance(120).unwrap();
        s.state_hash()
    };
    assert_eq!(hash(&a), hash(&b));
    assert_ne!(hash(&a), hash(&world(Some(Dormancy::flat()))));
}

#[test]
fn the_rule_changes_only_the_producers_births_and_adds_the_dormant_processes() {
    let (off, on) = (world(None), world(Some(Dormancy::flat())));
    let rule = on.rules.dormancy.clone().unwrap();
    assert_eq!(rule.seeds.len(), 2, "the two producer lineages seed");
    for (i, share) in [(0u32, rule.drawn(9, 0)), (3, rule.drawn(9, 3))] {
        assert!((5..=20).contains(&share));
        assert_eq!(rule.seeds[&format!("ecology:birth-{i}")], share);
        let birth = &on.rules.processes[&format!("ecology:birth-{i}")];
        assert!(matches!(&birth.effects[0], crate::rules::Effect::Birth { provision, .. }
            if provision[&format!("matter:{i}-0")] == share), "the share replaces the flat child mass of 8");
        // The matter a parent keeps after a birth is what it kept: 12 held, 8 given, 4 left.
        assert!(birth.requires.iter().any(|q| matches!(q, Query::Account { at_least, .. } if *at_least == 4 + share)));
        for kind in ["dormant-upkeep", "dormant-death", "wake"] {
            assert!(on.rules.processes.contains_key(&format!("ecology:{kind}-{i}")));
        }
    }
    for (id, p) in &off.rules.processes {
        if !["ecology:birth-0", "ecology:birth-3"].contains(&id.as_str()) {
            assert_eq!(on.rules.processes.get(id), Some(p), "{id} is as it was");
        }
    }
    assert_eq!(on.rules.processes.len(), off.rules.processes.len() + 6);
}

#[test]
fn the_dormant_rates_follow_the_active_ones_the_mass_law_scaled() {
    let mut f = founding(Rates::A1, Some(Dormancy::flat()));
    f.mass_law = Some(crate::ecology::MassLaw::default());
    let g = f.generate().unwrap();
    for i in [0, 3] {
        let active = &g.rules.processes[&format!("ecology:upkeep-{i}")];
        let dormant = &g.rules.processes[&format!("ecology:dormant-upkeep-{i}")];
        assert_eq!(dormant.period.unwrap(), active.period.unwrap() * 10, "a tenth of the scaled rate");
        assert_eq!(dormant.effects, active.effects, "the same payment");
    }
}

#[test]
fn the_stock_rates_found_and_hold_a_bank_too() {
    let mut f = founding(Rates::Stock, Some(Dormancy::flat()));
    f.played = None;
    let g = f.generate().unwrap();
    let share = g.rules.dormancy.as_ref().unwrap().seeds["ecology:birth-0"];
    let birth = &g.rules.processes["ecology:birth-0"];
    assert!(birth.requires.iter().any(|q| matches!(q, Query::Account { at_least, .. } if *at_least == 22 + share)), "30 less the 8");
    let mut sim = Simulation::new(g, Execution::Individuals).unwrap();
    let before = sim.matter();
    sim.advance(200).unwrap();
    assert_eq!(sim.matter(), before);
    assert!(!readings::propagules(&sim).is_empty());
}

#[test]
fn the_rule_needs_an_ecology_and_bodies() {
    let mut f = founding(Rates::A1, Some(Dormancy::flat()));
    f.bodies = None;
    f.played = None;
    assert!(f.generate().is_err());
    let mut f = founding(Rates::A1, Some(Dormancy::flat()));
    f.ecology = false;
    f.played = None;
    assert!(f.generate().is_err());
}

#[test]
fn a_seed_bank_returns_producers_a_lean_interval_ended_and_none_return_without_one() {
    let mut returned = Vec::new();
    for rule in [None, Some(Dormancy::flat())] {
        let on = rule.is_some();
        let mut sim = Simulation::new(world(rule), Execution::Individuals).unwrap();
        sim.advance(100).unwrap();
        let seeded = readings::propagules(&sim).values().map(|p| p.count).sum::<u64>();
        assert_eq!(seeded > 0, on, "only the world with dormancy holds a seed bank");
        // The lean interval: every adult producer is lost, and the world runs on.
        let held = sim.matter();
        let lost = adults(&sim, &PLANTS);
        assert!(!lost.is_empty());
        kill(&mut sim, &lost);
        assert_eq!(sim.matter(), held, "losing the adults makes and loses no matter");
        let mut most = 0;
        for _ in 0..6 {
            sim.advance(100).unwrap();
            most = most.max(PLANTS.iter().map(|l| living(&sim, l)).sum::<u64>());
        }
        assert_eq!(sim.matter(), held, "the interval makes and loses no matter");
        returned.push(most);
    }
    assert_eq!(returned[0], 0, "without dormancy the producers are gone");
    assert!(returned[1] > 0, "with it producers live again: {}", returned[1]);
}


/// A world where only `keep` acts, with `count` seeds of `matter` laid beside
/// a grazer and every other plant gone; whether they are `dormant` or awake.
fn fed_on(keep: &[&str], dormant: bool, matter: u64, count: u64) -> (Simulation, Id, Id) {
    let mut g = world(Some(rule(10, 1_000_000)));
    only(&mut g, keep);
    let mut sim = Simulation::new(g, Execution::Individuals).unwrap();
    let (parent, grazer) = (awake(&sim, "lineage:0")[0], awake(&sim, "lineage:1")[0]);
    // The played grazer is directed by its player; here it feeds as an ordinary one does.
    let method = sim.state().population.get(awake(&sim, "lineage:2")[0]).unwrap().method;
    sim.state.population.lift(grazer).unwrap().method = method;
    let (seed, site) = (lay_of(&mut sim, parent, Some(grazer), matter, count), site_of(&sim, grazer));
    let others: Vec<Id> = adults(&sim, &PLANTS);
    kill(&mut sim, &others);
    if !dormant {
        let key = sim.genesis().rules.dormancy.clone().unwrap().key;
        sim.state.population.lift(seed).unwrap().traits.remove(&key);
    }
    (sim, seed, site)
}

#[test]
fn a_propagule_is_never_fed_on_and_is_not_grown_or_bred_but_an_awake_one_is() {
    // Offered to a grazer beside the only plants left: a seed is no target, an awake plant is.
    let (seeds, seed, _) = fed_on(&["ecology:feed-1"], true, 10, 5);
    let (plants, plant, _) = fed_on(&["ecology:feed-1"], false, 10, 5);
    let offered = |sim: &Simulation| {
        let grazer = awake(sim, "lineage:1")[0];
        let feed = &sim.genesis().rules.processes["ecology:feed-1"];
        (sim.choose_target(grazer, feed), sim.prey_matter(grazer, feed))
    };
    assert_eq!(offered(&seeds), (None, 0), "{seed} is no prey");
    let (target, matter) = offered(&plants);
    assert!(target.is_some_and(|t| (plant..plant + 5).contains(&t)) && matter > 0, "the awake plant is");
    // Grown and bred: the producer's own growth and birth.
    let (mut seeds, seed, _) = fed_on(&["ecology:produce-0", "ecology:birth-0", "ecology:sunlight"], true, 10, 1);
    let (mut plants, plant, _) = fed_on(&["ecology:produce-0", "ecology:birth-0", "ecology:sunlight"], false, 10, 1);
    for sim in [&mut seeds, &mut plants] { sim.advance(300).unwrap(); }
    assert_eq!(matter_of(&seeds, seed), 10, "a seed is not grown");
    assert!(seeds.state().relations.iter().all(|r| !(r.subject == seed || r.object == seed)), "no birth from a propagule");
    assert!(plants.state().relations.iter().any(|r| r.subject == plant && r.kind == "sim:child") || matter_of(&plants, plant) > 10,
        "the awake one grows or breeds");
}

#[test]
fn a_propagule_is_no_count_no_heir_no_start_no_extinction_and_reports_apart() {
    let build = |lay_seed: bool| {
        let mut sim = Simulation::new(world(Some(rule(10, 1_000_000))), Execution::Individuals).unwrap();
        let adult = awake(&sim, "lineage:0");
        let site = site_of(&sim, adult[0]);
        let plants: BTreeSet<_> = PLANTS.iter().map(|s| s.to_string()).collect();
        let (crowd, members) = (sim.sensed(adult[0], "sense:crowding"), sim.members_at(site, &plants));
        let seed = lay_seed.then(|| lay(&mut sim, 10, 3));
        assert_eq!((sim.sensed(adult[0], "sense:crowding"), sim.members_at(site, &plants)), (crowd, members), "not counted");
        if let Some(seed) = seed {
            // Playable in every way but being dormant.
            sim.state.population.lift(seed).unwrap().method = Method::Deliberative;
        }
        (sim, adult, seed, site)
    };
    let (before, ..) = build(true);
    let (mut after, adults, seed, site) = build(true);
    let seed = seed.unwrap();
    kill(&mut after, &adults);
    assert!(!readings::raised(&before, &after).iter().any(|r| matches!(r, readings::Raised::Extinct { lineage } if lineage == "lineage:0")),
        "a lineage holding only propagules is not extinct");
    let apart = readings::propagules(&after);
    assert_eq!((apart["lineage:0"].count, apart["lineage:0"].matter), (3, 30));
    assert!(readings::heirs(&after, adults[0]).is_empty());
    assert!(readings::starts(&after, "lineage:0").is_empty());
    assert!(readings::start_at(&after, "lineage:0", site).is_err());
    // Awake, it would be offered: the dormant trait alone withholds it.
    let key = after.genesis().rules.dormancy.clone().unwrap().key;
    after.state.population.lift(seed).unwrap().traits.remove(&key);
    assert_eq!(readings::heirs(&after, adults[0]), vec![seed]);
    assert!(readings::starts(&after, "lineage:0").iter().any(|s| s.critter == seed && s.site == site));
    // A control with no seed bank does read as extinct.
    let (before, ..) = build(false);
    let (mut after, adults, ..) = build(false);
    kill(&mut after, &adults);
    assert!(readings::raised(&before, &after).iter().any(|r| matches!(r, readings::Raised::Extinct { lineage } if lineage == "lineage:0")));
}

#[test]
fn dormant_upkeep_is_its_fraction_of_the_active_rate_and_a_drained_seed_dies_to_the_site() {
    let mut g = world(Some(rule(10, 1_000_000)));
    only(&mut g, &["ecology:dormant-"]);
    let active = g.rules.processes["ecology:dormant-upkeep-0"].period.unwrap();
    assert_eq!(active, world(None).rules.processes["ecology:upkeep-0"].period.unwrap() * 10, "a tenth of the active rate");
    let mut sim = Simulation::new(g, Execution::Individuals).unwrap();
    let (total, parent) = (sim.matter(), awake(&sim, "lineage:0")[0]);
    let site = site_of(&sim, parent);
    let seed = lay(&mut sim, 3, 1);
    let at = soil(&sim, site);
    sim.advance(active * 2 + 1).unwrap();
    assert_eq!(matter_of(&sim, seed), 1, "two periods, two matter");
    assert_eq!(soil(&sim, site), at + 2);
    sim.advance(active + 5).unwrap();
    assert!(!sim.state().population.get(seed).unwrap().alive, "a propagule whose matter runs out dies");
    assert_eq!(matter_of(&sim, seed), 0);
    assert_eq!(soil(&sim, site), at + 3, "all its matter is in the site");
    assert_eq!(sim.matter(), total);
}

/// A world where only waking acts, with `n` seeds of `share` laid at one site
/// whose plants number `held`, the rule's crowding count `crowd`.
fn waking(share: u64, n: u64, crowd: Option<u64>, mode: Execution) -> (Simulation, Vec<Id>) {
    let mut r = rule(share, 100);
    r.crowd = crowd.unwrap_or(r.crowd);
    let mut g = world(Some(r));
    only(&mut g, &["ecology:wake-"]);
    g.rules.processes.remove("ecology:sunlight");
    let mut sim = Simulation::new(g, mode).unwrap();
    let ids = (0..n).map(|_| lay(&mut sim, share, 1)).collect();
    (sim, ids)
}

fn woken(sim: &Simulation, ids: &[Id]) -> Vec<Id> {
    let key = sim.genesis().rules.dormancy.clone().unwrap().key;
    ids.iter().copied().filter(|id| !sim.state().population.get(*id).unwrap().traits.contains(&key)).collect()
}

#[test]
fn about_the_wake_share_wakes_a_season_the_same_ones_under_both_executions_and_a_replay() {
    let n = 100;
    let mut runs = Vec::new();
    for mode in [Execution::Individuals, Execution::Grouped, Execution::Individuals] {
        let (mut sim, ids) = waking(10, n, Some(1_000), mode);
        sim.advance(99).unwrap();
        assert!(woken(&sim, &ids).is_empty(), "nothing wakes before the season");
        sim.advance(2).unwrap();
        let up = woken(&sim, &ids);
        // A waking seed's life is counted from waking.
        assert!(up.iter().all(|id| sim.state().population.get(*id).unwrap().born >= 99));
        runs.push(up);
    }
    let woke = runs[0].len() as u64;
    assert!((n / 10..=n * 2 / 5).contains(&woke), "{woke} of {n} woke, about a quarter");
    assert_eq!(runs[0], runs[1], "the same under both executions");
    assert_eq!(runs[0], runs[2], "the same on a replayed run");
}

#[test]
fn a_crowded_site_holds_its_seeds_asleep_and_a_heavier_seed_wakes_where_a_lighter_does_not() {
    // The count of plants at the site where the seeds lie.
    let (probe, _) = waking(10, 1, None, Execution::Individuals);
    let site = site_of(&probe, probe.state().population.groups.keys().copied().last().unwrap());
    let plants: BTreeSet<_> = PLANTS.iter().map(|s| s.to_string()).collect();
    let held = probe.members_at(site, &plants);
    assert!(held >= 1);
    let d = Dormancy::flat();
    assert_eq!((d.crowding(5), d.crowding(10), d.crowding(20)), (6, 12, 24));
    // A count of twice the plants held: a light seed's is exactly what is held, which is not fewer.
    let crowd = Some(2 * held * 10 / 10);
    for (share, wakes) in [(5, false), (10, true), (20, true)] {
        let (mut sim, ids) = waking(share, 100, crowd, Execution::Individuals);
        sim.advance(101).unwrap();
        assert_eq!(!woken(&sim, &ids).is_empty(), wakes, "seed of {share}");
    }
    // Where the site is not habitable no seed wakes, as no plant grows.
    let (mut sim, ids) = waking(20, 100, crowd, Execution::Individuals);
    let site = site_of(&sim, ids[0]);
    sim.state.sites.get_mut(&site).unwrap().conditions.insert("world:habitable".into(), 0);
    sim.advance(101).unwrap();
    assert!(woken(&sim, &ids).is_empty());
}

#[test]
fn a_run_with_the_rule_set_replays_to_the_same_hash_under_both_executions() {
    for mode in [Execution::Individuals, Execution::Grouped] {
        let mut s = Session::new(world(Some(Dormancy::flat())), mode).unwrap();
        s.advance(120).unwrap();
        assert!(!readings::propagules(&s.sim).is_empty());
        s.advance(120).unwrap();
        let json = serde_json::to_vec(&s.save()).unwrap();
        let mut loaded = Session::load_json(&json, mode).unwrap();
        assert_eq!(loaded.sim.state_hash(), s.sim.state_hash());
        s.advance(120).unwrap();
        loaded.advance(120).unwrap();
        assert_eq!(loaded.sim.state_hash(), s.sim.state_hash());
        assert_eq!(loaded.sim.matter(), s.sim.matter());
    }
}

#[test]
fn note_survival_a1_seed_9_unpiloted() {
    let mut rows = Vec::new();
    for rule in [None, Some(Dormancy::flat())] {
        let mut s = Session::new(world(rule.clone()), Execution::Individuals).unwrap();
        for t in (0..=600).step_by(100) {
            if t > 0 { s.advance(100).unwrap(); }
            let per: Vec<String> = (0..4).map(|i| {
                let l = format!("lineage:{i}");
                let seeds = readings::propagules(&s.sim).get(&l).map_or(0, |p| p.count);
                format!("{i}: {}+{seeds}", living(&s.sim, &l))
            }).collect();
            rows.push(format!("dormancy {} tick {t:3}  {}", if rule.is_some() { "on " } else { "off" }, per.join("  ")));
        }
    }
    println!("living+banked per lineage\n{}", rows.join("\n"));
}



// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Recovery of a lost patch by the generated world-time ecology's own
//! processes (sim plan S10.3 step 3), with feeding, births, upkeep and death
//! running. Two native routes back: crowded grazers leaving along a route
//! (case 1111) and producer births scattered by a medium (1089, 1093). Each
//! runs connected, disconnected (routes cut) and absent-source (no living
//! source population), in both execution modes.
//!
//! The walking arm's loss arises in the running web: the patch's three
//! grazers are founded hungry (one unit of body) beside no producers and
//! starve by native upkeep and death within hours; the hunger is an
//! initial condition. The scattering arm's loss is imposed (`Command::Kill`), because
//! nothing in this ecology kills a producer within a short span. The
//! fixture qualifies return, conservation and replay over days, not
//! long-term regional persistence.

use cosmograph::{
    Command, Execution, Founding, Session, anatomy,
    directing::readings::{Absence, Scope, absence, reading, sources},
    ecology::Rates,
    population::Population,
    rules::{AccountKind, minutes_ticks},
    schema::{Entity, Id, Tick},
    simulation::Genesis,
};
use std::collections::{BTreeMap, BTreeSet};

const PATCH: Id = 0;
const SOURCE: Id = 1;
const PRODUCER: &str = "lineage:0";
const GRAZER: &str = "lineage:1";
const HOUR: Tick = 60;
/// The processes whose accepted acts are kept and reported: every birth,
/// the animals' meals, the deaths in question and the walkers' return.
const WATCHED: [&str; 9] = [
    "ecology:birth-0",
    "ecology:birth-1",
    "ecology:birth-2",
    "ecology:feed-1",
    "ecology:feed-2",
    "ecology:death-0",
    "ecology:death-1",
    "move:1-0/lineage:1",
    "sim:birth",
];
const DAY: Tick = 1440;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Route {
    Walk,
    Scatter,
}

impl Route {
    fn lineage(self) -> &'static str {
        match self {
            Route::Walk => GRAZER,
            Route::Scatter => PRODUCER,
        }
    }
}

/// The world-time levels founding (producer, grazer, decomposer) on two
/// routed sites, its population replaced by the arm's.
fn world(route: Route, connected: bool, source: bool) -> (Genesis, Vec<Id>) {
    let mut g = Founding {
        seed: 9,
        sites: 2,
        population: 60,
        cohort_size: 1,
        lineages: 3,
        ecology: true,
        ecology_rates: Rates::WorldTime,
        ..Founding::default()
    }
    .generate()
    .unwrap();
    assert!(g.rules.processes.contains_key("move:1-0/lineage:1"));
    assert_eq!(minutes_ticks(1440, g.rules.tick_microseconds()), DAY, "a minute clock");
    let proto = |lineage: &str| -> Entity {
        g.population
            .groups
            .values()
            .find(|group| group.entity.lineage == lineage)
            .unwrap()
            .entity
            .as_ref()
            .clone()
    };
    let (producer, grazer, decomposer) = (proto(PRODUCER), proto(GRAZER), proto("lineage:2"));
    let breed = g.rules.processes["ecology:birth-0"].period.unwrap();
    let started = g.started;
    if !connected {
        for site in g.sites.values_mut() {
            site.routes.clear();
        }
    }
    let mut pop = Population::default();
    // The world's inert ground bodies stay as founded.
    for c in g.population.groups.values().filter(|c| c.entity.lineage == "world:ground") {
        pop.insert(c.entity.clone(), c.count).unwrap();
    }
    let mut put = |e: &Entity, site: Id, body: Option<u64>, n: usize| -> Vec<Id> {
        (0..n)
            .map(|_| {
                let mut e = e.clone();
                e.place = site;
                if let Some(body) = body {
                    let key = format!("matter:{}-0", &e.lineage["lineage:".len()..]);
                    e.accounts.insert(key, body);
                }
                pop.insert(e, 1).unwrap()
            })
            .collect()
    };
    put(&decomposer, PATCH, None, 3);
    put(&decomposer, SOURCE, None, 3);
    let locals = match route {
        Route::Walk => {
            // Hungry grazers beside no food, below the crowd that leaves.
            put(&producer, SOURCE, None, 16);
            if source {
                put(&grazer, SOURCE, None, 24);
            }
            put(&grazer, PATCH, Some(1), 3)
        }
        Route::Scatter => {
            if source {
                // Mature, stocked for a birth, and due an hour in.
                let mut ready = producer.clone();
                ready.born = started + HOUR - 2 * breed;
                put(&ready, SOURCE, Some(70), 48);
            }
            put(&producer, PATCH, None, 6)
        }
    };
    g.population = pop;
    (g, locals)
}

fn matter(s: &Session) -> u128 {
    let rules = &s.sim.genesis().rules;
    let mass = |ledger: &BTreeMap<String, u64>| {
        ledger
            .iter()
            .filter(|(key, _)| matches!(rules.accounts.get(*key), Some(AccountKind::Matter { .. })))
            .map(|(_, n)| u128::from(*n))
            .sum::<u128>()
    };
    let state = s.sim.state();
    let bodies: u128 = state
        .population
        .groups
        .values()
        .map(|g| mass(&anatomy::books(&g.entity)) * u128::from(g.count))
        .sum();
    bodies + state.sites.values().map(|site| mass(&site.accounts)).sum::<u128>()
}

fn members(s: &Session, lineage: &str, site: Id) -> BTreeSet<Id> {
    s.sim
        .state()
        .population
        .groups
        .iter()
        .filter(|(_, g)| g.entity.alive && g.entity.lineage == lineage && g.entity.resident_site() == Some(site))
        .flat_map(|(&first, g)| first..first + g.count)
        .collect()
}

/// Living members at the source or travelling from it.
fn donors(s: &Session, lineage: &str) -> BTreeSet<Id> {
    s.sim
        .state()
        .population
        .groups
        .iter()
        .filter(|(_, g)| {
            let e = &g.entity;
            e.alive && e.lineage == lineage && (e.resident_site() == Some(SOURCE) || e.journey.is_some_and(|j| j.from == SOURCE))
        })
        .flat_map(|(&first, g)| first..first + g.count)
        .collect()
}

/// What an arm saw: when the patch lost the lineage and when, and by whom,
/// it came back.
#[derive(Debug, Default)]
struct Outcome {
    lost: Option<Tick>,
    returned: Option<Tick>,
    returners: BTreeSet<Id>,
    /// Source members alive and resident when the loss was observed.
    donors: BTreeSet<Id>,
    acts: BTreeMap<String, u64>,
    /// Born members added by watched birth acts, and ids issued.
    births: u64,
    issued: u64,
    /// The first identity issued after founding.
    first_id: Id,
}

fn run(route: Route, mode: Execution, connected: bool, source: bool, horizon: Tick) -> Outcome {
    let lineage = route.lineage();
    let (g, locals) = world(route, connected, source);
    let mut s = Session::new(g, mode).unwrap();
    for p in WATCHED {
        s.sim.watch(p);
    }
    let initial = matter(&s);
    let first_id = s.sim.state().population.next_id;
    let start = s.sim.state().tick;
    assert_eq!(members(&s, lineage, PATCH).len(), locals.len());
    if route == Route::Scatter {
        for &id in &locals {
            s.command(Command::Kill { entity: id }).unwrap();
        }
    }
    let footprint = BTreeSet::from([PATCH, SOURCE]);
    let mut out = Outcome { first_id, ..Outcome::default() };
    let mut saved = None;
    loop {
        let tick = s.sim.state().tick;
        for act in s.sim.take_watched() {
            *out.acts.entry(act.process.clone()).or_default() += act.count;
            if act.process.ends_with("birth") || act.process.starts_with("ecology:birth-") {
                out.births += act.count;
            }
        }
        assert_eq!(matter(&s), initial, "{route:?} {mode:?}: matter conserved at {tick}");
        let here = members(&s, lineage, PATCH);
        match out.lost {
            None if here.is_empty() => {
                out.lost = Some(tick);
                out.donors = donors(&s, lineage);
                let patch = reading(&s.sim, &Scope::Patch(PATCH)).unwrap();
                assert!(!patch.holds(lineage) && patch.sites == BTreeSet::from([PATCH]));
                let found = sources(&s.sim, lineage, PATCH).unwrap();
                let available: Vec<_> = found.iter().filter(|f| f.available()).collect();
                let expected = if source { Absence::Patch } else { Absence::Global };
                assert_eq!(absence(&s.sim, lineage, PATCH, &footprint).unwrap(), expected);
                assert_eq!(found.len(), usize::from(source), "{found:?}");
                assert_eq!(available.len(), usize::from(source && connected), "{found:?}");
                if source && connected {
                    assert_eq!(found[0].site, SOURCE);
                    assert_eq!(found[0].members, members(&s, lineage, SOURCE).len() as u64);
                }
                saved = Some(s.save());
            }
            Some(_) if !here.is_empty() => {
                out.returned = Some(tick);
                out.returners = here;
            }
            _ => {}
        }
        if tick >= start + horizon || out.returned.is_some() {
            break;
        }
        s.advance(HOUR).unwrap();
    }
    let lost = out.lost.unwrap_or_else(|| panic!("{route:?} {mode:?}: the patch never lost {lineage}"));
    out.issued = s.sim.state().population.next_id - first_id;
    assert_eq!(out.issued, out.births, "every new member is a watched native birth");
    if let Some(saved) = saved {
        let mut replayed = Session::load(saved, mode).unwrap();
        replayed.advance(s.sim.state().tick - lost).unwrap();
        assert_eq!(replayed.sim.state_hash(), s.sim.state_hash(), "replay from the loss");
        for scope in [Scope::Patch(PATCH), Scope::Footprint(footprint.clone()), Scope::World] {
            assert_eq!(reading(&replayed.sim, &scope).unwrap(), reading(&s.sim, &scope).unwrap());
        }
    }
    eprintln!(
        "{route:?} {mode:?} connected={connected} source={source}: lost at +{}m, returned {:?}, \
         returners {:?}, donors {}, acts {:?}, births {}, matter {initial}",
        lost - start,
        out.returned.map(|t| t - start),
        out.returners,
        out.donors.len(),
        out.acts,
        out.births,
    );
    out
}

fn check(route: Route, mode: Execution, horizon: Tick) {
    for (connected, source) in [(true, true), (false, true), (true, false)] {
        let out = run(route, mode, connected, source, horizon);
        if !(connected && source) {
            assert_eq!(out.returned, None, "{route:?} connected={connected} source={source}: no return");
            continue;
        }
        assert!(out.returned.is_some(), "{route:?} {mode:?}: connected return within the horizon");
        match route {
            // The very members that stood at the source, arrived by route.
            Route::Walk => {
                assert!(out.returners.is_subset(&out.donors), "{out:?}");
                assert!(out.acts["move:1-0/lineage:1"] >= out.returners.len() as u64);
                assert!(out.acts["ecology:death-1"] >= 3, "the patch's grazers starved");
                assert!(out.acts["ecology:feed-1"] > 0, "source grazers fed on producers");
            }
            // New members, born of the source by scattered births.
            Route::Scatter => {
                assert!(out.returners.iter().all(|&id| id >= out.first_id), "born after founding: {out:?}");
                assert!(out.returners.is_disjoint(&out.donors), "{out:?}");
                assert!(out.acts["ecology:birth-0"] >= out.returners.len() as u64);
            }
        }
    }
}

#[test]
fn walkers_return_only_from_a_connected_living_source_individuals() {
    check(Route::Walk, Execution::Individuals, 6 * DAY);
}

#[test]
fn walkers_return_only_from_a_connected_living_source_grouped() {
    check(Route::Walk, Execution::Grouped, 6 * DAY);
}

#[test]
fn scattered_births_return_only_from_a_connected_living_source_individuals() {
    check(Route::Scatter, Execution::Individuals, DAY);
}

#[test]
fn scattered_births_return_only_from_a_connected_living_source_grouped() {
    check(Route::Scatter, Execution::Grouped, DAY);
}

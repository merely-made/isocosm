// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Checkpoint 9's controls (rulings 560 to 590), each run by the core and
//! the crowd alike: a route cut short carries nothing beyond the cut, while
//! the intact body carries to every part; a latent cell earns nothing until
//! a riff routes it, then earns; the limbs' share of a bite needs muscular
//! routes to the contracting parts; and with riffing off the domain's
//! systems never change, while with it on they do.

use isocosm::{
    Execution, Simulation, anatomy,
    probe::{BodyFounding, Crowd, ProbeWorld, Variant, readings, run_exact},
    rules::{Fill, Rules, Template},
    schema::*,
};
use std::collections::BTreeMap;

const GRAZER: &str = "lineage:1";
const GRAZE: &[&str] = &["body:graze-1"];
const FIX: &[&str] = &["body:fix-1"];

fn part(t: &Template, parent: Option<Id>, situs: [u8; 3]) -> Part {
    let cells: BTreeMap<Key, u32> = t
        .cells
        .iter()
        .filter(|(_, n)| **n > 0)
        .map(|(k, n)| (k.clone(), *n))
        .collect();
    Part {
        parent,
        half_extent: t.half_extent,
        situs: Some(situs),
        functions: cells.keys().cloned().collect(),
        cells,
        ..Default::default()
    }
}

/// One site of a frond holding plenty and one fed grazer: a lump, a limb
/// on it, a second limb on the first, and an eye, its tissue half its
/// parts' adult mass and its stores full, carrying the systems it
/// realizes, then changed by `change`; only `keep` runs, and nothing
/// varies.
fn world(seed: u64, keep: &[&str], change: impl FnOnce(&mut Entity, &Rules)) -> ProbeWorld {
    let mut w = BodyFounding {
        seed,
        ticks: 1,
        sites: [1, 1],
        variance: [0, 0],
        absence: [0, 0],
        riff: [0, 0],
        vary: [0, 0],
        semelparous: 0,
        ..Default::default()
    }
    .generate()
    .unwrap();
    let g = &mut w.genesis;
    g.rules.processes.retain(|id, _| keep.contains(&id.as_str()));
    let rules = g.rules.clone();
    let find = |lineage: &str| {
        let mut found = g.population.groups.values().map(|c| &c.entity);
        found.find(|e| e.lineage == lineage).unwrap().clone()
    };
    let (ground, mut frond, mut e) = (find("world:ground"), find("lineage:0"), find(GRAZER));
    for p in frond.parts.values_mut() {
        p.matter.insert("tissue:0".into(), 10_000);
    }
    let k = |name: &str| &rules.kinds[name];
    e.parts = BTreeMap::from([
        (0, part(k("kind:lump"), None, [0, 0, 0])),
        (1, part(k("kind:limb"), Some(0), [0, 0, 1])),
        (2, part(k("kind:limb"), Some(1), [0, 0, 2])),
        (3, part(k("kind:eye"), Some(0), [1, 0, 0])),
    ]);
    let b = rules.body();
    for p in e.parts.values_mut() {
        p.matter.insert("tissue:1".into(), anatomy::ceiling(p, b) / 2);
    }
    let full = anatomy::room(&e, &rules, "reserve:1");
    anatomy::give(&mut e, &rules, "reserve:1", full).unwrap().unwrap();
    e.systems = isocosm::systems::founded(&e, &rules);
    change(&mut e, &rules);
    let mut population = isocosm::population::Population::default();
    for (e, n) in [(ground, 1), (frond, 1), (e, 1)] {
        population.insert(e, n).unwrap();
    }
    g.population = population;
    w
}

/// The world's bodies after its run, by the core and by the crowd, zero
/// accounts dropped; the two must agree.
fn run(w: &ProbeWorld) -> Vec<Entity> {
    let states = |members: Vec<(&Entity, u64)>| {
        let mut v: BTreeMap<Entity, u64> = BTreeMap::new();
        for (e, n) in members {
            let mut e = e.clone();
            e.accounts.retain(|_, v| *v != 0);
            *v.entry(e).or_default() += n;
        }
        v.into_iter().collect::<Vec<_>>()
    };
    let exact = run_exact(w, 3, true).unwrap();
    let crowd = Crowd::new(w, 3, Variant::Histogram).unwrap().run().unwrap();
    let core = states(readings::exact_members(&exact));
    assert_eq!(core, states(readings::crowd_members(&crowd)), "crowd and core");
    core.into_iter().map(|(e, _)| e).collect()
}

/// The world's bodies as founded.
fn founded(w: &ProbeWorld) -> Vec<Entity> {
    let groups = w.genesis.population.groups.values();
    groups.map(|c| c.entity.clone()).collect()
}

fn of<'a>(bodies: &'a [Entity], lineage: &str) -> &'a Entity {
    bodies.iter().find(|e| e.lineage == lineage).unwrap()
}

fn tissue(e: &Entity, part: Id) -> u64 {
    e.parts[&part].matter.get("tissue:1").copied().unwrap_or(0)
}

fn held(e: &Entity, key: &str) -> u64 {
    anatomy::books(e).get(key).copied().unwrap_or(0)
}

/// The site's soil after the core's run.
fn soil(w: &ProbeWorld, ran: bool) -> u64 {
    let mut genesis = w.genesis.clone();
    genesis.dynamics = Some(3);
    let mut sim = Simulation::new(genesis, Execution::Individuals).unwrap();
    if ran {
        sim.advance(w.ticks).unwrap();
    }
    let site = sim.state().sites.values().next().unwrap();
    site.accounts.get("world:soil").copied().unwrap_or(0)
}

#[test]
fn a_cut_route_carries_nothing_beyond_the_cut() {
    for seed in 0..6 {
        let intact = world(seed, GRAZE, |_, _| {});
        let cut = world(seed, GRAZE, |e, _| {
            e.parts.get_mut(&1).unwrap().severed = true;
        });
        let start = of(&founded(&cut), GRAZER).clone();
        // Control: intact, the farther limb is fed.
        let fed = run(&intact);
        assert!(tissue(of(&fed, GRAZER), 2) > tissue(&start, 2), "seed {seed}");
        // Cut, it is alive and has room, and takes nothing; the lump still
        // takes its share.
        let severed = run(&cut);
        assert_eq!(tissue(of(&severed, GRAZER), 2), tissue(&start, 2), "seed {seed}");
        assert!(tissue(of(&severed, GRAZER), 0) > tissue(&start, 0), "seed {seed}");
    }
}

#[test]
fn a_latent_cell_earns_only_once_a_riff_routes_it() {
    let latent = |e: &mut Entity, _: &Rules| {
        let lump = e.parts.get_mut(&0).unwrap();
        *lump.cells.get_mut("function:intake").unwrap() -= 1;
        lump.cells.insert("function:fix".into(), 1);
        lump.functions.insert("function:fix".into());
    };
    for seed in 0..6 {
        let w = world(seed, FIX, latent);
        let start = of(&founded(&w), GRAZER).clone();
        assert_eq!(held(of(&run(&w), GRAZER), "tissue:1"), held(&start, "tissue:1"));
        assert_eq!(soil(&w, true), soil(&w, false), "latent: seed {seed}");
        let riffed = world(seed, FIX, |e, r| {
            latent(e, r);
            let gut = e.systems.get_mut("system:digestive").unwrap();
            gut.sources.insert(Fill::Function("function:fix".into()));
        });
        let after = run(&riffed);
        assert!(held(of(&after, GRAZER), "tissue:1") > held(&start, "tissue:1"));
        assert!(soil(&riffed, true) < soil(&riffed, false), "riffed: seed {seed}");
    }
}

#[test]
fn the_limbs_share_of_a_bite_needs_muscular_routes() {
    let mut fewer = 0;
    for seed in 0..6 {
        let whole = world(seed, GRAZE, |_, _| {});
        let lamed = world(seed, GRAZE, |e, _| {
            let muscle = e.systems.get_mut("system:muscular").unwrap();
            muscle.effects = [Fill::Function("function:sense".into())].into();
        });
        let eaten = |w: &ProbeWorld| 10_000 - held(of(&run(w), "lineage:0"), "tissue:0");
        let (a, b) = (eaten(&whole), eaten(&lamed));
        assert!(0 < b && b <= a, "seed {seed}: {b} against {a}");
        fewer += u64::from(b < a);
    }
    assert!(fewer > 0, "the limbs added nothing to any bite");
}

#[test]
fn with_riffing_off_no_system_changes_and_with_it_on_systems_do() {
    let riffed = |riff: [u32; 2]| -> u64 {
        let mut n = 0;
        for seed in 1..=6 {
            let w = BodyFounding {
                seed,
                riff,
                ..Default::default()
            }
            .generate()
            .unwrap();
            let defaults = &w.genesis.rules.systems;
            let exact = run_exact(&w, 3, true).unwrap();
            for (e, count) in readings::exact_members(&exact) {
                let changed = e.systems.iter().any(|(k, s)| defaults.get(k) != Some(s));
                n += u64::from(changed) * count;
            }
        }
        n
    };
    assert_eq!(riffed([0, 0]), 0);
    assert!(riffed([1, 1]) > 0);
}

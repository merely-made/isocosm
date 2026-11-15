// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! S10.4's paired producer comparison on the native protected-root path.
//! The founding changes allocations and shortens the producer season. All
//! subsequent matter moves use native processes. Run the ignored test in release.

use cosmograph::{
    Command, Execution, Founding, Session, anatomy,
    bodied::Bodies,
    development::{develop, soma},
    ecology::{Rates, Web},
    rules::{AccountKind, Causation, Dormancy, Effect, Query},
    schema::{Id, Key, Ledger},
    simulation::Genesis,
};
use serde::Serialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

const DAY: u64 = 1440;
const SEASON: u64 = 6 * DAY;
const MEADOW: Id = 0;
const GROWTH: &str = "experiment:growth-open";
const END_LEAN: &str = "experiment:end-lean";
const GRAZER: &str = "lineage:1";
const FEED: &str = "ecology:feed-1";

#[derive(Clone, Copy)]
struct Shares {
    exposed: u64,
    root: u64,
    seed: u64,
}

const EXPOSURE: Shares = Shares {
    exposed: 48,
    root: 12,
    seed: 6,
};
const RESERVES: Shares = Shares {
    exposed: 32,
    root: 28,
    seed: 14,
};

fn set_producer(g: &mut Genesis, line: u32, shares: Shares) {
    let tissue = format!("matter:{line}-0");
    let root = g.rules.root_stores.get_mut(&tissue).unwrap();
    (root.exposed, root.stored, root.seed) = (shares.exposed, shares.root, shares.seed);

    let birth = format!("ecology:birth-{line}");
    g.rules
        .dormancy
        .as_mut()
        .unwrap()
        .seeds
        .insert(birth.clone(), shares.seed);
    let process = g.rules.processes.get_mut(&birth).unwrap();
    process.period = Some(SEASON);
    for q in &mut process.requires {
        if let Query::Account { key, at_least, .. } = q
            && key == &tissue
        {
            *at_least = shares.exposed + shares.root + shares.seed;
        }
    }
    for effect in process
        .effects
        .iter_mut()
        .chain(process.risk.iter_mut().flat_map(|risk| &mut risk.effects))
    {
        if let Effect::Birth { provision, .. } = effect {
            *provision.get_mut(&tissue).unwrap() = shares.seed;
        }
    }
    let growth = g
        .rules
        .processes
        .get_mut(&format!("ecology:produce-{line}"))
        .unwrap();
    let light = growth
        .requires
        .iter()
        .find_map(|q| match q {
            Query::Sensed { reading, at_least } if reading == "sense:light-share" => {
                Some(*at_least)
            },
            _ => None,
        })
        .unwrap();
    for q in &mut growth.requires {
        if let Query::Below { key, amount, .. } = q
            && key == &tissue
        {
            *amount = shares.exposed + shares.root + shares.seed;
        }
    }
    let gate = Query::Condition {
        key: GROWTH.into(),
        at_least: 1,
    };
    growth.requires.push(gate.clone());
    let wake = g
        .rules
        .processes
        .get_mut(&format!("ecology:wake-{line}"))
        .unwrap();
    for q in &mut wake.requires {
        if let Query::Sensed { reading, at_least } = q
            && reading == "sense:light-share"
        {
            *at_least = Dormancy::waking(light, shares.seed);
        }
    }
    wake.requires.push(gate);
}

fn founding(seed: u64, shares: [Shares; 2]) -> (Genesis, Id) {
    let mut g = Founding {
        seed,
        sites: 2,
        population: 64,
        cohort_size: 1,
        lineages: 4,
        ecology: true,
        ecology_rates: Rates::WorldTime,
        web: Web::Levels,
        experimental_food_contract: true,
        bodies: Some(Bodies::default()),
        dormancy: Some(Dormancy {
            season: SEASON,
            ..Dormancy::default()
        }),
        ..Founding::default()
    }
    .generate()
    .unwrap();
    assert!(g.sites[&MEADOW].routes.iter().any(|r| r.to == 1));
    assert!(g.sites[&1].routes.iter().any(|r| r.to == MEADOW));
    g.rules.conditions.insert(GROWTH.into());
    for (&id, site) in &mut g.sites {
        site.conditions
            .insert(GROWTH.into(), i64::from(id != MEADOW));
    }
    let mut opening = g.rules.processes["ecology:sunlight"].clone();
    opening.id = END_LEAN.into();
    opening.period = None;
    opening.causation = Causation::Choice;
    opening.effects = vec![Effect::Condition {
        key: GROWTH.into(),
        delta: 1,
    }];
    opening.requires = vec![Query::Condition {
        key: GROWTH.into(),
        at_least: 0,
    }];
    g.rules.processes.insert(opening.id.clone(), opening);
    let ground = g
        .population
        .groups
        .iter()
        .find(|(_, group)| group.entity.lineage == "world:ground" && group.entity.place == MEADOW)
        .map(|(&id, _)| id)
        .unwrap();

    // Give both lineages the same underlying plant recipe. The arms only
    // change their exposed, stored and seed allocations. The same founder
    // IDs draw the same somas in every arm.
    let mut development = g.lineages["lineage:0"].development.clone().unwrap();
    development.recipe.tagmata.last_mut().unwrap().variance = Some(1);
    for (line, allocation) in [0, 3].into_iter().zip(shares) {
        set_producer(&mut g, line, allocation);
        let lineage = format!("lineage:{line}");
        g.lineages.get_mut(&lineage).unwrap().development = Some(development.clone());
        let tissue = format!("matter:{line}-0");
        let mut draws = BTreeSet::new();
        for (&id, group) in &mut g.population.groups {
            if group.entity.lineage != lineage {
                continue;
            }
            let e = group.entity_mut();
            let held = anatomy::books(e)[&tissue];
            let draw = soma(
                &g.rules,
                &development.recipe,
                cosmograph::draw(seed, "founder-soma", &[id]),
            );
            draws.insert(draw.segments.clone());
            e.embody(develop(&g.rules, &development, &draw).unwrap());
            e.soma = draw.segments;
            e.accounts.remove(&tissue);
            assert_eq!(held, allocation.exposed + allocation.root);
            let root = e.body.as_ref().unwrap().root;
            Arc::make_mut(e.parts.get_mut(&root).unwrap())
                .matter
                .insert(tissue.clone(), allocation.root);
            let given = anatomy::give(e, &g.rules, &tissue, allocation.exposed)
                .expect("a rooted body")
                .expect("room for the exposed provision");
            assert!(given.iter().all(|(part, _)| *part != root));
            assert_eq!(anatomy::books(e)[&tissue], held);
        }
        assert!(draws.len() > 1, "founders of {lineage} should vary");
    }
    g.validate().unwrap();
    (g, ground)
}

#[derive(Default, Serialize)]
struct Plants {
    active: u64,
    dormant: u64,
    exposed: u64,
    root: u64,
    dormant_matter: u64,
}

#[derive(Serialize)]
struct Reading {
    day: u64,
    site: Id,
    edible: Option<u64>,
    meal_per_grazer: u64,
    grazers: u64,
    plants: BTreeMap<Key, Plants>,
    grazed: BTreeMap<Key, u64>,
    recruited: BTreeMap<Key, u64>,
}

fn matter(session: &Session) -> u128 {
    let rules = &session.sim.genesis().rules;
    let mass = |book: &Ledger| {
        book.iter()
            .filter(|(k, _)| matches!(rules.accounts.get(*k), Some(AccountKind::Matter { .. })))
            .map(|(_, n)| u128::from(*n))
            .sum::<u128>()
    };
    let state = session.sim.state();
    state
        .population
        .groups
        .values()
        .map(|g| mass(&anatomy::books(&g.entity)) * u128::from(g.count))
        .sum::<u128>()
        + state
            .sites
            .values()
            .map(|s| mass(&s.accounts))
            .sum::<u128>()
}

type Counts = BTreeMap<(Id, Key), u64>;

fn read(session: &Session, day: u64, site: Id, grazed: &Counts, recruited: &Counts) -> Reading {
    let sim = &session.sim;
    let rules = &sim.genesis().rules;
    let mut plants = BTreeMap::<Key, Plants>::new();
    let mut grazers = 0;
    let mut actor = None;
    for (&id, g) in &sim.state().population.groups {
        let e = &g.entity;
        if !e.alive || e.resident_site() != Some(site) {
            continue;
        }
        if e.lineage == GRAZER {
            grazers += g.count;
            actor = Some(id);
        }
        if e.kingdom != "kingdom:flora" {
            continue;
        }
        let p = plants.entry(e.lineage.clone()).or_default();
        let tissue = format!("matter:{}-0", &e.lineage["lineage:".len()..]);
        if rules.dormant(e) {
            p.dormant += g.count;
            p.dormant_matter += anatomy::books(e).get(&tissue).copied().unwrap_or(0) * g.count;
        } else {
            p.active += g.count;
            let root = e.body.as_ref().unwrap().root;
            for (&id, part) in &e.parts {
                let held = part.matter.get(&tissue).copied().unwrap_or(0) * g.count;
                if id == root {
                    p.root += held;
                } else if anatomy::root_store::exposed(e, rules, id) {
                    p.exposed += held;
                }
            }
        }
    }
    let at_site = |counts: &Counts| {
        counts
            .iter()
            .filter(|((at, _), _)| *at == site)
            .map(|((_, key), &n)| (key.clone(), n))
            .collect()
    };
    Reading {
        day,
        site,
        plants,
        grazers,
        edible: actor.map(|id| sim.edible_matter(id, FEED).unwrap()),
        meal_per_grazer: sim.feed_demand(FEED, &GRAZER.into()).unwrap(),
        grazed: at_site(grazed),
        recruited: at_site(recruited),
    }
}

#[derive(Serialize)]
struct Run {
    seed: u64,
    arm: &'static str,
    lean_days: u64,
    genesis_digest: String,
    initial_matter: u128,
    final_hash: u64,
    readings: Vec<Reading>,
}

fn run(g: Genesis, ground: Id, arm: &'static str, lean_days: u64) -> Run {
    let seed = g.seed;
    let genesis_digest = cosmograph::digest(&g);
    let mut s = Session::new(g, Execution::Individuals).unwrap();
    let initial_matter = matter(&s);
    for process in ["ecology:wake-0", "ecology:wake-3"] {
        s.sim.watch(process);
    }
    let mut grazed = Counts::new();
    let mut recruited = Counts::new();
    let mut readings = Vec::new();
    for elapsed in 0..=2 * SEASON {
        if elapsed % DAY == 0 {
            assert_eq!(matter(&s), initial_matter, "{seed} {arm} at tick {elapsed}");
            for site in [MEADOW, 1] {
                readings.push(read(&s, elapsed / DAY, site, &grazed, &recruited));
            }
        }
        if elapsed == lean_days * DAY {
            s.command(Command::Act {
                actor: ground,
                process: END_LEAN.into(),
                target: None,
                cause: None,
            })
            .unwrap();
            assert_eq!(s.sim.state().sites[&MEADOW].conditions[GROWTH], 1);
        }
        if elapsed == 2 * SEASON {
            break;
        }
        let tick = s.advance_tick_with_flows().unwrap();
        for flow in tick.flows.iter().filter(|flow| flow.by_process(FEED)) {
            if let Some(e) = flow
                .from
                .0
                .body()
                .and_then(|id| s.sim.state().population.get(id))
                && e.kingdom == "kingdom:flora"
            {
                *grazed.entry((e.place, e.lineage.clone())).or_default() +=
                    flow.amount * flow.count;
            }
        }
        for act in s.sim.take_watched() {
            let e = s.sim.state().population.get(act.actor).unwrap();
            if e.alive && !s.sim.genesis().rules.dormant(e) {
                *recruited.entry((e.place, e.lineage.clone())).or_default() += act.count;
            }
        }
    }
    let final_hash = s.sim.state_hash();
    let saved = serde_json::from_slice(&serde_json::to_vec(&s.save()).unwrap()).unwrap();
    let replay = Session::load(saved, Execution::Individuals).unwrap();
    assert_eq!(replay.sim.state_hash(), final_hash);
    Run {
        seed,
        arm,
        lean_days,
        genesis_digest,
        initial_matter,
        final_hash,
        readings,
    }
}

#[test]
#[ignore = "S10.4 paired ecology experiment"]
fn two_producer_experiment() {
    let seeds = std::env::var("COSMOGRAPH_PRODUCER_SEEDS").unwrap_or_else(|_| "3,9,17".into());
    let mut results = Vec::new();
    for seed in seeds.split(',').map(|seed| seed.parse::<u64>().unwrap()) {
        let mut initial = None;
        for (arm, shares) in [
            ("exposure/exposure", [EXPOSURE; 2]),
            ("reserves/reserves", [RESERVES; 2]),
            ("exposure/reserves", [EXPOSURE, RESERVES]),
            ("reserves/exposure", [RESERVES, EXPOSURE]),
        ] {
            let (g, ground) = founding(seed, shares);
            assert_eq!(
                cosmograph::digest(&g),
                cosmograph::digest(&founding(seed, shares).0)
            );
            for lean in [2, 4] {
                let result = run(g.clone(), ground, arm, lean);
                assert_eq!(
                    result.initial_matter,
                    *initial.get_or_insert(result.initial_matter)
                );
                eprintln!(
                    "seed {seed} {arm}, {lean} lean days: replay {}",
                    result.final_hash
                );
                results.push(result);
            }
        }
    }
    let report = std::env::var_os("COSMOGRAPH_PRODUCER_REPORT").expect("report path");
    use std::io::Write as _;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(report)
        .unwrap();
    serde_json::to_writer_pretty(&mut file, &results).unwrap();
    file.write_all(b"\n").unwrap();
}

// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Death drawn once (the execution plan's E7b, case 1181). In distribution,
//! a death tick drawn once at maturity matches the per-tick draw it
//! replaces, by the probe's paired difference and equivalence tests, while
//! a draw at doubled odds fails them. In a world, each hazard death falls on
//! the tick its draw names, matter is conserved, both execution modes agree
//! and a save replays exactly.

use cosmograph::{
    Execution, Founding, Session,
    bodied::Bodies,
    ecology::{Rates, Web},
    probe::check::{Settings, compare},
    rules::Aging,
    schema::{Id, Tick},
};

/// The world-time roles' hazards (`ecology::world_time::role_time`):
/// maturity in days, the first year's odds in millionths, doubling years.
const ROLES: [(&str, Tick, u32, u32); 5] = [
    ("producer", 30, 100_000, 3),
    ("grazer", 120, 200_000, 2),
    ("omnivore", 180, 150_000, 3),
    ("predator", 365, 120_000, 3),
    ("decomposer", 21, 500_000, 1),
];

/// A year of a tick a day, so the per-tick arm stays affordable; the draw
/// is the same arithmetic at any year.
const YEAR: Tick = 365;

fn aging(&(_, days, per_million, doubling): &(&str, Tick, u32, u32)) -> Aging {
    Aging { maturity: days, year: YEAR, per_million_per_year: per_million, doubling_years: doubling }
}

/// Age at death by a draw each tick at `factor` times the odds.
fn per_tick(a: &Aging, k: u64, arm: &str, factor: u64) -> u64 {
    let mut age = a.maturity;
    loop {
        let chance = a.per_tick(age).unwrap_or(0).saturating_mul(factor);
        if cosmograph::draw(7, arm, &[k, age]) % 1_000_000_000_000 < chance || age > a.maturity + 200 * YEAR {
            return age;
        }
        age += 1;
    }
}

/// Age at death drawn once at maturity.
fn once(a: &Aging, k: u64) -> u64 {
    a.maturity + a.falls_after(a.maturity, cosmograph::draw(7, "once", &[k])).expect("the hazard falls")
}

fn arms(n: u64, b: impl Fn(&Aging, u64) -> u64) -> (Vec<Vec<u64>>, Vec<Vec<u64>>) {
    let roles: Vec<Aging> = ROLES.iter().map(aging).collect();
    let a = (0..n).map(|k| roles.iter().map(|r| per_tick(r, k, "tick", 1)).collect()).collect();
    let b = (0..n).map(|k| roles.iter().map(|r| b(r, k)).collect()).collect();
    (a, b)
}

/// The distribution check, run in release: `DEATH_DRAWS` replicates per
/// role (20,000 by default), each reading's distance bounded at 50 per
/// mille, both test families Holm-corrected at 5%.
#[test]
#[ignore = "tens of millions of per-tick draws; run in release"]
fn a_death_drawn_once_matches_the_per_tick_draw_in_distribution() {
    let n = std::env::var("DEATH_DRAWS").ok().and_then(|v| v.parse().ok()).unwrap_or(20_000);
    let readings: Vec<(String, u32)> = ROLES.iter().map(|r| (format!("death age, {}", r.0), 50)).collect();
    let settings = Settings { alpha: 0.05, permutations: 1999, seed: 1181 };
    let report = |c: &cosmograph::probe::check::Comparison| {
        for r in &c.readings {
            eprintln!(
                "{}: {}: distance {:.4}, means {:.1} / {:.1}, difference p {:.4}, equivalence p {:.2e}",
                c.name, r.key, r.distance, r.mean_a, r.mean_b, r.p_difference_holm, r.p_equivalence_holm
            );
        }
        eprintln!("{}: pass {}", c.name, c.pass);
    };
    let (a, b) = arms(n, once);
    let drawn = compare("per tick against drawn once", &readings, &a, &b, settings);
    report(&drawn);
    let (a, b) = arms(n, |r, k| per_tick(r, k, "again", 1));
    let control = compare("per tick against per tick (positive control)", &readings, &a, &b, settings);
    report(&control);
    let (a, b) = arms(n, |r, k| per_tick(r, k, "broken", 2));
    let broken = compare("per tick against doubled odds (broken draw)", &readings, &a, &b, settings);
    report(&broken);
    assert!(drawn.pass, "the single draw must match the per-tick draw");
    assert!(control.pass, "the positive control must pass");
    assert!(!broken.pass && broken.different, "doubled odds must be told apart");
}

/// The drawn tick is the per-tick survival's inverse: never before
/// maturity, monotone in the draw, and certain when the odds are.
#[test]
fn the_drawn_tick_follows_the_survival_curve() {
    for role in &ROLES {
        let a = aging(role);
        let ticks: Vec<u64> = [0, 1 << 40, 1 << 62, 1 << 63, u64::MAX - 1].iter()
            .map(|u| a.falls_after(a.maturity, *u).expect("falls")).collect();
        assert!(ticks.windows(2).all(|w| w[0] >= w[1]), "{}: {ticks:?}", role.0);
        assert_eq!(a.falls_after(0, u64::MAX), a.falls_after(a.maturity, u64::MAX).map(|t| t + a.maturity));
    }
    let never = Aging { maturity: 10, year: YEAR, per_million_per_year: 0, doubling_years: 1 };
    assert_eq!(never.falls_after(10, 0), None);
}

const DAY: Tick = 1440;

/// A small world-time web whose hazards run a year to the day, so deaths
/// come within days.
fn world(mode: Execution) -> Session {
    let seed = 9;
    let grid = cosmograph::map::Grid { width: 2, height: 2, ..cosmograph::map::Grid::drawn(seed) };
    let mut genesis = Founding {
        seed,
        sites: 4,
        population: 120,
        cohort_size: 1,
        lineages: 10,
        ecology: true,
        web: Web::Diets,
        ecology_rates: Rates::WorldTime,
        experimental_food_contract: true,
        bodies: Some(Bodies { clutch: [1, 1], sensing: true, grazing: true, hazard: [0, 0], wound: [0, 0], ..Bodies::default() }),
        map: Some(cosmograph::map::Layout::Grid(grid)),
        ..Default::default()
    }
    .generate()
    .unwrap();
    for p in genesis.rules.processes.values_mut().filter(|p| p.id.starts_with("ecology:hazard-")) {
        p.risk.as_mut().unwrap().aging.as_mut().unwrap().year = DAY;
    }
    Session::new(genesis, mode).unwrap()
}

/// The tick a hazard's draw names for `actor`, worked out here from the
/// rules as `death::Drawn` does.
fn drawn(s: &Session, process: &str, actor: Id, born: Tick) -> Option<Tick> {
    let g = s.sim.genesis();
    let p = &g.rules.processes[process];
    let a = p.risk.as_ref()?.aging?;
    let gate = p.requires.iter().filter_map(|q| match q {
        cosmograph::rules::Query::Age { at_least } => Some(*at_least),
        _ => None,
    }).max().unwrap_or(0);
    let from = (born + gate.max(a.maturity)).max(g.started + 1);
    let u = cosmograph::draw(g.dynamics_seed(), process, &[actor, from]);
    Some(from + a.falls_after(from - born, u)?)
}

#[test]
fn each_hazard_death_falls_on_its_drawn_tick() {
    let mut s = world(Execution::Grouped);
    s.advance(2 * DAY).unwrap();
    let state = s.sim.state();
    let deaths: Vec<_> = state.events.values().filter(|e| e.process.starts_with("ecology:hazard-")).collect();
    assert!(deaths.len() >= 5, "only {} hazard deaths", deaths.len());
    let mut checked = 0;
    for e in &deaths {
        let born = state.population.get(e.subject).map(|b| b.born);
        if let Some(born) = born {
            assert_eq!(Some(e.tick), drawn(&s, &e.process, e.subject, born), "member {}", e.subject);
            checked += 1;
        }
    }
    eprintln!("{} hazard deaths, {checked} with their bodies still held", deaths.len());
    assert!(checked * 2 >= deaths.len(), "most deaths are read against their draws");
    // No living member under a hazard is past its drawn tick.
    let hazards: Vec<String> = s.sim.genesis().rules.processes.keys().filter(|k| k.starts_with("ecology:hazard-")).cloned().collect();
    for (&first, g) in &state.population.groups {
        let e = &g.entity;
        if !e.alive {
            continue;
        }
        for h in &hazards {
            let trait_key = format!("ability:cycle-{}", h.trim_start_matches("ecology:hazard-"));
            if e.traits.contains(&trait_key) {
                for actor in first..first + g.count {
                    assert!(drawn(&s, h, actor, e.born).is_none_or(|t| t > state.tick), "member {actor} outlived its tick");
                }
            }
        }
    }
}

#[test]
fn drawn_deaths_conserve_matter_agree_across_modes_and_replay() {
    let mut runs: Vec<Session> = [Execution::Individuals, Execution::Grouped].map(world).into();
    let matter = runs[0].sim.matter();
    for s in &mut runs {
        for _ in 0..4 {
            s.advance(DAY / 2).unwrap();
        }
        assert_eq!(s.sim.matter(), matter, "matter is conserved");
    }
    assert_eq!(runs[0].sim.state_hash(), runs[1].sim.state_hash(), "both modes agree");
    let saved = runs[1].save();
    let at_save = runs[1].sim.state_hash();
    runs[1].advance(DAY / 2).unwrap();
    for mode in [Execution::Grouped, Execution::Individuals] {
        let mut loaded = Session::load(saved.clone(), mode).expect("the save replays exactly");
        assert_eq!(loaded.sim.state_hash(), at_save);
        loaded.advance(DAY / 2).unwrap();
        assert_eq!(loaded.sim.state_hash(), runs[1].sim.state_hash(), "a loaded world goes on alike");
    }
}

// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The decomposer line of the seed-9 A1 played world (Mesocosm's founding,
//! as `played_world_decay` states it): upkeep runs on every body's own clock,
//! so a decomposer must be fed from the dead matter that exists, not die all
//! at once when its founding reserve runs out. The ignored tests print the
//! diagnosis (release, `--ignored --nocapture`): `decomposer_diagnosis` per
//! tick (knobs `DIAG_TICKS`, `DECAY_SEED`, `DECAY_DORMANT=1`, and the probes
//! `BARE`, `NOPLAYED`, `NO_MOVES`, `NO_DIRECTING`, `DECIDES_ON`,
//! `L2_DECIDES=none`), `decomposer_feed_attempts` (the food diagnostic's
//! tallies), `decomposer_events` and `survival_table` (`DECAY_SEEDS=9,3`).

use cosmograph::{
    Execution, Founding, Session,
    anatomy::books,
    bodied::Bodies,
    directing::Played,
    ecology::Rates,
    flows::MadeBy,
};

fn founding(seed: u64) -> Founding {
    let grid = cosmograph::map::Grid { width: 3, height: 2, ..cosmograph::map::Grid::drawn(seed) };
    Founding {
        seed,
        population: 256,
        sites: grid.width * grid.height,
        map: Some(cosmograph::map::Layout::Grid(grid)),
        ecology: true,
        ecology_rates: Rates::A1,
        bodies: Some(Bodies {
            hazard: cosmograph::harm::hazard::RATE,
            wound: cosmograph::harm::hazard::CELLS,
            ..Bodies::default()
        }),
        played: Some(Played { lineage: 1, region_sites: 2 }),
        dormancy: (std::env::var("DECAY_DORMANT").as_deref() == Ok("1"))
            .then(cosmograph::rules::Dormancy::flat),
        ..Founding::default()
    }
}

fn is(lineage: &str, i: u32) -> bool {
    lineage == format!("lineage:{i}")
}

/// Living members of lineage 2.
fn decomposers_alive(s: &Session) -> u64 {
    s.sim.state().population.groups.values()
        .filter(|g| g.entity.alive && is(&g.entity.lineage, 2)).map(|g| g.count).sum()
}

/// The founding cohort (128 decomposers, one reserve of 10) paid upkeep on
/// every body's own clock and died together at tick 50, before any dead
/// matter existed and unable to feed: hungry, they moved before they ate (a move runs
/// before a meal in the same decision, and a hungry body is always due to move).
/// They must outlive that, feed, and still be there past the first hazard
/// deaths and the first upkeep periods after them.
#[test]
#[ignore = "110 ticks: release build"]
fn the_decomposer_line_outlives_its_founding_reserve() {
    let mut s = Session::new(founding(9).generate().unwrap(), Execution::Individuals).unwrap();
    assert_eq!(decomposers_alive(&s), 128);
    s.advance(55).unwrap();
    assert!(decomposers_alive(&s) > 100, "decomposers dead at the old tick 50: {}", decomposers_alive(&s));
    s.advance(55).unwrap();
    let now = decomposers_alive(&s);
    assert!(now > 40, "decomposers at tick 110: {now}");
    let born = s.sim.state().events.values()
        .filter(|e| e.process == "ecology:birth-2").count();
    assert!(born > 20, "decomposers fed to birth: {born}");
}

/// The short sibling, readable in a debug build: what the long run rests on.
/// The founding reserve outlasts the time to the first hazard deaths (the
/// first dead matter), a decomposer's meal is due at every decision, and a
/// hungry decomposer's meal runs before the moves its mood sends it on.
#[test]
fn a_decomposer_can_last_to_its_first_dead_matter_and_eat_it() {
    let g = founding(9).generate().unwrap();
    let p = |id: &str| g.rules.processes[id].clone();
    let reserve = g.population.groups.values()
        .find(|c| c.entity.lineage == "lineage:2").map(|c| books(&c.entity)["matter:2-0"]).unwrap();
    let lasts = reserve * p("ecology:upkeep-2").period.unwrap();
    assert!(lasts > 60, "a reserve of {reserve} lasts {lasts} ticks, the first dead come after 60");
    let decides = g.lineages["lineage:2"].decides.unwrap();
    assert!(p("ecology:feed-2").period.unwrap() <= decides, "a meal is due at every decision");
    let feed = p("ecology:feed-2").priority;
    for (id, m) in g.rules.processes.iter().filter(|(id, _)| id.starts_with("move:")) {
        assert!(feed < m.priority, "{id} runs before the decomposer's meal");
    }
}

/// Living members of lineage 2 and the tissue the dead hold, per tick.
fn decomposers(s: &Session) -> (u64, u64, u64, Vec<u64>) {
    let (mut live, mut dead_tissue, mut reserve, mut dead_bodies) = (0, 0, 0, 0u64);
    let mut by_reserve = vec![0u64; 14];
    let _ = ();
    for g in s.sim.state().population.groups.values() {
        let e = &g.entity;
        let tissue: u64 = books(e).iter().filter(|(k, _)| k.starts_with("matter:") ).map(|(_, v)| *v).sum();
        if !e.alive {
            dead_tissue += tissue * g.count;
            dead_bodies += g.count;
        } else if is(&e.lineage, 2) {
            live += g.count;
            reserve += tissue * g.count;
            by_reserve[(tissue as usize).min(13)] += g.count;
        }
    }
    let _ = dead_bodies;
    (live, dead_tissue, reserve, by_reserve)
}

#[test]
#[ignore = "diagnostic table"]
fn decomposer_diagnosis() {
    let seed = std::env::var("DECAY_SEED").ok().and_then(|v| v.parse().ok()).unwrap_or(9);
    let mut f = founding(seed);
    if std::env::var("BARE").is_ok() { f.bodies = None; }
    if std::env::var("NOPLAYED").is_ok() { f.played = None; }
    let g = f.generate().unwrap();
    for id in ["ecology:feed-2", "ecology:upkeep-2", "ecology:birth-2"] {
        eprintln!("{id}: {:?}", g.rules.processes[id]);
    }
    let mut g = g;
    for (id, p) in g.rules.processes.iter().filter(|(id, _)| id.starts_with("move:")) { eprintln!("{id} period {:?} priority {} requires {:?}", p.period, p.priority, p.requires); }
    if std::env::var("NO_MOVES").is_ok() { g.rules.processes.retain(|id, _| !id.starts_with("move:")); }
    if std::env::var("DECIDES_ON").is_ok() { for (i, d) in [(0, 10), (1, 2), (2, 5), (3, 10)] { g.lineages.get_mut(&format!("lineage:{i}")).unwrap().decides = Some(d); } }
    if std::env::var("NO_DIRECTING").is_ok() { g.rules.directing = None; }
    if std::env::var("L2_DECIDES").as_deref() == Ok("none") { g.lineages.get_mut("lineage:2").unwrap().decides = None; }
    eprintln!("decides L2: {:?}", g.lineages["lineage:2"].decides);
    let mut s = Session::new(g, Execution::Individuals).unwrap();
    eprintln!("tick  live  deadTissue  reserve  meals(f) upkeep(f) | reserve histogram 0..13");
    let to: u64 = std::env::var("DIAG_TICKS").ok().and_then(|v| v.parse().ok()).unwrap_or(60);
    for n in 1..=to {
        let r = s.advance_tick_with_flows().unwrap();
        let (mut meals, mut pays) = (0u128, 0u128);
        for f in r.flows.iter().filter(|f| !f.internal()) {
            if let MadeBy::Process(p) = &f.made_by {
                if p.starts_with("ecology:feed-") && p == "ecology:feed-2" { let prey = f.from.0.body().and_then(|b| s.sim.state().population.get(b)).map(|e| (e.alive, e.lineage.clone()));
                    if n % 5 == 0 { eprintln!("   {n} feed-2 prey {prey:?} {}x{}", f.amount, f.count); } }
                if p == "ecology:feed-2" {
                    meals += u128::from(f.amount) * u128::from(f.count);
                }
                if p == "ecology:upkeep-2" {
                    pays += u128::from(f.count);
                }
            }
        }
        let (live, dead, reserve, h) = decomposers(&s);
        if n % 10 == 0 && n >= 60 {
            let mut at: std::collections::BTreeMap<_, (u64, u64, u64)> = Default::default();
            for g in s.sim.state().population.groups.values() {
                let e = &g.entity;
                let Some(site) = e.resident_site() else { continue };
                let tissue: u64 = books(e).iter().filter(|(k, _)| k.starts_with("matter:")).map(|(_, v)| *v).sum();
                let row = at.entry(site).or_default();
                if !e.alive { row.1 += g.count; row.2 += tissue * g.count; }
                else if is(&e.lineage, 2) { row.0 += g.count; }
            }
            eprintln!("   sites (L2 here, dead bodies, dead tissue) at {n}: {at:?}");
        }
        let moving: u64 = s.sim.state().population.groups.values().filter(|g| g.entity.alive && g.entity.journey.is_some() && is(&g.entity.lineage, 2)).map(|g| g.count).sum();
        eprint!("[L2 travelling {moving}] ");
        eprintln!("{n:>4} {live:>5} {dead:>11} {reserve:>8} {meals:>8} {pays:>9} | {h:?}");
    }
}

/// Living members by lineage at the table's ticks: `DECAY_SEEDS=9,3`,
/// `DECAY_DORMANT=1`, `DECAY_TICKS=600`.
#[test]
#[ignore = "diagnostic table"]
fn survival_table() {
    let seeds: Vec<u64> = std::env::var("DECAY_SEEDS").unwrap_or("9".into())
        .split(',').filter_map(|v| v.parse().ok()).collect();
    let ticks: u64 = std::env::var("DECAY_TICKS").ok().and_then(|v| v.parse().ok()).unwrap_or(600);
    let at = [0u64, 50, 100, 200, 300, 450, 600];
    for seed in seeds {
        let mut s = Session::new(founding(seed).generate().unwrap(), Execution::Individuals).unwrap();
        let mut rows = vec![];
        for n in 0..=ticks {
            if at.contains(&n) {
                let mut c = [0u64; 4];
                for g in s.sim.state().population.groups.values() {
                    if let (true, Some(i)) = (g.entity.alive, g.entity.lineage.strip_prefix("lineage:").and_then(|v| v.parse::<usize>().ok())) {
                        if i < 4 { c[i] += g.count; }
                    }
                }
                rows.push(format!("{n}:{c:?}"));
            }
            if n < ticks { s.advance(1).unwrap(); }
        }
        eprintln!("seed {seed} dormant={:?}: {}", std::env::var("DECAY_DORMANT").ok(), rows.join("  "));
    }
}

#[test]
#[ignore = "diagnostic table"]
fn decomposer_feed_attempts() {
    let g = founding(9).generate().unwrap();
    let mut s = Session::new(g, Execution::Individuals).unwrap();
    s.sim
        .begin_food_diagnostic(cosmograph::watch::diagnostic::FoodDiagnosticConfig {
            focus_lineages: ["lineage:2".to_string()].into(),
            examples_per_kind: 2,
            bucket_ticks: 25,
        })
        .unwrap();
    s.advance(100).unwrap();
    let r = s.sim.end_food_diagnostic().unwrap();
    for f in r.feeds.iter().filter(|f| f.lineage == "lineage:2") {
        eprintln!("{} {} tgt={:?}/{:?} phase={} acc={} stage={} attempts={} selected={} no_target={}", f.bucket, f.process, f.target_lineage, f.target_alive, f.phase, f.accepted, f.stage, f.attempts, f.selected, f.no_target);
    }
}

#[test]
#[ignore = "diagnostic table"]
fn decomposer_events() {
    let mut s = Session::new(founding(9).generate().unwrap(), Execution::Individuals).unwrap();
    s.advance(250).unwrap();
    let st = s.sim.state();
    let mut by: std::collections::BTreeMap<(u64, String), u64> = Default::default();
    for e in st.events.values() {
        let Some(b) = st.population.get(e.subject) else { continue };
        if b.lineage != "lineage:2" { continue }
        *by.entry((e.tick / 25 * 25, e.process.clone())).or_default() += 1;
    }
    for ((w, p), n) in by { eprintln!("{w:>4} {p} {n}"); }
}

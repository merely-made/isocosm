// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Diagnosis, not a gate: why the A1 played founding (seed 9, 6 sites, 4
//! lineages, Mesocosm's `Founded::generated_with_affinity`) runs to total
//! extinction as a plain unpiloted simulation. Every test is ignored and
//! prints tables; none asserts a result. Run them all, in release:
//!
//! `cargo test --release --test played_world_decay -- --ignored --nocapture --test-threads=1`
//!
//! Knobs: `DECAY_TICKS` (600), `DECAY_SEED` (9), `DECAY_POP` (256),
//! `DECAY_DORMANT` (0; 1 founds the flat rate set's dormancy, `Dormancy::flat()`).
//! Observation only: a run advances a tick at a time with its matter moves
//! handed over, which changes no outcome.

use cosmograph::{
    Execution, Founding, Session,
    anatomy::books,
    bodied::Bodies,
    directing::Played,
    ecology::{MassLaw, Rates},
    flows::{Flow, Holder},
    schema::Id,
    simulation::Genesis,
};
use std::collections::BTreeMap;

fn env(key: &str, default: u64) -> u64 {
    std::env::var(key).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
}

/// Mesocosm's founding (`Founded::generated_with_affinity` then
/// `a1_setup_with`): 3x2 grid, wound and hazard at the probe's rates, the
/// played lineage 1 in a region of 2 sites, ecology on, A1 rates.
fn founding(seed: u64, population: u64) -> Founding {
    let grid = cosmograph::map::Grid { width: 3, height: 2, ..cosmograph::map::Grid::drawn(seed) };
    Founding {
        seed,
        population,
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
        dormancy: (env("DECAY_DORMANT", 0) == 1).then(cosmograph::rules::Dormancy::flat),
        ..Founding::default()
    }
}

fn world(seed: u64, population: u64) -> Genesis {
    founding(seed, population).generate().unwrap()
}

fn index(lineage: &str) -> Option<u32> {
    lineage.strip_prefix("lineage:")?.parse().ok()
}

fn role(i: u32) -> &'static str {
    // Web::Levels: [Producer, Grazer, Decomposer][i % 3]
    ["producer", "grazer", "decomposer"][i as usize % 3]
}

/// Living members by lineage.
fn alive(s: &Session) -> BTreeMap<u32, u64> {
    let mut out = BTreeMap::new();
    for g in s.sim.state().population.groups.values() {
        if let (true, Some(i)) = (g.entity.alive, index(&g.entity.lineage)) {
            *out.entry(i).or_default() += g.count;
        }
    }
    out
}

/// Living members by (lineage, site).
fn by_site(s: &Session) -> BTreeMap<(u32, Id), u64> {
    let mut out = BTreeMap::new();
    for g in s.sim.state().population.groups.values() {
        if let (true, Some(i)) = (g.entity.alive, index(&g.entity.lineage)) {
            *out.entry((i, g.entity.place)).or_default() += g.count;
        }
    }
    out
}

/// World matter by holder class: living bodies per lineage (body matter and
/// the soil inside them), dead bodies per lineage, site soil, other.
fn matter_classes(s: &Session) -> BTreeMap<String, u128> {
    let mut out: BTreeMap<String, u128> = BTreeMap::new();
    let st = s.sim.state();
    let accounts = &s.sim.genesis().rules.accounts;
    let is_matter = |k: &str| matches!(accounts.get(k), Some(cosmograph::rules::AccountKind::Matter { .. }));
    for g in st.population.groups.values() {
        let e = &g.entity;
        let Some(i) = index(&e.lineage) else { continue };
        let tag = if e.alive { "alive" } else { "dead" };
        for (k, v) in books(e) {
            if !is_matter(&k) {
                continue;
            }
            let what = if k == "world:soil" { "soil-in-body" } else { "tissue" };
            *out.entry(format!("{tag} L{i} {what}")).or_default() += u128::from(v) * u128::from(g.count);
        }
    }
    for site in st.sites.values() {
        for (k, v) in &site.accounts {
            if !is_matter(k) {
                continue;
            }
            *out.entry(format!("site {k}")).or_default() += u128::from(*v);
        }
    }
    out
}

/// What the entity a holder names is, now.
fn class(s: &Session, h: Holder, account: &str) -> String {
    let short = if account == "world:soil" { "soil" } else if account.starts_with("matter:") { "tissue" } else { account };
    match h {
        Holder::Site(_) => format!("site {short}"),
        Holder::Entity(id) | Holder::Part(id, _) => match s.sim.state().population.get(id) {
            Some(e) => format!("{} L{} {short}", if e.alive { "alive" } else { "dead" }, index(&e.lineage).map_or(99, |i| i)),
            None => format!("gone {short}"),
        },
        _ => "other".into(),
    }
}

#[derive(Default)]
struct Trace {
    /// Living by lineage at every tick (index = tick since start).
    counts: Vec<BTreeMap<u32, u64>>,
    /// Per-site counts every 25 ticks.
    sites: Vec<(u64, BTreeMap<(u32, Id), u64>)>,
    /// Matter classes every 50 ticks, with the world's own total.
    matter: Vec<(u64, BTreeMap<String, u128>, u128)>,
    /// Matter moves per 50-tick window, by "process | from -> to".
    flows: BTreeMap<(u64, String), u128>,
    /// Events by (window, process kind, lineage).
    events: BTreeMap<(u64, String, u32), u64>,
}

fn kind(process: &str) -> String {
    process.rsplit_once('-').map_or(process, |(k, _)| k).to_string()
}

fn run(genesis: Genesis, mode: Execution, ticks: u64, with_flows: bool) -> Trace {
    let mut s = Session::new(genesis, mode).unwrap();
    let start = s.sim.state().tick;
    let mut t = Trace::default();
    t.counts.push(alive(&s));
    t.sites.push((0, by_site(&s)));
    t.matter.push((0, matter_classes(&s), s.sim.matter()));
    for n in 1..=ticks {
        if with_flows {
            let r = s.advance_tick_with_flows().expect("a native tick");
            for f in r.flows.iter().filter(|f| !f.internal()) {
                record(&s, f, n, &mut t);
            }
        } else {
            s.advance(1).expect("a native tick");
        }
        t.counts.push(alive(&s));
        if n % 25 == 0 {
            t.sites.push((n, by_site(&s)));
        }
        if n % 50 == 0 {
            t.matter.push((n, matter_classes(&s), s.sim.matter()));
        }
    }
    let state = s.sim.state();
    for e in state.events.values().filter(|e| e.tick > start) {
        let lineage = state.population.get(e.subject).and_then(|b| index(&b.lineage)).unwrap_or(99);
        let w = (e.tick - start - 1) / 50 * 50;
        *t.events.entry((w, kind(&e.process), lineage)).or_default() += 1;
    }
    t
}

fn record(s: &Session, f: &Flow, n: u64, t: &mut Trace) {
    let process = match &f.made_by {
        cosmograph::flows::MadeBy::Process(p) => p.clone(),
        cosmograph::flows::MadeBy::Command(c) => c.clone(),
    };
    let key = format!("{process} | {} -> {}", class(s, f.from.0, &f.from.1), class(s, f.to.0, &f.to.1));
    *t.flows.entry(((n - 1) / 50 * 50, key)).or_default() += u128::from(f.amount) * u128::from(f.count);
}

fn total(m: &BTreeMap<u32, u64>) -> u64 {
    m.values().sum()
}

/// "ticks 0/100/200/300" style row for each lineage, and first-extinction ticks.
fn members_table(label: &str, t: &Trace, lineages: &[u32], every: usize) {
    let last = t.counts.len() - 1;
    let ticks: Vec<usize> = (0..=last).step_by(every).collect();
    eprintln!("-- {label}");
    eprint!("{:<26}", "lineage (role)");
    for k in &ticks {
        eprint!("{k:>6}");
    }
    eprintln!("   peak@tick   extinct@");
    for &i in lineages {
        eprint!("{:<26}", format!("L{i} ({})", role(i)));
        for &k in &ticks {
            eprint!("{:>6}", t.counts[k].get(&i).copied().unwrap_or(0));
        }
        let (peak_tick, peak) = t.counts.iter().enumerate().map(|(k, c)| (k, c.get(&i).copied().unwrap_or(0)))
            .fold((0, 0), |a, b| if b.1 > a.1 { b } else { a });
        let dead = t.counts.iter().position(|c| c.get(&i).copied().unwrap_or(0) == 0);
        eprintln!("   {peak:>5}@{peak_tick:<5} {}", dead.map_or("never".into(), |k| k.to_string()));
    }
    eprint!("{:<26}", "total");
    for &k in &ticks {
        eprint!("{:>6}", total(&t.counts[k]));
    }
    let dead = t.counts.iter().position(|c| total(c) == 0);
    eprintln!("   all extinct@ {}", dead.map_or("never".into(), |k| k.to_string()));
}

fn mode_name(m: Execution) -> &'static str {
    if m == Execution::Individuals { "Individuals" } else { "Grouped" }
}

// ---------------------------------------------------------------------------

/// Q1: the finding, reproduced on five seeds under both execution modes, and
/// how long it is in world time.
#[test]
#[ignore = "diagnostic table"]
fn q1_reproduce_across_seeds_and_modes() {
    let ticks = env("DECAY_TICKS", 600);
    let g = world(9, 256);
    let unit = g.rules.tick_microseconds();
    eprintln!("tick = {} microseconds ({} s); epoch_ticks as founded = {}; 300 ticks = {:.2} hours = {:.3} days; 600 ticks = {:.2} days",
        unit, unit / 1_000_000, g.rules.epoch_ticks, 300.0 * unit as f64 / 3.6e9, 300.0 * unit as f64 / 8.64e10, 600.0 * unit as f64 / 8.64e10);
    eprintln!("Mesocosm: pace.round = 1 tick, scoring 6; played-world epoch (config.play_epoch / bench_epoch) = 60 ticks, so 300 ticks = 5 epochs = 300 rounds");
    for seed in [9u64, 3, 5, 11, 17] {
        for mode in [Execution::Individuals, Execution::Grouped] {
            let t = run(world(seed, 256), mode, ticks, false);
            members_table(&format!("seed {seed} {} pop 256 (lineages 0,3 producers; 1 grazer; 2 decomposer)", mode_name(mode)),
                &t, &[0, 1, 2, 3], 50);
        }
    }
    eprintln!("== founders 60 (what Mesocosm's --play uses)");
    for seed in [9u64, 3, 5, 11, 17] {
        let t = run(world(seed, 60), Execution::Individuals, ticks, false);
        members_table(&format!("seed {seed} Individuals pop 60"), &t, &[0, 1, 2, 3], 50);
    }
}

/// Q2: sequence and per-site counts.
#[test]
#[ignore = "diagnostic table"]
fn q2_sequence_and_sites() {
    let seed = env("DECAY_SEED", 9);
    let t = run(world(seed, env("DECAY_POP", 256)), Execution::Individuals, env("DECAY_TICKS", 600), false);
    members_table(&format!("seed {seed} every 10 ticks"), &t, &[0, 1, 2, 3], 10);
    eprintln!("-- per-site counts (site: L0 L1 L2 L3) every 25 ticks");
    let sites: Vec<Id> = t.sites[0].1.keys().map(|k| k.1).collect::<std::collections::BTreeSet<_>>().into_iter().collect();
    eprint!("{:>5}", "tick");
    for s in &sites {
        eprint!("  site{s:<2}(L0 L1 L2 L3)  ");
    }
    eprintln!();
    for (n, m) in &t.sites {
        eprint!("{n:>5}");
        for s in &sites {
            let c = |i| m.get(&(i, *s)).copied().unwrap_or(0);
            eprint!("  {:>9}{:>4}{:>4}{:>4}  ", c(0), c(1), c(2), c(3));
        }
        eprintln!();
    }
}

/// Q3 and Q4: where matter goes, and what killed what.
#[test]
#[ignore = "diagnostic table"]
fn q3_q4_matter_deaths_and_flows() {
    let seed = env("DECAY_SEED", 9);
    let ticks = env("DECAY_TICKS", 600);
    let t = run(world(seed, env("DECAY_POP", 256)), Execution::Individuals, ticks, true);
    eprintln!("-- Q3 matter by holder (sim.matter() is the world's own total; `sum` is the table's)");
    let keys: Vec<String> = t.matter.iter().flat_map(|m| m.1.keys().cloned()).collect::<std::collections::BTreeSet<_>>().into_iter().collect();
    eprint!("{:<28}", "holder");
    for (n, _, _) in &t.matter {
        eprint!("{n:>8}");
    }
    eprintln!();
    for k in &keys {
        eprint!("{k:<28}");
        for (_, m, _) in &t.matter {
            eprint!("{:>8}", m.get(k).copied().unwrap_or(0));
        }
        eprintln!();
    }
    eprint!("{:<28}", "sum of table");
    for (_, m, _) in &t.matter {
        eprint!("{:>8}", m.values().sum::<u128>());
    }
    eprintln!();
    eprint!("{:<28}", "sim.matter()");
    for (_, _, w) in &t.matter {
        eprint!("{w:>8}");
    }
    eprintln!();

    eprintln!("-- Q4 events per 50-tick window by process kind and lineage (death = body below 1 matter: starved or eaten out; age = lifespan reached; hazard = per-tick draw after maturity; birth)");
    let kinds: Vec<(String, u32)> = t.events.keys().map(|k| (k.1.clone(), k.2)).collect::<std::collections::BTreeSet<_>>().into_iter().collect();
    eprint!("{:<26}", "process / lineage");
    for w in (0..ticks).step_by(50) {
        eprint!("{w:>6}");
    }
    eprintln!();
    for (k, l) in &kinds {
        eprint!("{:<26}", format!("{k} L{l} ({})", if *l == 99 { "gone" } else { role(*l) }));
        for w in (0..ticks).step_by(50) {
            eprint!("{:>6}", t.events.get(&(w, k.clone(), *l)).copied().unwrap_or(0));
        }
        eprintln!();
    }

    eprintln!("-- matter moves per 50-tick window (what moved, from -> to)");
    let fk: Vec<String> = t.flows.keys().map(|k| k.1.clone()).collect::<std::collections::BTreeSet<_>>().into_iter().collect();
    eprint!("{:<64}", "process | from -> to");
    for w in (0..ticks).step_by(50) {
        eprint!("{w:>7}");
    }
    eprintln!();
    for k in &fk {
        eprint!("{k:<64}");
        for w in (0..ticks).step_by(50) {
            eprint!("{:>7}", t.flows.get(&(w, k.clone())).copied().unwrap_or(0));
        }
        eprintln!();
    }
}

/// Q5: the A1 rates as the genesis states them, per lineage.
#[test]
#[ignore = "diagnostic table"]
fn q5_rate_budget() {
    let g = world(env("DECAY_SEED", 9), 256);
    eprintln!("founding: stock_min {} stock_max {}", g.founding.as_ref().unwrap().stock_min, g.founding.as_ref().unwrap().stock_max);
    for i in 0..4u32 {
        eprintln!("== lineage {i} ({})", role(i));
        for kind in ["upkeep", "produce", "feed", "birth", "age", "hazard", "death"] {
            if let Some(p) = g.rules.processes.get(&format!("ecology:{kind}-{i}")) {
                eprintln!("  {kind}: period {:?} priority {} requires {:?}\n      effects {:?}\n      risk {:?}", p.period, p.priority, p.requires, p.effects, p.risk);
            }
        }
    }
    let sun = &g.rules.processes["ecology:sunlight"];
    eprintln!("sunlight: {:?} {:?}; sun rules {:?}", sun.effects, sun.requires, g.rules.sun);
    eprintln!("conditions: {:?}", g.rules.conditions);
    let st = Session::new(g.clone(), Execution::Individuals).unwrap();
    for (id, site) in &st.sim.state().sites {
        eprintln!("site {id}: {:?}", site.accounts);
    }
    for gr in st.sim.state().population.groups.values() {
        let e = &gr.entity;
        if index(&e.lineage).is_some() {
            eprintln!("founder cohort {} x{} at site {} accounts {:?} method {:?}", e.lineage, gr.count, e.place, e.accounts, e.method);
        }
    }
}

// ---------------------------------------------------------------------------
// Q6 probes.

fn survivors(t: &Trace, lineages: &[u32], at: &[usize]) -> String {
    let mut s = String::new();
    for &k in at {
        let k = k.min(t.counts.len() - 1);
        s += &format!(" t{k}:[");
        for (j, i) in lineages.iter().enumerate() {
            if j > 0 {
                s += "/";
            }
            s += &t.counts[k].get(i).copied().unwrap_or(0).to_string();
        }
        s += "]";
    }
    s
}

#[test]
#[ignore = "diagnostic table"]
fn q6_probes() {
    let seed = env("DECAY_SEED", 9);
    let ticks = env("DECAY_TICKS", 600);
    let at = [0usize, 100, 200, 300, 450, 600];
    let mode = Execution::Individuals;
    let mut rows: Vec<(String, Trace, Vec<u32>)> = vec![];
    let mut add = |name: &str, g: Genesis, show: Vec<u32>| {
        eprintln!("running {name}");
        rows.push((name.to_string(), run(g, mode, ticks, false), show));
    };

    add("baseline A1 (L0 prod, L1 graz, L2 dec, L3 prod)", world(seed, 256), vec![0, 1, 2, 3]);
    // (a) producers alone: one lineage, no played lineage.
    let only = |lineages: u32, played: Option<Played>, mass: Option<MassLaw>| {
        let mut f = founding(seed, 256);
        f.lineages = lineages;
        f.played = played;
        f.mass_law = mass;
        f.generate()
    };
    add("(a) producers alone (lineages=1, no played)", only(1, None, None).unwrap(), vec![0]);
    // (b) producers + grazers.
    add("(b) producers+grazers (lineages=2, played L1)", only(2, Some(Played { lineage: 1, region_sites: 2 }), None).unwrap(), vec![0, 1]);
    add("(b2) producers+grazers, no played", only(2, None, None).unwrap(), vec![0, 1]);
    add("(b3) producers+grazers+decomposers (lineages=3)", only(3, Some(Played { lineage: 1, region_sites: 2 }), None).unwrap(), vec![0, 1, 2]);
    // (c) upkeep periods doubled.
    let mut g = world(seed, 256);
    for (id, p) in g.rules.processes.iter_mut() {
        if id.starts_with("ecology:upkeep-") {
            p.period = p.period.map(|x| x * 2);
        }
    }
    add("(c) upkeep periods doubled (all lineages)", g, vec![0, 1, 2, 3]);
    // (d) the played lineage removed.
    let mut g = world(seed, 256);
    g.population.groups.retain(|_, c| c.entity.lineage != "lineage:1");
    add("(d) lineage 1 (grazer, played) founders removed", g, vec![0, 2, 3]);
    let mut f = founding(seed, 256);
    f.played = None;
    add("(d2) no played lineage declared (L1 still a grazer)", f.generate().unwrap(), vec![0, 1, 2, 3]);
    // (e) the mass law on.
    let mut f = founding(seed, 256);
    f.mass_law = Some(MassLaw::default());
    add("(e) mass law on", f.generate().unwrap(), vec![0, 1, 2, 3]);
    // extra: more soil per site, more founders of producers, producer-heavy.
    let mut g = world(seed, 256);
    for site in g.sites.values_mut() {
        if let Some(soil) = site.accounts.get_mut("world:soil") {
            *soil *= 4;
        }
    }
    add("(x1) soil x4 at every site", g, vec![0, 1, 2, 3]);
    let mut g = world(seed, 256);
    for (id, p) in g.rules.processes.iter_mut() {
        if id.starts_with("ecology:feed-1") {
            p.period = p.period.map(|x| x * 3);
        }
    }
    add("(x2) grazer feeds 3x less often", g, vec![0, 1, 2, 3]);
    let mut g = world(seed, 256);
    g.rules.processes.retain(|id, _| !(id.starts_with("ecology:hazard-") || id.starts_with("ecology:age-")));
    add("(x3) no hazard / no age deaths for anyone", g, vec![0, 1, 2, 3]);
    let mut g = world(seed, 256);
    g.rules.processes.retain(|id, _| !(id.starts_with("ecology:hazard-") || id.starts_with("ecology:age-")) || id.ends_with("-1") || id.ends_with("-2"));
    add("(x4) producers immortal (no age/hazard for L0, L3)", g, vec![0, 1, 2, 3]);
    let mut g = world(seed, 256);
    g.rules.processes.retain(|id, _| !(id.starts_with("ecology:hazard-") || id.starts_with("ecology:age-")) || id.ends_with("-0") || id.ends_with("-3"));
    add("(x5) consumers+decomposer immortal (no age/hazard for L1, L2)", g, vec![0, 1, 2, 3]);

    // The played founding gives each role a decision period (D19): 10, 2, 5.
    let mut g = world(seed, 256);
    for l in g.lineages.values_mut() {
        l.decides = None;
    }
    add("(x6) played, but every lineage decides every tick", g, vec![0, 1, 2, 3]);
    let mut g = world(seed, 256);
    for i in [0, 3] {
        g.lineages.get_mut(&format!("lineage:{i}")).unwrap().decides = None;
    }
    add("(x7) played, only producers decide every tick", g, vec![0, 1, 2, 3]);
    let mut g = world(seed, 256);
    for i in [0, 2, 3] {
        g.lineages.get_mut(&format!("lineage:{i}")).unwrap().decides = None;
    }
    add("(x8) played, only grazers keep decides=2", g, vec![0, 1, 2, 3]);

    eprintln!("== probes, seed {seed}, {} ticks, {} (members [listed lineages] at ticks)", ticks, mode_name(mode));
    for (name, t, show) in &rows {
        let dead = t.counts.iter().position(|c| total(c) == 0);
        eprintln!("{name:<62}{}  all-extinct@{}", survivors(t, show, &at), dead.map_or("never".into(), |k| k.to_string()));
    }
}

// ---------------------------------------------------------------------------
// Q5 measured: the per-member-tick budget the run actually shows, per window,
// and how many bodies ever pay upkeep.

fn parse_amount(t: &Trace, w: u64, pred: impl Fn(&str) -> bool) -> u128 {
    t.flows.iter().filter(|((fw, k), _)| *fw == w && pred(k)).map(|(_, v)| *v).sum()
}

#[test]
#[ignore = "diagnostic table"]
fn q5_measured_budget_and_upkeep_participation() {
    let seed = env("DECAY_SEED", 9);
    let ticks = env("DECAY_TICKS", 300);
    for (label, played) in [("played (decides 10/2/5 per role)", true), ("no played lineage (every member decides every tick)", false)] {
        let mut f = founding(seed, 256);
        if !played {
            f.played = None;
        }
        let g = f.generate().unwrap();
        let mut s = Session::new(g, Execution::Individuals).unwrap();
        let start = s.sim.state().tick;
        let mut t = Trace::default();
        let mut member_ticks: BTreeMap<(u64, u32), u128> = BTreeMap::new();
        let mut paid: BTreeMap<Id, u32> = BTreeMap::new();
        for n in 1..=ticks {
            let r = s.advance_tick_with_flows().unwrap();
            for fl in r.flows.iter().filter(|f| !f.internal()) {
                record(&s, fl, n, &mut t);
                if let cosmograph::flows::MadeBy::Process(p) = &fl.made_by {
                    if p.starts_with("ecology:upkeep-") && matches!(fl.to.0, Holder::Site(_)) {
                        if let Some(b) = fl.from.0.body() {
                            *paid.entry(b).or_default() += 1;
                        }
                    }
                }
            }
            for (i, c) in alive(&s) {
                *member_ticks.entry(((n - 1) / 50 * 50, i)).or_default() += u128::from(c);
            }
        }
        eprintln!("== {label}, seed {seed}: matter per member per tick (x1000), by 50-tick window");
        eprintln!("{:<30}{}", "lineage / term", (0..ticks).step_by(50).map(|w| format!("{w:>8}")).collect::<String>());
        for i in 0..4u32 {
            let rows: Vec<(&str, Box<dyn Fn(&str) -> bool>)> = vec![
                ("produce (soil->tissue)", Box::new(move |k: &str| k.starts_with(&format!("ecology:produce-{i} |")) && k.contains(&format!("-> alive L{i} soil")))),
                ("meals eaten", Box::new(move |k: &str| k.starts_with(&format!("ecology:feed-{i} |")) && (k.contains(&format!("-> alive L{i} tissue")) || k.contains(&format!("-> dead L{i} tissue"))))),
                ("upkeep paid to site", Box::new(move |k: &str| k.starts_with(&format!("ecology:upkeep-{i} |")) && k.contains("-> site soil"))),
                ("births (provisioned)", Box::new(move |k: &str| k.starts_with(&format!("ecology:birth-{i} |")))),
                ("eaten by others (alive)", Box::new(move |k: &str| k.starts_with("ecology:feed-") && !k.starts_with(&format!("ecology:feed-{i} |")) && k.contains(&format!("| alive L{i} tissue ->")))),
                ("rest/wound loss", Box::new(move |k: &str| k.starts_with("body:rest") && k.contains(&format!("| alive L{i} tissue ->")))),
            ];
            for (name, pred) in &rows {
                eprint!("{:<30}", format!("L{i} {} {name}", role(i)));
                for w in (0..ticks).step_by(50) {
                    let mt = member_ticks.get(&(w, i)).copied().unwrap_or(0);
                    let amt = parse_amount(&t, w, |k| pred(k));
                    eprint!("{:>8}", if mt == 0 { "-".into() } else { format!("{:.0}", amt as f64 * 1000.0 / mt as f64) });
                }
                eprintln!();
            }
            eprint!("{:<30}", format!("L{i} member-ticks"));
            for w in (0..ticks).step_by(50) {
                eprint!("{:>8}", member_ticks.get(&(w, i)).copied().unwrap_or(0));
            }
            eprintln!();
        }
        // Upkeep participation: bodies born in the first 20 ticks and alive at tick 100.
        eprintln!("-- upkeep payers: of the bodies founded at tick {start} (still the cohort members that exist at the end of the run, dead or alive, with a lifetime >= 60 ticks)");
        let st = s.sim.state();
        let mut by: BTreeMap<u32, (u64, u64, u64)> = BTreeMap::new();
        for g in st.population.groups.values() {
            let e = &g.entity;
            let Some(i) = index(&e.lineage) else { continue };
            if e.born > start + 20 {
                continue;
            }
            let id = *st.population.groups.iter().find(|(_, gg)| std::ptr::eq(&gg.entity, &g.entity)).unwrap().0;
            let n = paid.get(&id).copied().unwrap_or(0);
            let row = by.entry(i).or_default();
            row.0 += 1;
            row.1 += u64::from(n > 0);
            row.2 += u64::from(n);
        }
        for (i, (bodies, payers, pays)) in by {
            eprintln!("L{i} ({}): founders-or-early-born {bodies}, ever paid upkeep {payers}, total payments {pays}", role(i));
        }
    }
}

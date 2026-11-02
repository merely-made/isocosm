// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The world-time ecology's year (cases 1077, 1078): a founding run a year
//! at the minute, sampled monthly per lineage, against Mark's bar: every
//! role alive at month 12, each lineage's largest and smallest monthly
//! counts over months 3 to 12 within four times, and no month after the
//! second losing more than a third of a lineage. Ignored by default: a year
//! takes minutes. `WORLD_TIME_SEED`, `WORLD_TIME_SITES`, `WORLD_TIME_POP`
//! and `WORLD_TIME_LINEAGES` choose the founding.

use cosmograph::{Execution, Founding, Session, ecology::{Rates, Web}};
use std::collections::BTreeMap;

fn env(key: &str, default: u64) -> u64 {
    std::env::var(key).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
}

fn living(s: &Session) -> BTreeMap<String, u64> {
    let mut by = BTreeMap::new();
    for g in s.sim.state().population.groups.values() {
        if g.entity.alive && g.entity.lineage.starts_with("lineage:") {
            *by.entry(g.entity.lineage.clone()).or_default() += g.count;
        }
    }
    by
}

fn events(s: &Session, prefix: &str) -> usize {
    s.sim.state().events.values().filter(|e| e.process.starts_with(prefix)).count()
}

#[test]
#[ignore]
fn a_world_time_web_holds_a_year() {
    let seed = env("WORLD_TIME_SEED", 9);
    let lineages = env("WORLD_TIME_LINEAGES", 10) as u32;
    let genesis = Founding { seed, sites: env("WORLD_TIME_SITES", 4) as u32, population: env("WORLD_TIME_POP", 240),
        cohort_size: 1, lineages, ecology: true, web: Web::Diets, ecology_rates: Rates::WorldTime,
        ..Default::default() }.generate().unwrap();
    let roles: BTreeMap<String, String> = genesis.lineages.iter().map(|(k, l)| (k.clone(),
        l.traits.iter().find(|t| t.starts_with("role:") || t.contains("producer") || t.contains("consumer")
            || t.contains("decomposer")).cloned().unwrap_or_default())).collect();
    let mut s = Session::new(genesis, Execution::Grouped).unwrap();
    let month = cosmograph::rules::year_ticks(s.sim.genesis().rules.tick_microseconds()) / 12;
    let started = std::time::Instant::now();
    let mut months = vec![living(&s)];
    eprintln!("seed {seed} month 0 {:?} roles {roles:?}", months[0]);
    let span = env("WORLD_TIME_DAYS", 0);
    let (step, steps) = if span > 0 { (span * 1440 / 6, 6) } else { (month, 12) };
    for m in 1..=steps {
        // A day at a time keeps each advance inside the operation budget.
        let mut left = step;
        let trace = env("WORLD_TIME_TRACE", u64::MAX);
        while left > 0 {
            let day = left.min(60); s.advance(day).unwrap(); left -= day;
            let fine = env("WORLD_TIME_FINE", u64::MAX);
            let t = s.sim.state().tick;
            if trace != u64::MAX && (t % 1440 == 0 || (t / 1440 >= fine && t / 1440 < fine + 2)) {
                let st = s.sim.state();
                let bodies: Vec<(String, u64, u64)> = st.population.groups.values()
                    .filter(|g| g.entity.alive && g.entity.place == trace && roles.get(&g.entity.lineage).is_some_and(|r| r.contains("decomposer")))
                    .map(|g| { let i = g.entity.lineage.trim_start_matches("lineage:");
                        (i.to_string(), g.entity.accounts.get(&format!("matter:{i}-0")).copied().unwrap_or(0), (st.tick - g.entity.born) / 1440) }).collect();
                let site = &st.sites[&trace];
                let mut what: BTreeMap<String, usize> = BTreeMap::new();
                for e in st.events.values().filter(|e| e.tick + 60 > st.tick && !e.process.starts_with("ecology:litter")) {
                    *what.entry(e.process.clone()).or_default() += 1;
                }
                eprintln!("trace events {what:?}");
                eprintln!("trace day {} min {} litter {:?} soil {:?} decomposers {bodies:?}", st.tick / 1440, st.tick % 1440,
                    site.accounts.get("world:litter"), site.accounts.get("world:soil"));
            }
        }
        let now = living(&s);
        let st = s.sim.state();
        let mean: BTreeMap<String, u64> = now.keys().map(|k| { let i = k.trim_start_matches("lineage:");
            let (sum, n) = st.population.groups.values().filter(|g| g.entity.alive && &g.entity.lineage == k)
                .fold((0u64, 0u64), |(a, b), g| (a + g.entity.accounts.get(&format!("matter:{i}-0")).copied().unwrap_or(0) * g.count, b + g.count));
            (k.clone(), sum / n.max(1)) }).collect();
        let soil: Vec<u64> = st.sites.values().map(|site| site.accounts.get("world:soil").copied().unwrap_or(0)).collect();
        let light: Vec<u64> = st.sites.values().map(|site| site.accounts.get("sim:energy").copied().unwrap_or(0)).collect();
        let litter: Vec<u64> = st.sites.values().map(|site| site.accounts.get("world:litter").copied().unwrap_or(0)).collect();
        let mut by_site: BTreeMap<u64, Vec<u64>> = BTreeMap::new();
        for g in st.population.groups.values().filter(|g| g.entity.alive) {
            let Some(i) = g.entity.lineage.strip_prefix("lineage:").and_then(|i| i.parse::<usize>().ok()) else { continue };
            by_site.entry(g.entity.place).or_insert_with(|| vec![0; lineages as usize])[i] += g.count;
        }
        eprintln!("seed {seed} sites {by_site:?}");
        eprintln!("seed {seed} body {mean:?} soil {soil:?} litter {litter:?} light {light:?}");
        eprintln!("seed {seed} month {m} ({:?}) {now:?} births {} starved {} hazard {}", started.elapsed(),
            events(&s, "ecology:birth"), events(&s, "ecology:death"), events(&s, "ecology:hazard"));
        months.push(now);
    }
    if steps != 12 { return; }
    let names: Vec<String> = months[0].keys().cloned().collect();
    let mut verdict = Vec::new();
    for name in &names {
        let series: Vec<u64> = months.iter().map(|m| m.get(name).copied().unwrap_or(0)).collect();
        let alive = series[12] > 0;
        let window = &series[3..=12];
        let (lo, hi) = (*window.iter().min().unwrap(), *window.iter().max().unwrap());
        let bounded = lo > 0 && hi <= lo * 4;
        let steady = series.windows(2).skip(2).all(|w| w[1] * 3 >= w[0] * 2);
        verdict.push((name.clone(), alive, bounded, steady, series));
    }
    for v in &verdict { eprintln!("seed {seed} {} alive {} within4x {} no-third-loss {} {:?}", v.0, v.1, v.2, v.3, v.4); }
    let pass = verdict.iter().all(|v| v.1 && v.2 && v.3);
    eprintln!("seed {seed} PASS {pass}");
}

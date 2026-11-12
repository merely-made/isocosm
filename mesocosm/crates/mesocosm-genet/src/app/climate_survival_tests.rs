// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Measurement, not a gate (ruling D26): the climate's strain scale against
//! founding and the played line's survival, through the real startup path
//! (the 60-tick witness and the native admission) with the product defaults
//! (the crowding brake and `Limits::default()`) and the climate on. Prints one
//! `CLIMATE` row per scale and seed: whether it founded and, if not, what
//! refused it; the four lineages' living counts at ticks 100, 300 and 600;
//! each line's peak and minimum after tick 100 (a run stops once the played
//! line is dead, and says at which tick, so the other lines' figures are then partial); and the matter each strain
//! term charged each lineage, per individual and tick, before tick 150.
//! Ignored; run explicitly, release:
//!
//! `cargo test --release -p mesocosm-genet --lib climate_survival -- --ignored --nocapture`
//!
//! Knobs: `CLIMATE_SCALES` (`off;1;1/10;1/50`, split by `;`: `off` is no climate,
//! `n/d` multiplies the three strain rates of the first cut, so `1/30` is the default), `CLIMATE_SEEDS` (`0-39` or `3,5,9`),
//! `CLIMATE_TICKS` (600), `CLIMATE_THREADS` (8).

use super::*;
use cosmograph::{climate, rules::Climate};

/// The full sunlight credit the ecology stages (`ecology::LIGHT`, not public).
const LIGHT: u64 = 500;
use mesocosm_runtime::world_rules::{WorldRules, with_climate};
use std::collections::BTreeMap;

const MARKS: [u64; 3] = [100, 300, 600];
const DIAG_TICKS: u64 = 150;

/// A scale of the three strain rates over the first cut's (a tenth of a matter
/// a degree, a twentieth a point of thirst, a 5,000th a point of shade), which
/// is what scale 1 means whatever the default now is; the default is 1/30.
fn scaled([n, d]: [u32; 2]) -> Climate {
    let by = |[a, b]: [u32; 2]| [a * n, b * d];
    Climate { strain: by([1, 10]), drought: by([1, 20]), dim: by([1, 5_000]), ..Climate::default() }
}

fn seeds() -> Vec<u64> {
    let text = std::env::var("CLIMATE_SEEDS").unwrap_or_else(|_| "0-39".into());
    match text.split_once('-') {
        Some((a, b)) => (a.parse().unwrap()..=b.parse().unwrap()).collect(),
        None => text.split(',').filter_map(|v| v.trim().parse().ok()).collect(),
    }
}

fn counts(runtime: &Runtime) -> [u64; 4] {
    let mut row = [0; 4];
    for group in runtime.sim().state().population.groups.values().filter(|g| g.entity.alive) {
        if let Some(i) = group.entity.lineage.strip_prefix("lineage:").and_then(|i| i.parse::<usize>().ok()).filter(|i| *i < 4) {
            row[i] += group.count;
        }
    }
    row
}

/// Matter each strain term charges each lineage this tick, `[heat, water, light]`.
fn charged(runtime: &Runtime, into: &mut BTreeMap<usize, ([f64; 3], u64)>) {
    let sim = runtime.sim();
    let rules = &sim.genesis().rules;
    let Some(c) = rules.climate.as_ref() else { return };
    let tick = sim.state().tick;
    for group in sim.state().population.groups.values().filter(|g| g.entity.alive) {
        let e = &group.entity;
        let Some(i) = e.lineage.strip_prefix("lineage:").and_then(|i| i.parse::<usize>().ok()) else { continue };
        let Some(site) = sim.state().sites.get(&e.place) else { continue };
        let [want, shade] = climate::needs(rules, e, &sim.state().lineages).unwrap_or([0, 0]);
        let heat = match (climate::comfort(rules, e, &sim.state().lineages),
            climate::shifted(rules, sim.genesis().seed, e.place, site, tick, sim.state().turn)) {
            (Some(band), Some(degrees)) => climate::strain(band, degrees),
            _ => 0,
        };
        let water = site.conditions.get("world:water").copied().unwrap_or(100);
        let dry = u64::try_from(want.saturating_sub(water)).unwrap_or(0);
        let sky = cosmograph::light::at(rules, tick, Some(site));
        let lit = (LIGHT as f64 * sky.sun.map_or(0.0, |s| f64::from(s.intensity))).round() as i64;
        let dark = if shade > 0 { u64::try_from(shade - lit).unwrap_or(0) } else { 0 };
        let rate = |points: u64, [n, d]: [u32; 2]| points as f64 * f64::from(n) / f64::from(d) * group.count as f64;
        let row = into.entry(i).or_default();
        row.0[0] += rate(heat, c.strain);
        row.0[1] += rate(dry, c.drought);
        row.0[2] += rate(dark, c.dim);
        row.1 += group.count;
    }
}

fn measure(scale: &str, seed: u64, ticks: u64, authored: &mesocosm_runtime::Authored) -> String {
    let climate_on = scale != "off";
    let rules = WorldRules { climate: climate_on, ..WorldRules::generated() };
    let config = HostConfig { seed, world_rules: rules, ..Default::default() };
    let found = match scale.split_once('/') {
        _ if !climate_on => startup::configured(&config, authored),
        Some((n, d)) => with_climate(scaled([n.parse().unwrap(), d.parse().unwrap()]), || startup::configured(&config, authored)),
        None => with_climate(scaled([scale.parse().unwrap(), 1]), || startup::configured(&config, authored)),
    };
    let mut runtime = match found {
        Ok(runtime) => runtime,
        Err(why) => {
            let why: String = why.chars().take(160).collect();
            return format!("CLIMATE scale={scale} seed={seed} founded=no why={why:?}");
        },
    };
    let founded_at = runtime.sim().state().tick;
    let (mut at, mut peak, mut low, mut step) = (vec![], [0u64; 4], [u64::MAX; 4], 0);
    let (mut diag, mut gone, mut stalled) = (BTreeMap::new(), None, 0u32);
    let mut before = runtime.sim().state().tick;
    let mut last = counts(&runtime);
    let mut marks = MARKS.iter().peekable();
    while runtime.sim().state().tick < ticks {
        crate::played::demo_step(&mut runtime, step);
        step += 1;
        let tick = runtime.sim().state().tick;
        // A loop that takes no tick for many steps is waiting on a question the demo cannot answer.
        stalled = if tick == before { stalled + 1 } else { 0 };
        before = tick;
        if stalled > 300 {
            return format!("CLIMATE scale={scale} seed={seed} founded=yes at={founded_at} stalled={tick} counts={last:?}");
        }
        if tick <= DIAG_TICKS { charged(&runtime, &mut diag); }
        last = counts(&runtime);
        if tick > 100 { for k in 0..4 { peak[k] = peak[k].max(last[k]); low[k] = low[k].min(last[k]); } }
        while marks.peek().is_some_and(|m| **m <= tick) { at.push((*marks.next().unwrap(), last)); }
        // A played line that has died stays dead: stop, and let the later marks read as it is.
        if last[1] == 0 {
            gone = Some(tick);
            low[1] = 0;
            for m in marks.by_ref() { at.push((*m, last)); }
            break;
        }
    }
    let alive = last.iter().all(|n| *n > 0);
    let diag: Vec<String> = diag.iter().map(|(i, (t, n))| {
        let per = |x: f64| x / (*n).max(1) as f64;
        format!("{i}:{:.4}/{:.4}/{:.4}", per(t[0]), per(t[1]), per(t[2]))
    }).collect();
    format!("CLIMATE scale={scale} seed={seed} founded=yes at={founded_at} counts={at:?} played_alive={} gone={gone:?} all_alive={alive} peak={peak:?} min={low:?} strain_per_individual_tick(heat/water/light)={diag:?}",
        last[1] > 0)
}

#[test]
#[ignore = "a measurement: founds and runs worlds to tick 600"]
fn climate_survival_over_scales_and_seeds() {
    let ticks: u64 = std::env::var("CLIMATE_TICKS").ok().and_then(|v| v.parse().ok()).unwrap_or(600);
    let threads: usize = std::env::var("CLIMATE_THREADS").ok().and_then(|v| v.parse().ok()).unwrap_or(8);
    let scales = std::env::var("CLIMATE_SCALES").unwrap_or_else(|_| "off;1;1/10;1/50".into());
    let jobs: Vec<(String, u64)> = scales.split(';').flat_map(|s| seeds().into_iter().map(move |seed| (s.to_string(), seed))).collect();
    let authored = mesocosm_runtime::Authored::load(&pack_root()).unwrap();
    let next = std::sync::atomic::AtomicUsize::new(0);
    std::thread::scope(|scope| for _ in 0..threads { scope.spawn(|| loop {
        let Some((scale, seed)) = jobs.get(next.fetch_add(1, std::sync::atomic::Ordering::Relaxed)) else { break };
        let row = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| measure(scale, *seed, ticks, &authored)))
            .unwrap_or_else(|_| format!("CLIMATE scale={scale} seed={seed} founded=panic"));
        println!("{row}");
    }); });
}

/// The measurement's scale of 1/30 is the climate a world gets, so a change to
/// the default's rates is a change to what was measured.
#[test]
fn the_measurements_thirtieth_is_the_default_climate() {
    assert_eq!(scaled([1, 30]), Climate::default());
}

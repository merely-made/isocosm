// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use cosmograph::{
    Execution, Founding, Session,
    band::Banding,
    bodied::Bodies,
    ecology::{Limits, Rates, Rot, Web},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    time::Instant,
};

#[test]
#[ignore = "headless ecology measurement"]
fn measure_cohort_ecology() {
    let number = |name, fallback| {
        std::env::var(name)
            .ok()
            .map_or(fallback, |value| value.parse::<u64>().unwrap())
    };
    let seed = number("COSMOGRAPH_MEASURE_SEED", 9);
    let population = number("COSMOGRAPH_MEASURE_POPULATION", 256);
    let ticks = number("COSMOGRAPH_MEASURE_TICKS", 120);
    let counted = number("COSMOGRAPH_MEASURE_COUNTED", 1) != 0;
    let genesis = Founding {
        seed,
        sites: 4,
        lineages: 5,
        population,
        cohort_size: 4,
        ecology: true,
        ecology_rates: Rates::A1,
        web: Web::Diets,
        limits: Some(Limits::default()),
        rot: Some(Rot::default()),
        bodies: Some(Bodies::default()),
        inherit: true,
        inherited_variation: 60,
        population_bands: counted.then_some(Banding { age_band: 1 }),
        cohort_births: counted,
        ..Founding::default()
    }
    .generate()
    .unwrap();
    let mut session = Session::new(genesis, Execution::Individuals).unwrap();
    let matter = session.sim.matter();
    let start = Instant::now();
    let work = session.advance_traced(ticks, |_| {}).unwrap();
    let elapsed = start.elapsed();
    let state = session.sim.state();
    let mut live = 0;
    let mut dead = 0;
    let mut empty_dead = 0;
    let mut forms = BTreeSet::new();
    let mut lineages = BTreeMap::new();
    for row in state.population.groups.values() {
        let entity = &row.entity;
        if entity.alive && entity.body.is_some() {
            live += row.count;
            *lineages.entry(&entity.lineage).or_insert(0u64) += row.count;
            forms.insert((&entity.lineage, &entity.soma, &entity.absent));
        } else if !entity.alive {
            dead += row.count;
            if entity.accounts.values().all(|&n| n == 0)
                && entity
                    .parts
                    .values()
                    .all(|p| p.matter.values().all(|&n| n == 0))
                && row.pool.is_empty()
            {
                empty_dead += row.count;
            }
        }
    }
    assert_eq!(session.sim.matter(), matter);
    println!(
        "seed={seed} counted={counted} founders={population} ticks={ticks} elapsed_ms={} \
         evaluations={} accepted={} live={live} dead={dead} empty_dead={empty_dead} forms={} \
         rows={} events={} notes={} relations={} witness={:016x}",
        elapsed.as_millis(),
        work.evaluations,
        work.accepted,
        forms.len(),
        state.population.groups.len(),
        state.events.len(),
        state.notes.len(),
        state.relations.len(),
        session.sim.state_hash(),
    );
    println!("live_by_lineage={lineages:?}");
    let mut profile = cosmograph::profile_totals();
    profile.sort_by_key(|(_, values)| std::cmp::Reverse(values[0]));
    for (label, [ns, calls, evaluations]) in profile.into_iter().take(15) {
        println!(
            "{label} ms={} calls={calls} evaluations={evaluations}",
            ns / 1_000_000
        );
    }
}

// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use cosmograph::{
    Execution, Founding, Session,
    coarse::{Region, RegionalModel},
    ecology::{Rates, Web},
};
use std::collections::{BTreeMap, BTreeSet};
use std::time::Instant;

fn mass(sites: &BTreeMap<u64, Region>) -> u128 {
    sites
        .values()
        .flat_map(|s| {
            s.biomass
                .values()
                .chain(s.carcasses.values())
                .chain(s.surfaces.values())
                .chain(s.resources.values())
        })
        .sum()
}

fn generated(seed: u64) -> Founding {
    Founding {
        seed,
        sites: 3,
        population: 45,
        lineages: 5,
        cohort_size: 3,
        ecology: true,
        web: Web::Diets,
        ecology_rates: Rates::A1,
        bodies: Some(cosmograph::bodied::Bodies {
            canopy_growth: true,
            fixing_budget: 16,
            ..Default::default()
        }),
        inherit: true,
        guard: Some(Default::default()),
        categories: Some(Default::default()),
        nutrients: Some(Default::default()),
        mutualism: Some(Default::default()),
        detritus: Some(Default::default()),
        repertoire: Some(cosmograph::repertoire::Repertoire {
            founders: true,
            generative_ecology: true,
            across_kingdoms: true,
            kingdom_draw: Some(cosmograph::generate::KingdomDraw {
                pool: [
                    "kingdom:flora",
                    "kingdom:fauna",
                    "kingdom:myco",
                    "kingdom:micro",
                ]
                .into_iter()
                .map(str::to_owned)
                .collect(),
                kingdoms: [4, 4],
                per_kingdom: [1, 2],
            }),
            ..Default::default()
        }),
        ..Default::default()
    }
}

fn count(sites: &BTreeMap<u64, Region>) -> (u64, usize) {
    let members = sites.values().flat_map(|s| s.living.values()).sum();
    let lineages = sites
        .values()
        .flat_map(Region::extant_lineages)
        .collect::<BTreeSet<_>>()
        .len();
    (members, lineages)
}

#[test]
fn regional_step_uses_live_traffic_and_keeps_the_playable_world() {
    for seed in [41, 42, 43] {
        check(&generated(seed));
    }
}

fn check(f: &Founding) {
    let world = Session::new(f.generate().unwrap(), Execution::Grouped).unwrap();
    let hash = world.sim.state_hash();
    let sample = 12;
    let span = 96;
    let t = Instant::now();
    let model = RegionalModel::observe(&world, sample).unwrap();
    let observed = t.elapsed();
    let projection = model.project(span).unwrap();
    assert_eq!(model.start(), &Region::read(&world.sim).unwrap());
    assert_eq!(mass(&projection.sites), world.sim.matter());
    assert!(model.start().values().any(|s| !s.surfaces.is_empty()));
    assert!(
        projection
            .sites
            .values()
            .all(|r| r.resources.keys().all(|k| k != "sim:energy"))
    );
    assert!(
        projection
            .sites
            .iter()
            .any(|(site, r)| r.living.keys().any(|line| projection
                .strongest_flow(*site, line, true)
                .is_some()
                || projection.strongest_flow(*site, line, false).is_some()))
    );
    assert_eq!(world.sim.state_hash(), hash);
    assert_eq!(projection.end_tick, world.sim.state().tick + span);
    assert_ne!(model.project(sample).unwrap().sites, projection.sites);
    assert_eq!(model.project(span).unwrap().sites, projection.sites);
    assert!(model.project(sample - 1).is_err());

    if f.seed == 41 {
        let mut fine = world.clone();
        let mut bulk = world.clone();
        for _ in 0..sample {
            fine.advance(1).unwrap();
        }
        bulk.advance(sample).unwrap();
        let single = Region::read(&fine.sim).unwrap();
        let batched = Region::read(&bulk.sim).unwrap();
        let sampled = model.project(sample).unwrap();
        println!(
            "sample cadence: {} one-tick calls {:?}, recorded {:?}, bulk {:?}",
            sample,
            count(&single),
            count(&sampled.sites),
            count(&batched)
        );
    }

    let mut exact = world.clone();
    let mut precise = std::time::Duration::ZERO;
    for horizon in [sample, 24, 36, 48, span] {
        let t = Instant::now();
        while exact.sim.state().tick < world.sim.state().tick + horizon {
            let _ = exact.advance_tick_with_flows().unwrap();
        }
        precise += t.elapsed();
        let fine = Region::read(&exact.sim).unwrap();
        let t = Instant::now();
        let future = model.project(horizon).unwrap();
        let projected = t.elapsed();
        if horizon == sample {
            assert_eq!(future.sites, fine);
        }
        assert_eq!(mass(&future.sites), mass(&fine));
        let (expected, lines) = count(&future.sites);
        let (actual, known) = count(&fine);
        println!(
            "seed {}, {horizon} ticks: observation {observed:?}, coarse steps {projected:?}, fine {precise:?}; members {expected}/{actual}, lineages {lines}/{known} (estimate/fine)",
            f.seed
        );
    }
}

// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Case 1094's coarse copy, as grouped execution and bands make it today,
//! beside the exact one: whether it picks the same candidates, and the time.

use super::*;
use crate::{Execution, Founding, band::Banding, directing::{Played, policy}, rules::PolicyVariation, ecology::{Rates, Web}};
use std::time::{Duration, Instant};

fn played(seed: u64) -> Session {
    let mut g = Founding {
        seed,
        sites: 8,
        population: 200,
        cohort_size: 4,
        lineages: 5,
        ecology: true,
        ecology_rates: Rates::A1,
        web: Web::Diets,
        played: Some(Played { lineage: 1, region_sites: 4 }),
        ..Default::default()
    }
    .generate()
    .unwrap();
    g.rules.directing.as_mut().unwrap().policy = PolicyVariation { founding: 16, expression: 16, candidates: 3 };
    let mut s = Session::new(g, Execution::Individuals).unwrap();
    let found = policy::founding(&s, "lineage:1");
    s.command(found).unwrap();
    s
}

const BANDS: Banding = Banding { age_band: 20 };

/// The copy made coarse by what exists today: grouped execution, which
/// since case 1076 chooses as individuals do, and E4's bands folding what
/// no one attends. Measured slower than the exact copy, so it stays here.
fn coarsen(copy: &mut Session, banding: Banding) {
    copy.sim.set_mode(Execution::Grouped);
    std::sync::Arc::make_mut(&mut copy.sim.genesis).rules.banding = Some(banding);
}

/// The played line's policy candidates every 30 ticks to 150, scored
/// exactly and coarsely over several horizons: how often the two pick the
/// same, and the time. Unbodied lines offer no revisions, so the policy's
/// are the candidates. `cargo test coarse_readout -- --ignored --nocapture`.
#[test]
#[ignore = "a readout, not a control"]
fn coarse_readout() {
    let lineage = "lineage:1";
    for horizon in [20, 180, 600] {
        let (mut agree, mut turns) = (0, 0);
        let (mut exact_t, mut coarse_t) = (Duration::ZERO, Duration::ZERO);
        for seed in [9, 3, 7] {
            let mut s = played(seed);
            for _ in 0..5 {
                s.advance(30).unwrap();
                if standing(&s, lineage).1 == 0 {
                    break;
                }
                let offered = policy::candidates(&s, lineage);
                let stay = Candidate { name: "candidate:stay".into(), commands: vec![] };
                let all: Vec<&Candidate> = std::iter::once(&stay).chain(&offered).collect();
                let pick = |score: &mut dyn FnMut(&Candidate) -> Score| {
                    let mut best: Option<(Score, Key)> = None;
                    for c in &all {
                        let sc = score(c);
                        if best.as_ref().is_none_or(|(b, _)| sc.beats(b)) {
                            best = Some((sc, c.name.clone()));
                        }
                    }
                    best.unwrap().1
                };
                // One replayed source per turn, as Mesocosm's scorer keeps,
                // so the times are the copies' own ticks.
                let source = ScoringSource::new(&s).unwrap();
                let t = Instant::now();
                let exact = pick(&mut |c| source.score_using(lineage, c, horizon,
                    &mut |copy, t| copy.advance(t).map(|_| ())).unwrap());
                exact_t += t.elapsed();
                let t = Instant::now();
                let coarse = pick(&mut |c| source.score_using(lineage, c, horizon, &mut |copy, t| {
                    coarsen(copy, BANDS);
                    copy.advance(t).map(|_| ())
                }).unwrap());
                coarse_t += t.elapsed();
                turns += 1;
                agree += usize::from(exact == coarse);
                if exact != coarse {
                    let tick = s.sim.state().tick;
                    println!("  seed {seed} tick {tick} h{horizon}: exact {exact}, coarse {coarse} of {}", all.len());
                }
            }
        }
        println!("horizon {horizon}: {agree}/{turns} turns agree; exact {exact_t:.2?}, coarse {coarse_t:.2?}");
    }
}

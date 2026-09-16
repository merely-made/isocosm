// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Measurement for the deep-time assessment (2026-09-16), not a receipt.
//! Runs a bare world headless across several epochs and reports, at each
//! boundary, what the ecology came to and how many readings beat a mark that
//! stood before that reckoning.
//!
//! Usage: deep_time_probe [seed] [organisms] [epochs] [bare|foundation|door]
//! `bare` is `World::new`; `foundation` is the generation door's played world,
//! with its drawn soil pattern and the first admitted candidate entered;
//! `door` prepares through the generation door with `epochs` as the request's
//! deep-time span (D7a) and reports the handover.

use mesocosm_core::world::generation::Request;
use mesocosm_core::{DeepTimeSpan, Founding, History, Intent, World};
use std::collections::BTreeSet;
use std::time::Instant;

fn main() {
    let args: Vec<u64> = std::env::args()
        .skip(1)
        .filter_map(|a| a.parse().ok())
        .collect();
    let seed = args.first().copied().unwrap_or(7);
    let organisms = args.get(1).copied().unwrap_or(60) as u32;
    let epochs = args.get(2).copied().unwrap_or(8);
    if std::env::args().any(|a| a == "door") {
        return door(seed, organisms, epochs as u32);
    }
    let foundation = std::env::args().any(|a| a == "foundation");

    let mut world = if foundation {
        Request {
            seed,
            organisms,
            ..Request::default()
        }
        .prepare(Founding::Drawn.palette())
        .expect("a prepared habitat")
        .enter(0)
        .expect("the first admitted candidate")
    } else {
        World::new(seed, organisms)
    };
    let mut history = History::new();
    let started = Instant::now();
    let mut lap = Instant::now();
    let kind = if foundation { "foundation" } else { "bare" };
    println!("seed={seed} organisms={organisms} epochs={epochs} world={kind}");
    println!("epoch tick living species readings took beat first filled secs");
    while world.epoch < epochs {
        let before = world.epoch;
        world.apply(Intent::Idle);
        history.record_all(world.drain_events());
        if world.epoch == before {
            continue;
        }
        let standing = world.record().clone();
        let readings = world.reckon(&history);
        let took = readings.iter().filter(|r| r.took).count();
        let beat = readings
            .iter()
            .filter(|r| {
                standing
                    .standing(r.feat, r.scale)
                    .is_some_and(|m| r.value > m.high)
            })
            .count();
        let first = readings
            .iter()
            .filter(|r| standing.standing(r.feat, r.scale).is_none())
            .count();
        let species: BTreeSet<_> = world.living().map(|o| o.species).collect();
        println!(
            "{} {} {} {} {} {} {} {} {} {:.1}",
            world.epoch,
            world.tick,
            world.living().count(),
            species.len(),
            readings.len(),
            took,
            beat,
            first,
            world.record().filled(),
            lap.elapsed().as_secs_f64()
        );
        lap = Instant::now();
    }
    println!(
        "total_secs {:.1} history_entries {}",
        started.elapsed().as_secs_f64(),
        history.len()
    );
}

/// One generation-door preparation with a deep-time span: wall time, the
/// handed-over world and the past it carries.
fn door(seed: u64, organisms: u32, epochs: u32) {
    let started = Instant::now();
    let prepared = Request {
        seed,
        organisms,
        deep_time: DeepTimeSpan { epochs },
        ..Request::default()
    }
    .prepare(Founding::Drawn.palette())
    .expect("a prepared habitat");
    let secs = started.elapsed().as_secs_f64();
    let world = prepared.habitat_world();
    let species: BTreeSet<_> = world.living().map(|o| o.species).collect();
    println!("seed={seed} organisms={organisms} span={epochs} world=door");
    println!(
        "tick {} epoch {} at_boundary {} living {} species {} history_entries {} filled {} candidates {} secs {secs:.1}",
        world.tick,
        world.epoch,
        world.at_boundary(),
        world.living().count(),
        species.len(),
        prepared.history().len(),
        world.record().filled(),
        prepared.draft().candidates.len(),
    );
    println!(
        "lineages {species:?} rejected {:?}",
        prepared.draft().rejected
    );
}

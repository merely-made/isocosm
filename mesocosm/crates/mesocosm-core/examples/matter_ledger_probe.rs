// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Measurement for the deep-time assessment (2026-09-16), not a receipt.
//! Where the enclosure's matter sits at each epoch of deep time: soil in total
//! and near the draft's place, living matter by kingdom, carrion, and how many
//! decomposers are alive, so the soil cycle can be read rather than assumed.
//!
//! Usage: matter_ledger_probe [seed] [organisms] [epochs]
//! Builds the generation door's played world with no span, then runs one epoch
//! of deep time at a time through `World::run_deep_time`.

use mesocosm_core::rules::{DeepTimeSpan, WorldRules};
use mesocosm_core::world::generation::Request;
use mesocosm_core::{Founding, History, Kingdom, Stage, World};
use std::collections::BTreeMap;

fn main() {
    let args: Vec<u64> = std::env::args()
        .skip(1)
        .filter_map(|a| a.parse().ok())
        .collect();
    let seed = args.first().copied().unwrap_or(7);
    let organisms = args.get(1).copied().unwrap_or(24) as u32;
    let epochs = args.get(2).copied().unwrap_or(6);

    let request = Request {
        seed,
        organisms,
        ..Request::default()
    };
    let world = request
        .prepare(Founding::Drawn.palette())
        .expect("a prepared habitat")
        .enter(0)
        .expect("the first admitted candidate");
    let rules = world.rules();
    let mut world = world.with_rules(WorldRules {
        deep_time: DeepTimeSpan { epochs: 1 },
        ..rules
    });
    let mut history = History::new();

    println!(
        "seed={seed} organisms={organisms} epochs={epochs} place={}",
        request.place
    );
    println!("total matter mg: {}", world.total_matter_mg());
    report(&world, request.place);
    for _ in 0..epochs {
        world
            .run_deep_time(&mut history)
            .expect("one epoch of deep time");
        report(&world, request.place);
    }
    println!("total matter mg: {}", world.total_matter_mg());
}

fn report(world: &World, place: u16) {
    let soil = world.soil();
    let near = world
        .places()
        .all()
        .find(|p| p.id.0 == place)
        .map(|p| {
            let column = soil.column_at([p.centre[0], 0, p.centre[1]]);
            soil.columns_within(column, 3)
                .map(|c| soil.stock(c).total())
                .sum::<u128>()
        })
        .unwrap_or(0);

    let mut living: BTreeMap<&str, (u32, u64)> = BTreeMap::new();
    let mut lineages: BTreeMap<u32, u32> = BTreeMap::new();
    let (mut carrion_count, mut carrion_mg) = (0u32, 0u64);
    for o in &world.organisms {
        if o.stage == Stage::Carrion {
            carrion_count += 1;
            carrion_mg += o.biomass_mg();
        } else if o.is_alive() {
            let kingdom = match o.kingdom() {
                Kingdom::Producer => "producer",
                Kingdom::Consumer => "consumer",
                Kingdom::Decomposer => "decomposer",
            };
            let entry = living.entry(kingdom).or_default();
            entry.0 += 1;
            entry.1 += o.biomass_mg() + o.energy_mg;
            *lineages.entry(o.species.0).or_default() += 1;
        }
    }
    println!(
        "epoch {} tick {} soil_mg {} near_place_mg {} living {:?} carrion {} ({} mg) lineages {:?}",
        world.epoch,
        world.tick,
        soil.total_mg(),
        near,
        living,
        carrion_count,
        carrion_mg,
        lineages
    );
}

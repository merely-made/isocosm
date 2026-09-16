// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Measurement for the M1/M2 assessment (soil cycle plan, ruling 8), not a
//! receipt.
//!
//! Runs the readings test's control arm -- `World::new(seed, founders)` idled,
//! `ARM_FOUNDERS = 200`, `ARM_TICKS = 2_000` -- and reports, per window:
//!
//! * producers alive and their mass, consumers alive, decomposers alive;
//! * grazing, predation and scavenging milligrams, and the number of meals;
//! * grazing per consumer per tick, against producer density, so that column
//!   pair is the enclosure's measured functional response;
//! * producers (and producer mass) inside a consumer's sight floor, which is
//!   the density a body actually reads;
//! * consumers per crowding cell, the density term producers answer to and
//!   consumers do not (M2).
//!
//! Densities are read at the end of each window; flows are summed across it.
//!
//! The header restates each founding consumer's per-tick bite ceiling by
//! `rates::feeding_rate_for_body`'s own arithmetic, which is crate-private:
//! `GRAZES_BASE_MG * m^0.75 * (ceiling + span * 100) / (100^0.75 * ceiling)`.
//! The gap between that ceiling and the measured mean bite is the point.
//!
//! Usage: response_probe [seed] [founders] [ticks] [window]

use std::collections::BTreeMap;

use mesocosm_core::{Event, Intent, Kingdom, MealKind, World};

/// `rates::GRAZES_BASE_MG`.
const GRAZES_BASE_MG: u64 = 3;
/// `rates::three_quarter_power(rates::REFERENCE_MASS_MG)`.
const REFERENCE_TQP: u64 = 31;
/// `rates::REFERENCE_SEGMENT_MG`.
const REFERENCE_SEGMENT_MG: u64 = 100;
/// `rates::CROWD_CELL`.
const CROWD_CELL: i32 = 8;
/// `perception::NEAR_SIGHT_RANGE`, the sight floor a blind body reads.
const SIGHT_FLOOR: i32 = 8;

fn integer_sqrt(value: u128) -> u128 {
    let mut low = 0u128;
    let mut high = value.saturating_add(1);
    while high - low > 1 {
        let middle = low + (high - low) / 2;
        if middle <= value / middle.max(1) {
            low = middle;
        } else {
            high = middle;
        }
    }
    low
}

fn three_quarter_power(mass_mg: u64) -> u64 {
    let mass = mass_mg.max(1) as u128;
    integer_sqrt(mass * integer_sqrt(mass)) as u64
}

/// `rates::feeding_rate_for_body`, restated.
fn bite_mg(mass_mg: u64, actuator_span: u32, ceiling_mg: u64) -> u64 {
    let ceiling = ceiling_mg.max(1);
    let priced = ceiling + u64::from(actuator_span) * REFERENCE_SEGMENT_MG;
    (GRAZES_BASE_MG * three_quarter_power(mass_mg) * priced / (REFERENCE_TQP * ceiling)).max(1)
}

fn chebyshev(from: [i32; 3], to: [i32; 3]) -> i32 {
    (0..3)
        .map(|axis| (from[axis] - to[axis]).abs())
        .max()
        .unwrap_or(0)
}

#[derive(Default)]
struct Pane {
    grazing_mg: u64,
    grazing_meals: u64,
    predation_mg: u64,
    scavenging_mg: u64,
    births: u64,
    deaths: u64,
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let nums: Vec<u64> = args.iter().filter_map(|a| a.parse().ok()).collect();
    let seed = nums.first().copied().unwrap_or(7);
    let founders = nums.get(1).copied().unwrap_or(200) as u32;
    let ticks = nums.get(2).copied().unwrap_or(2_000);
    let window = nums.get(3).copied().unwrap_or(100).max(1);

    let mut world = World::new(seed, founders);

    // What each founding consumer could bite, before anything is in its way.
    let mut bites: Vec<u64> = Vec::new();
    let mut upkeeps: Vec<u64> = Vec::new();
    let mut rooms: Vec<u64> = Vec::new();
    for organism in world.living().filter(|o| o.kingdom() == Kingdom::Consumer) {
        bites.push(bite_mg(
            organism.biomass_mg(),
            organism.actuator_span(),
            organism.mass_ceiling_mg(),
        ));
        upkeeps.push(organism.upkeep_mg());
        rooms.push(organism.intake_room_mg());
    }
    bites.sort_unstable();
    upkeeps.sort_unstable();
    rooms.sort_unstable();
    let quote = |sorted: &[u64]| {
        if sorted.is_empty() {
            "n/a".to_string()
        } else {
            format!(
                "{}/{}/{}",
                sorted[0],
                sorted[sorted.len() / 2],
                sorted[sorted.len() - 1]
            )
        }
    };
    println!("seed {seed}, {founders} founders, {ticks} ticks, window {window}");
    println!(
        "founding consumers {}: bite min/median/max {}, upkeep {}, intake room {}",
        bites.len(),
        quote(&bites),
        quote(&upkeeps),
        quote(&rooms)
    );
    println!(
        "tick\tprod\tprod_mg\tcons\tdecomp\tgraze_mg\tmeals\tbite_mean\tgraze_per_cons_tick\t\
         meals_per_cons_tick\tprod_in_8\tprod_mg_in_8\tcons_per_cell\tcell_max\tcells\t\
         prod_per_cell\tprod_cell_max\tcrowd_divisor\tof_it_not_prod\troom_mean\tsated\t\
         fullness\tpred_mg\tscav_mg\tborn\tdied"
    );

    let mut pane = Pane::default();
    for tick in 1..=ticks {
        world.apply(Intent::Idle);
        for event in world.drain_events() {
            match event.record {
                Event::Fed { mass_mg, kind, .. } => match kind {
                    MealKind::Grazing => {
                        pane.grazing_mg += mass_mg;
                        pane.grazing_meals += 1;
                    },
                    MealKind::Predation => pane.predation_mg += mass_mg,
                    MealKind::Scavenging => pane.scavenging_mg += mass_mg,
                },
                Event::Born { .. } => pane.births += 1,
                Event::Died { .. } => pane.deaths += 1,
                _ => {},
            }
        }
        let _ = world.drain_flows();
        if tick % window != 0 {
            continue;
        }

        let producers: Vec<([i32; 3], u64)> = world
            .living()
            .filter(|o| o.kingdom() == Kingdom::Producer)
            .map(|o| (o.position, o.biomass_mg()))
            .collect();
        // Position, intake room, and how full the body itself is: the two
        // halves of the only satiation the tick has (`intake_room_mg`).
        let bodies: Vec<([i32; 3], u64, f64)> = world
            .living()
            .filter(|o| o.kingdom() == Kingdom::Consumer)
            .map(|o| {
                let ceiling = o.mass_ceiling_mg().max(1);
                (
                    o.position,
                    o.intake_room_mg(),
                    o.biomass_mg() as f64 / ceiling as f64,
                )
            })
            .collect();
        let consumers: Vec<[i32; 3]> = bodies.iter().map(|(at, ..)| *at).collect();
        let decomposers = world
            .living()
            .filter(|o| o.kingdom() == Kingdom::Decomposer)
            .count();
        let producer_mg: u64 = producers.iter().map(|(_, mg)| mg).sum();

        // The crowd divisor a producer's income is actually divided by:
        // `ecology.rs:232-236` counts **every** living body in the cell, not
        // only producers, so this is the throttle as the tick applies it, and
        // the second column is the share of it that is not a producer.
        let mut all_cells: BTreeMap<(i32, i32), (u32, u32)> = BTreeMap::new();
        for organism in world.living() {
            let key = (
                organism.position[0].div_euclid(CROWD_CELL),
                organism.position[2].div_euclid(CROWD_CELL),
            );
            let entry = all_cells.entry(key).or_default();
            entry.0 += 1;
            if organism.kingdom() != Kingdom::Producer {
                entry.1 += 1;
            }
        }
        let (mut crowd_total, mut crowd_other) = (0u64, 0u64);
        for (at, _) in &producers {
            let key = (at[0].div_euclid(CROWD_CELL), at[2].div_euclid(CROWD_CELL));
            let (all, other) = all_cells.get(&key).copied().unwrap_or((1, 0));
            crowd_total += u64::from(all.max(1));
            crowd_other += u64::from(other);
        }
        let (crowd_mean, crowd_other_mean) = if producers.is_empty() {
            (0.0, 0.0)
        } else {
            (
                crowd_total as f64 / producers.len() as f64,
                crowd_other as f64 / producers.len() as f64,
            )
        };

        // What a consumer reads: producers, and producer mass, inside the
        // sight floor every body has whether or not it grew a sense organ.
        let (mut near_count, mut near_mg) = (0u64, 0u64);
        for at in &consumers {
            for (position, mg) in &producers {
                if chebyshev(*at, *position) <= SIGHT_FLOOR {
                    near_count += 1;
                    near_mg += mg;
                }
            }
        }
        let per_consumer = |total: u64| {
            if consumers.is_empty() {
                0.0
            } else {
                total as f64 / consumers.len() as f64
            }
        };

        // Consumers per crowding cell: the density term producers answer to
        // and consumers do not.
        let mut cells: BTreeMap<(i32, i32), u32> = BTreeMap::new();
        for at in &consumers {
            *cells
                .entry((at[0].div_euclid(CROWD_CELL), at[2].div_euclid(CROWD_CELL)))
                .or_default() += 1;
        }
        let occupied = cells.len();
        let cell_max = cells.values().copied().max().unwrap_or(0);
        let cell_mean = if occupied == 0 {
            0.0
        } else {
            consumers.len() as f64 / occupied as f64
        };

        // The same read for producers, who alone answer to `CROWD_COMFORT`.
        let mut producer_cells: BTreeMap<(i32, i32), u32> = BTreeMap::new();
        for (at, _) in &producers {
            *producer_cells
                .entry((at[0].div_euclid(CROWD_CELL), at[2].div_euclid(CROWD_CELL)))
                .or_default() += 1;
        }
        let producer_cell_mean = if producer_cells.is_empty() {
            0.0
        } else {
            producers.len() as f64 / producer_cells.len() as f64
        };
        let producer_cell_max = producer_cells.values().copied().max().unwrap_or(0);

        // Satiation as it exists today: room left, and the share of consumers
        // whose room has fallen under one median founding bite.
        let room_mean = if bodies.is_empty() {
            0.0
        } else {
            bodies.iter().map(|(_, room, _)| *room).sum::<u64>() as f64 / bodies.len() as f64
        };
        let median_bite = bites.get(bites.len() / 2).copied().unwrap_or(1);
        let sated = bodies
            .iter()
            .filter(|(_, room, _)| *room < median_bite)
            .count();
        let fullness = if bodies.is_empty() {
            0.0
        } else {
            bodies.iter().map(|(.., share)| share).sum::<f64>() / bodies.len() as f64
        };

        let span = window as f64;
        let graze_rate = if consumers.is_empty() {
            0.0
        } else {
            pane.grazing_mg as f64 / consumers.len() as f64 / span
        };
        let meal_rate = if consumers.is_empty() {
            0.0
        } else {
            pane.grazing_meals as f64 / consumers.len() as f64 / span
        };
        let bite_mean = if pane.grazing_meals == 0 {
            0.0
        } else {
            pane.grazing_mg as f64 / pane.grazing_meals as f64
        };

        println!(
            "{tick}\t{}\t{producer_mg}\t{}\t{decomposers}\t{}\t{}\t{bite_mean:.2}\t\
             {graze_rate:.3}\t{meal_rate:.3}\t{:.2}\t{:.0}\t{cell_mean:.2}\t{cell_max}\t\
             {occupied}\t{producer_cell_mean:.2}\t{producer_cell_max}\t{crowd_mean:.2}\t\
             {crowd_other_mean:.2}\t{room_mean:.0}\t{sated}\t{fullness:.2}\t{}\t{}\t{}\t{}",
            producers.len(),
            consumers.len(),
            pane.grazing_mg,
            pane.grazing_meals,
            per_consumer(near_count),
            per_consumer(near_mg),
            pane.predation_mg,
            pane.scavenging_mg,
            pane.births,
            pane.deaths,
        );
        pane = Pane::default();
    }
}

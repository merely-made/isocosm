// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Measurement for the soil cycle plan's S2 (2026-09-16): deep time measured
//! near against far now that S1 fixed far-tier movement.
//!
//! Isoscape ruling 19, everything runs far during deep time, rested on the
//! premise that a released founder starts away from the player; the
//! isoscape family plan's §2.6 assessment found that premise wrong (every
//! founder starts near), and Mark ruled the near/far question be measured
//! rather than assumed once far movement was fixed (soil cycle plan
//! rulings 2 and 7, step S2).
//!
//! **Far arm**: exactly `World::run_deep_time` per epoch, the ruled
//! behaviour `matter_ledger_probe.rs` reports and this probe's positive
//! control.
//!
//! **Near arm**: the same door world with the played hand released, the way
//! `decomposer_probe.rs`'s `release_control`/`freeze_tiers_far` workaround
//! does (`release_control` and `freeze_tiers_far` are crate-private, so
//! control is released by editing `controlled` out of the postcard snapshot
//! through the public `snapshot`/`restore`), but tiers left exactly where
//! genesis set them, all near, ticked one tick at a time by the mirrored
//! loop `decomposer_probe.rs`'s `deep` mode uses. The technique is copied
//! rather than shared, because examples cannot share code.
//!
//! Before either arm is trusted, the mirrored loop is checked once per seed
//! against the ruled call: released, every living body set far, ticked for
//! one epoch, and asserted to match `run_deep_time`'s `state_hash` and
//! `History` exactly — the same positive control `decomposer_probe.rs`
//! established for its `deep` mode.
//!
//! Usage: `deep_time_arms_probe [seed] [organisms] [epochs] [far|near|both]`
//! With no seed given, runs seeds 7, 1 and 42 in turn, 24 organisms, 6
//! epochs, both arms. A seed given makes `organisms` and `epochs`
//! positional (defaults 24 and 6); `far`, `near` or `both` selects the arm
//! (default `both`).

use std::collections::BTreeMap;
use std::time::Instant;

use mesocosm_core::places::Tier;
use mesocosm_core::rules::{DeepTimeSpan, WorldRules};
use mesocosm_core::world::generation::Request;
use mesocosm_core::{
    Founding, History, Intent, Kingdom, OrganismId, Stage, World, restore, snapshot, state_hash,
};

fn main() {
    let raw: Vec<String> = std::env::args().skip(1).collect();
    let mode = if raw.iter().any(|a| a == "far") {
        "far"
    } else if raw.iter().any(|a| a == "near") {
        "near"
    } else {
        "both"
    };
    let nums: Vec<u64> = raw.iter().filter_map(|a| a.parse().ok()).collect();
    let (seeds, organisms, epochs) = if nums.is_empty() {
        (vec![7, 1, 42], 24u32, 6u64)
    } else {
        let organisms = nums.get(1).copied().unwrap_or(24) as u32;
        let epochs = nums.get(2).copied().unwrap_or(6);
        (vec![nums[0]], organisms, epochs)
    };

    for seed in seeds {
        run_seed(seed, organisms, epochs, mode);
    }
}

/// One seed: the door world entered once, the positive control checked
/// against it, then each requested arm run from a fresh clone of the same
/// entered world so neither arm's ticks can leak into the other's.
fn run_seed(seed: u64, organisms: u32, epochs: u64, mode: &str) {
    println!("\n==== seed {seed} organisms {organisms} epochs {epochs} mode {mode} ====");
    let request = Request {
        seed,
        organisms,
        ..Request::default()
    };
    let entered = request
        .prepare(Founding::Drawn.palette())
        .expect("a prepared habitat")
        .enter(0)
        .expect("the first admitted candidate");
    let rules = entered.rules();
    let entered = entered.with_rules(WorldRules {
        deep_time: DeepTimeSpan { epochs: 1 },
        ..rules
    });

    positive_control(&entered);

    let far = (mode == "far" || mode == "both").then(|| run_arm(&entered, epochs, Arm::Far));
    let near = (mode == "near" || mode == "both").then(|| run_arm(&entered, epochs, Arm::Near));
    if let (Some(near), Some(far)) = (&near, &far) {
        print_summary(seed, epochs, near, far);
    }
}

/// The mirrored loop, checked once per seed against the ruled call it
/// stands in for: released, every living body set far, ticked for one
/// epoch. `decomposer_probe.rs` established this as a faithful
/// reproduction (state hash and history both match, six runs there); this
/// probe re-proves it on each seed it measures before trusting the near
/// arm's own use of the same loop.
fn positive_control(entered: &World) {
    let mut control = entered.clone();
    let mut control_history = History::new();
    control
        .run_deep_time(&mut control_history)
        .expect("one epoch of deep time");

    let mut mirrored = released(entered);
    for o in mirrored.organisms.iter_mut().filter(|o| o.is_alive()) {
        o.tier = Tier::Far;
    }
    let mut mirrored_history = History::new();
    tick_one_epoch(&mut mirrored, &mut mirrored_history);

    let hash_ok = state_hash(&mirrored) == state_hash(&control);
    let history_ok = mirrored_history == control_history;
    println!(
        "positive control: mirrored loop (released, every body far, one epoch) vs \
         run_deep_time: state_hash {} history {}",
        if hash_ok { "MATCH" } else { "DIFFER" },
        if history_ok { "MATCH" } else { "DIFFER" }
    );
    assert!(
        hash_ok,
        "the mirrored loop must reproduce run_deep_time's state hash before either arm is trusted"
    );
    assert!(
        history_ok,
        "the mirrored loop must reproduce run_deep_time's history before either arm is trusted"
    );
}

#[derive(Clone, Copy)]
enum Arm {
    Far,
    Near,
}

/// What an arm came to at its last measured epoch, and the wall time and
/// matter conservation across every epoch it ran.
struct ArmSummary {
    wall_total_s: f64,
    start_matter_mg: u64,
    end_matter_mg: u64,
    last: EpochStats,
}

/// One epoch's reading, for both the per-epoch line and the final table.
struct EpochStats {
    living: u32,
    species: usize,
    decomposers_alive: u32,
    soil_mg: u64,
    carrion_count: u32,
    carrion_mg: u64,
}

/// Runs one arm for `epochs` epochs from a fresh clone of `entered`, far
/// exactly as `run_deep_time` runs it and near by the mirrored loop with
/// tiers left as genesis set them, reporting every epoch as it goes.
fn run_arm(entered: &World, epochs: u64, arm: Arm) -> ArmSummary {
    let label = match arm {
        Arm::Far => "far",
        Arm::Near => "near",
    };
    let mut world = match arm {
        Arm::Far => entered.clone(),
        Arm::Near => {
            let released = released(entered);
            assert!(
                released.living().all(|o| o.tier == Tier::Near),
                "a door world at tick 0 is all near"
            );
            released
        },
    };
    let start_matter_mg = world.total_matter_mg();
    let mut history = History::new();
    report_epoch(&world, label, 0, 0.0, None, None);

    let mut last = None;
    let mut wall_total_s = 0.0f64;
    for epoch in 1..=epochs {
        let standing = world.record().clone();
        let clock = Instant::now();
        match arm {
            Arm::Far => {
                world
                    .run_deep_time(&mut history)
                    .expect("one epoch of deep time");
            },
            Arm::Near => tick_one_epoch(&mut world, &mut history),
        }
        let wall = clock.elapsed().as_secs_f64();
        wall_total_s += wall;

        // `reckon` is idempotent when nothing changed since the last call
        // (hagiograph's `Mark::note` is a join: a repeat of the same value
        // neither raises the mark nor changes its holders), so calling it
        // again here only reads back this epoch's readings. Far already
        // reckoned once inside `run_deep_time`; near reckoned once inside
        // `tick_one_epoch`. Either way this call changes nothing and lets
        // both arms report `beat`/`first` the same way `deep_time_probe.rs`
        // does, against the mark standing before this epoch's reckoning.
        let readings = world.reckon(&history);
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
        last = Some(report_epoch(
            &world,
            label,
            epoch,
            wall,
            Some(beat),
            Some(first),
        ));
    }
    let end_matter_mg = world.total_matter_mg();
    ArmSummary {
        wall_total_s,
        start_matter_mg,
        end_matter_mg,
        last: last.expect("at least one epoch ran"),
    }
}

/// Ticks one world one epoch at a time, the way `Unheld::advance` in
/// `deep_time.rs` does for the ruled call: idle, record, drain flows, and
/// reckon on the tick that closes the epoch. No release and no tier freeze
/// here — the caller decides both before this is reached.
fn tick_one_epoch(world: &mut World, history: &mut History) {
    let start_epoch = world.epoch;
    let mut seen = world.epoch;
    while world.epoch == start_epoch {
        world.apply(Intent::Idle);
        history.record_all(world.drain_events());
        world.drain_flows();
        if world.epoch != seen {
            seen = world.epoch;
            world.reckon(history);
        }
    }
}

/// Prints one epoch's line for one arm and returns the figures the final
/// summary table reads back.
fn report_epoch(
    world: &World,
    label: &str,
    epoch: u64,
    wall_s: f64,
    beat: Option<usize>,
    first: Option<usize>,
) -> EpochStats {
    let mut living: BTreeMap<&str, (u32, u64)> = BTreeMap::new();
    let mut lineages: BTreeMap<u32, u32> = BTreeMap::new();
    let mut near = 0u32;
    let mut far = 0u32;
    let (mut carrion_count, mut carrion_mg) = (0u32, 0u64);
    for o in &world.organisms {
        if o.stage == Stage::Carrion {
            carrion_count += 1;
            carrion_mg += o.biomass_mg();
            continue;
        }
        if !o.is_alive() {
            continue;
        }
        let kingdom = match o.kingdom() {
            Kingdom::Producer => "producer",
            Kingdom::Consumer => "consumer",
            Kingdom::Decomposer => "decomposer",
        };
        let entry = living.entry(kingdom).or_default();
        entry.0 += 1;
        entry.1 += o.biomass_mg() + o.energy_mg;
        *lineages.entry(o.species.0).or_default() += 1;
        match o.tier {
            Tier::Near => near += 1,
            Tier::Far => far += 1,
        }
    }
    let soil_mg = world.soil().total_mg();
    let decomposers_alive = living.get("decomposer").map_or(0, |(n, _)| *n);
    let living_total: u32 = living.values().map(|(n, _)| *n).sum();
    println!(
        "[{label}] epoch {epoch} tick {} soil_mg {soil_mg} living {living:?} carrion {carrion_count} \
         ({carrion_mg} mg) lineages {lineages:?} near {near} far {far} beat {beat:?} first {first:?} \
         wall_s {wall_s:.3}",
        world.tick,
    );
    EpochStats {
        living: living_total,
        species: lineages.len(),
        decomposers_alive,
        soil_mg,
        carrion_count,
        carrion_mg,
    }
}

fn print_summary(seed: u64, epochs: u64, near: &ArmSummary, far: &ArmSummary) {
    println!("\n-- seed {seed}: near vs far at epoch {epochs} --");
    println!("{:<20} {:>14} {:>14}", "metric", "near", "far");
    println!(
        "{:<20} {:>14} {:>14}",
        "living", near.last.living, far.last.living
    );
    println!(
        "{:<20} {:>14} {:>14}",
        "species", near.last.species, far.last.species
    );
    println!(
        "{:<20} {:>14} {:>14}",
        "decomposers_alive", near.last.decomposers_alive, far.last.decomposers_alive
    );
    println!(
        "{:<20} {:>14} {:>14}",
        "soil_mg", near.last.soil_mg, far.last.soil_mg
    );
    println!(
        "{:<20} {:>14} {:>14}",
        "carrion_count", near.last.carrion_count, far.last.carrion_count
    );
    println!(
        "{:<20} {:>14} {:>14}",
        "carrion_mg", near.last.carrion_mg, far.last.carrion_mg
    );
    println!(
        "{:<20} {:>14.3} {:>14.3}",
        "wall_s_total", near.wall_total_s, far.wall_total_s
    );
    println!(
        "matter conservation: near start {} end {} ({}), far start {} end {} ({})",
        near.start_matter_mg,
        near.end_matter_mg,
        if near.start_matter_mg == near.end_matter_mg {
            "OK"
        } else {
            "MISMATCH"
        },
        far.start_matter_mg,
        far.end_matter_mg,
        if far.start_matter_mg == far.end_matter_mg {
            "OK"
        } else {
            "MISMATCH"
        },
    );
}

/// The world with `controlled` cleared, through its postcard snapshot.
/// Copied from `decomposer_probe.rs`'s `released`, verbatim technique,
/// because examples cannot share code: `release_control` is crate-private,
/// so control is released by editing it out of the public snapshot bytes.
///
/// Postcard writes fields in declaration order with no names: `tick`,
/// `epoch`, `rules`, then `rng` (one `u64` varint), then `controlled`
/// (`world.rs:153-198`; `ruleset` is skipped). The prefix is re-encoded from
/// public reads, so the field is located rather than searched for, and
/// every step is checked.
fn released(world: &World) -> World {
    let bytes = snapshot(world).expect("a world encodes");
    let same = restore(&bytes).expect("and decodes");
    assert_eq!(
        state_hash(&same),
        state_hash(world),
        "the round trip is exact"
    );
    let prefix = postcard::to_allocvec(&(world.tick, world.epoch, world.rules())).unwrap();
    assert!(
        bytes.starts_with(&prefix),
        "tick, epoch and rules lead the snapshot"
    );
    let rng_len = 1 + bytes[prefix.len()..]
        .iter()
        .position(|b| b & 0x80 == 0)
        .expect("the rng varint ends");
    let at = prefix.len() + rng_len;
    let id = world.controlled_id().expect("a world opens under the hand");
    let some = postcard::to_allocvec(&Some(id)).unwrap();
    assert_eq!(
        &bytes[at..at + some.len()],
        &some[..],
        "controlled follows rng"
    );
    let mut edited = bytes[..at].to_vec();
    edited.extend(postcard::to_allocvec(&None::<OrganismId>).unwrap());
    edited.extend(&bytes[at + some.len()..]);
    let released = restore(&edited).expect("the released world decodes");
    assert_eq!(released.controlled_id(), None);
    assert_eq!(released.organisms, world.organisms, "no body moved");
    assert_eq!(released.soil(), world.soil(), "no soil moved");
    assert_eq!(snapshot(&released).unwrap(), edited, "and nothing else did");
    released
}

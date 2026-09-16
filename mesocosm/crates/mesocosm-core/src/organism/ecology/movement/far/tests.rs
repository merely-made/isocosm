// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! What far-tier travel has to keep true. (soil cycle plan S1)

use super::*;
use crate::body::{SpeciesId, VolumeRef};
use crate::development::PartPalette;
use crate::flow::Process;
use crate::history::Event;
use crate::organism::ecology::tests::{Sink, organism, registry, soil};
use crate::organism::ecology::{dispersal_for, is_hungry, step_with_places};
use crate::organism::{Kingdom, Organism, OrganismId};
use crate::places::{PlaceId, Soil, Tier};
use crate::process::BodyProcesses;
use crate::{Intent, World};

const WALKER: OrganismId = OrganismId(0);
const PREY: OrganismId = OrganismId(9);

/// Places wide enough that a place is several ticks across.
fn enclosure() -> Places {
    Places::scatter(&mut Rng::from_seed(55), 3, 64)
}

fn centre(places: &Places, id: u16) -> [i32; 3] {
    let [x, z] = places.get(PlaceId(id)).unwrap().centre;
    [x, 0, z]
}

/// A far predator: a long, thin plan grows a limb, so it walks (TD8).
fn walker(at: [i32; 3], energy_mg: u64) -> Organism {
    let mut body = Organism::founding(
        WALKER,
        SpeciesId(2),
        Kingdom::Consumer,
        VolumeRef::from_tag(16),
        [8, 1, 1],
        at,
        300,
    );
    body.tier = Tier::Far;
    body.energy_mg = energy_mg;
    body
}

/// Far prey that stays put: an unlimbed consumer is sessile (TD8).
fn prey(at: [i32; 3]) -> Organism {
    let mut body = Organism::founding(
        PREY,
        SpeciesId(3),
        Kingdom::Consumer,
        VolumeRef::from_tag(18),
        [1, 1, 1],
        at,
        300,
    );
    body.tier = Tier::Far;
    body
}

/// What one graph-only tick did to one body.
struct Moved {
    from: [i32; 3],
    to: [i32; 3],
    paid_mg: u64,
}

/// One tick with no focus, so every tier stays where the fixture set it.
fn tick(world: &mut Vec<Organism>, places: &Places, ground: &mut Soil, who: OrganismId) -> Moved {
    let from = world.iter().find(|o| o.id == who).unwrap().position;
    let mut sink = Sink::default();
    let lines = registry(world);
    step_with_places(
        world,
        &mut 100,
        &mut Rng::from_seed(3),
        &mut sink.stream(),
        &lines,
        PartPalette::primitive(),
        ground,
        places,
        None,
    );
    let paid_mg = sink
        .flows
        .records()
        .iter()
        .filter(|f| f.record.process == Process::Travel)
        .filter(|f| f.record.from.is_some_and(|s| s.organism == who))
        .map(|f| f.record.amount_mg)
        .sum();
    let to = world.iter().find(|o| o.id == who).unwrap().position;
    Moved { from, to, paid_mg }
}

fn alive(world: &[Organism], id: OrganismId) -> bool {
    world.iter().any(|o| o.id == id && o.is_alive())
}

#[test]
fn a_far_body_in_its_targets_place_closes_on_it_and_stays_there() {
    // The bounce §2.6 measured: every neighbour is one hop from the target's
    // place, so the old hop always left it. Walker and prey start at opposite
    // edges of the middle place.
    let places = enclosure();
    let home = Some(PlaceId(4));
    let edge = |dx: i32, dz: i32| {
        let mut at = centre(&places, 4);
        while places.at([at[0] + dx, 0, at[2] + dz]) == home {
            at = [at[0] + dx, 0, at[2] + dz];
        }
        at
    };
    let target = edge(1, 0);
    let mut world = vec![walker(edge(-1, -1), 5_000), prey(target)];
    let range = world[0].body().reach() + crate::organism::ecology::movement::GRAZE_RANGE;
    let budget = dispersal_for(&world[0]) as i32;
    let start = chebyshev(world[0].position, target);
    assert!(
        start > range + budget,
        "the fixture starts out of reach: {start}"
    );
    let mut ground = soil();

    for _ in 0..40 {
        if !alive(&world, PREY) {
            break;
        }
        let moved = tick(&mut world, &places, &mut ground, WALKER);
        assert_eq!(places.at(moved.to), home, "left the target's place");
        assert!(chebyshev(moved.to, target) <= chebyshev(moved.from, target));
        assert!(chebyshev(moved.from, moved.to) <= budget);
    }
    let end = world.iter().find(|o| o.id == WALKER).unwrap().position;
    assert!(chebyshev(end, target) <= range, "and it arrived");
}

#[test]
fn a_step_that_would_leave_the_targets_place_slides_along_it() {
    // A diagonal-first path can clip a neighbouring place even between two
    // points of one. Found by search, and proven to clip before it is trusted.
    let places = enclosure();
    let home = places.at(centre(&places, 4));
    let naive_leaves = |from: [i32; 3], to: [i32; 3]| {
        let mut at = from;
        while at != to {
            at = [
                at[0] + (to[0] - at[0]).signum(),
                0,
                at[2] + (to[2] - at[2]).signum(),
            ];
            if places.at(at) != home {
                return true;
            }
        }
        false
    };
    let inside: Vec<[i32; 3]> = (-64..=64)
        .step_by(3)
        .flat_map(|x| (-64..=64).step_by(3).map(move |z| [x, 0, z]))
        .filter(|at| places.at(*at) == home)
        .collect();
    let (from, to) = inside
        .iter()
        .flat_map(|from| inside.iter().map(move |to| (*from, *to)))
        .find(|(from, to)| naive_leaves(*from, *to))
        .expect("the middle place has a pair a diagonal-first path clips");

    let mut at = from;
    for _ in 0..100 {
        let next = walk(&places, None, WalkerShape::STANDARD, at, to, 3, None);
        assert_eq!(places.at(next), home, "stepped out at {next:?}");
        assert!(chebyshev(next, to) <= chebyshev(at, to));
        if next == at {
            break;
        }
        at = next;
    }
    assert!(
        chebyshev(at, to) < chebyshev(from, to),
        "and it still closed"
    );
}

#[test]
fn no_far_body_moves_more_than_its_dispersal_budget() {
    let places = enclosure();
    let (start, across) = (centre(&places, 0), centre(&places, 8));
    let mut ground = soil();

    // With a target across the enclosure a walker spends exactly its budget,
    // sated or hungry, and pays exactly what it moved.
    for (energy_mg, hungry) in [(5_000, false), (0, true)] {
        let mut world = vec![walker(start, energy_mg), prey(across)];
        assert_eq!(is_hungry(&world[0]), hungry);
        let budget = dispersal_for(&world[0]);
        let moved = tick(&mut world, &places, &mut ground, WALKER);
        assert_eq!(chebyshev(moved.from, moved.to), budget as i32);
        assert_eq!(moved.paid_mg, u64::from(budget));
    }

    // Hungry with nothing to reach for anywhere: the wander, one voxel.
    let mut world = vec![walker(start, 0)];
    assert!(dispersal_for(&world[0]) > 1);
    let moved = tick(&mut world, &places, &mut ground, WALKER);
    assert_eq!(chebyshev(moved.from, moved.to), 1);
    assert_eq!(moved.paid_mg, 1);

    // A far producer creeps nowhere, however hungry.
    let mut plant = organism(Kingdom::Producer, 300);
    (plant.tier, plant.energy_mg, plant.position) = (Tier::Far, 0, start);
    let mut world = vec![plant];
    assert_eq!(dispersal_for(&world[0]), 0);
    assert!(is_hungry(&world[0]));
    let moved = tick(&mut world, &places, &mut ground, OrganismId(0));
    assert_eq!((moved.to, moved.paid_mg), (moved.from, 0));
}

#[test]
fn a_far_body_reaches_another_place_over_several_ticks() {
    let places = enclosure();
    let (start, target) = (centre(&places, 0), centre(&places, 8));
    let mut world = vec![walker(start, 5_000), prey(target)];
    let budget = dispersal_for(&world[0]) as i32;
    let mut ground = soil();

    let mut ticks = 0;
    let mut distance = chebyshev(start, target);
    while places.at(world[0].position) != places.at(target) {
        let moved = tick(&mut world, &places, &mut ground, WALKER);
        let now = chebyshev(moved.to, target);
        assert!(now < distance, "tick {ticks}: {distance} -> {now}");
        assert!(chebyshev(moved.from, moved.to) <= budget);
        assert_eq!(moved.paid_mg, chebyshev(moved.from, moved.to) as u64);
        distance = now;
        ticks += 1;
        assert!(ticks < 200, "never arrived");
    }
    assert!(ticks > 1, "a place away is several ticks, not one hop");
}

/// Every far move on generated ground, for a released world and a held one:
/// within the hungry budget, paid voxel for voxel, onto a legal stance; and
/// the far cohorts conserve every tick.
fn run_far(seed: u64, released: bool, ticks: u32) -> (World, u32) {
    let mut world = World::new(seed, 24);
    if released {
        world.release_control();
        world.freeze_tiers_far();
    }
    let mut far_moves = 0;
    for _ in 0..ticks {
        world.apply(Intent::Idle);
        let flows = world.drain_flows();
        for event in world.drain_events() {
            let Event::Moved { organism, from, to } = event.record else {
                continue;
            };
            let Some(body) = world.organisms.iter().find(|o| o.id == organism) else {
                continue;
            };
            if body.tier != Tier::Far {
                continue;
            }
            far_moves += 1;
            let voxels = chebyshev(from, to);
            let most = match body.actuator_span() {
                0 => 0,
                span => (span / 4).max(1) as i32 + 1,
            };
            assert!(voxels <= most, "{organism:?} moved {voxels}, budget {most}");
            let paid: u64 = flows
                .iter()
                .filter(|f| f.record.process == Process::Travel)
                .filter(|f| f.record.from.is_some_and(|s| s.organism == organism))
                .map(|f| f.record.amount_mg)
                .sum();
            assert_eq!(paid, voxels as u64, "{organism:?} paid for other voxels");
            assert!(body.walker_shape().stands(world.ground(), to));
        }
        let far = world
            .organisms
            .iter()
            .filter(|o| o.is_alive() && o.tier == Tier::Far)
            .fold((0, 0, 0), |(n, mass, energy), o| {
                (n + 1, mass + o.biomass_mg(), energy + o.energy_mg)
            });
        assert_eq!(crate::cohort::conserved_totals(&world.far_cohorts()), far);
    }
    (world, far_moves)
}

#[test]
fn far_travel_on_generated_ground_is_bounded_paid_conserved_and_replays() {
    for released in [true, false] {
        let (a, moves) = run_far(4_242, released, 120);
        assert!(moves > 0, "released {released}: no far body moved");
        let (b, _) = run_far(4_242, released, 120);
        assert_eq!(
            crate::snapshot::state_hash(&a),
            crate::snapshot::state_hash(&b)
        );
    }
}

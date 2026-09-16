// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The far tier runs near's rules with cheaper geometry (ruled by Mark
//! 2026-09-16, soil cycle plan ruling 7): a near and a far body in the same
//! situation choose the same target and take the same heading.
//!
//! The ground is flat, so a near body's grounded step and a far body's column
//! step land on the same voxel and a heading can be compared as a position.

use super::super::perception::sight_range;
use super::super::*;
use super::{predator, strangers, target};
use crate::body::{Attachment, Provenance, VolumeRef, Yaw};
use crate::development::PartPalette;
use crate::history::{Event, MealKind};
use crate::organism::ecology::step_with_ground;
use crate::organism::ecology::tests::{Sink, organism, soil};
use crate::organism::{Kingdom, OrganismId, Signal};
use crate::places::Tier;
use crate::species::Lineages;
use isometer_core::ground::Terrain;

/// Ground whose every column tops out at y 3.
struct Flat;

impl Terrain for Flat {
    fn sea_level(&self, _extent: i32) -> i32 {
        0
    }

    fn surface(&self, _extent: i32, _x: i32, _z: i32) -> i32 {
        3
    }
}

/// The stance height on [`Flat`].
const FLOOR: i32 = 4;

fn flat() -> Ground {
    Ground::grow(&Flat, 48)
}

fn places() -> Places {
    Places::scatter(&mut Rng::from_seed(55), 3, 48)
}

/// `body` once per tier, near first.
fn both(body: &Organism) -> [Organism; 2] {
    [Tier::Near, Tier::Far].map(|tier| {
        let mut body = body.clone();
        body.tier = tier;
        body
    })
}

/// A predator at `at` with the palette's sensor on its root, so a warning is
/// sensed and the policy has something to avoid.
fn sensing(at: [i32; 3]) -> Organism {
    let mut body = predator(2, 300);
    let root = body.body().root;
    body.phenotype
        .attach(
            VolumeRef::from_tag(16),
            1,
            [1, 1, 1],
            Attachment {
                parent: root,
                offset: [0, 0, 4],
                yaw: Yaw::Zero,
            },
            Provenance::founding(),
        )
        .expect("a sensor attaches to the root");
    body.position = at;
    body
}

#[test]
fn a_near_and_a_far_body_choose_by_one_policy() {
    // Before ruling 7 a far body ranked by distance alone and could only ever
    // pursue. Every situation here is read by both tiers, and each leaves the
    // same choice, the same decision trace and the same recurrent state.
    let lineages = strangers();
    let kin = Kin::new(&lineages);
    let ground = flat();
    let body = sensing([0, FLOOR, 0]);
    let sight = sight_range(&body, 0, Some(&ground));
    let no_carrion: Vec<CarrionTarget> = Vec::new();
    let at = |x: i32| [x, FLOOR, 0];
    let mut warning = target(1, 9, at(2), 300);
    warning.signal = Signal::Warning;

    let mut drives = Vec::new();
    for (living, expected) in [
        (vec![target(1, 9, at(3), 300)], Some("pursue")),
        (vec![warning], Some("avoid")),
        (
            vec![target(1, 2, at(1), 900), target(2, 9, at(3), 300)],
            None,
        ),
        (vec![target(1, 2, at(2), 300)], Some("pursue")),
        (vec![target(1, 9, at(sight + 1), 300)], Some("nothing")),
    ] {
        let cells = living_cells(&living);
        let [near, far] = both(&body).map(|mut body| {
            let chosen = preferred_target(
                &mut body,
                &living,
                &cells,
                &no_carrion,
                &carrion_cells(&no_carrion),
                Some(&ground),
                &kin,
            );
            (chosen, body.last_fauna_decision, body.fauna_policy)
        });
        assert_eq!(near, far, "{living:?}");
        let drive = match near.0 {
            Some(MovementTarget::Seen(..)) => "pursue",
            Some(MovementTarget::Avoid(..)) => "avoid",
            Some(MovementTarget::Hold(..)) => "hold",
            Some(MovementTarget::Other(_)) => "carrion",
            None => "nothing",
        };
        if let Some(expected) = expected {
            assert_eq!(drive, expected, "{living:?}");
        }
        drives.push(drive);
    }
    assert!(drives.contains(&"avoid"), "the far body ran the policy");
}

#[test]
fn a_near_and_a_far_body_keep_one_sighting_across_the_tier_line() {
    let ground = flat();
    let mut body = predator(2, 300);
    body.position = [0, FLOOR, 0];
    let quarry = OrganismId(900);
    let living = vec![target(900, 7, [4, FLOOR, 0], 300)];
    body.last_seen = Some(LastSeen {
        target: quarry,
        position: [4, FLOOR, 0],
        ticks_left: 3,
    });

    let [near, far] = both(&body).map(|mut body| {
        let heading = remembered_target(&mut body, &living, Some(&ground));
        (heading, body.last_seen)
    });
    assert_eq!(near, far);
    assert_eq!(near.0, Some([4, FLOOR, 0]));

    // Near, then demoted, then promoted: the memory runs out on its own clock.
    for (tier, left) in [
        (Tier::Near, Some(2)),
        (Tier::Far, Some(1)),
        (Tier::Near, Some(0)),
    ] {
        body.tier = tier;
        assert_eq!(
            remembered_target(&mut body, &living, Some(&ground)),
            Some([4, FLOOR, 0]),
            "{tier:?}"
        );
        assert_eq!(body.last_seen.map(|memory| memory.ticks_left), left);
    }
    body.tier = Tier::Far;
    assert_eq!(remembered_target(&mut body, &living, Some(&ground)), None);
}

/// One `disperse` for `body` in each tier, from the same random stream.
fn disperse_both(
    body: &Organism,
    living: &[LivingTarget],
    lineages: &Lineages,
) -> [([i32; 3], u64); 2] {
    let (ground, places, kin) = (flat(), places(), Kin::new(lineages));
    let no_carrion: Vec<CarrionTarget> = Vec::new();
    both(body).map(|mut body| {
        let mut rng = Rng::from_seed(11);
        let mut sink = Sink::default();
        disperse(
            &mut body,
            &places,
            Some(&ground),
            &mut rng,
            &mut soil(),
            living,
            &living_cells(living),
            &no_carrion,
            &carrion_cells(&no_carrion),
            &mut sink.stream(),
            &kin,
        );
        (body.position, rng.below(u64::MAX))
    })
}

#[test]
fn a_hungry_near_and_far_body_step_up_the_same_gradient() {
    // Nothing in sight; a sibling in a nearer bucket, a stranger in a farther
    // one, both inside the gradient's horizon. Before ruling 7 the far body
    // took a random voxel toward a neighbouring place instead.
    let lineages = strangers();
    let hunter = hungry();
    assert!(is_hungry(&hunter));
    let sight = sight_range(&hunter, 0, Some(&flat()));
    let horizon = GRAZE_RANGE + hunter.body().reach();
    let far_off = sight + 3;
    assert!(horizon >= far_off, "horizon {horizon}, sight {sight}");
    let living = vec![
        target(1, 2, [sight + 2, FLOOR, 0], 300),
        target(2, 9, [-far_off, FLOOR, 0], 300),
    ];
    let heading = forage_gradient(
        &hunter,
        &living,
        &living_cells(&living),
        &[],
        &carrion_cells(&[]),
        &Kin::new(&lineages),
    )
    .expect("the stranger's bucket");
    assert!(heading[0] < 0, "toward the stranger: {heading:?}");

    let [near, far] = disperse_both(&hunter, &living, &lineages);
    assert_eq!(near, far, "the same voxel, and no wander drawn");
    assert_eq!(chebyshev(near.0, hunter.position), 1);
    assert!(chebyshev(near.0, heading) < chebyshev(hunter.position, heading));
}

/// The fixture predator at the origin with no reserve left.
fn hungry() -> Organism {
    let mut body = predator(2, 300);
    (body.position, body.energy_mg) = ([0, FLOOR, 0], 0);
    body
}

#[test]
fn with_no_gradient_a_near_and_a_far_body_draw_the_same_wander() {
    // Only its own line within the horizon: no gradient, so the random wander,
    // the same draw from the same stream, one voxel on the same axis.
    let lineages = strangers();
    let hunter = hungry();
    let sight = sight_range(&hunter, 0, Some(&flat()));
    let living = vec![target(1, 2, [sight + 2, FLOOR, 0], 300)];
    let [near, far] = disperse_both(&hunter, &living, &lineages);
    assert_eq!(near, far);
    assert_eq!(chebyshev(near.0, hunter.position), 1);

    // A hungry stand creeps by the same wander in either tier.
    let mut plant = organism(Kingdom::Producer, 300);
    (plant.position, plant.energy_mg) = ([0, FLOOR, 0], 0);
    let [near, far] = disperse_both(&plant, &[], &lineages);
    assert_eq!(near, far);
    assert_eq!(chebyshev(near.0, plant.position), 1);
}

/// A sessile consumer of `species`: unlimbed, so it never walks (TD8).
fn sessile(id: u32, species: u32, at: [i32; 3]) -> Organism {
    let mut body = Organism::founding(
        OrganismId(id),
        crate::body::SpeciesId(species),
        Kingdom::Consumer,
        VolumeRef::from_tag(18),
        [1, 1, 1],
        at,
        300,
    );
    body.energy_mg = 5_000;
    body
}

/// A hungry predator amid its own line, a stranger beyond sight, run for
/// `ticks` in one tier. Returns each tick's position and who it ate.
fn amid_kin(tier: Tier, ticks: u32) -> (Vec<[i32; 3]>, Vec<(OrganismId, MealKind)>) {
    let (ground, places, lineages) = (flat(), places(), strangers());
    let hunter = hungry();
    let sight = sight_range(&hunter, 0, Some(&ground));
    let ring = sight + 2;
    let mut world = vec![hunter];
    for (id, [x, z]) in [
        [ring, 0],
        [ring, ring],
        [ring, -ring],
        [0, ring + 4],
        [0, -ring - 4],
    ]
    .into_iter()
    .enumerate()
    {
        world.push(sessile(id as u32 + 1, 2, [x, FLOOR, z]));
    }
    world.push(sessile(9, 9, [-(sight + 6), FLOOR, 0]));
    for body in &mut world {
        body.tier = tier;
    }
    let mut soil = soil();
    let (mut rng, mut next) = (Rng::from_seed(5), 100);
    let (mut path, mut meals) = (Vec::new(), Vec::new());
    for _ in 0..ticks {
        let mut sink = Sink::default();
        step_with_ground(
            &mut world,
            &mut next,
            &mut rng,
            &mut sink.stream(),
            &lineages,
            PartPalette::primitive(),
            &mut soil,
            &places,
            &ground,
            None,
            None,
            0,
        );
        path.push(world[0].position);
        meals.extend(sink.events().into_iter().filter_map(|event| match event {
            Event::Fed {
                eater, from, kind, ..
            } if eater == OrganismId(0) => Some((from, kind)),
            _ => None,
        }));
    }
    (path, meals)
}

#[test]
fn a_far_consumer_amid_its_own_line_follows_the_gradient_to_a_stranger() {
    // The ruling's case: offspring stand near parents, and a far body with
    // nothing in sight used to wander among them until kin were all it could
    // see, and ate them. Now it heads up the gradient, which scores its own line
    // at zero, to the stranger beyond sight, and eats that.
    let (path, meals) = amid_kin(Tier::Far, 40);
    assert!(path[0][0] < 0, "the first step heads for the stranger");
    assert!(
        meals.iter().all(|(from, _)| *from == OrganismId(9)),
        "never its own line: {meals:?}"
    );
    assert!(
        meals.contains(&(OrganismId(9), MealKind::Predation)),
        "and it reached the stranger: {path:?}"
    );

    // A near body in the same enclosure heads the same way and eats the same.
    let (near_path, near_meals) = amid_kin(Tier::Near, 40);
    assert_eq!(near_path[0], path[0]);
    assert!(near_meals.iter().all(|(from, _)| *from == OrganismId(9)));
    assert!(!near_meals.is_empty());

    // And a replay is the same run.
    assert_eq!(amid_kin(Tier::Far, 40), (path, meals));
}

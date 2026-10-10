// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! P4 and PD5's commit, refused by name: outside the lineage checkpoint, for
//! a condition the line has not come to, and for a revision this world could
//! never express. Births under a committed program are `lineage.rs`'s.

use isocosm::legacy::mesocosm::discovery::HUNGER_TICKS;
use isocosm::legacy::mesocosm::{
    Founding, Intent, Outcome, Rejection, SpeciesId, Stage, Unrevised, World,
};

use super::bulk_world;
use super::discovery::{endure, hunger};
use super::gland::frond_on;
use super::lineage::at_the_checkpoint;

#[test]
fn the_commit_is_gated_to_the_lineage_checkpoint_and_says_so_in_one_place() {
    // **The placeholder replaced** (PE3). Bodies change between epochs and not
    // during them, so the commit consults `revision_admitted_now`, that reads
    // `World::at_boundary`, and every other tick is refused `NotYet`. The
    // played door and the unplayed one are the same transaction, so they are
    // gated together and neither can be a second way for a program to move.
    let mut world = bulk_world(9_001, 24);
    frond_on(&mut world);
    endure(&mut world, HUNGER_TICKS + 1);
    assert!(world.discovered(hunger()));
    assert!(
        !world.revision_admitted_now(),
        "the default budget is nowhere near spent"
    );
    assert_eq!(
        world.apply(Intent::Revise {
            condition: hunger()
        }),
        Outcome::Rejected(Rejection::Unrevised(Unrevised::NotYet)),
        "and mid-epoch it is refused by name"
    );

    let mut world = at_the_checkpoint(world);
    assert!(world.revision_admitted_now(), "inside, it is admitted");
    assert!(matches!(
        world.apply(Intent::Revise {
            condition: hunger()
        }),
        Outcome::Revised { .. }
    ));
}

#[test]
fn revising_a_condition_the_line_has_not_come_to_is_refused_by_name() {
    let mut world = at_the_checkpoint(bulk_world(4_242, 24));
    assert_eq!(
        world.apply(Intent::Revise {
            condition: hunger()
        }),
        Outcome::Rejected(Rejection::Unrevised(Unrevised::Undiscovered(hunger())))
    );
    assert_eq!(
        world.revise(SpeciesId(9_999), hunger()),
        Err(Unrevised::NoSuchSpecies(SpeciesId(9_999))),
        "and a line this world never heard of is its own answer"
    );
}

#[test]
fn a_revision_this_world_could_never_express_is_refused_at_the_commit() {
    // The gland removed from the admitted set. The condition table is native,
    // so the line still comes to a candidate citing it — and every descendant's
    // development would then refuse `UnknownProcess` forever. So the commit
    // refuses once instead: a program that can never be expressed is not a
    // program, and this is the honest place to say so.
    let mut defs: Vec<_> = isocosm::process::Registry::native()
        .all()
        .cloned()
        .collect();
    defs.retain(|def| def.id.name != "secrete");
    let without =
        std::sync::Arc::new(isocosm::process::Registry::admit(defs).expect("no collision"));
    let mut world =
        World::founded_on(4_242, 24, Founding::default(), without).expect("the palette is valid");

    let me = world.controlled_id().expect("embodied");
    let organism = world.organisms.iter_mut().find(|o| o.id == me).unwrap();
    let (species, position) = (organism.species, organism.position);
    *organism = isocosm::legacy::mesocosm::Organism {
        stage: Stage::Mature,
        ..isocosm::legacy::mesocosm::Organism::founding(
            me,
            species,
            isocosm::legacy::mesocosm::Kingdom::Consumer,
            isocosm::legacy::mesocosm::VolumeRef::from_tag(1),
            [2, 2, 2],
            position,
            1_500,
        )
    };
    endure(&mut world, HUNGER_TICKS + 1);
    assert!(world.discovered(hunger()), "the line came to it anyway");

    let mut world = at_the_checkpoint(world);
    assert_eq!(
        world.apply(Intent::Revise {
            condition: hunger()
        }),
        Outcome::Rejected(Rejection::Unrevised(Unrevised::Nothing))
    );
    assert!(
        world
            .lineages()
            .get(species)
            .expect("the line")
            .program()
            .is_empty(),
        "and nothing was committed"
    );
}

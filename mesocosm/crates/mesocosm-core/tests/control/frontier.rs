// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! P1's frontier: a player may step down a lineage but not across, may always
//! return to a line it has lived, and the frontier only rises.

use super::*;

#[test]
fn you_may_step_down_but_not_across() {
    // The complexity frontier, finally binding at the point control moves.
    // It was ruled long ago and lived in `epoch::can_switch_to`, which nothing
    // outside its own tests ever called, so control could take anything alive
    // however elaborate. That module was deleted on 2026-09-04; this is where
    // the rule binds now.
    let world = World::new(4_242, 40);
    let frontier = world.frontier();

    let simpler = world
        .organisms
        .iter()
        .find(|o| {
            o.is_alive() && world.intricacy(o) < frontier && Some(o.id) != world.controlled_id()
        })
        .map(|o| o.id)
        .expect("something in the world is simpler than the player");

    assert!(
        world.is_eligible(simpler),
        "stepping down into a simpler niche is the point"
    );

    // Something more elaborate than anything earned is refused, and says why.
    let mut world = World::new(4_242, 40);
    let grand = OrganismId(9_500);
    world.organisms.push(mesocosm_core::Organism {
        stage: mesocosm_core::Stage::Mature,
        ..mesocosm_core::Organism::founding(
            grand,
            mesocosm_core::SpeciesId(99),
            mesocosm_core::Kingdom::Consumer,
            mesocosm_core::VolumeRef::from_tag(3),
            [4, 4, 4],
            world.position().unwrap(),
            50_000,
        )
    });

    assert!(
        matches!(
            world.eligibility(grand),
            Err(mesocosm_core::Ineligible::AboveTheFrontier { .. })
        ),
        "an unearned peer is refused"
    );
    assert!(matches!(
        world.apply(Intent::TakeControl { organism: grand }),
        Outcome::Rejected(Rejection::Ineligible(
            mesocosm_core::Ineligible::AboveTheFrontier { .. }
        ))
    ));
    let _ = simpler;
}

#[test]
fn a_line_you_have_lived_is_always_yours_to_return_to() {
    // The frontier gates reaching outward, not going home. Otherwise growing a
    // body would lock you out of the line you grew it in.
    let mut world = World::new(4_242, 40);
    let mine = world.controlled().unwrap().species;

    for _ in 0..40 {
        world.apply(Intent::Idle);
    }

    let kin = world
        .organisms
        .iter()
        .find(|o| o.species == mine && o.is_alive() && Some(o.id) != world.controlled_id())
        .map(|o| o.id);

    if let Some(kin) = kin {
        assert!(
            world.is_eligible(kin),
            "your own kind is never above your frontier"
        );
    }
    assert!(world.unlocked().any(|s| s == mine));
}

#[test]
fn the_frontier_only_goes_up() {
    // What you reach, you keep. An earlier cut read the frontier from living
    // organisms, so a lineage dying out collapsed it to zero and left the
    // world permanently uninhabitable, which contradicts disembodiment being a
    // seam rather than a dead end.
    let mut world = World::new(4_242, 40);
    world.apply(Intent::Idle);
    let earned = world.frontier();
    assert!(earned > 0);

    let me = world.controlled_id().unwrap();
    world.organisms.retain(|o| o.id != me);
    for _ in 0..20 {
        world.apply(Intent::Idle);
    }

    assert!(!world.is_embodied(), "the body is gone");
    assert_eq!(
        world.frontier(),
        earned,
        "and the standing it earned is not"
    );
}

#[test]
fn growing_raises_the_frontier() {
    let mut world = World::new(4_242, 40);
    let before = world.frontier();

    // Since 2026-08-03 the frontier reads the *recipe's* intricacy rather than
    // the body's part count, so bulk does not lift it. Learning does: teach
    // the line a word it did not have and the ceiling rises.
    let mine = world.controlled().unwrap().species;
    {
        let species = world.lineages_mut().get_mut(mine).unwrap();
        assert!(species.recipe.acquire(mesocosm_core::Appendage::Vane));
    }
    world.apply(Intent::Idle);

    assert!(
        world.frontier() > before,
        "the ceiling rose with what the line learned"
    );
}

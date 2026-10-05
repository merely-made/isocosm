// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! P1's lineages: speciation as the player's act, founders crossing alone,
//! named lines, kinship, and what a line learns and a fork inherits.

use super::*;

#[test]
fn speciation_is_an_act_and_the_name_is_the_doing() {
    // Lineages could never split before this: reproduction copied the parent's
    // species verbatim and nothing else ever assigned one, so no new species
    // was ever born in a Mesocosm world.
    let mut world = World::new(4_242, 40);
    let me = world.controlled_id().unwrap();
    let before = world.controlled().unwrap().species;

    let outcome = world.apply(Intent::Speciate {
        name: "the pale kind".into(),
    });

    let Outcome::Speciated {
        species,
        from,
        founder,
    } = outcome
    else {
        panic!("expected a split, got {outcome:?}")
    };
    assert_eq!((from, founder), (before, me));
    assert_ne!(species, before, "a new line, not a rename");

    let forked = world.lineages().get(species).unwrap();
    assert_eq!(forked.name.as_deref(), Some("the pale kind"));
    assert_eq!(
        forked.parent,
        Some(before),
        "and it remembers what it came from"
    );
}

#[test]
fn a_founder_crosses_alone() {
    // Forking takes the creature you are holding and nothing else, which makes
    // it a commitment rather than a free rename. A new line begins with one
    // individual, which is how a founder effect actually works.
    let mut world = World::new(4_242, 40);
    let me = world.controlled_id().unwrap();
    let old = world.controlled().unwrap().species;
    let kin: Vec<OrganismId> = world
        .organisms
        .iter()
        .filter(|o| o.species == old && o.id != me)
        .map(|o| o.id)
        .collect();

    let Outcome::Speciated { species, .. } = world.apply(Intent::Speciate {
        name: "alone".into(),
    }) else {
        panic!("expected a split")
    };

    assert_eq!(
        world.controlled().unwrap().species,
        species,
        "the founder crossed"
    );
    for other in kin {
        let still = world.organisms.iter().find(|o| o.id == other);
        if let Some(still) = still {
            assert_eq!(still.species, old, "its former kin kept the old line");
        }
    }
}

#[test]
fn a_world_begins_with_unnamed_lineages_and_gains_named_ones() {
    // Naming promotes a line out of being a variation, which is the same rule
    // that promotes a critter out of being a statistic.
    let mut world = World::new(4_242, 40);
    assert_eq!(
        world.lineages().named().count(),
        0,
        "nobody was there to name them"
    );
    assert!(world.lineages().len() > 1, "but the world has lineages");

    world.apply(Intent::Speciate {
        name: "named".into(),
    });
    assert_eq!(world.lineages().named().count(), 1);
}

#[test]
fn kinship_becomes_computable() {
    // The axis graft compatibility was ruled to scale with. Before lineages
    // could split, every pair of creatures was either identical or unrelated
    // and the measure said nothing.
    let mut world = World::new(4_242, 40);
    let me = world.controlled_id().unwrap();

    world.apply(Intent::Speciate {
        name: "first".into(),
    });
    let stranger = world
        .organisms
        .iter()
        .find(|o| o.id != me && o.is_alive())
        .map(|o| o.id)
        .expect("the world is populated");

    assert_eq!(
        world.kinship(me, me),
        Some(0),
        "a creature is no distance from itself"
    );
    // A founding lineage and a line forked off a different founder share no
    // ancestor, which is a real answer rather than a large number.
    let apart = world.kinship(me, stranger);
    assert!(apart.is_none() || apart == Some(1), "got {apart:?}");
}

#[test]
fn speciating_needs_a_body() {
    let mut world = World::new(13, 16);
    let me = world.controlled_id().unwrap();
    world.organisms.retain(|o| o.id != me);
    world.apply(Intent::Idle);

    assert_eq!(
        world.apply(Intent::Speciate {
            name: "nobody".into()
        }),
        Outcome::Rejected(Rejection::Disembodied),
        "a line with nobody in it cannot be split"
    );
}

#[test]
fn a_split_is_recorded_in_the_founders_own_history() {
    use mesocosm_core::{Event, History};

    let mut world = World::new(4_242, 40);
    let mut history = History::new();
    history.record_all(world.drain_events());

    let me = world.controlled_id().unwrap();
    world.apply(Intent::Speciate {
        name: "recorded".into(),
    });
    history.record_all(world.drain_events());

    let split = history
        .log()
        .entries()
        .iter()
        .any(|e| matches!(e.record, Event::Speciated { founder, .. } if founder == me));
    assert!(split, "the founder's line shows where it forked");
}

#[test]
fn eating_teaches_the_line_a_word() {
    // The acquisition half of kleptoplasty, ruled 2026-08-03: a lineage cannot
    // express an appendage it has never eaten, and a meal that teaches is a
    // different kind of event from a meal that feeds.
    let mut world = World::new(4_242, 40);
    let mine = world.controlled().unwrap().species;

    // Find something whose line grows an appendage ours has not learned.
    let unknown: Vec<_> = world
        .lineages()
        .all()
        .flat_map(|s| {
            s.recipe
                .tagmata
                .iter()
                .map(|t| t.appendage)
                .collect::<Vec<_>>()
        })
        .filter(|a| !a.is_innate())
        .filter(|a| !world.lineages().get(mine).unwrap().recipe.can_express(*a))
        .collect();
    assert!(
        !unknown.is_empty(),
        "the enclosure holds words we do not have"
    );

    // Teach it directly and confirm the rule the world enforces.
    let word = unknown[0];
    assert!(!world.lineages().get(mine).unwrap().recipe.can_express(word));
    assert!(
        world
            .lineages_mut()
            .get_mut(mine)
            .unwrap()
            .recipe
            .acquire(word)
    );
    assert!(world.lineages().get(mine).unwrap().recipe.can_express(word));
    assert!(
        !world
            .lineages_mut()
            .get_mut(mine)
            .unwrap()
            .recipe
            .acquire(word),
        "the second one is just a meal"
    );
}

#[test]
fn a_seeded_world_holds_several_body_plans() {
    // The enclosure used to be one shape at several sizes. Every founding
    // line now draws its own recipe.
    let world = World::new(4_242, 40);
    let shapes: std::collections::BTreeSet<String> = world
        .lineages()
        .all()
        .map(|s| format!("{:?}", s.recipe.tagmata))
        .collect();

    assert!(shapes.len() > 1, "a world of one plan is the old world");
    assert!(
        world.lineages().all().any(|s| s.recipe.appendages() > 1),
        "something out there has appendages"
    );
}

#[test]
fn a_fork_inherits_what_its_parent_learned() {
    let mut world = World::new(4_242, 40);
    let mine = world.controlled().unwrap().species;
    world
        .lineages_mut()
        .get_mut(mine)
        .unwrap()
        .recipe
        .acquire(mesocosm_core::Appendage::Vane);

    let Outcome::Speciated { species, .. } = world.apply(Intent::Speciate {
        name: "heir".into(),
    }) else {
        panic!("the fork happened");
    };
    assert!(
        world
            .lineages()
            .get(species)
            .unwrap()
            .recipe
            .can_express(mesocosm_core::Appendage::Vane),
        "a founder does not forget what its line had learned"
    );
}

// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use std::collections::BTreeSet;

use super::*;
use crate::places::Tier;
use crate::rules::{DeepTimeSpan, WorldRules};
use crate::state_hash;
use crate::{Organism, OrganismId, Stage};

/// Twenty-tick epochs keep a debug build fast; the span is the world's rule,
/// set here the way a generation door sets it.
fn spanned(seed: u64, epoch: EpochRule, epochs: u32) -> World {
    let world = World::new(seed, 60);
    let rules = world.rules();
    world.with_rules(WorldRules {
        epoch,
        deep_time: DeepTimeSpan { epochs },
        ..rules
    })
}

const BRISK: EpochRule = EpochRule::Timed { ticks: 20 };

/// Drives `world` one idle tick at a time until its next boundary, recording
/// and reckoning the way a runtime does, and returns that reckoning together
/// with the record as it stood before it.
fn reckon_next_boundary(
    world: &mut World,
    history: &mut History,
) -> (crate::WorldRecord, Vec<crate::Reading>) {
    let epoch = world.epoch;
    while world.epoch == epoch {
        world.apply(Intent::Idle);
        history.record_all(world.drain_events());
        drop(world.drain_flows());
    }
    let standing = world.record().clone();
    (standing, world.reckon(history))
}

#[test]
fn a_span_of_two_lands_exactly_on_the_second_boundary() {
    let mut world = spanned(7, BRISK, 2);
    let mut history = History::new();
    let handover = world.run_deep_time(&mut history).unwrap();

    assert_eq!(handover.span, DeepTime { epochs: 2 });
    assert_eq!((handover.from_epoch, handover.from_tick), (0, 0));
    assert_eq!(handover.to_epoch, 2, "the epoch rose by the span");
    assert_eq!(world.epoch, handover.to_epoch);
    assert_eq!(handover.to_tick, 40, "two whole twenty-tick budgets");
    assert_eq!(world.tick, handover.to_tick);
    assert!(world.at_boundary(), "handed over standing on the boundary");
    assert!(!history.is_empty(), "the past was recorded");
    assert!(world.record().filled() > 0, "and reckoned into the record");
    assert!(world.drain_events().is_empty(), "nothing left undrained");
}

#[test]
fn deep_time_is_deterministic() {
    let run = || {
        let mut world = spanned(7, BRISK, 2);
        let mut history = History::new();
        let handover = world.run_deep_time(&mut history).unwrap();
        (state_hash(&world), history, handover)
    };
    let (hash, history, handover) = run();
    let (again_hash, again_history, again_handover) = run();
    assert_eq!(hash, again_hash);
    assert_eq!(history, again_history);
    assert_eq!(handover, again_handover);
}

/// The body is let go before the first tick, every tier freezes far in the
/// same moment, and the run is otherwise exactly released control followed
/// by the runtime's own loop: idle, record, drain, reckon on the tick that
/// closes an epoch.
#[test]
fn the_controlled_body_is_released_for_the_run() {
    let mut world = spanned(7, BRISK, 2);
    let body = world.controlled_id().expect("a world opens under the hand");
    let mut by_hand = world.clone();
    let mut history = History::new();
    world.run_deep_time(&mut history).unwrap();
    assert_eq!(
        world.controlled_id(),
        None,
        "{body:?} is let go for the run"
    );
    assert_eq!(world.control_lost(), None, "and letting go was not a loss");

    by_hand.release_control();
    by_hand.freeze_tiers_far();
    let mut by_hand_history = History::new();
    for _ in 0..2 {
        reckon_next_boundary(&mut by_hand, &mut by_hand_history);
    }
    assert_eq!(state_hash(&by_hand), state_hash(&world));
    assert_eq!(by_hand_history, history);
}

#[test]
fn a_zero_span_changes_nothing() {
    for rule in [BRISK, EpochRule::Gated, EpochRule::PlayerTriggered] {
        let mut world = spanned(7, rule, 0);
        let before = state_hash(&world);
        let mut history = History::new();
        let handover = world.run_deep_time(&mut history).unwrap();
        assert_eq!(state_hash(&world), before, "{rule:?}");
        assert!(history.is_empty());
        assert!(world.controlled_id().is_some(), "no span, nobody released");
        assert_eq!(
            (handover.from_tick, handover.from_epoch),
            (handover.to_tick, handover.to_epoch)
        );
        assert_eq!((handover.to_tick, handover.to_epoch), (0, 0));
    }
}

#[test]
fn a_rule_that_never_closes_an_epoch_refuses_a_span_before_any_tick() {
    for rule in [
        EpochRule::Gated,
        EpochRule::PlayerTriggered,
        EpochRule::Timed { ticks: 0 },
    ] {
        let mut world = spanned(7, rule, 3);
        let before = state_hash(&world);
        let mut history = History::new();
        assert_eq!(
            world.run_deep_time(&mut history),
            Err(DeepTimeError::NeverCloses { rule, epochs: 3 })
        );
        assert_eq!(world.tick, 0, "{rule:?}");
        assert_eq!(state_hash(&world), before, "not even the hand was released");
        assert!(history.is_empty());
    }
}

/// The point of the whole step. A fresh world's first reckoning lands every
/// reading on an axis nobody had marked; after deep time, the played world's
/// first reckoning is judged against marks deep time set.
#[test]
fn the_first_reckoning_after_deep_time_is_judged_against_its_record() {
    let mut fresh = spanned(7, BRISK, 0);
    let mut fresh_history = History::new();
    let (standing, readings) = reckon_next_boundary(&mut fresh, &mut fresh_history);
    assert!(!readings.is_empty(), "the control reckons something");
    assert!(
        readings.iter().all(|r| standing.untouched(r.feat, r.scale)),
        "a fresh record has nothing to judge against"
    );

    let mut world = spanned(7, BRISK, 2);
    let mut history = History::new();
    world.run_deep_time(&mut history).unwrap();
    let (standing, readings) = reckon_next_boundary(&mut world, &mut history);
    assert_eq!(world.epoch, 3);
    let judged = readings
        .iter()
        .filter(|r| !standing.untouched(r.feat, r.scale))
        .count();
    assert!(
        judged > 0,
        "at least one reading meets a mark deep time set: {judged} of {}",
        readings.len()
    );
}

#[test]
fn the_ceiling_allows_one_budget_past_the_span() {
    assert_eq!(ceiling(EpochRule::Timed { ticks: 20 }, 2), Ok(60));
    assert_eq!(
        ceiling(EpochRule::Timed { ticks: u64::MAX }, u32::MAX),
        Ok(u64::MAX)
    );
}

/// Ruled 2026-09-16: with no player, no body is near anyone. Every mature
/// founder is pushed past its own gestation gate first, so the run is all but
/// guaranteed to breed at least one newcomer — the case the freeze at release
/// alone cannot cover, since a newborn inherits its parent's tier rather than
/// reading one off a focus that does not exist.
#[test]
fn deep_time_leaves_every_living_organism_far_including_the_newly_born() {
    let mut world = spanned(7, BRISK, 2);
    for organism in world.organisms.iter_mut() {
        if organism.stage == Stage::Mature {
            organism.since_offspring = u32::MAX;
        }
    }
    let before: BTreeSet<OrganismId> = world.living().map(|o| o.id).collect();

    let mut history = History::new();
    world.run_deep_time(&mut history).unwrap();

    let after: Vec<&Organism> = world.living().collect();
    let newcomers = after.iter().filter(|o| !before.contains(&o.id)).count();
    assert!(
        newcomers > 0,
        "fixture must breed at least one newcomer to exercise inheritance"
    );
    assert!(
        after.iter().all(|o| o.tier == Tier::Far),
        "every living organism, inherited or original, runs the far tier"
    );
}

/// A span does not merely leave the near tier where it stood; it levels a
/// mixed enclosure down to nothing but far.
#[test]
fn a_world_mixed_near_and_far_ends_with_none_near() {
    let mut world = spanned(7, BRISK, 2);
    for (index, organism) in world.organisms.iter_mut().enumerate() {
        organism.tier = if index % 2 == 0 {
            Tier::Near
        } else {
            Tier::Far
        };
    }
    assert!(
        world.living().any(|o| o.tier == Tier::Near),
        "fixture must start with some bodies near"
    );
    assert!(
        world.living().any(|o| o.tier == Tier::Far),
        "fixture must start with some bodies far"
    );

    let mut history = History::new();
    world.run_deep_time(&mut history).unwrap();

    assert!(
        world.living().all(|o| o.tier == Tier::Far),
        "deep time levels every body to the far tier"
    );
}

/// Deep time's freeze is not a standing rule: the ordinary tier update
/// resumes the moment a focus exists again, promoting bodies around whoever
/// took control, the same way `mesocosm-runtime`'s trial fixture inhabits a
/// world after core's own deep time.
#[test]
fn taking_control_after_deep_time_resumes_tiers_around_the_heir() {
    let mut world = spanned(7, BRISK, 2);
    let mut history = History::new();
    world.run_deep_time(&mut history).unwrap();
    assert!(
        world.living().all(|o| o.tier == Tier::Far),
        "deep time leaves the whole enclosure far"
    );

    let heir = world
        .living()
        .map(|organism| organism.id)
        .find(|id| world.is_eligible(*id))
        .expect("a playable critter survived deep time");
    world.apply(Intent::TakeControl { organism: heir });
    for _ in 0..2 {
        world.apply(Intent::Idle);
    }
    assert!(
        world.living().any(|o| o.tier == Tier::Near),
        "the ordinary tier update resumes around the new focus"
    );
}

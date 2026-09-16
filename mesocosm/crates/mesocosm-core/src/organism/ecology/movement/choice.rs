// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! What a body reaches for: the bite, the target it walks toward, and the
//! sighting it remembers.
//!
//! Split out of `movement.rs` at the six-hundred-line ceiling (soil cycle plan
//! S1), along the seam between choosing and travelling. **Both tiers choose by
//! the same rules** (ruled by Mark 2026-09-16, ruling 7): a far body differs
//! only in how it perceives, a distance with no ray (`perception`), and how it
//! steps (`far`). Only a graph-only fixture's near body, with no ground to see
//! across, keeps the reach-and-omniscience model.

use super::perception::{
    Cells, can_perceive, can_perceive_position, nearby_indexes, sight_range, sighted,
};
use super::{CarrionTarget, DECOMPOSE_RANGE, GRAZE_RANGE, LivingTarget, chebyshev};
use crate::organism::ecology::is_hungry;
use crate::organism::ecology::kinship::Kin;
use crate::organism::{
    FaunaDecisionTrace, FaunaDrive, FaunaSenses, FaunaTraits, LastSeen, Organism, OrganismId,
};
use crate::organism::{Kingdom, Signal};
use crate::places::Ground;
use crate::process::{BodyProcesses, FeedingMode, NisKind};
use std::cmp::Reverse;

/// Chooses a food source within the body's actual reach. This is local for
/// both tiers, so it never needs a global scan.
pub(in crate::organism::ecology) fn choose_living_target(
    organism: &Organism,
    living: &[LivingTarget],
    cells: &Cells,
    ground: Option<&Ground>,
    kin: &Kin,
) -> Option<usize> {
    let reach = GRAZE_RANGE + organism.body().reach();
    let sight = sight_range(organism, reach, ground);
    let observer_shape = organism.walker_shape();
    let hungry = is_hungry(organism);
    let mut candidates: Vec<(u64, usize, usize)> = Vec::new();
    for order in nearby_indexes(cells, organism.position, sight) {
        let Some(target) = living.get(order) else {
            continue;
        };
        if target.id == organism.id
            || !organism.admits(target.kingdom.nis_kind(), false)
            || chebyshev(organism.position, target.position) > reach
            || (target.signal != Signal::Plain && target.kingdom != Kingdom::Producer)
        {
            continue;
        }
        // **Kinship tempers the appetite** (TD10). The remove is in the same
        // voxels the distance term is, so kin read as further off rather than
        // as forbidden: a body of the eater's own line ranks behind everything
        // else in reach, and is still taken when it is the only thing there.
        let remove = kin.remove(organism.species, target.species, reach, hungry);
        let distance = (chebyshev(organism.position, target.position) + remove) as u64;
        let danger = u64::from(target.signal == Signal::Warning) * 4;
        let score =
            (distance.saturating_mul(16) + danger).saturating_sub(target.mass_mg.min(256) / 64);
        candidates.push((score, order, target.organism_index));
    }
    candidates.sort_unstable();
    candidates.into_iter().find_map(|(_, order, index)| {
        living
            .get(order)
            .is_some_and(|target| can_perceive(organism, observer_shape, target, sight, ground))
            .then_some(index)
    })
}

/// Carrion feeding is local too. Returning the original organism index keeps
/// the drain pass independent from the derived bucket representation.
pub(in crate::organism::ecology) fn choose_carrion_target(
    organism: &Organism,
    carrion: &[CarrionTarget],
    cells: &Cells,
    ground: Option<&Ground>,
) -> Option<usize> {
    if !organism.admits(NisKind::Producer, true) {
        return None;
    }
    let observer_shape = organism.walker_shape();
    nearby_indexes(cells, organism.position, DECOMPOSE_RANGE)
        .filter_map(|order| carrion.get(order).map(|target| (order, target)))
        .filter(|(_, target)| {
            (0..3).all(|axis| {
                (target.position[axis] - organism.position[axis]).abs() <= DECOMPOSE_RANGE
            }) && can_perceive_position(
                organism,
                observer_shape,
                target.position,
                target.shape,
                DECOMPOSE_RANGE,
                ground,
            )
        })
        .min_by_key(|(order, _)| *order)
        .map(|(_, target)| target.organism_index)
}

/// The graph-only fixture's pursuit ranking: nearest after the kin remove,
/// then fattest, then roster order.
fn preferred_living<'a>(
    organism: &Organism,
    candidates: impl Iterator<Item = (usize, &'a LivingTarget)>,
    ground: Option<&Ground>,
    sight: i32,
    kin: &Kin,
) -> Option<(OrganismId, [i32; 3])> {
    let observer_shape = organism.walker_shape();
    let hungry = is_hungry(organism);
    let mut ranked: Vec<(i32, u64, usize, &'a LivingTarget)> = candidates
        .filter(|(_, target)| {
            target.id != organism.id && organism.admits(target.kingdom.nis_kind(), false)
        })
        .map(|(order, target)| {
            // The same remove the bite applies (TD10), against sight rather
            // than reach: a hunter should not walk toward its own line either.
            (
                chebyshev(organism.position, target.position)
                    + kin.remove(organism.species, target.species, sight, hungry),
                target.mass_mg,
                order,
                target,
            )
        })
        .collect();
    ranked.sort_unstable_by_key(|(distance, mass, order, _)| (*distance, Reverse(*mass), *order));
    ranked.into_iter().find_map(|(_, _, _, target)| {
        can_perceive(organism, observer_shape, target, sight, ground)
            .then_some((target.id, target.position))
    })
}

/// The inherited fauna policy's choice among what this body sees: pursue,
/// avoid or hold, with its trace. Every body that sees runs it, near or far
/// (ruling 7); a far body read the graph-only ranking above until then, so it
/// could neither avoid nor hold, and its policy never remembered a score.
fn policy_living<'a>(
    organism: &mut Organism,
    candidates: impl Iterator<Item = (usize, &'a LivingTarget)>,
    ground: Option<&Ground>,
    sight: i32,
    kin: &Kin,
) -> Option<MovementTarget> {
    let traits = FaunaTraits::read(organism);
    let own_mass = organism.biomass_mg();
    let policy = organism.fauna_policy;
    let observer_shape = organism.walker_shape();
    let hungry = is_hungry(organism);
    let candidate = candidates
        .filter(|(_, target)| {
            target.id != organism.id && organism.admits(target.kingdom.nis_kind(), false)
        })
        .filter_map(|(order, target)| {
            // Kin read as further away here too (TD10), and it is the same one
            // remove. The bite alone was measured not to reach: a hunter that
            // still *walked* toward its own line arrived where its own line was
            // the only thing it could see, and then had to eat it. The decision
            // trace below therefore records the discounted distance rather than
            // the geometric one, because that is the reading the body acted on.
            let distance = chebyshev(organism.position, target.position)
                + kin.remove(organism.species, target.species, sight, hungry);
            if !can_perceive(organism, observer_shape, target, sight, ground) {
                return None;
            }
            let senses = FaunaSenses::read(
                organism,
                traits,
                target.id,
                distance,
                target.mass_mg,
                target.signal,
            );
            let scores = policy.score(senses, own_mass, sight);
            let drive = scores.selected();
            let rank = (
                scores.score(drive),
                Reverse(distance),
                target.mass_mg,
                Reverse(order),
            );
            Some((rank, target, senses, scores, drive))
        })
        .max_by_key(|(rank, ..)| *rank);

    let Some((_, target, senses, scores, drive)) = candidate else {
        organism.last_fauna_decision = None;
        return None;
    };
    organism.fauna_policy.remember(scores);
    organism.last_fauna_decision = Some(FaunaDecisionTrace {
        traits,
        senses,
        selected_drive: drive,
        selected_target: Some(target.id),
        scores,
    });
    Some(match drive {
        FaunaDrive::Pursue => MovementTarget::Seen(target.id, target.position),
        FaunaDrive::Avoid => MovementTarget::Avoid(target.id, target.position),
        FaunaDrive::Hold => MovementTarget::Hold(target.id, target.position),
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum MovementTarget {
    Seen(OrganismId, [i32; 3]),
    Avoid(OrganismId, [i32; 3]),
    Hold(OrganismId, [i32; 3]),
    Other([i32; 3]),
}

fn preferred_carrion<'a>(
    organism: &Organism,
    candidates: impl Iterator<Item = (usize, &'a CarrionTarget)>,
    ground: Option<&Ground>,
    sight: i32,
) -> Option<[i32; 3]> {
    if !organism.admits(NisKind::Producer, true) {
        return None;
    }
    let observer_shape = organism.walker_shape();
    let mut ranked: Vec<(i32, usize, &'a CarrionTarget)> = candidates
        .map(|(order, target)| (chebyshev(organism.position, target.position), order, target))
        .collect();
    ranked.sort_unstable_by_key(|(distance, order, _)| (*distance, *order));
    ranked.into_iter().find_map(|(_, _, target)| {
        can_perceive_position(
            organism,
            observer_shape,
            target.position,
            target.shape,
            sight,
            ground,
        )
        .then_some(target.position)
    })
}

/// Where this body means to go this tick, if anywhere in sight is worth it.
///
/// A body that sees reads only the sensory buckets its sight overlaps, which
/// is also what took the far tier's whole-roster scan away (scale plan S3's
/// "distance cap and bucketing"): with far sight capped, a bucket read and a
/// roster scan answer alike, and the bucket read is local.
#[allow(clippy::too_many_arguments)]
pub(super) fn preferred_target(
    organism: &mut Organism,
    living: &[LivingTarget],
    living_cells: &Cells,
    carrion: &[CarrionTarget],
    carrion_cells: &Cells,
    ground: Option<&Ground>,
    kin: &Kin,
) -> Option<MovementTarget> {
    match organism.feeding_mode() {
        FeedingMode::Grazer | FeedingMode::Predator | FeedingMode::Omnivore => {
            let reach = GRAZE_RANGE + organism.body().reach();
            let sight = sight_range(organism, reach, ground);
            let living = if sighted(organism, ground) {
                policy_living(
                    organism,
                    nearby_indexes(living_cells, organism.position, sight)
                        .filter_map(|order| living.get(order).map(|target| (order, target))),
                    ground,
                    sight,
                    kin,
                )
            } else {
                organism.last_fauna_decision = None;
                preferred_living(organism, living.iter().enumerate(), ground, sight, kin)
                    .map(|(id, at)| MovementTarget::Seen(id, at))
            };
            living.or_else(|| preferred_carrion_target(organism, carrion, carrion_cells, ground))
        },
        FeedingMode::Scavenger => {
            preferred_carrion_target(organism, carrion, carrion_cells, ground)
        },
        FeedingMode::Producer => {
            organism.last_fauna_decision = None;
            None
        },
    }
}

fn preferred_carrion_target(
    organism: &mut Organism,
    carrion: &[CarrionTarget],
    carrion_cells: &Cells,
    ground: Option<&Ground>,
) -> Option<MovementTarget> {
    organism.last_fauna_decision = None;
    let sight = sight_range(organism, DECOMPOSE_RANGE + organism.body().reach(), ground);
    let target = if sighted(organism, ground) {
        preferred_carrion(
            organism,
            nearby_indexes(carrion_cells, organism.position, sight)
                .filter_map(|order| carrion.get(order).map(|target| (order, target))),
            ground,
            sight,
        )
    } else {
        preferred_carrion(organism, carrion.iter().enumerate(), ground, sight)
    };
    target.map(MovementTarget::Other)
}

/// The sighting a body keeps pursuing for a few ticks after losing it.
///
/// **It crosses the tier line** (ruling 7). It used to be dropped on the far
/// side, because a far body saw the whole enclosure and a near body saw along
/// rays, so a sighting from one model meant nothing in the other. Both tiers
/// now read one sight, so a sighting is a sighting in either, and a far body
/// that loses its prey keeps after it as a near body does. Only a graph-only
/// fixture's near body, which does not see at all, drops it.
pub(super) fn remembered_target(
    organism: &mut Organism,
    living: &[LivingTarget],
    ground: Option<&Ground>,
) -> Option<[i32; 3]> {
    if !sighted(organism, ground) {
        organism.last_seen = None;
        return None;
    }
    let memory = organism.last_seen?;
    if memory.ticks_left == 0
        || !living.iter().any(|target| {
            target.id == memory.target && organism.admits(target.kingdom.nis_kind(), false)
        })
    {
        organism.last_seen = None;
        return None;
    }
    organism.last_seen = Some(LastSeen {
        ticks_left: memory.ticks_left - 1,
        ..memory
    });
    Some(memory.position)
}

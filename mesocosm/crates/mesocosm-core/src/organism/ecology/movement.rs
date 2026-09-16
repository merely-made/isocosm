// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Local affordance lookup and locomotion for ecology ticks.
//!
//! The spatial buckets are rebuilt from the tick's immutable reading. They
//! are an acceleration structure, never state: their only job is to avoid
//! asking every embodied body about every other body before a sight query.
//!
//! **The far tier runs near's rules with cheaper geometry** (ruled by Mark
//! 2026-09-16, soil cycle plan ruling 7). What a body reaches for is `choice`,
//! for both tiers; what differs is sight without a ray (`perception`) and a
//! column step without brick kinematics (`far`).

use super::kinship::Kin;
use super::{dispersal_for, is_hungry, travels};
use crate::flow::Records;
use crate::history::Event;
use crate::organism::{LastSeen, Organism};
use crate::places::{
    Ground, Places, Soil, Tier, WalkerShape, route_step_for, step_for as grounded_step,
    surface_stance_for,
};
use crate::process::BodyProcesses;
use crate::rng::Rng;

mod choice;
mod far;
mod perception;

use choice::{MovementTarget, preferred_target, remembered_target};
pub(super) use choice::{choose_carrion_target, choose_living_target};
pub(super) use perception::{CarrionTarget, LivingTarget, carrion_cells, living_cells};
use perception::{Cells, forage_gradient, sighted};

/// How far a consumer reaches for a meal, in voxel units.
pub(super) const GRAZE_RANGE: i32 = 5;
/// How far a decomposer reaches for the dead, in voxel units.
pub(super) const DECOMPOSE_RANGE: i32 = 6;
/// A remembered sight line can take a short detour, but never authorizes a
/// global navigation search in an ecology tick.
const MEMORY_ROUTE_BUDGET: i32 = 8;
/// Direct observation is fresh for this many failed perception ticks.
const MEMORY_TICKS: u8 = 8;

fn chebyshev(from: [i32; 3], to: [i32; 3]) -> i32 {
    (0..3)
        .map(|axis| (from[axis] - to[axis]).abs())
        .max()
        .unwrap_or(0)
}

/// Finds a valid surface stance for a graph position, if it is resident in the
/// grown ground. Callers choose the fallback: a near-tier birth remains beside
/// its grounded parent when its scatter leaves the grown enclosure.
pub(super) fn surface_stance(
    ground: &Ground,
    shape: WalkerShape,
    position: [i32; 3],
) -> Option<[i32; 3]> {
    surface_stance_for(ground, shape, position)
}

/// Moves an organism toward the affordance it currently needs, by one set of
/// rules for both tiers (ruling 7). Near bodies move by legal integer steps;
/// far bodies by column steps within the same budget (`far`, soil cycle plan
/// S1); a graph-only fixture's near body by integer steps and place hops.
#[allow(clippy::too_many_arguments)]
pub(super) fn disperse(
    organism: &mut Organism,
    places: &Places,
    ground: Option<&Ground>,
    rng: &mut Rng,
    soil: &mut Soil,
    living: &[LivingTarget],
    living_cells: &Cells,
    carrion: &[CarrionTarget],
    carrion_cells: &Cells,
    records: &mut Records<'_>,
    kin: &Kin,
) -> bool {
    let target = preferred_target(
        organism,
        living,
        living_cells,
        carrion,
        carrion_cells,
        ground,
        kin,
    );
    let (target, pursuing_memory) = match target {
        Some(MovementTarget::Seen(id, at)) => {
            organism.last_seen = Some(LastSeen {
                target: id,
                position: at,
                ticks_left: MEMORY_TICKS,
            });
            (Some(at), false)
        },
        Some(MovementTarget::Avoid(id, at)) => {
            organism.last_seen = Some(LastSeen {
                target: id,
                position: at,
                ticks_left: MEMORY_TICKS,
            });
            (
                Some([
                    organism.position[0] + (organism.position[0] - at[0]),
                    organism.position[1],
                    organism.position[2] + (organism.position[2] - at[2]),
                ]),
                false,
            )
        },
        Some(MovementTarget::Hold(id, at)) => {
            organism.last_seen = Some(LastSeen {
                target: id,
                position: at,
                ticks_left: MEMORY_TICKS,
            });
            (None, false)
        },
        Some(MovementTarget::Other(at)) => (Some(at), false),
        None => (remembered_target(organism, living, ground), true),
    };
    let old = organism.position;
    let shape = organism.walker_shape();
    // **No actuator, no travel** (TD8). Zeroing `dispersal_for` is necessary
    // and not sufficient: the hungry wander and the far tier's graph hop never
    // asked for a budget, so a body with nothing that contracts still walked —
    // and it walked at a plant's rent, which is the free lunch the ruling
    // withdraws. It reads its drives and its memory above, exactly as before;
    // what it cannot do is act on them by going somewhere.
    //
    // **Producers creep** (TD9). TD8 made the *stand* sessile as a side-effect,
    // because a producer is unlimbed by construction, and that retired the one
    // way a shaded plant had of leaving its own shade. `travels` restores it as
    // a rule about how a producer lives rather than about bodies without limbs,
    // so the free lunch stays withdrawn: an unlimbed consumer still reads false
    // here and still goes nowhere.
    let next = if !travels(organism) {
        organism.position
    } else if let Some(target) = target {
        if organism.tier == Tier::Far {
            far::walk(
                places,
                ground,
                shape,
                organism.position,
                target,
                dispersal_for(organism),
                Some(organism.body().reach() + GRAZE_RANGE),
            )
        } else if let Some(ground) = ground {
            walk_grounded(organism, ground, shape, target, pursuing_memory)
        } else {
            let mut at = organism.position;
            for _ in 0..dispersal_for(organism) {
                at = integer_step(at, target);
                if chebyshev(at, target) <= organism.body().reach() + GRAZE_RANGE {
                    break;
                }
            }
            at
        }
    } else if is_hungry(organism) {
        // **The size of the creep** (TD9). A producer gets no target from
        // `preferred_target`, so this branch is its entire travel budget: one
        // voxel, only while its reserve is under `HUNGRY_UPKEEP_TICKS` of rent,
        // paid for in substance like every other step. A far stand creeps as a
        // near one does (ruling 7): the far wander is near's one voxel now, not
        // a heading for the next place. With no ground and no tier that sees,
        // there is nothing to creep across.
        if sighted(organism, ground) {
            forage(
                organism,
                places,
                ground,
                rng,
                living,
                living_cells,
                carrion,
                carrion_cells,
                kin,
            )
        } else if organism.actuator_span() == 0 {
            organism.position
        } else {
            diffuse(places, organism.position, rng)
        }
    } else {
        organism.position
    };

    if next != old {
        super::flows::pay_travel(organism, soil, records, old, chebyshev(old, next) as u64);
        organism.position = next;
        records.event(
            next,
            Event::Moved {
                organism: organism.id,
                from: old,
                to: next,
            },
        );
        true
    } else {
        false
    }
}

/// One tick's grounded travel toward a place, spending the dispersal budget a
/// step at a time and stopping where the ground refuses or the body arrives.
///
/// Lifted out of `disperse` in TD11 so the hungry gradient walks by exactly the
/// same rule a pursuit does; the arithmetic is unchanged.
fn walk_grounded(
    organism: &Organism,
    ground: &Ground,
    shape: WalkerShape,
    target: [i32; 3],
    pursuing_memory: bool,
) -> [i32; 3] {
    let mut at = organism.position;
    for _ in 0..dispersal_for(organism) {
        let next = if pursuing_memory {
            route_step_for(ground, shape, at, target, MEMORY_ROUTE_BUDGET)
                .unwrap_or_else(|| grounded_step(ground, shape, at, target))
        } else {
            grounded_step(ground, shape, at, target)
        };
        if next == at {
            break;
        }
        at = next;
        if chebyshev(at, target) <= organism.body().reach() + GRAZE_RANGE {
            break;
        }
    }
    at
}

/// A hungry body with nothing in sight takes one voxel up the forage gradient,
/// or the random wander when the gradient gives nothing or its step is refused.
///
/// **Hunger follows a gradient** (TD11), and it is exactly the same *one*
/// voxel the random wander took: the direction changes, never the size. See
/// `perception::forage_gradient` for the rule, for why a bucket centre is not a
/// second sight, and for what the pursuit budget cost when it was tried here
/// instead. A heading the ground refuses falls back to the wander: the gradient
/// has no route-finder, so it is the only thing here that can get around an
/// obstruction.
///
/// **Both tiers, one rule** (ruling 7). A far body only wandered, toward a
/// random neighbouring place, so a hungry one with nothing in sight stayed
/// among its own line until kin were all it could see (150,757 of 157,626 mg
/// of far predation on seed 7 at 200 founders was own-line). The voxel is the
/// tier's: a grounded step near, a column step far.
#[allow(clippy::too_many_arguments)]
fn forage(
    organism: &Organism,
    places: &Places,
    ground: Option<&Ground>,
    rng: &mut Rng,
    living: &[LivingTarget],
    living_cells: &Cells,
    carrion: &[CarrionTarget],
    carrion_cells: &Cells,
    kin: &Kin,
) -> [i32; 3] {
    let (at, shape) = (organism.position, organism.walker_shape());
    let voxel = |toward: [i32; 3]| match (organism.tier, ground) {
        (Tier::Near, Some(ground)) => grounded_step(ground, shape, at, toward),
        _ => far::walk(places, ground, shape, at, toward, 1, None),
    };
    forage_gradient(organism, living, living_cells, carrion, carrion_cells, kin)
        .map(voxel)
        .filter(|next| *next != at)
        .unwrap_or_else(|| {
            const WANDER: [[i32; 2]; 4] = [[1, 0], [-1, 0], [0, 1], [0, -1]];
            let [dx, dz] = WANDER[rng.below(WANDER.len() as u64) as usize];
            voxel([at[0] + dx, at[1], at[2] + dz])
        })
}

fn integer_step(from: [i32; 3], to: [i32; 3]) -> [i32; 3] {
    [
        from[0] + (to[0] - from[0]).signum(),
        from[1] + (to[1] - from[1]).signum(),
        from[2] + (to[2] - from[2]).signum(),
    ]
}

/// A neighbouring place's centre, drawn at random: the graph-only near
/// wander's destination.
fn diffuse(places: &Places, position: [i32; 3], rng: &mut Rng) -> [i32; 3] {
    let Some(current) = places.at(position) else {
        return position;
    };
    let neighbours = places.neighbours(current);
    if neighbours.is_empty() {
        return position;
    }
    let id = neighbours[rng.below(neighbours.len() as u64) as usize];
    let Some(place) = places.get(id) else {
        return position;
    };
    [place.centre[0], position[1], place.centre[1]]
}

#[cfg(test)]
mod tests;

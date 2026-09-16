// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Far-tier travel. (soil cycle plan S1)
//!
//! A far body used to cross one place-graph edge a tick and arrive at the
//! neighbour's centre: 39 to 111 voxels, paid from body substance, and a body
//! already in its target's place always left it, because every neighbour is
//! one hop from there. Mark ruled both out on 2026-09-16 (soil cycle plan
//! ruling 1). A far body now walks:
//!
//! - **toward its target, and never out of the target's place** once inside
//!   it;
//! - **at most its dispersal budget a tick**, in the unit travel is paid in:
//!   the Chebyshev distance, height included, from where the tick began.
//!
//! The far tier has no brick kinematics, so a step is one column, diagonal
//! first and then each axis alone, and the body stands on that column's
//! surface where there is ground, as the old hop's arrival did. Every far
//! position is therefore a legal stance, which is all a later promotion to
//! near needs; nothing else is carried between ticks.

use super::{chebyshev, diffuse, surface_stance};
use crate::places::{Ground, Places, WalkerShape};
use crate::rng::Rng;

/// One tick's travel from `from` toward `toward`, spending at most `budget`
/// voxels and stopping once within `stop_within` of it.
///
/// A step is refused if the body could not stand on it, if it would put the
/// body further than `budget` from `from` or further from `toward` than it
/// stood, or if it would leave `toward`'s place while the body is inside it.
/// A refused diagonal slides to an axis; with nothing left, the walk ends.
pub(super) fn walk(
    places: &Places,
    ground: Option<&Ground>,
    shape: WalkerShape,
    from: [i32; 3],
    toward: [i32; 3],
    budget: u32,
    stop_within: Option<i32>,
) -> [i32; 3] {
    let budget = i32::try_from(budget).unwrap_or(i32::MAX);
    let home = places.at(toward);
    let step = |at: [i32; 3]| {
        let want = [(toward[0] - at[0]).signum(), (toward[2] - at[2]).signum()];
        let inside = home.is_some() && places.at(at) == home;
        [want, [want[0], 0], [0, want[1]]]
            .into_iter()
            .filter(|&[dx, dz]| dx != 0 || dz != 0)
            .find_map(|[dx, dz]| {
                let column = [at[0] + dx, at[1], at[2] + dz];
                let next = match ground {
                    Some(ground) => surface_stance(ground, shape, column)?,
                    None => column,
                };
                (chebyshev(from, next) <= budget
                    && chebyshev(next, toward) <= chebyshev(at, toward)
                    && (!inside || places.at(next) == home))
                    .then_some(next)
            })
    };
    let mut at = from;
    for _ in 0..budget {
        let Some(next) = step(at) else {
            break;
        };
        at = next;
        if stop_within.is_some_and(|range| chebyshev(at, toward) <= range) {
            break;
        }
    }
    at
}

/// A hungry far body with nothing to reach for: one voxel toward a
/// neighbouring place, drawn as the old hop drew it.
///
/// One, not the pursuit budget: that is the near wander's size, for the reason
/// `perception::forage_gradient` records (TD11 measured the pursuit budget
/// spending hungry bodies with nothing in sight to death).
pub(super) fn wander(
    places: &Places,
    ground: Option<&Ground>,
    shape: WalkerShape,
    from: [i32; 3],
    rng: &mut Rng,
) -> [i32; 3] {
    let heading = diffuse(places, from, rng);
    walk(places, ground, shape, from, heading, 1, None)
}

#[cfg(test)]
mod tests;

// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The grown place graph: adjacency derived from the landscape.
//!
//! `Places::scatter` asserts a lattice, so every world is topologically the
//! same world (uniform interior degree, fixed diameter, no bridges, no dead
//! ends), and its links can disagree with the partition `at` answers from.
//! This module derives links from traversability over a [`Relief`]: you can
//! get between two regions if the ground between them permits it. Ridges
//! make chokepoints, basins make clusters, and no two worlds share a shape
//! unless they share a seed (plan §0.5).
//!
//! Additive on purpose: `scatter` and its serialized shape are untouched
//! while play is in flight. Genesis adopts this constructor by swapping one
//! call when its owner is ready.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use super::relief::Relief;
use super::{Place, PlaceId, Places};
use isocosm::rng::Rng;

/// How much rise above the higher endpoint a crossing tolerates.
const CLIMB: i32 = 26;
/// How many consecutive underwater samples a crossing tolerates.
const FORD: u32 = 2;
/// Samples taken along a candidate crossing.
const STRIDES: i32 = 9;
/// Ruggedness at which a region grows an interior at all, then deepens.
const POCKET: i32 = 34;
const WARREN: i32 = 58;

/// An elective interior: a region whose ground has structure worth
/// entering. Graph-level fact only; the rooms get geometry at G1.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Nest {
    pub host: PlaceId,
    pub rooms: u8,
    pub depth: u8,
}

/// A world's places, the landscape they were derived from, and the
/// interiors the landscape earned.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Grown {
    pub places: Places,
    pub relief: Relief,
    pub nests: Vec<Nest>,
}

impl Places {
    /// Grows a place graph from its own seed: stratified jittered sites,
    /// relief, links where the ground permits travel, nests where it folds.
    /// Consumes no caller stream, so worlds already replay-hashed stay put.
    pub fn grown(seed: u64, side: u16, extent: i32) -> Grown {
        assert!(side >= 2, "a grown world has something to travel between");
        let n = side as usize;
        let mut rng = Rng::from_seed(seed);
        let edge = |i: usize| -extent + (2 * extent * i as i32) / side as i32;

        let mut places = Vec::with_capacity(n * n);
        for row in 0..n {
            for column in 0..n {
                let (x, z) = (edge(column), edge(row));
                places.push(Place {
                    id: PlaceId((row * n + column) as u16),
                    centre: [
                        rng.range_i32(x, (edge(column + 1) - 1).max(x)),
                        rng.range_i32(z, (edge(row + 1) - 1).max(z)),
                    ],
                });
            }
        }

        let relief = Relief::generate(seed);

        // Candidates reach two grid cells out, so jitter can earn diagonal
        // and skip-one links the lattice never had.
        let mut candidates: Vec<(usize, usize)> = Vec::new();
        for index in 0..n * n {
            let (row, col) = (index / n, index % n);
            for dr in -2i32..=2 {
                for dc in -2i32..=2 {
                    let (r2, c2) = (row as i32 + dr, col as i32 + dc);
                    if r2 < 0 || c2 < 0 || r2 >= n as i32 || c2 >= n as i32 {
                        continue;
                    }
                    let other = r2 as usize * n + c2 as usize;
                    if other > index {
                        candidates.push((index, other));
                    }
                }
            }
        }

        let mut links: Vec<BTreeSet<PlaceId>> = vec![BTreeSet::new(); n * n];
        let mut blocked: Vec<(i32, usize, usize)> = Vec::new();
        for &(a, b) in &candidates {
            match crossing(&relief, extent, places[a].centre, places[b].centre) {
                Ok(()) => {
                    links[a].insert(PlaceId(b as u16));
                    links[b].insert(PlaceId(a as u16));
                },
                Err(rise) => blocked.push((rise, a, b)),
            }
        }

        // The world stays whole: reconnect components over the least-bad
        // blocked crossings, in deterministic order. A pass through a ridge
        // is still a pass; it is just the only one.
        let mut component = (0..n * n).collect::<Vec<_>>();
        fn root(component: &mut [usize], index: usize) -> usize {
            let mut at = index;
            while component[at] != at {
                component[at] = component[component[at]];
                at = component[at];
            }
            at
        }
        for (index, set) in links.iter().enumerate() {
            for other in set {
                let (a, b) = (
                    root(&mut component, index),
                    root(&mut component, other.0 as usize),
                );
                component[a.max(b)] = a.min(b);
            }
        }
        blocked.sort_unstable();
        for (_, a, b) in blocked {
            let (ra, rb) = (root(&mut component, a), root(&mut component, b));
            if ra != rb {
                links[a].insert(PlaceId(b as u16));
                links[b].insert(PlaceId(a as u16));
                component[ra.max(rb)] = ra.min(rb);
            }
        }

        let places = Places {
            places,
            links: links
                .into_iter()
                .map(|set| set.into_iter().collect())
                .collect(),
        };

        let nests = places
            .all()
            .filter_map(|place| {
                let folds = relief.ruggedness(extent, place.centre[0], place.centre[1]);
                if folds < POCKET {
                    return None;
                }
                let deep = folds >= WARREN;
                Some(Nest {
                    host: place.id,
                    rooms: 2 + (folds % 5) as u8 + if deep { 3 } else { 0 },
                    depth: 1 + deep as u8,
                })
            })
            .collect();

        Grown {
            places,
            relief,
            nests,
        }
    }
}

/// Whether the ground between two sites permits travel. `Err` carries how
/// far over the climb limit the worst stride rose, for reconnection order.
pub(crate) fn crossing(
    relief: &Relief,
    extent: i32,
    from: [i32; 2],
    to: [i32; 2],
) -> Result<(), i32> {
    // Canonical direction, or integer stride rounding makes A→B and B→A
    // sample different ground and disagree about the same crossing.
    let (from, to) = if from <= to { (from, to) } else { (to, from) };
    let base = relief
        .sample(extent, from[0], from[1])
        .max(relief.sample(extent, to[0], to[1]));
    let mut worst_rise = 0;
    let mut wet_run = 0u32;
    let mut worst_wet = 0u32;
    for stride in 1..STRIDES {
        let x = from[0] + (to[0] - from[0]) * stride / STRIDES;
        let z = from[1] + (to[1] - from[1]) * stride / STRIDES;
        let height = relief.sample(extent, x, z);
        worst_rise = worst_rise.max(height - base);
        if height < relief.sea {
            wet_run += 1;
            worst_wet = worst_wet.max(wet_run);
        } else {
            wet_run = 0;
        }
    }
    if worst_rise <= CLIMB && worst_wet <= FORD {
        Ok(())
    } else {
        Err((worst_rise - CLIMB).max(worst_wet as i32 * CLIMB))
    }
}

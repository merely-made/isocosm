// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A world's competitions (rulings 115, 218, 236 and 240) and the bounds its
//! crowds keep their readings within (rulings 113 and 218).

use super::Query;
use crate::schema::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// One competing lineage in a competition: how its members are told apart,
/// what they grow into, when they want the resource, and the acts the
/// competition executes for them.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Competitor {
    pub identity: Key,
    pub body: Key,
    pub hungry: Query,
    pub eat: Key,
    pub share: Key,
    /// One unit of reserve spent in a fight, and the strain it costs.
    pub spend: Key,
}

/// Ruling 115: members wanting one scarce thing at a site, each side's own
/// way of deciding picking contest or share, the sim resolving the choices.
/// Two contesters size each other up and only a close match escalates
/// (ruling 116), into rounds that strain both sides against their bearing
/// (rulings 221 to 223). A world's competitions are keyed by the site
/// account each contests (ruling 236); they run at once in a tick, each
/// against its members' state at the tick's start, and settle at its end
/// (ruling 240).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Competition {
    pub ration: u64,
    /// The leaning trait: members that carry it contest, the rest share.
    pub contest: Key,
    /// The widest gap in standing that still reads as a close match.
    pub margin: u64,
    /// Reserve the side losing an exchange spends, capped at what it holds.
    pub cost: u64,
    /// The act each side takes for each round: the round's strain.
    pub round: Key,
    /// Per mille, the chance an exchange goes against the side standing
    /// higher.
    pub upset: u32,
    /// How far a break up raises its side's standing, or a break down
    /// lowers it, for the rest of the fight (ruling 222).
    pub advantage: u64,
    pub kinds: Vec<Competitor>,
}

/// Ruling 113's tolerance: each reading's Kolmogorov-Smirnov distance
/// between the two ways, per mille, within its own bound.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Similitude {
    pub default_bound: u32,
    pub bounds: BTreeMap<Key, u32>,
}

impl Similitude {
    pub fn bound(&self, reading: &str) -> u32 {
        self.bounds
            .get(reading)
            .copied()
            .unwrap_or(self.default_bound)
    }
}

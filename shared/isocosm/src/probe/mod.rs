// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Ruling 115's competing instance for feeding when food is short, run two
//! ways under ruling 113: member by member through the core's exact
//! individual runner, and as a crowd, an exact-state histogram advanced by
//! random count draws. The process language cannot yet express a
//! competition, so its definition and the world's similitude bounds sit
//! beside the core rules in `ProbeWorld` until S2 moves them in.

mod aggregate;
pub mod check;
mod crowd;
mod draws;
mod exact;
mod found;
pub mod readings;
#[cfg(test)]
mod tests;

pub use crowd::{Crowd, Variant};
pub use exact::{ExactRun, run_exact};
pub use found::ProbeFounding;

use crate::{rules::Query, schema::*, simulation::Genesis};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// One competing lineage: how its members are told apart, what they eat
/// into, when they are hungry, and the acts the competition executes.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Kind {
    pub identity: Key,
    pub body: Key,
    pub hungry: Query,
    pub eat: Key,
    pub share: Key,
    pub strain: Key,
}

/// Ruling 115 for food: hungry members pair at random; a contester takes the
/// ration from a sharer or from a rival that yields on sizing up; sharers
/// split it; a close match between contesters escalates at a cost to both,
/// paid from body reserve until vigour is in the ledger. Trade waits.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Competition {
    pub food: Key,
    pub ration: u64,
    /// The leaning trait: members that carry it contest, the rest share.
    pub contest: Key,
    pub margin: u64,
    pub cost: u64,
    pub kinds: Vec<Kind>,
}

/// Ruling 113's tolerance: every reading's Kolmogorov-Smirnov distance
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProbeWorld {
    pub genesis: Genesis,
    pub competition: Competition,
    pub similitude: Similitude,
    pub ticks: Tick,
}

/// One side of a contested pair, as the rules read it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Side {
    pub contest: bool,
    pub body: u64,
}

/// What one contested ration does to each side. On an exact tie a seeded
/// coin gives the ration to one side; `gain` is then empty.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Resolution {
    pub gain: [u64; 2],
    pub pay: [u64; 2],
    pub tie: bool,
}

pub fn resolve(c: &Competition, a: Side, b: Side) -> Resolution {
    let r = c.ration;
    let (gain, pay, tie) = match (a.contest, b.contest) {
        (false, false) => ([r / 2, r / 2], [0, 0], false),
        (true, false) => ([r, 0], [0, 0], false),
        (false, true) => ([0, r], [0, 0], false),
        (true, true) => {
            // Sizing up settles all but close matches; an escalated fight
            // costs both, and the side with more reserve outlasts the other.
            let pay = if a.body.abs_diff(b.body) > c.margin {
                [0, 0]
            } else {
                [c.cost.min(a.body), c.cost.min(b.body)]
            };
            let gain = match a.body.cmp(&b.body) {
                std::cmp::Ordering::Greater => [r, 0],
                std::cmp::Ordering::Less => [0, r],
                std::cmp::Ordering::Equal => [0, 0],
            };
            (gain, pay, a.body == b.body)
        },
    };
    Resolution { gain, pay, tie }
}

/// How a short site's rations fall over its hungry members once they are
/// paired at random: every pair gets one ration before any pair gets two,
/// so each pair left on one ration is two wanting one scarce thing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Allocation {
    /// Leading pairs that eat a ration each, uncontested.
    pub doubles: u64,
    /// The pairs after them that contend for one ration each.
    pub contested: u64,
}

/// None when the site can feed every hungry member.
pub fn allocate(hungry: u64, rations: u64) -> Option<Allocation> {
    if rations >= hungry {
        return None;
    }
    let pairs = hungry / 2;
    let doubles = rations.saturating_sub(pairs);
    let contested = (rations - 2 * doubles).min(pairs - doubles);
    Some(Allocation { doubles, contested })
}

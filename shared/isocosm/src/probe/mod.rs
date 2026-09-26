// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Ruling 115's competing instance for feeding when food is short, run two
//! ways under ruling 113: member by member through the core's exact
//! individual runner, and as a crowd, an exact-state histogram advanced by
//! random count draws. The competition's definition and the similitude
//! bounds live in the world's rules (ruling 218); the rounds that resolve a
//! competition live here until the core's scheduler runs them.

mod aggregate;
pub mod check;
mod crowd;
mod draws;
mod exact;
mod found;
pub mod readings;
#[cfg(test)]
mod tests;

pub use crate::rules::{Competition, Competitor, Similitude};
pub use crowd::{Crowd, Variant};
pub use exact::{ExactRun, run_exact};
pub use found::ProbeFounding;

use crate::{Result, schema::*, simulation::Genesis};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProbeWorld {
    pub genesis: Genesis,
    pub ticks: Tick,
}

impl ProbeWorld {
    /// The world's one competition. How several would interact within a
    /// round is not designed, so the probe refuses more than one.
    pub fn competition(&self) -> Result<&Competition> {
        let mut all = self.genesis.rules.competitions.values();
        match (all.next(), all.next()) {
            (Some(c), None) => Ok(c),
            _ => Err("the probe runs worlds with exactly one competition".into()),
        }
    }
    pub fn similitude(&self) -> Result<&Similitude> {
        self.genesis
            .rules
            .similitude
            .as_ref()
            .ok_or_else(|| "the world states no similitude".into())
    }
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

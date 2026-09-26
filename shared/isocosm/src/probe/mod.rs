// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Ruling 115's competing instance for feeding when food is short, run two
//! ways under ruling 113: member by member through the core's exact
//! individual runner, and as a crowd, an exact-state histogram advanced by
//! random count draws. The competition's definition and the similitude
//! bounds live in the world's rules (ruling 218), beside the mind its fights
//! strain (rulings 221 and 227); the rounds that resolve a competition live
//! here until the core's scheduler runs them.

mod aggregate;
pub mod check;
mod crowd;
mod draws;
mod exact;
pub mod fight;
mod found;
pub mod readings;
#[cfg(test)]
mod tests;

pub use crate::rules::{Competition, Competitor, Mind, Need, Similitude};
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
    pub fn mind(&self) -> Result<&Mind> {
        self.genesis
            .rules
            .mind
            .as_ref()
            .ok_or_else(|| "the world has no mind".into())
    }
}

/// How a contested ration first falls between two sides (ruling 206):
/// sharers split it, a contester takes it from a sharer, and two contesters
/// size each other up by reserve, only a close match escalating to a fight.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Meeting {
    /// Each side takes its gain.
    Settled([u64; 2]),
    Fight,
}

pub fn meet(c: &Competition, contest: [bool; 2], reserve: [u64; 2]) -> Meeting {
    let r = c.ration;
    match contest {
        [false, false] => Meeting::Settled([r / 2, r / 2]),
        [true, false] => Meeting::Settled([r, 0]),
        [false, true] => Meeting::Settled([0, r]),
        [true, true] if reserve[0].abs_diff(reserve[1]) <= c.margin => Meeting::Fight,
        [true, true] if reserve[0] > reserve[1] => Meeting::Settled([r, 0]),
        [true, true] => Meeting::Settled([0, r]),
    }
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

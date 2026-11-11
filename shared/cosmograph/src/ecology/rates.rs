// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Which rates a generated ecology runs at (ruling 907). The generator's
//! defaults are the stock rates and stay so; the A1 preset is the profile
//! that carried a played consumer lineage through three epochs on draws
//! (after-pass plan A1, Findings 2026-10-15). Its key is a plain technical
//! name; a player-facing name would come back as a naming question.

use serde::{Deserialize, Serialize};

// Life history is world time (case 1072): lifespans, maturity, the hazard,
// birth and growth periods are stated in minutes, A1's values at the minute
// it was tuned at, and founding reads them at the world's own unit.

/// Minutes an individual lives before the death hazard begins (906).
pub(super) const MATURE: u64 = 60;
/// The hazard per minute once mature: 2.5 percent, a mean wait of 40 minutes (906).
pub(super) const HAZARD_PER_MILLION: u32 = 25_000;
/// How often a played world's producers, consumers and decomposers decide,
/// in ticks (the execution plan's D19).
pub(crate) const DECIDES: [u64; 3] = [10, 2, 5];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Rates {
    /// The generator's rates before A1: birth gate `(stock_min + 2) * 3`,
    /// upkeep every 5 ticks, meals of 1, births every 7, lifespans 20 to 60.
    #[default]
    Stock,
    /// Declared at `stock_min` 8: birth gate 12 (consumers 9), consumers
    /// eat 3, are born every 14 ticks and pay upkeep every 10, decomposers
    /// pay upkeep every 10 and feed at every decision, and every lineage's
    /// lifespan, the ceiling under its death hazard, is drawn from 100 to 140.
    A1,
    /// The ecology in world time (cases 1077, 1078): each role's life,
    /// metabolism, feeding and growth from `world_time::role_time`.
    WorldTime,
}

impl Rates {
    pub fn is_stock(&self) -> bool {
        *self == Self::Stock
    }

    pub(super) fn birth_gate(self, stock_min: u64, consumer: bool) -> u64 {
        match (self, consumer) {
            (Self::Stock, _) => (stock_min + 2) * 3,
            (Self::A1 | Self::WorldTime, true) => 9,
            (Self::A1 | Self::WorldTime, false) => 12,
        }
    }

    pub(super) fn birth_period(self, consumer: bool) -> u64 {
        if self == Self::A1 && consumer { 14 } else { 7 }
    }

    /// Under A1 a decomposer, like a grazer, pays every 10 ticks. Upkeep runs
    /// on every body's own clock, and the dead it eats are few until the
    /// first hazard deaths (maturity, 60 minutes): a founding reserve of 10
    /// paid down every 5 ticks was gone at tick 50, the whole line with it.
    pub(super) fn upkeep_period(self, role: super::Role) -> u64 {
        if self == Self::A1 && role.level() != 0 { 10 } else { 5 }
    }

    pub(super) fn meal(self, consumer: bool) -> u64 {
        if self == Self::A1 && consumer { 3 } else { 1 }
    }

    /// How often a feeding process falls due on a body's own clock, in ticks.
    /// A decomposer decides every 5 ticks (`DECIDES`), so a period of 3 left
    /// it a bite every 15 and a bite of dead matter is small: under A1 it is
    /// due at every decision, as often as its decisions allow.
    pub(super) fn feed_period(self, role: super::Role) -> u64 {
        if self == Self::A1 && role.level() == 2 { 1 } else { 3 }
    }

    /// When in a tick a feeding process runs among the others (they run in
    /// priority order). A played world's moves run at 0, before a meal at 10,
    /// and a hungry body is due to move at every decision (some route of its
    /// site takes a tick), so a move left it in transit at the very decision
    /// its meal fell due: a hungry decomposer could never feed. Under A1 its
    /// meal runs first. (Not the moves' priority: a played lineage's choice
    /// scores a process by it, and raising it would bury the needs.)
    pub(super) fn feed_priority(self, role: super::Role) -> i32 {
        if self == Self::A1 && role.level() == 2 { -1 } else { 10 }
    }

    /// A lineage's lifespan in minutes: the base plus a draw of 0 to 40 (the
    /// ceiling the hazard works under).
    pub(super) fn lifespan(self, draw: u64) -> u64 {
        let base = if self == Self::A1 { 100 } else { 20 };
        base + draw % 41
    }
}

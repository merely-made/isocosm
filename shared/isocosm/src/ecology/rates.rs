// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Which rates a generated ecology runs at (ruling 907). The generator's
//! defaults are the stock rates and stay so; the A1 preset is the profile
//! that carried a played consumer lineage through three epochs on draws
//! (after-pass plan A1, Findings 2026-10-15). Its key is a plain technical
//! name; a player-facing name would come back as a naming question.

use serde::{Deserialize, Serialize};

/// Ticks an individual lives before the death hazard begins (906).
pub(super) const MATURE: u64 = 60;
/// The hazard per tick once mature: 2.5 percent, a mean wait of 40 ticks (906).
pub(super) const HAZARD_PER_MILLION: u32 = 25_000;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Rates {
    /// The generator's rates before A1: birth gate `(stock_min + 2) * 3`,
    /// upkeep every 5 ticks, meals of 1, births every 7, lifespans 20 to 60.
    #[default]
    Stock,
    /// Declared at `stock_min` 8: birth gate 12 (consumers 9), consumers
    /// eat 3, are born every 14 ticks and pay upkeep every 10, and every
    /// lineage's lifespan is drawn from 80 to 120.
    A1,
}

impl Rates {
    pub fn is_stock(&self) -> bool {
        *self == Self::Stock
    }

    pub(super) fn birth_gate(self, stock_min: u64, consumer: bool) -> u64 {
        match (self, consumer) {
            (Self::Stock, _) => (stock_min + 2) * 3,
            (Self::A1, true) => 9,
            (Self::A1, false) => 12,
        }
    }

    pub(super) fn birth_period(self, consumer: bool) -> u64 {
        if self == Self::A1 && consumer { 14 } else { 7 }
    }

    pub(super) fn upkeep_period(self, consumer: bool) -> u64 {
        if self == Self::A1 && consumer { 10 } else { 5 }
    }

    pub(super) fn meal(self, consumer: bool) -> u64 {
        if self == Self::A1 && consumer { 3 } else { 1 }
    }

    /// A lineage's lifespan: the base plus a draw of 0 to 40 (the ceiling the
    /// hazard works under).
    pub(super) fn lifespan(self, draw: u64) -> u64 {
        let base = if self == Self::A1 { 80 } else { 20 };
        base + draw % 41
    }
}

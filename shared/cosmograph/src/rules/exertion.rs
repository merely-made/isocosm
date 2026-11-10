// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The world rule of exertion (case 1234): ordinary activity is paid inside
//! upkeep, and acts past a body's ease spend. Unset, a world has no
//! exertion and hashes as before.

use crate::{Result, schema::Tick};
use serde::{Deserialize, Serialize};

/// Ordinary activity is paid inside upkeep; acts past a body's ease spend
/// (case 1234).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Exertion {
    /// Ticks one span of activity lasts.
    pub span: Tick,
    /// Chosen acts a span a body with no contracting cells makes at ease.
    pub ease: u32,
    /// Further acts at ease for a body all of whose cells contract.
    pub fit: u32,
    /// Matter each act past its ease spends.
    pub cost: u64,
}

impl Default for Exertion {
    /// A first cut, to be tuned.
    fn default() -> Self {
        // A span is a day at a minute a tick; four acts at ease, twelve for
        // a body all of muscle; each act beyond costs a matter.
        Self { span: 24, ease: 4, fit: 8, cost: 1 }
    }
}

/// The acts a span a body makes at ease: `ease` and `fit` in proportion to
/// its share of `contract` cells among its `cells` (case 1234).
#[must_use]
pub fn ease(exertion: &Exertion, contract: u64, cells: u64) -> u32 {
    if cells == 0 {
        return exertion.ease;
    }
    let share = u128::from(exertion.fit) * u128::from(contract.min(cells)) / u128::from(cells);
    exertion.ease.saturating_add(u32::try_from(share).unwrap_or(u32::MAX))
}

impl Exertion {
    /// # Errors
    /// Fails if a span lasts no ticks.
    pub fn validate(&self) -> Result<()> {
        if self.span == 0 {
            return Err("an exertion's span must last some ticks".into());
        }
        Ok(())
    }
}

// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! What ends an epoch, and how many epochs a world lives before anyone steps
//! in (rulings 451 and 452), moved from Mesocosm's `rules` and `deep_time`.

use crate::{Result, schema::Tick};
use serde::{Deserialize, Serialize};

/// A year of 365.25 days in microseconds, a timed epoch's default (ruling 451).
pub const YEAR_MICROSECONDS: u64 = 31_557_600_000_000;

/// Ticks in a year at a unit of `tick_microseconds`, never fewer than one.
pub fn year_ticks(tick_microseconds: u64) -> Tick {
    (YEAR_MICROSECONDS / tick_microseconds.max(1)).max(1)
}

/// What ends an epoch, Mesocosm's three rules (ruling 451). A timed epoch's
/// budget is the rules' `epoch_ticks`.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub enum EpochRule {
    /// The epoch ends when its budget of ticks is spent.
    #[default]
    Timed,
    /// The epoch ends when named world conditions are all met. Named and not
    /// built: a world holding it never ends an epoch.
    Gated,
    /// The epoch ends on demand and on nothing else.
    PlayerTriggered,
}

impl EpochRule {
    pub fn is_timed(&self) -> bool {
        matches!(self, Self::Timed)
    }
    /// Whether this rule is implemented; false for the named-only one.
    pub fn built(self) -> bool {
        !matches!(self, Self::Gated)
    }
    /// Whether a demand may end the epoch now: a timed epoch ends early on
    /// one, as in Mesocosm, and the gated one refuses it.
    pub fn admits_demand(self) -> bool {
        self.built()
    }
}

/// How many epochs a world lives before anyone steps in (ruling 452), its
/// world time following the epoch rule.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct DeepTimeSpan {
    pub epochs: u32,
}

impl DeepTimeSpan {
    pub fn is_bare(&self) -> bool {
        self.epochs == 0
    }
}

/// The hagiograph's form of the same count.
impl From<DeepTimeSpan> for hagiograph::DeepTime {
    fn from(span: DeepTimeSpan) -> Self {
        Self {
            epochs: span.epochs,
        }
    }
}

/// The most ticks a deep-time span may take before it is refused as stalled:
/// one epoch more than the span, so the ceiling never ends a run its rule is
/// closing. A span under a rule that never closes an epoch on its own is
/// refused before any tick.
pub fn deep_time_ceiling(rule: EpochRule, epoch_ticks: Tick, span: DeepTimeSpan) -> Result<Tick> {
    match (span.epochs, rule) {
        (0, _) => Ok(0),
        (epochs, EpochRule::Timed) if epoch_ticks > 0 => Ok(u64::from(epochs)
            .saturating_add(1)
            .saturating_mul(epoch_ticks)),
        (epochs, rule) => Err(format!(
            "a deep-time span of {epochs} epoch(s) cannot run under {rule:?}, which never \
             closes an epoch on its own"
        )),
    }
}

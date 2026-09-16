// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Deep time: a world lives through its own span before anyone steps in.
//! (isoscape family plan §2.4, step D7a)
//!
//! The hagiograph drives and this module is what it drives: Mesocosm's own
//! simulation, one idle tick at a time, with no hand, no checkpoint and no
//! answer to any question. Every event goes into the one history and every
//! boundary reckons into the one record, the way a played run's do, so a
//! generated past is made of the same stuff as a played one (Law C).

use std::fmt;

use crate::rules::EpochRule;
use crate::{History, Intent, World};

pub use hagiograph::{DeepTime, Handover};

/// Why deep time refused to run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeepTimeError {
    /// The world's epoch rule never closes an epoch on its own, so a span
    /// under it could only spin. Refused before any tick.
    NeverCloses { rule: EpochRule, epochs: u32 },
    /// The world stopped closing epochs inside the ceiling its rule allows.
    Stalled(hagiograph::DeepTimeError),
}

impl fmt::Display for DeepTimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NeverCloses { rule, epochs } => write!(
                f,
                "a deep-time span of {epochs} epoch(s) cannot run under {rule:?}, \
                 which never closes an epoch on its own"
            ),
            Self::Stalled(stalled) => write!(f, "{stalled}"),
        }
    }
}

impl std::error::Error for DeepTimeError {}

/// A world and the past it is making, as the hagiograph advances them.
struct Unheld<'a> {
    world: &'a mut World,
    history: &'a mut History,
    /// The epoch already reckoned: the runtime's `epoch_seen`, and started at
    /// the world's own epoch for the same reason a runtime with a past is.
    seen: u64,
}

impl hagiograph::Epochal for Unheld<'_> {
    /// The runtime's `absorb`, less everything that serves a hand: the tick's
    /// events into the past, its flows drained and dropped, and the reckoning
    /// if this tick closed an epoch.
    fn advance(&mut self) {
        self.world.apply(Intent::Idle);
        self.history.record_all(self.world.drain_events());
        drop(self.world.drain_flows());
        if self.world.epoch != self.seen {
            self.seen = self.world.epoch;
            self.world.reckon(self.history);
        }
    }

    fn epochs(&self) -> u64 {
        self.world.epoch
    }

    fn tick(&self) -> u64 {
        self.world.tick
    }
}

/// The most advances `epochs` may take under `rule`, or the refusal.
///
/// Under `Timed { ticks }` an epoch closes on the advance that spends its
/// budget, so `epochs × ticks` advances always suffice: exactly from a world
/// standing on a boundary, fewer from one mid-epoch or one whose budget was
/// shortened below its elapsed ticks. **The margin is one more budget**, so
/// the ceiling never ends a run the rule is closing, and a refusal means the
/// world overran its own rule by a whole epoch rather than by a tick.
/// Saturating, since a span and budget near their maxima overflow `u64`.
///
/// Every other rule never closes an epoch unasked: `Gated` has no condition
/// yet, `PlayerTriggered` waits for a demand deep time never makes, and a
/// zero-tick budget is never spent (`EpochRule::spent`).
fn ceiling(rule: EpochRule, epochs: u32) -> Result<u64, DeepTimeError> {
    match rule {
        EpochRule::Timed { ticks } if ticks > 0 => {
            Ok(u64::from(epochs).saturating_add(1).saturating_mul(ticks))
        },
        rule => Err(DeepTimeError::NeverCloses { rule, epochs }),
    }
}

impl World {
    /// Runs this world's own deep-time span and hands it over on the boundary
    /// its last epoch closed. (D7a)
    ///
    /// The span is the world's rule, [`crate::rules::WorldRules::deep_time`].
    /// The played body is released first, because deep time has no hand; see
    /// the release door in `world::release`. A zero span releases nothing,
    /// ticks nothing and returns a handover naming the same tick and epoch. A
    /// span under a rule that never closes an epoch is refused before any
    /// tick, with the world untouched.
    pub fn run_deep_time(&mut self, history: &mut History) -> Result<Handover, DeepTimeError> {
        let rules = self.rules();
        let span = rules.deep_time;
        let max_ticks = match span.epochs {
            0 => 0,
            epochs => {
                let max_ticks = ceiling(rules.epoch, epochs)?;
                self.release_control();
                max_ticks
            },
        };
        let mut unheld = Unheld {
            seen: self.epoch,
            world: self,
            history,
        };
        hagiograph::run(&mut unheld, span.into(), max_ticks).map_err(DeepTimeError::Stalled)
    }
}

#[cfg(test)]
mod tests;

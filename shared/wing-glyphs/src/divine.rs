// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! A fixed domain observation window, not a world evaluator or currency issuer.
//!
//! The product selects a metric and supplies one authoritative aggregate per
//! simulation tick, including explicit zeros. Units belong to that metric:
//! occurrences, entity-ticks, and another time integral are different choices.
//! This accumulator neither averages nor multiplies by window duration. It
//! cannot verify the supplied observations against a world it does not own.

use serde::Serialize;

/// A proposed selection. Once admitted, the window retains its own copy.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DomainChoice {
    pub metric_id: String,
    pub unit: String,
    pub start_tick: u64,
    pub duration_ticks: u64,
}

/// Product-configured duration bounds, checked at construction.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct WindowLimits {
    pub min_ticks: u64,
    pub max_ticks: u64,
}

/// A completed measurement. This conveys no authority to mint or spend.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DomainReceipt {
    pub metric_id: String,
    pub unit: String,
    pub start_tick: u64,
    /// Exclusive end of the selected interval, independent of settlement time.
    pub end_tick: u64,
    pub total: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WindowError {
    InvalidLimits,
    EmptyMetric,
    EmptyUnit,
    DurationOutsideLimits {
        duration: u64,
        min: u64,
        max: u64,
    },
    EndOverflow,
    WrongMetric {
        expected: String,
        supplied: String,
    },
    OutsideWindow {
        tick: u64,
        start: u64,
        end: u64,
    },
    OutOfOrder {
        expected_tick: u64,
        supplied_tick: u64,
    },
    TotalOverflow,
    TooEarly {
        at_tick: u64,
        end_tick: u64,
    },
    Incomplete {
        next_tick: u64,
        end_tick: u64,
    },
    AlreadySettled,
}

/// Constant-space sequential observation state.
///
/// No unchecked deserialization or selection setters are exposed. Serialized
/// state is inspection output, not an accepted restore format. Settlement is
/// one-time per instance; products still own durable identity, replay, and
/// deduplication of receipts across cloned/reconstructed application state.
#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct DomainWindow {
    choice: DomainChoice,
    end_tick: u64,
    next_tick: u64,
    total: u64,
    settled: bool,
}

impl DomainWindow {
    pub fn new(choice: DomainChoice, limits: WindowLimits) -> Result<Self, WindowError> {
        if limits.min_ticks == 0 || limits.min_ticks > limits.max_ticks {
            return Err(WindowError::InvalidLimits);
        }
        if choice.metric_id.trim().is_empty() {
            return Err(WindowError::EmptyMetric);
        }
        if choice.unit.trim().is_empty() {
            return Err(WindowError::EmptyUnit);
        }
        if !(limits.min_ticks..=limits.max_ticks).contains(&choice.duration_ticks) {
            return Err(WindowError::DurationOutsideLimits {
                duration: choice.duration_ticks,
                min: limits.min_ticks,
                max: limits.max_ticks,
            });
        }
        let end_tick = choice
            .start_tick
            .checked_add(choice.duration_ticks)
            .ok_or(WindowError::EndOverflow)?;
        Ok(Self {
            next_tick: choice.start_tick,
            choice,
            end_tick,
            total: 0,
            settled: false,
        })
    }

    pub fn choice(&self) -> &DomainChoice {
        &self.choice
    }
    pub fn end_tick(&self) -> u64 {
        self.end_tick
    }
    pub fn next_tick(&self) -> u64 {
        self.next_tick
    }
    pub fn observed_ticks(&self) -> u64 {
        self.next_tick - self.choice.start_tick
    }
    pub fn total(&self) -> u64 {
        self.total
    }
    pub fn is_settled(&self) -> bool {
        self.settled
    }

    /// Accept exactly the next tick. Every refusal leaves all state unchanged.
    /// An absence of occurrences must be supplied as zero, not skipped.
    pub fn observe(&mut self, metric_id: &str, tick: u64, amount: u64) -> Result<(), WindowError> {
        if self.settled {
            return Err(WindowError::AlreadySettled);
        }
        if metric_id != self.choice.metric_id {
            return Err(WindowError::WrongMetric {
                expected: self.choice.metric_id.clone(),
                supplied: metric_id.into(),
            });
        }
        if tick < self.choice.start_tick || tick >= self.end_tick {
            return Err(WindowError::OutsideWindow {
                tick,
                start: self.choice.start_tick,
                end: self.end_tick,
            });
        }
        if tick != self.next_tick {
            return Err(WindowError::OutOfOrder {
                expected_tick: self.next_tick,
                supplied_tick: tick,
            });
        }
        let total = self
            .total
            .checked_add(amount)
            .ok_or(WindowError::TotalOverflow)?;
        // tick < end_tick <= u64::MAX proves this increment representable.
        self.next_tick = tick + 1;
        self.total = total;
        Ok(())
    }

    /// Settle at or after the fixed end, once every tick has been observed.
    /// Waiting longer never adds unobserved time or increases the total.
    pub fn settle(&mut self, at_tick: u64) -> Result<DomainReceipt, WindowError> {
        if self.settled {
            return Err(WindowError::AlreadySettled);
        }
        if at_tick < self.end_tick {
            return Err(WindowError::TooEarly {
                at_tick,
                end_tick: self.end_tick,
            });
        }
        if self.next_tick != self.end_tick {
            return Err(WindowError::Incomplete {
                next_tick: self.next_tick,
                end_tick: self.end_tick,
            });
        }
        let receipt = DomainReceipt {
            metric_id: self.choice.metric_id.clone(),
            unit: self.choice.unit.clone(),
            start_tick: self.choice.start_tick,
            end_tick: self.end_tick,
            total: self.total,
        };
        self.settled = true;
        Ok(receipt)
    }
}

#[cfg(test)]
#[path = "divine_tests.rs"]
mod tests;

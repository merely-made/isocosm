// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The wait ladder (case 1075): a span of any length in world time, its
//! named rungs a convenience, stepped tick by tick on the one clock until the
//! execution plan's E7 and E8 let the unwatched aggregate take a coarser step.
//! The VTT waits a span as consented downtime; any world can be advanced by
//! one, as a seed world is.

use super::epoch::{MINUTE_MICROSECONDS, YEAR_MICROSECONDS};
use crate::{Result, schema::Tick};
use serde::{Deserialize, Serialize};

/// A named length on the ladder, fixed in world time: a month is a twelfth of
/// the 365.25-day year the epoch uses.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Rung {
    Minute,
    Day,
    Week,
    Month,
    Year,
    Decade,
    Century,
    Millennium,
}

impl Rung {
    pub const ALL: [Rung; 8] = [Rung::Minute, Rung::Day, Rung::Week, Rung::Month, Rung::Year,
        Rung::Decade, Rung::Century, Rung::Millennium];

    pub fn microseconds(self) -> u64 {
        const DAY: u64 = 1_440 * MINUTE_MICROSECONDS;
        match self {
            Rung::Minute => MINUTE_MICROSECONDS,
            Rung::Day => DAY,
            Rung::Week => 7 * DAY,
            Rung::Month => YEAR_MICROSECONDS / 12,
            Rung::Year => YEAR_MICROSECONDS,
            Rung::Decade => 10 * YEAR_MICROSECONDS,
            Rung::Century => 100 * YEAR_MICROSECONDS,
            Rung::Millennium => 1_000 * YEAR_MICROSECONDS,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Rung::Minute => "minute",
            Rung::Day => "day",
            Rung::Week => "week",
            Rung::Month => "month",
            Rung::Year => "year",
            Rung::Decade => "decade",
            Rung::Century => "century",
            Rung::Millennium => "millennium",
        }
    }
}

/// A wait: so many of one rung.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Span {
    pub count: u64,
    pub rung: Rung,
}

impl Span {
    pub fn new(count: u64, rung: Rung) -> Self {
        Self { count, rung }
    }

    /// The span in microseconds of world time.
    pub fn microseconds(self) -> Result<u64> {
        self.count.checked_mul(self.rung.microseconds()).ok_or_else(|| "span overflow".into())
    }

    /// The ticks the span takes at a unit of `tick_microseconds`, rounded up,
    /// so a wait never ends short of its span.
    pub fn ticks(self, tick_microseconds: u64) -> Result<Tick> {
        if tick_microseconds == 0 {
            return Err("zero world clock unit".into());
        }
        Ok(self.microseconds()?.div_ceil(tick_microseconds))
    }
}

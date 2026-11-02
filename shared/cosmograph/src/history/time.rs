// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Time across saves and waits: how a world's life history was founded
//! (case 1073) and waiting a span on the one clock (case 1075).

use super::*;

/// How a world's life history was founded. A world founded from v5 on states
/// it in world time, read at its tick unit (case 1072); a v3 or v4 world's
/// counts were ticks. At the minute every game keeps, the two are the same
/// numbers; the meaning is kept so an older world replays as it was made.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LifeHistory {
    Ticks,
    #[default]
    WorldTime,
}

impl Session {
    /// Waits a span of world time on the one clock, tick by tick (case 1075).
    pub fn advance_span(&mut self, span: crate::rules::Span) -> Result<Work> {
        let ticks = span.ticks(self.sim.genesis.rules.tick_microseconds())?;
        self.advance(ticks)
    }
}

// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Bounded ecology windows over the native flow record (the record family,
//! 750): the producer stand's change and what mouths took from it over a
//! judgement window, and births and deaths over a longer one. Derivable
//! from a save, so kept beside the world, never in it.

use isocosm::directing::interim::Happening;
use isocosm::flows::{Flow, Holder};
use isocosm::simulation::Simulation;
use serde::{Deserialize, Serialize};

/// Rounds the long window keeps.
pub const RETENTION_TICKS: usize = 240;
/// Rounds the stand is judged over.
pub const JUDGEMENT_TICKS: usize = 60;

/// The native key the producer kingdom's flows carry.
const FLORA: &str = "kingdom:flora";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
struct Totals {
    stand_change: i64,
    grazed: u64,
    born: u32,
    died: u32,
}

fn flora(kind: &Option<isocosm::flows::Kind>) -> bool {
    kind.as_ref().is_some_and(|k| k.kingdom == FLORA)
}

impl Totals {
    fn of(living: (u64, u64), flows: &[Flow]) -> Self {
        let mut totals = Self {
            born: living.1.saturating_sub(living.0) as u32,
            died: living.0.saturating_sub(living.1) as u32,
            ..Self::default()
        };
        for flow in flows.iter().filter(|f| !f.internal()) {
            let amount = flow.amount.saturating_mul(flow.count);
            let body = |h: Holder| h.body().is_some();
            if flora(&flow.from_kind) && body(flow.from.0) {
                totals.stand_change -= amount as i64;
                if body(flow.to.0) && !flora(&flow.to_kind) {
                    totals.grazed += amount;
                }
            }
            if flora(&flow.to_kind) && body(flow.to.0) {
                totals.stand_change += amount as i64;
            }
        }
        totals
    }
}

/// What the windows read, with the windows they cover.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Trend {
    pub replacement_ticks: u64,
    pub born: u32,
    pub died: u32,
    pub stand_ticks: u64,
    pub stand_change: i64,
    pub grazed: u64,
    /// Consecutive rounds the stand has been shrinking.
    pub shortfall_ticks: u64,
}

impl Trend {
    /// Whether the stand has shrunk for a whole judgement window.
    pub fn warns(&self) -> bool {
        self.shortfall_ticks >= JUDGEMENT_TICKS as u64
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FlowWindows {
    ring: Vec<Totals>,
    absorbed: u64,
    shortfall_ticks: u64,
    living: u64,
}

impl Default for FlowWindows {
    fn default() -> Self {
        Self::new()
    }
}

impl FlowWindows {
    pub fn new() -> Self {
        Self {
            ring: Vec::with_capacity(RETENTION_TICKS),
            absorbed: 0,
            shortfall_ticks: 0,
            living: 0,
        }
    }

    /// Takes one round: its flows, and the living count it left.
    pub fn absorb(&mut self, sim: &Simulation, _happenings: &[Happening], flows: &[Flow]) {
        let groups = sim.state().population.groups.values();
        let living = groups.filter(|g| g.entity.alive).count() as u64;
        let first = self.absorbed == 0;
        let before = if first { living } else { self.living };
        self.living = living;
        let totals = Totals::of((before, living), flows);
        let slot = (self.absorbed % RETENTION_TICKS as u64) as usize;
        if self.ring.len() < RETENTION_TICKS {
            self.ring.push(totals);
        } else {
            self.ring[slot] = totals;
        }
        self.absorbed += 1;
        self.shortfall_ticks = if self.stand().0 < 0 {
            self.shortfall_ticks + 1
        } else {
            0
        };
    }

    pub fn retained(&self) -> u64 {
        self.absorbed.min(RETENTION_TICKS as u64)
    }

    fn recent(&self, ticks: usize) -> impl Iterator<Item = &Totals> {
        let held = self.ring.len();
        let want = ticks.min(held);
        (0..want).map(move |back| {
            let index = ((self.absorbed % held as u64) as usize + held - back - 1) % held;
            &self.ring[index]
        })
    }

    fn stand(&self) -> (i64, u64) {
        self.recent(JUDGEMENT_TICKS)
            .fold((0, 0), |(c, g), t| (c + t.stand_change, g + t.grazed))
    }

    pub fn trend(&self) -> Trend {
        let (born, died) = self
            .recent(RETENTION_TICKS)
            .fold((0, 0), |(b, d), t| (b + t.born, d + t.died));
        let (stand_change, grazed) = self.stand();
        Trend {
            replacement_ticks: self.retained(),
            born,
            died,
            stand_ticks: self.retained().min(JUDGEMENT_TICKS as u64),
            stand_change,
            grazed,
            shortfall_ticks: self.shortfall_ticks,
        }
    }
}

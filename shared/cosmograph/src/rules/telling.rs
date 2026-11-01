// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! What a world rules of telling (case 1066): the baseline belief draw's
//! numbers, tuned in play; whether a visit leaves a note about its place;
//! and how far an untold event may lie from what a hearer knows to be worth
//! offering it.

use serde::{Deserialize, Serialize};

/// The note a visit leaves on its visitor, and a telling copies, where the
/// world keeps place notes. Its object is the site's id.
pub const PLACE_NOTE: &str = "sim:place";

/// A world's telling rules. Absent means these defaults, which are the
/// numbers telling was built with, so nothing replays differently.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Telling {
    /// The baseline chance, in thousandths, that a hearer takes a telling.
    pub base: i64,
    /// Added when teller and hearer hold any relation, either way.
    pub related: i64,
    /// Taken off per point of the hearer's caution.
    pub per_caution: i64,
    /// The caution read is clamped to plus or minus this.
    pub caution_clamp: i64,
    /// The chance stays within these, never certain either way.
    pub floor: i64,
    pub ceiling: i64,
    /// A visit leaves a place note on its visitor, and telling about a place
    /// copies one. Off in worlds founded before it: turning it on adds notes,
    /// so existing replays would move.
    #[serde(default)]
    pub place_notes: bool,
    /// How many hops of cause or shared party an untold event may lie from
    /// what the hearer knows and still be offered to tell.
    #[serde(default = "default_hops")]
    pub event_hops: u32,
}

fn default_hops() -> u32 { DEFAULT_HOPS }

/// The offered events' default reach; see `Simulation::tellable_events`.
pub const DEFAULT_HOPS: u32 = 2;

impl Default for Telling {
    fn default() -> Self {
        Self { base: 700, related: 150, per_caution: 2, caution_clamp: 100, floor: 50, ceiling: 950,
            place_notes: false, event_hops: DEFAULT_HOPS }
    }
}

impl Telling {
    pub fn validate(&self, rules: &super::Rules) -> crate::Result<()> {
        if !(0..=1000).contains(&self.floor) || self.floor > self.ceiling || self.ceiling > 1000
            || self.caution_clamp < 0 {
            return Err("telling chances must lie within 0 to 1000 with floor below ceiling".into());
        }
        if self.place_notes && !rules.note_kinds.contains(PLACE_NOTE) {
            return Err("place notes need the world to keep the sim:place note kind".into());
        }
        Ok(())
    }
}

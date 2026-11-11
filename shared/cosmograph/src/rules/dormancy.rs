// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The world rule of dormancy (cases 1192 to 1197): a producer's young are
//! propagules, bodies that carry a dormant trait and their own matter. Gated
//! processes skip them, their upkeep is a fraction of the active rate, and
//! each wakes on a season's draw when its site's light share is enough for
//! its mass. Unset, no body is ever dormant and a world hashes as before.

use super::{Process, Query, Rules};
use crate::{Result, schema::{Entity, Key}};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// The seed share a producer's birth gave before dormancy (case 1114): the
/// weight a waking light share is stated against (case 1195).
pub const REFERENCE_SEED: u64 = 10;

/// The flat rate sets' first cut of `Dormancy::crowd`.
const CROWD: u64 = 12;

fn crowd() -> u64 {
    CROWD
}

/// How a producer's young lie dormant (cases 1192 to 1197).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Dormancy {
    /// The trait a dormant body carries (case 1197); an awake one lacks it.
    pub key: Key,
    /// A dormant body's upkeep as a share (numerator, denominator) of the
    /// active rate (case 1196).
    pub upkeep: [u64; 2],
    /// The share of a site's dormant stock that wakes each season (case 1195),
    /// as each body's own keyed chance.
    pub wake: [u64; 2],
    /// A season, in minutes, as world time counts them: the producers'
    /// seeding period. Read at the world's tick unit when processes are made.
    pub season: u64,
    /// A season in ticks, where the flat rate sets run it: they have no day
    /// to count, and a season of days outlasts any life they state. Unset,
    /// the season is read from `season` at the world's tick unit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub season_ticks: Option<u64>,
    /// How many plants a site may hold, at the reference seed share, for a
    /// seed there to wake under the flat rate sets, which have no light
    /// share to sense; a heavier seed wakes at a greater count. World time
    /// wakes on light and does not read it.
    #[serde(default = "crowd")]
    pub crowd: u64,
    /// Whether waking starts the body's age afresh, so a flat lineage's
    /// lifespan is lived awake. World time keeps its own ages.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub renew: bool,
    /// The range a lineage's seed share is drawn within, both ends included
    /// (case 1194).
    pub seed: [u64; 2],
    /// The birth processes whose young are propagules, each with its
    /// lineage's drawn seed share. A strain keeps its parent's birth process,
    /// so the share is inherited. Filled when the world is founded.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub seeds: BTreeMap<Key, u64>,
}

impl Default for Dormancy {
    /// A first cut, to be tuned.
    fn default() -> Self {
        Self {
            key: "stage:dormant".into(),
            // A tenth of the active upkeep rate.
            upkeep: [1, 10],
            // A quarter of the stock each season.
            wake: [1, 4],
            // Ninety days, as a producer's seeding period counts a season.
            season: 90 * 1440,
            season_ticks: None,
            crowd: CROWD,
            renew: false,
            // About the fixed ten, from half to twice.
            seed: [5, 20],
            seeds: BTreeMap::new(),
        }
    }
}

impl Dormancy {
    /// The rule for a flat-rate ecology (stock or A1): the defaults, with a
    /// season of forty ticks, which a lineage living a hundred and more
    /// ticks sees about three times, and waking that renews the age.
    #[must_use]
    pub fn flat() -> Self {
        Self { season_ticks: Some(40), renew: true, ..Self::default() }
    }

    /// The plants a site may hold for a seed of `share` matter to wake there
    /// (the flat sets' stand-in for the light share of case 1195): rises
    /// with the seed's weight, so heavier seeds wake where it is more crowded.
    #[must_use]
    pub fn crowding(&self, share: u64) -> u64 {
        (self.crowd.saturating_mul(share) / REFERENCE_SEED).max(1)
    }

    /// # Errors
    /// Fails if a share has no denominator or is more than whole or the
    /// upkeep share is nought, the season lasts no time, the seed range is
    /// empty or starts at nought, the trait is not one the world declares,
    /// or a drawn seed share is nought.
    pub fn validate(&self, rules: &Rules) -> Result<()> {
        if [self.upkeep, self.wake].iter().any(|[n, d]| *d == 0 || n > d) {
            return Err("dormancy's shares need denominators and cannot exceed a whole".into());
        }
        if self.upkeep[0] == 0 {
            return Err("dormancy's upkeep cannot be nought of the active rate".into());
        }
        if self.season == 0 || self.season_ticks == Some(0) {
            return Err("dormancy's season must last some time".into());
        }
        if self.crowd == 0 {
            return Err("dormancy's crowding count cannot be nought".into());
        }
        if self.seed[0] == 0 || self.seed[0] > self.seed[1] {
            return Err("dormancy's seed range must be a nonempty range above nought".into());
        }
        if !rules.traits.contains(&self.key) {
            return Err("dormancy's trait must be one the world declares".into());
        }
        if self.seeds.values().any(|share| *share == 0) {
            return Err("dormancy's seed shares cannot be nought".into());
        }
        Ok(())
    }

    /// Lineage `index`'s seed share, drawn within the range by the world's
    /// seed as a scatter propensity is (case 1194).
    #[must_use]
    pub fn drawn(&self, seed: u64, index: u32) -> u64 {
        let [low, high] = self.seed;
        low + crate::draw(seed, "seed-share", &[u64::from(index)]) % (high - low + 1)
    }

    /// The light share a seed of `share` matter needs to wake, given what a
    /// seedling of the reference share needs (case 1195): lower as the seed
    /// is heavier.
    #[must_use]
    pub fn waking(light: u64, share: u64) -> u64 {
        light.saturating_mul(REFERENCE_SEED) / share.max(1)
    }

    /// A body's chance in a draw of the wake share, in millionths.
    #[must_use]
    pub fn wakes(&self) -> u32 {
        let [n, d] = self.wake;
        u32::try_from(n.saturating_mul(1_000_000) / d).unwrap_or(1_000_000)
    }
}

impl Rules {
    /// Whether `e` is a propagule: the world has dormancy and it carries the
    /// dormant trait (case 1193).
    #[must_use]
    pub fn dormant(&self, e: &Entity) -> bool {
        self.dormancy.as_ref().is_some_and(|d| e.traits.contains(&d.key))
    }

    /// The dormant trait where `p` passes dormant bodies by: the world has
    /// dormancy and `p` does not itself require the trait (case 1197).
    #[must_use]
    pub fn passes_dormant(&self, p: &Process) -> Option<&Key> {
        let key = &self.dormancy.as_ref()?.key;
        let wanted = |q: &Query| matches!(q, Query::Trait { who: super::Binding::Actor, key: k } if k == key);
        (!p.requires.iter().any(wanted)).then_some(key)
    }
}

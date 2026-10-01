// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Founding presets (ruling 272): Mesocosm's three authored worlds, each a
//! question and the pressures that answer it, set on every site at founding.

use crate::schema::{Id, Key, Site};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// The seven pressures as site condition keys, a world's strength in each
/// nominally 0 to 10 and nothing clamping it.
pub const PRESSURES: [&str; 7] = [
    "pressure:gravity",
    "pressure:cold",
    "pressure:drought",
    "pressure:dark",
    "pressure:corrosive",
    "pressure:crowding",
    "pressure:predation",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Preset {
    TidalShelf,
    HeavyDeep,
    LongYear,
}

impl Preset {
    pub const ALL: [Preset; 3] = [Preset::TidalShelf, Preset::HeavyDeep, Preset::LongYear];

    /// The question the world was authored from.
    pub fn question(self) -> &'static str {
        match self {
            Self::TidalShelf => "what lives where the light never moves?",
            Self::HeavyDeep => "what shape is life when the air is heavy enough to swim in?",
            Self::LongYear => "what survives a year that tries to kill it twice?",
        }
    }

    /// The pressures it puts on what lives there, as Mesocosm authored them.
    pub fn forces(self) -> &'static [(&'static str, i64)] {
        match self {
            Self::TidalShelf => &[
                ("pressure:crowding", 8),
                ("pressure:dark", 5),
                ("pressure:cold", 4),
                ("pressure:predation", 3),
            ],
            Self::HeavyDeep => &[
                ("pressure:gravity", 9),
                ("pressure:corrosive", 6),
                ("pressure:dark", 5),
            ],
            Self::LongYear => &[
                ("pressure:cold", 8),
                ("pressure:drought", 7),
                ("pressure:crowding", 4),
            ],
        }
    }

    /// The strength of one pressure here; an absent pressure is none.
    pub fn strength(self, key: &str) -> i64 {
        self.forces()
            .iter()
            .find(|(k, _)| *k == key)
            .map_or(0, |(_, s)| *s)
    }

    /// Declares the seven pressures and sets each site's strength in them.
    pub fn apply(self, conditions: &mut BTreeSet<Key>, sites: &mut BTreeMap<Id, Site>) {
        conditions.extend(PRESSURES.iter().map(|k| k.to_string()));
        for site in sites.values_mut() {
            for key in PRESSURES {
                site.conditions.insert(key.into(), self.strength(key));
            }
        }
    }
}

// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Founding presets (ruling 272): Mesocosm's three authored worlds, each a
//! question and the pressures that answer it, set on every site at founding.
//! Mesocosm's legacy `pressure` module, re-expressed here whole.

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

    /// The world's plain name, as Mesocosm authored it.
    pub fn name(self) -> &'static str {
        match self {
            Self::TidalShelf => "the tidal shelf",
            Self::HeavyDeep => "the heavy deep",
            Self::LongYear => "the long year",
        }
    }

    /// How the world answers its question, by grammar family: authoring
    /// notes, read by nothing.
    pub fn parameters(self) -> &'static [(&'static str, &'static str)] {
        match self {
            Self::TidalShelf => &[
                (
                    "energy schedule",
                    "tidally locked; no day, no year, a fixed terminator",
                ),
                ("medium", "thin air, standing meltwater along the ring"),
                (
                    "chemistry",
                    "solvent liquid only within the band; ice on one side, vapour on the other",
                ),
                (
                    "topology",
                    "one continuous habitable ring, dark side and bright side both lethal",
                ),
                (
                    "cycles",
                    "none — the defining absence, so nothing is seasonal and nothing gets a reprieve",
                ),
                (
                    "initial ecology",
                    "producers anchored to the light edge, consumers working the shade",
                ),
            ],
            Self::HeavyDeep => &[
                (
                    "energy schedule",
                    "a dim red sun, most light scattered before it lands",
                ),
                (
                    "medium",
                    "roughly three gravities; atmosphere dense enough to be buoyant in",
                ),
                (
                    "chemistry",
                    "reducing atmosphere with acidic aerosols; abundant solvent",
                ),
                (
                    "topology",
                    "vertical stratification -- everything is a layer, and layers are the niches",
                ),
                (
                    "cycles",
                    "slow, deep convection storms that move whole layers",
                ),
                (
                    "initial ecology",
                    "floaters and anchored filterers; nothing walks far",
                ),
            ],
            Self::LongYear => &[
                (
                    "energy schedule",
                    "eccentric orbit; a short fierce summer and a long deep winter",
                ),
                (
                    "medium",
                    "thin air, thickening as volatiles boil off each summer",
                ),
                (
                    "chemistry",
                    "solvent locked as ice for most of the cycle, then abundant, then gone",
                ),
                (
                    "topology",
                    "basins that hold meltwater and highlands that never thaw",
                ),
                (
                    "cycles",
                    "the defining feature -- freeze, flood, bloom, desiccation, freeze",
                ),
                (
                    "initial ecology",
                    "everything reproduces explosively in the bloom and waits out the rest",
                ),
            ],
        }
    }

    /// Total pressure, to check the worlds are hard in different ways.
    pub fn severity(self) -> i64 {
        self.forces().iter().map(|(_, s)| s).sum()
    }

    /// The pressure the world is about: its strongest.
    pub fn defining(self) -> Option<&'static str> {
        self.forces()
            .iter()
            .max_by_key(|(_, s)| *s)
            .map(|(k, _)| *k)
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

/// Mesocosm's checks on its three authored worlds, ported with them.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_authored_world_answers_a_question() {
        for world in Preset::ALL {
            assert!(world.question().ends_with('?'), "{}", world.name());
            assert!(world.parameters().len() >= 5, "{}", world.name());
            assert!(!world.forces().is_empty(), "{}", world.name());
        }
    }

    #[test]
    fn the_three_worlds_are_hard_in_different_ways() {
        let defining: BTreeSet<_> = Preset::ALL.iter().filter_map(|w| w.defining()).collect();
        assert_eq!(
            defining.len(),
            3,
            "each world is about a different pressure"
        );
        for world in Preset::ALL {
            assert!(
                world.forces().len() < PRESSURES.len(),
                "{} leaves an axis alone",
                world.name()
            );
            assert!(world.forces().iter().all(|(k, _)| PRESSURES.contains(k)));
        }
        let severities: BTreeSet<i64> = Preset::ALL.iter().map(|w| w.severity()).collect();
        assert!(
            severities.iter().all(|s| *s > 10),
            "all three are demanding"
        );
        assert!(severities.len() > 1, "and not equally");
    }
}

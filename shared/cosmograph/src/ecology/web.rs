// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Which ways of living a generated ecology's lineages take, in turn. The
//! three trophic levels are the default; a web of diets splits consumers by
//! what they eat. Every consumer keeps the consumer level, so a region's
//! slots are unchanged, and its diet is what its feeding admits, read from
//! what it eats rather than declared (case 1055's measured reading).

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Web {
    /// Producer, consumer, decomposer: the generator's ecology before diets.
    #[default]
    Levels,
    /// Producer, grazer, decomposer, predator, omnivore.
    Diets,
}

impl Web {
    pub fn is_levels(&self) -> bool {
        *self == Self::Levels
    }

    /// The lineage a founding cohort joins, by `draw`: evenly in a web of
    /// levels, as the generator always drew; by each role's share in a web
    /// of diets.
    pub(crate) fn founder(self, lineages: u32, draw: u64, rates: super::Rates) -> u64 {
        let world_time = rates == super::Rates::WorldTime;
        if self == Self::Levels && !world_time {
            return draw % u64::from(lineages);
        }
        // A world-time founding is a biomass pyramid (1078).
        let share = |i: u32| if world_time { super::world_time::founders(Role::of(self, i)) }
            else { Role::of(self, i).founders() };
        let total: u64 = (0..lineages).map(share).sum();
        let mut left = draw % total.max(1);
        for i in 0..lineages {
            if left < share(i) {
                return u64::from(i);
            }
            left -= share(i);
        }
        0
    }
}

/// A generated lineage's way of living.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Role {
    Producer,
    /// Eats living producers: the consumer of a web of levels.
    Grazer,
    /// Eats the dead of every other lineage.
    Decomposer,
    /// Legacy selectors eat living grazers and omnivores. Ported world-time
    /// founding admits Consumer tissue, including other predators.
    Predator,
    /// Eats living producers and grazers.
    Omnivore,
}

impl Role {
    /// Lineage `i`'s role in `web`.
    pub(crate) fn of(web: Web, i: u32) -> Self {
        use Role::*;
        match web {
            Web::Levels => [Producer, Grazer, Decomposer][i as usize % 3],
            Web::Diets => [Producer, Grazer, Decomposer, Predator, Omnivore][i as usize % 5],
        }
    }

    /// Its trophic level: 0 producer, 1 consumer, 2 decomposer, the order
    /// of the levels, kingdoms and paces the generator names.
    pub(crate) fn level(self) -> usize {
        match self {
            Self::Producer => 0,
            Self::Decomposer => 2,
            Self::Grazer | Self::Predator | Self::Omnivore => 1,
        }
    }

    pub(crate) fn consumer(self) -> bool {
        self.level() == 1
    }

    /// Its share of a web of diets' founders: a pyramid, the food beneath
    /// outnumbering what eats it.
    fn founders(self) -> u64 {
        match self {
            Self::Producer => 5,
            Self::Grazer => 3,
            Self::Decomposer => 2,
            Self::Predator | Self::Omnivore => 1,
        }
    }

    /// The legacy founding selector. An actual ported meal and its food
    /// counts use admitted provenance, not this role lookup.
    pub(crate) fn eats(self, other: Role) -> bool {
        use Role::*;
        match self {
            Producer => false,
            Grazer => other == Producer,
            Predator => matches!(other, Grazer | Omnivore),
            Omnivore => matches!(other, Producer | Grazer),
            Decomposer => true,
        }
    }
}

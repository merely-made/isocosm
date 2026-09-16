// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Mesocosm's axis set over the hagiograph's standing record.
//!
//! [`Feat`] and [`Scale`] are this game's own vocabulary: six kinds of thing a
//! lineage can do, at three scales. `Symbiosis` and `Construction` stay
//! untouched today, deliberately — nothing in the simulation yet gives to
//! another creature or changes the enclosure itself, and `score.rs` (which
//! computes the readings [`World::reckon`](crate::world::World::reckon) notes
//! here) explains why that is worth leaving unwritten rather than noted as a
//! zero.
//!
//! The mechanism lives in [`hagiograph`], mere's history organ, since the
//! isoscape family plan (rulings 8 to 10): the join-semilattice merge, the
//! rule that a mark keeps its holders rather than forgetting who, and the
//! abnormality query itself. [`WorldRecord`] is a thin newtype over
//! [`hagiograph::Record`] keyed by `(Feat, Scale)` and
//! [`SpeciesId`](crate::body::SpeciesId), `#[serde(transparent)]` so its
//! postcard bytes are the inner record's own.

use hagiograph::Record;
use serde::{Deserialize, Serialize};

use crate::body::SpeciesId;

/// The kind of thing a lineage did.
///
/// Harmony and domination sit beside each other rather than opposing, so a
/// lineage can score highly on both. That is a real ecological posture: made
/// itself indispensable and dangerous at once.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Feat {
    /// Biomass gained.
    Growth,
    /// Taken from others.
    Predation,
    /// Given to, or depended on by, others.
    Symbiosis,
    /// Survived what others did not.
    Endurance,
    /// Reached where others had not.
    Spread,
    /// Changed the world rather than living in it.
    Construction,
}

impl Feat {
    pub const ALL: [Feat; 6] = [
        Feat::Growth,
        Feat::Predation,
        Feat::Symbiosis,
        Feat::Endurance,
        Feat::Spread,
        Feat::Construction,
    ];
}

/// How far a feat reached.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Scale {
    Local,
    Regional,
    Worldwide,
}

/// A high-water mark, and who stands at it. The hagiograph's generic mark,
/// fixed to this game's holder type.
pub type Mark = hagiograph::Mark<SpeciesId>;

/// Everything a world has seen, keyed by what and how far.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct WorldRecord(Record<(Feat, Scale), SpeciesId>);

impl WorldRecord {
    pub fn new() -> Self {
        Self::default()
    }

    /// The standing mark for one axis, if anyone has set one.
    pub fn standing(&self, feat: Feat, scale: Scale) -> Option<&Mark> {
        self.0.standing(&(feat, scale))
    }

    /// Whether this would be the first, or the best, anyone has managed.
    ///
    /// **The abnormality query.** One comparison, which is the whole reason
    /// the record is a handful of integers rather than an index.
    pub fn is_unprecedented(&self, feat: Feat, scale: Scale, value: i64) -> bool {
        self.0.is_unprecedented(&(feat, scale), value)
    }

    /// Whether anyone has ever done this at all, at any magnitude.
    ///
    /// A goal generator wants this: "something no species on the planet has
    /// done" is a different question from "more than anyone has done."
    pub fn untouched(&self, feat: Feat, scale: Scale) -> bool {
        self.0.untouched(&(feat, scale))
    }

    /// Records what a lineage did. Returns whether it took the record.
    pub fn note(&mut self, feat: Feat, scale: Scale, value: i64, by: SpeciesId) -> bool {
        self.0.note((feat, scale), value, by)
    }

    /// Joins another world's record into this one.
    ///
    /// Order-independent and repeatable, so a moot can fold records from peers
    /// as they arrive without sequencing them.
    pub fn merge(&mut self, other: &WorldRecord) {
        self.0.merge(&other.0);
    }

    /// Axes anyone has reached, in a deterministic order.
    pub fn axes(&self) -> impl Iterator<Item = (Feat, Scale)> + '_ {
        self.0.axes().copied()
    }

    /// How much of the possible record this world has filled.
    ///
    /// A world with nothing left untouched has no significance left to offer,
    /// which is the storyteller's actual brief: keep this below one by opening
    /// possibility rather than by manufacturing calamity.
    pub fn filled(&self) -> usize {
        self.0.filled()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;

    const A: SpeciesId = SpeciesId(1);
    const B: SpeciesId = SpeciesId(2);
    const C: SpeciesId = SpeciesId(3);

    fn record(entries: &[(Feat, Scale, i64, SpeciesId)]) -> WorldRecord {
        let mut record = WorldRecord::new();
        for (feat, scale, value, by) in entries {
            record.note(*feat, *scale, *value, *by);
        }
        record
    }

    #[test]
    fn the_first_of_anything_is_unprecedented() {
        let mut world = WorldRecord::new();
        assert!(world.untouched(Feat::Spread, Scale::Worldwide));
        assert!(world.is_unprecedented(Feat::Spread, Scale::Worldwide, 1));

        assert!(
            world.note(Feat::Spread, Scale::Worldwide, 1, A),
            "and it takes the record"
        );
        assert!(!world.untouched(Feat::Spread, Scale::Worldwide));
    }

    #[test]
    fn a_world_gets_harder_to_impress() {
        // The difficulty curve nobody authored: the same act stops being
        // remarkable once the record has moved past it.
        let mut world = WorldRecord::new();
        world.note(Feat::Growth, Scale::Regional, 40, A);

        assert!(
            !world.is_unprecedented(Feat::Growth, Scale::Regional, 40),
            "matching is not beating"
        );
        assert!(!world.is_unprecedented(Feat::Growth, Scale::Regional, 10));
        assert!(world.is_unprecedented(Feat::Growth, Scale::Regional, 41));
    }

    #[test]
    fn beating_a_record_takes_it_and_matching_it_shares() {
        let mut world = WorldRecord::new();
        world.note(Feat::Predation, Scale::Local, 10, A);

        world.note(Feat::Predation, Scale::Local, 10, B);
        let mark = world.standing(Feat::Predation, Scale::Local).unwrap();
        assert_eq!(mark.holders, BTreeSet::from([A, B]), "a tie is shared");

        world.note(Feat::Predation, Scale::Local, 20, C);
        let mark = world.standing(Feat::Predation, Scale::Local).unwrap();
        assert_eq!(mark.high, 20);
        assert_eq!(
            mark.holders,
            BTreeSet::from([C]),
            "being beaten gives it up"
        );
    }

    #[test]
    fn scales_and_feats_are_separate_records() {
        // Doing something locally says nothing about having done it worldwide,
        // and a predator's record is not a symbiote's.
        let mut world = WorldRecord::new();
        world.note(Feat::Growth, Scale::Local, 90, A);

        assert!(world.untouched(Feat::Growth, Scale::Worldwide));
        assert!(world.untouched(Feat::Symbiosis, Scale::Local));
        assert!(world.is_unprecedented(Feat::Growth, Scale::Worldwide, 1));
    }

    #[test]
    fn harmony_and_domination_are_not_opposites() {
        // A lineage can hold both, which is a real posture rather than a
        // contradiction: indispensable and dangerous at once.
        let mut world = WorldRecord::new();
        world.note(Feat::Symbiosis, Scale::Regional, 50, A);
        world.note(Feat::Predation, Scale::Regional, 50, A);

        for feat in [Feat::Symbiosis, Feat::Predation] {
            let mark = world.standing(feat, Scale::Regional).unwrap();
            assert!(mark.holders.contains(&A));
        }
    }

    // --- the three laws that make merging safe without a protocol ---

    #[test]
    fn merging_is_commutative() {
        let left = record(&[
            (Feat::Growth, Scale::Local, 10, A),
            (Feat::Spread, Scale::Local, 5, A),
        ]);
        let right = record(&[(Feat::Growth, Scale::Local, 30, B)]);

        let mut a = left.clone();
        a.merge(&right);
        let mut b = right.clone();
        b.merge(&left);

        assert_eq!(a, b, "which side merged first cannot matter");
    }

    #[test]
    fn merging_is_associative() {
        let x = record(&[(Feat::Growth, Scale::Local, 10, A)]);
        let y = record(&[(Feat::Growth, Scale::Local, 30, B)]);
        let z = record(&[
            (Feat::Growth, Scale::Local, 20, C),
            (Feat::Endurance, Scale::Worldwide, 7, C),
        ]);

        let mut left = x.clone();
        left.merge(&y);
        left.merge(&z);

        let mut yz = y.clone();
        yz.merge(&z);
        let mut right = x.clone();
        right.merge(&yz);

        assert_eq!(left, right, "grouping cannot matter either");
    }

    #[test]
    fn merging_is_idempotent() {
        // The property that lets a peer resend a record without harm, which is
        // what removes the need for a protocol.
        let mine = record(&[(Feat::Growth, Scale::Local, 10, A)]);
        let theirs = record(&[(Feat::Growth, Scale::Local, 30, B)]);

        let mut once = mine.clone();
        once.merge(&theirs);

        let mut twice = once.clone();
        twice.merge(&theirs);
        twice.merge(&theirs);

        assert_eq!(once, twice, "merging again changes nothing");
    }

    #[test]
    fn merging_worlds_keeps_the_better_mark_and_both_ties() {
        let mut old = record(&[
            (Feat::Growth, Scale::Worldwide, 40, A),
            (Feat::Spread, Scale::Local, 12, A),
        ]);
        let new = record(&[
            (Feat::Growth, Scale::Worldwide, 60, B),
            (Feat::Spread, Scale::Local, 12, C),
            (Feat::Construction, Scale::Regional, 3, B),
        ]);

        old.merge(&new);

        let growth = old.standing(Feat::Growth, Scale::Worldwide).unwrap();
        assert_eq!(
            (growth.high, growth.holders.len()),
            (60, 1),
            "the better mark wins outright"
        );

        let spread = old.standing(Feat::Spread, Scale::Local).unwrap();
        assert_eq!(
            spread.holders,
            BTreeSet::from([A, C]),
            "an equal mark keeps both names"
        );

        assert!(
            !old.untouched(Feat::Construction, Scale::Regional),
            "and axes only they had arrive"
        );
    }

    #[test]
    fn a_record_round_trips() {
        let world = record(&[
            (Feat::Growth, Scale::Local, 10, A),
            (Feat::Spread, Scale::Worldwide, 2, B),
        ]);
        let bytes = crate::snapshot::encode(&world).unwrap();
        assert_eq!(
            crate::snapshot::decode::<WorldRecord>(&bytes).unwrap(),
            world
        );
    }

    #[test]
    fn an_empty_world_has_everything_left_to_offer() {
        let world = WorldRecord::new();
        assert_eq!(world.filled(), 0);
        for feat in Feat::ALL {
            for scale in [Scale::Local, Scale::Regional, Scale::Worldwide] {
                assert!(world.untouched(feat, scale));
            }
        }
    }
}

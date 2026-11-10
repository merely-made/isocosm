// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! What a world rules of directing (rulings 177, 178, 683, 687 to 689): how
//! a bond passes down, how far it moves, how long a nudge stays live, and
//! each trophic level's slot in a region.

use crate::schema::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Whether a bond carries to the next of a lineage (178): all three are
/// options, seeded the default.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Inheritance {
    /// Each life starts at `fresh`.
    Fresh,
    /// Each life starts `seed` per mille of the way from `fresh` to the
    /// bond its forebear ended with.
    #[default]
    Seeded,
    /// The lineage's bond carries whole.
    Lineage,
}

/// How a deliberative critter resolves an equally scored movement menu.
/// Older and hand-authored worlds keep identity order unless they opt in.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum MovementTies {
    #[default]
    Identity,
    Seeded,
}

/// A world's directing rules. Absent means these defaults.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Directing {
    pub inheritance: Inheritance,
    /// A first bond's weight, out of `most`.
    pub fresh: i64,
    /// Per mille, how much of a forebear's bond a seeded life keeps.
    pub seed: i64,
    /// The strongest a bond grows; nought is the weakest, with no sway.
    pub most: i64,
    /// How far one answered nudge moves the bond.
    pub step: i64,
    /// How long a nudge stays live, unanswered.
    pub span: Tick,
    /// What a nudge adds to a choice at the strongest bond (683).
    pub sway: i64,
    /// Each trophic level's slot in a region (226, 689): matter, by the
    /// trait that names the level. A level unnamed has no slot to fill.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub slots: BTreeMap<Key, u128>,
    /// The most recipe variants a line weighs at a boundary (752).
    #[serde(default = "four")]
    pub variants: u32,
    /// How equally best movement acts are resolved (ruling 958).
    #[serde(default, skip_serializing_if = "is_identity_movement_ties")]
    pub movement_ties: MovementTies,
    /// How lineage policies vary (mesocosm-overlay D2, M3.5); nought by
    /// default, which serializes and hashes as before.
    #[serde(default, skip_serializing_if = "PolicyVariation::is_none")]
    pub policy: PolicyVariation,
    /// Move each lineage's template to its members' success-weighted
    /// average at a boundary (case 1208; lineage-rounds D9).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub drift: bool,
    /// The player's draft: meals a kind needs, this round's meals only and
    /// a mass budget (cases 1200, 1203; lineage-rounds D10 to D12); unset,
    /// a kind is learned at the first meal.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub draft: Option<Draft>,
}

/// The player's draft (cases 1200, 1203; lineage-rounds D10 to D12).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Draft {
    /// The budget: this share (numerator, denominator) of what the line's
    /// average living member holds.
    pub share: [u32; 2],
}

impl Default for Draft {
    fn default() -> Self {
        Self { share: [1, 2] }
    }
}

/// How far a lineage policy varies, in its weights' units: at founding
/// (D2) and in each member's expression; and how many prospering members'
/// variants a line weighs at a boundary (M3.5).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyVariation {
    pub founding: i16,
    pub expression: i16,
    pub candidates: u32,
}

impl PolicyVariation {
    fn is_none(&self) -> bool {
        *self == Self::default()
    }
}

fn is_identity_movement_ties(value: &MovementTies) -> bool {
    *value == MovementTies::Identity
}

fn four() -> u32 {
    4
}

impl Default for Directing {
    fn default() -> Self {
        Self {
            movement_ties: MovementTies::Identity,
            inheritance: Inheritance::Seeded,
            fresh: 250,
            seed: 500,
            most: 1000,
            step: 50,
            span: 60,
            sway: 1000,
            slots: BTreeMap::new(),
            variants: four(),
            policy: PolicyVariation::default(),
            drift: false,
            draft: None,
        }
    }
}

impl Directing {
    /// A new life's bond, from the bond its forebear ended with, if any.
    pub fn inherit(&self, forebear: Option<i64>) -> i64 {
        let start = match (self.inheritance, forebear) {
            (_, None) | (Inheritance::Fresh, _) => self.fresh,
            (Inheritance::Seeded, Some(b)) => self.fresh + (b - self.fresh) * self.seed / 1000,
            (Inheritance::Lineage, Some(b)) => b,
        };
        start.clamp(0, self.most)
    }

    pub fn validate(&self) -> crate::Result<()> {
        let fits = |v: i64| (0..=self.most).contains(&v);
        if self.most <= 0 || !fits(self.fresh) || !(0..=1000).contains(&self.seed) {
            return Err("a bond's weights lie between nought and the most".into());
        }
        if !fits(self.step) || self.span == 0 || self.sway < 0 {
            return Err("invalid directing step, span or sway".into());
        }
        for level in self.slots.keys() {
            crate::validation::key(level)?;
        }
        if self.draft.is_some_and(|d| d.share[1] == 0) {
            return Err("the draft's share has a denominator of nought".into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inheritance_has_three_settings_and_seeds_by_default() {
        let mut d = Directing::default();
        assert_eq!(d.inheritance, Inheritance::Seeded);
        assert_eq!(d.inherit(None), 250);
        assert_eq!(d.inherit(Some(850)), 550);
        d.inheritance = Inheritance::Fresh;
        assert_eq!(d.inherit(Some(850)), 250);
        d.inheritance = Inheritance::Lineage;
        assert_eq!(d.inherit(Some(850)), 850);
        assert!(d.validate().is_ok());
        d.fresh = 2000;
        assert!(d.validate().is_err());
    }
}

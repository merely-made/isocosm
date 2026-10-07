// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use std::fmt;

use crate::legacy::mesocosm::axis::Recipe;
use crate::legacy::mesocosm::axis::archetype::datasheet::{FoundingSheet, foundings};
use crate::legacy::mesocosm::{development::PartPalette, organism::Kingdom};

/// Where a founding tier's bodies come from: one founding in
/// `datasheets/foundings/foundings.toml`, by name.
///
/// The sheet says what each founding admits and installs, and why (wing
/// design record rulings 625 to 632). The eight it declares today are named
/// here too, so call sites read as they always did. Debug prints the bare
/// name, which is what recorded receipts store (`"palette": "SpacedRoster"`).
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Founding(&'static str);

#[allow(
    non_upper_case_globals,
    reason = "the sheet's names, as receipts record them"
)]
impl Founding {
    pub const Drawn: Self = Self("Drawn");
    pub const BrowsingConsumer: Self = Self("BrowsingConsumer");
    pub const RosterStand: Self = Self("RosterStand");
    pub const RosterFauna: Self = Self("RosterFauna");
    pub const Roster: Self = Self("Roster");
    pub const BranchingRoster: Self = Self("BranchingRoster");
    pub const JointedRoster: Self = Self("JointedRoster");
    pub const SpacedRoster: Self = Self("SpacedRoster");
}

impl Founding {
    /// The founding the sheet declares by this name.
    pub fn named(name: &str) -> Option<Self> {
        Self::all().find(|founding| founding.0 == name)
    }

    /// Every founding the sheet declares, in name order.
    pub fn all() -> impl Iterator<Item = Self> {
        foundings().names().map(Self)
    }

    pub fn name(self) -> &'static str {
        self.0
    }

    /// The vocabulary a world founded this way has to admit.
    pub fn palette(self) -> PartPalette {
        self.sheet().palette
    }

    /// The authored bodies this founding installs for a tier, one lineage
    /// each, in founding order. Empty means the tier still draws.
    pub(super) fn tier(self, kingdom: Kingdom) -> &'static [Recipe] {
        self.sheet().tier(kingdom)
    }

    /// How many non-played lineages this tier founds. A drawn tier is one
    /// interbreeding species, which is the structural fact TD10 found and the
    /// roster exists to change.
    pub(super) fn lineages(self, kingdom: Kingdom) -> usize {
        self.tier(kingdom).len().max(1)
    }

    fn sheet(self) -> &'static FoundingSheet {
        foundings()
            .get(self.0)
            .unwrap_or_else(|| panic!("the founding datasheet declares no {:?}", self.0))
    }
}

/// The founding the sheet ships: how the enclosure opens.
impl Default for Founding {
    fn default() -> Self {
        Self(foundings().default.as_str())
    }
}

impl fmt::Debug for Founding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}

#[cfg(test)]
mod tests;

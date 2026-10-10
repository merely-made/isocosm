// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! What rules a legacy world actually realized. (PD3)
//!
//! The record a world is founded under, folded into its identity: the digest
//! of the admitted [`Registry`], the epoch rule and its budget, the scoring
//! window, the trophic grammar, the soil completion rate, graft
//! compatibility and the deep-time span. Its parts are native Isocosm's
//! (`process`, `rules::epoch`); the record itself is the legacy world's, and
//! leaves with the world family (wing rulings 666, 678).

use serde::{Deserialize, Serialize};

use crate::process::{Registry, RulesetDigest};
use crate::rules::{DeepTimeSpan, EpochRule};

/// Ticks an epoch runs for by default: a hundred seconds at ten ticks a
/// second, a third of a 1,000 mg starter's 3,000-tick lifespan.
pub const DEFAULT_EPOCH_TICKS: u64 = 1_000;

/// Version of the trophic semantics embodied in a world and its trace.
/// Zero names the pre-port grammar, so a decoded historical world can be
/// distinguished at the `WorldRules` admission gate rather than replayed as
/// TG1. Revision 2 introduces typed soil; revision 3 adds part scruple storage.
/// Revision 4 preserves whole-meal mixtures and records reserve digestion.
/// Revision 5 preserves birth provisioning and records completed tissue returns.
/// Revision 6 introduces producer synthesis and explicit founder tissue recipes.
/// Revision 7 completes pending typed soil at a declared per-column rate.
/// Revision 8 admits bounded, priced disfavoured carries.
pub const TROPHIC_GRAMMAR_REVISION: u32 = 8;

/// Pending typed matter completed into nutrients per column per ecology tick.
/// Provisional, not a calibrated decomposition model; zero keeps pending
/// tissue intact for a paired control.
pub const DEFAULT_SOIL_MINERALIZATION_MG_PER_COLUMN_PER_TICK: u64 = 1;

/// How long a candidate is grown before its flow record is read (P4b): one
/// brood interval at the ecology's reference body, the shortest named span
/// past the 240 to 600 ticks where a gland turns worth carrying.
pub const DEFAULT_SCORE_TICKS: u64 =
    crate::legacy::mesocosm::organism::ecology::GESTATION_BASE as u64;

/// The immutable record of the rules a world realized.
///
/// `epoch_ticks` follows `epoch` so a timed world encodes as it did when the
/// budget lived inside the rule.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct WorldRules {
    /// The process ruleset this world admitted.
    pub processes: RulesetDigest,
    /// What ends an epoch here. (PE3)
    #[serde(default)]
    pub epoch: EpochRule,
    /// A timed epoch's budget, unread under the other rules.
    #[serde(default = "default_epoch_ticks")]
    pub epoch_ticks: u64,
    /// Ticks a candidate is grown for before its flow record is read. (P4b)
    #[serde(default = "default_score_ticks")]
    pub score_ticks: u64,
    /// The trophic grammar that decides which body can admit which food.
    #[serde(default)]
    pub trophic_grammar: u32,
    /// Typed soil completed after bodies settle and before percolation.
    #[serde(default = "default_soil_mineralization")]
    pub soil_mineralization_mg_per_column_per_tick: u64,
    /// Cumulative disfavoured tissue allowance and the incoming graft's cost.
    #[serde(
        default = "crate::legacy::mesocosm::graft::compatibility::Compatibility::legacy_disabled"
    )]
    pub graft_compatibility: crate::legacy::mesocosm::graft::compatibility::Compatibility,
    /// The epochs this world ran before handover. See [`DeepTimeSpan`].
    #[serde(default)]
    pub deep_time: DeepTimeSpan,
}

fn default_epoch_ticks() -> u64 {
    DEFAULT_EPOCH_TICKS
}

fn default_score_ticks() -> u64 {
    DEFAULT_SCORE_TICKS
}

fn default_soil_mineralization() -> u64 {
    DEFAULT_SOIL_MINERALIZATION_MG_PER_COLUMN_PER_TICK
}

/// Written out, not derived: a derived default would score over zero ticks.
impl Default for WorldRules {
    fn default() -> Self {
        Self {
            processes: RulesetDigest::default(),
            epoch: EpochRule::default(),
            epoch_ticks: DEFAULT_EPOCH_TICKS,
            score_ticks: DEFAULT_SCORE_TICKS,
            trophic_grammar: 0,
            soil_mineralization_mg_per_column_per_tick: default_soil_mineralization(),
            graft_compatibility:
                crate::legacy::mesocosm::graft::compatibility::Compatibility::legacy_disabled(),
            deep_time: DeepTimeSpan::default(),
        }
    }
}

impl WorldRules {
    /// The rules a world founded on this build's own definitions realizes.
    pub fn native() -> Self {
        Self::of(Registry::native())
    }

    /// The rules an admitted registry amounts to, under this build's defaults
    /// for everything a registry does not decide.
    pub fn of(registry: &Registry) -> Self {
        Self {
            processes: registry.digest(),
            trophic_grammar: TROPHIC_GRAMMAR_REVISION,
            graft_compatibility:
                crate::legacy::mesocosm::graft::compatibility::Compatibility::native(),
            ..Self::default()
        }
    }

    /// The same rules under a different epoch rule, the budget kept.
    pub fn ending(self, epoch: EpochRule) -> Self {
        Self { epoch, ..self }
    }

    /// The same rules under a timed epoch of `ticks`.
    pub fn timed(self, ticks: u64) -> Self {
        Self {
            epoch: EpochRule::Timed,
            epoch_ticks: ticks,
            ..self
        }
    }

    /// The same rules under a different scoring window.
    pub fn scoring_over(self, score_ticks: u64) -> Self {
        Self {
            score_ticks,
            ..self
        }
    }

    /// Whether `elapsed` ticks of the current epoch spend its budget: only a
    /// timed rule with a budget does.
    pub fn epoch_spent(self, elapsed: u64) -> bool {
        self.epoch.is_timed() && self.epoch_ticks > 0 && elapsed >= self.epoch_ticks
    }

    /// The epoch rule's rule-bearing bytes, the budget only when timed.
    fn epoch_bytes(self) -> Vec<u8> {
        match self.epoch {
            EpochRule::Timed => {
                let mut bytes = vec![0u8];
                bytes.extend_from_slice(&self.epoch_ticks.to_le_bytes());
                bytes
            },
            EpochRule::Gated => vec![1],
            EpochRule::PlayerTriggered => vec![2],
        }
    }

    /// This whole record's identity: every component folded, in a fixed order.
    pub fn digest(self) -> u64 {
        let mut bytes = self.processes.0.to_le_bytes().to_vec();
        bytes.extend_from_slice(&self.epoch_bytes());
        bytes.extend_from_slice(&self.score_ticks.to_le_bytes());
        bytes.extend_from_slice(&self.trophic_grammar.to_le_bytes());
        bytes.extend_from_slice(
            &self
                .soil_mineralization_mg_per_column_per_tick
                .to_le_bytes(),
        );
        bytes.extend_from_slice(&self.graft_compatibility.digest().to_le_bytes());
        bytes.extend_from_slice(&self.deep_time.epochs.to_le_bytes());
        crate::legacy::mesocosm::snapshot::hash_bytes(&bytes)
    }
}

#[cfg(test)]
mod tests;

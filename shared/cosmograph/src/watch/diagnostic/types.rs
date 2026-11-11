// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FoodDiagnosticConfig {
    pub focus_lineages: BTreeSet<Key>,
    /// Detailed examples per lineage/process/phase/outcome stage, per drain.
    pub examples_per_kind: usize,
    /// Exact buckets anchored at the capture's starting native tick.
    pub bucket_ticks: Tick,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FoodDiagnosticReport {
    pub started: Tick,
    pub through: Tick,
    pub bucket_ticks: Tick,
    pub feeds: Vec<FeedTally>,
    pub life: Vec<LifeTally>,
    pub breeding: Vec<BirthTally>,
    pub feed_examples: Vec<FeedAttempt>,
    pub breeding_examples: Vec<BirthOpportunity>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FeedTally {
    pub bucket: Tick,
    pub first_tick: Tick,
    pub last_tick: Tick,
    pub process: Key,
    pub lineage: Key,
    pub target_lineage: Option<Key>,
    pub target_alive: Option<bool>,
    /// `plan`, `scheduled`, or `direct`; plans never commit matter.
    pub phase: String,
    pub accepted: bool,
    pub stage: String,
    pub attempts: u64,
    pub selected: u64,
    pub no_target: u64,
    pub requested_by_destination: Ledger,
    /// Sums of observed per-Eat offerings, not unique prey biomass.
    pub offered: Ledger,
    pub reserved: Ledger,
    pub granted: Ledger,
    pub staged_taken: Ledger,
    pub selected_parts: BTreeMap<crate::schema::PartId, u64>,
    pub staged_given: Ledger,
    pub committed_given: Ledger,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FeedAttempt {
    pub tick: Tick,
    pub process: Key,
    pub actor: Id,
    pub lineage: Key,
    pub target: Option<Id>,
    pub target_lineage: Option<Key>,
    pub target_alive: Option<bool>,
    pub count: u64,
    pub phase: String,
    pub accepted: bool,
    pub stage: String,
    pub outcome: String,
    /// Staged observations: a refused act commits none of these amounts.
    pub meals: Vec<MealObservation>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MealObservation {
    pub at: Vec<usize>,
    pub part: Option<crate::schema::PartId>,
    pub into: Key,
    pub asked: u64,
    pub holdings: Ledger,
    pub offered: Ledger,
    pub reserved: Option<Ledger>,
    pub planned_at: Vec<usize>,
    pub planned_part: Option<crate::schema::PartId>,
    pub quota: Option<Ledger>,
    pub taken: Ledger,
    pub given: Ledger,
    pub ingesta_before: Option<u64>,
    pub ingesta_after: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LifeTally {
    pub bucket: Tick,
    pub first_tick: Tick,
    pub last_tick: Tick,
    pub process: Key,
    pub lineage: Key,
    pub kind: String,
    pub members: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueryObservation {
    pub query: String,
    pub passed: bool,
    pub reading: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BirthOpportunity {
    pub tick: Tick,
    pub process: Key,
    pub actor: Id,
    pub lineage: Key,
    pub count: u64,
    pub site: Option<Id>,
    pub age: Tick,
    pub age_required: Tick,
    pub tissue: Option<Key>,
    pub tissue_held: u64,
    pub tissue_required: u64,
    pub role_members: u64,
    pub port_members: u64,
    /// Actual admitted exposed holders, from the native FoodPerHead contract.
    pub food_members: Option<u64>,
    pub food_gate_ready: Option<bool>,
    /// Edible matter the feed's admitted holders hold, from the native
    /// EdiblePerHead contract, beside the headcount above.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub edible_matter: Option<u64>,
    pub consumer_members: u64,
    pub at_least: u64,
    pub per_members: u64,
    pub feed_process: Option<Key>,
    pub ready: bool,
    pub requirements: Vec<QueryObservation>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Range {
    pub min: u64,
    pub max: u64,
}

impl Range {
    pub(crate) fn is_unset(&self) -> bool {
        *self == Self::default()
    }
}

impl Range {
    pub(super) fn include(&mut self, value: u64, first: bool) {
        if first {
            self.min = value;
            self.max = value;
        } else {
            self.min = self.min.min(value);
            self.max = self.max.max(value);
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BirthTally {
    pub bucket: Tick,
    pub first_tick: Tick,
    pub last_tick: Tick,
    pub process: Key,
    pub lineage: Key,
    pub opportunities: u64,
    pub ready: u64,
    pub age_ready: u64,
    pub tissue_ready: u64,
    pub role_gate_ready: u64,
    pub port_gate_ready: u64,
    pub food_gate_ready: u64,
    pub age: Range,
    pub tissue: Range,
    pub role_members: Range,
    pub port_members: Range,
    pub food_members: Range,
    #[serde(default, skip_serializing_if = "Range::is_unset")]
    pub edible_matter: Range,
    pub consumer_members: Range,
}

// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use isocosm::{Command, schema::Id, directing::{interim::{Happening, OnCollapse, Pace,
    Start, Handover}, readings::Mode}, lineage::boundary::Score};
use isocosm_overlay::{EntityHandle, ParticipantHandle, Tick, mesocosm::CheckpointAnswer};
use serde::{Deserialize, Serialize};
use crate::{Checkpoint, FlowWindows, Review};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LoopSettings { pub mode: Mode, pub pace: Pace, pub on_collapse: OnCollapse }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LoopCut { pub native: String, pub game: String, pub question: Option<String> }

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoopPhase { Running, Held, LineageEnded, Watching, NewWorldRequested }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LoopInput {
    pub kind: String, pub version: u32, pub participant: ParticipantHandle, pub tick: Tick,
    pub expected: LoopCut, pub action: LoopAction,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoopAction { Configure(LoopSettings), Step, Answer(LoopAnswer) }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoopAnswer {
    Checkpoint(CheckpointAnswer), OtherLineage(EntityHandle), Watch, NewWorld,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LoopOffer {
    pub index: usize, pub name: String, pub score: Score, pub why_not: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PersonalQuestion { Birth { parent: Id }, Loss { critter: Id }, Epoch { entity: Id } }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LoopReading {
    pub cut: LoopCut, pub settings: Option<LoopSettings>, pub phase: LoopPhase,
    pub checkpoint: Option<Checkpoint>, pub offers: Vec<LoopOffer>,
    pub happenings: Vec<Happening>, pub lineages: Vec<EntityHandle>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub personal: Option<PersonalQuestion>,
}

/// Only an owned run has these actual founding/deep-time facts.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Origin { pub start: Start, pub handover: Handover, pub habitable_at: u64 }

/// Concrete authored expression work retained at this exact held cut.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SavedProposal {
    pub offer: usize, pub script: String, pub cells: Result<u32, String>,
    pub commands: Vec<Command>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SavedExpressionSource {
    pub scripts: Vec<(String, String)>, pub fuel: i32,
    pub max_output_bytes: usize, pub max_depth: usize, pub max_entries: usize,
    pub terms: mesocosm_phenotype::TermTable, pub affinity: Option<isocosm::rules::Affinity>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RoundSource {
    pub first_entry: String, pub from: u64, pub native: String, pub played: Option<Id>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Continuation {
    pub version: u32, pub participant: Id, pub settings: LoopSettings,
    pub native: String, pub native_entries: u64, pub native_tick: u64,
    pub physical: String, pub phase: LoopPhase,
    pub checkpoint: Option<Checkpoint>, pub review: Option<Review>,
    pub proposed: Vec<SavedProposal>, pub happenings: Vec<Happening>, pub windows: FlowWindows,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expression: Option<SavedExpressionSource>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ended_lineage: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub queued: Vec<crate::Envelope>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub round: Option<RoundSource>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origin: Option<Origin>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fault: Option<String>,
}

// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use cosmograph::{Command, schema::Id, directing::{interim::{Happening, OnCollapse, Pace,
    Start, Handover, Ending}, readings::Mode}, lineage::{boundary::Score, sheet::Sheet}};
use cosmograph_overlay::{EntityHandle, ParticipantHandle, Tick, mesocosm::CheckpointAnswer};
use serde::{Deserialize, Serialize};
use crate::{Checkpoint, FlowWindows, Review};

/// How a round runs past its span (lineage-rounds D7; cases 1199, 1217):
/// capped at the epoch, held for the player's call, or continuous, where
/// other lineages adapt every span and the played line's review waits for a call.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Rounds { #[default] Capped, Waiting, Continuous }

impl Rounds { fn is_default(&self) -> bool { *self == Self::Capped } }

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LoopSettings {
    pub mode: Mode, pub pace: Pace, pub on_collapse: OnCollapse,
    #[serde(default, skip_serializing_if = "Rounds::is_default")]
    pub rounds: Rounds,
    /// Each epoch's par is what the played line left alone would hold (D8).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub condition: bool,
}

fn no_ending(ending: &Ending) -> bool { *ending == Ending::default() }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LoopCut { pub native: String, pub game: String, pub question: Option<String> }

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoopPhase { Running, Held, LineageEnded, Watching, NewWorldRequested, WorldEnded }

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
    /// Begin as another lineage at an offered start site (case 1211; lineage-rounds D5, D6).
    Start { lineage: String, site: u64 },
    /// End the epoch (case 1199); `Extend` skips the next capped boundary once
    /// (case 1217); `Forfeit` ends it for a player with no living member (case 1212).
    Call, Extend, Forfeit,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LoopOffer {
    pub index: usize, pub name: String, pub score: Score, pub why_not: Option<String>,
    /// The adult mass it adds, in a draft review (case 1203).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cost: Option<u128>,
    /// What a fully grown body of it would be (case 1215).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sheet: Option<Sheet>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PersonalQuestion { Birth { parent: Id }, Loss { critter: Id }, Epoch { entity: Id } }

/// One place a round may begin for a lineage: the body taken there, and
/// whether it must first journey to the site (a fringe start).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LoopStart { pub lineage: String, pub site: u64, pub critter: EntityHandle, pub journey: bool }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LoopReading {
    pub cut: LoopCut, pub settings: Option<LoopSettings>, pub phase: LoopPhase,
    pub checkpoint: Option<Checkpoint>, pub offers: Vec<LoopOffer>,
    pub happenings: Vec<Happening>, pub lineages: Vec<EntityHandle>,
    /// The starts of those lineages (case 1211; lineage-rounds D5, D6).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub starts: Vec<LoopStart>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub personal: Option<PersonalQuestion>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub world_ended_at: Option<u64>,
    #[serde(default, skip_serializing_if = "no_ending")]
    pub ending: Ending,
    /// What a fully grown body of the played line's recipe is (case 1215).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sheet: Option<Sheet>,
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
    pub terms: mesocosm_phenotype::TermTable, pub affinity: Option<cosmograph::rules::Affinity>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RoundSource {
    pub first_entry: String, pub from: u64, pub native: String, pub played: Option<Id>,
    /// The ending as it stood before this round's ticks, par included.
    #[serde(default, skip_serializing_if = "no_ending")]
    pub ending: Ending,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorldEndSource { pub entry: String, pub tick: u64 }

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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub world_end: Option<WorldEndSource>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub queued: Vec<crate::Envelope>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub round: Option<RoundSource>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origin: Option<Origin>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fault: Option<String>,
    #[serde(default, skip_serializing_if = "no_ending")]
    pub ending: Ending,
}

// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Mesocosm's side of the overlay contract (Mesocosm overlay plan §3; wing
//! design record §5.5): a player directs only the entity they play, never
//! drives it (rulings 152, 175). The directive vocabulary is four kinds —
//! priorities, places, stances and nudges (ruling 176) — plus the checkpoint
//! answers and dev intents below. See the crate README for the table
//! mapping every one of `mesocosm-core`'s sixteen `Intent` variants onto
//! this module.

mod checkpoint;
mod dev;
mod directive;
mod handoff;

pub use checkpoint::{BirthAnswer, CheckpointAnswer, CheckpointAnswerKind, RevisionAnswer};
pub use dev::{DevIntent, WorldPoint};
pub use directive::{
    Boldness, CompetitorStance, Directive, DirectiveKind, Nudge, Places, Priorities, PriorityKey,
    Stances,
};
pub use handoff::MesocosmHandoff;

use serde::{Deserialize, Serialize};

use crate::{HandoffEnvelope, IntentEnvelope};

/// Everything a Mesocosm host may send the sim: a directive, a checkpoint
/// answer, or a dev intent (never play; see [`DevIntent`]).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MesocosmIntent {
    Directive(Directive),
    Checkpoint(CheckpointAnswer),
    Dev(DevIntent),
}

impl MesocosmIntent {
    /// Whether this is a dev intent (mesocosm-core's `Intent::is_dev`,
    /// carried into the contract for the same reason: a run that used one is
    /// labelled assisted, never branched on).
    pub fn is_dev(&self) -> bool {
        matches!(self, Self::Dev(_))
    }
}

/// A Mesocosm intent, stamped for its tick.
pub type MesocosmIntentEnvelope = IntentEnvelope<MesocosmIntent>;

/// Mesocosm's handoff envelope. Uninhabited today (see [`MesocosmHandoff`]),
/// so no value of this type can be constructed.
pub type MesocosmHandoffEnvelope = HandoffEnvelope<MesocosmHandoff>;

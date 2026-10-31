// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use serde::{Deserialize, Serialize};
use cosmograph_overlay::{ParticipantHandle, Tick};
use crate::runtime::Envelope;

/// Existing envelopes remain readable beside explicit product operations.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Input { Envelope(Envelope), Game(GameInput), Graft(crate::runtime::graft::GraftInput), Loop(crate::continuation::LoopInput) }

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GameInput {
    pub participant: ParticipantHandle,
    pub tick: Tick,
    pub envelope: Option<Envelope>,
    pub advance: bool,
}

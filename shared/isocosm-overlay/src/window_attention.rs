// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Window-scoped attention input and effective player publication (815).
//! Native admission issues window identity and validates every request.

use crate::{AttentionChange, EntityHandle, ParticipantHandle, PlaceHandle, Pointable, Tick};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct WindowHandle(pub u64);

/// User-authored attention for one window. Played and care are native-derived.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WindowAttention {
    pub pinned: BTreeSet<Pointable>,
    pub examined: Option<PlaceHandle>,
}

/// One player's published union. Multiple active windows can examine regions.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerAttention {
    pub participant: ParticipantHandle,
    pub played: BTreeSet<EntityHandle>,
    pub pinned: BTreeSet<Pointable>,
    pub examined: BTreeSet<PlaceHandle>,
    pub care: BTreeSet<Pointable>,
}

/// A window attention change stamped for native validation and replay.
/// The existing game IntentEnvelope remains unchanged in this source slice.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WindowAttentionEnvelope {
    pub tick: Tick,
    pub participant: ParticipantHandle,
    pub window: WindowHandle,
    pub change: AttentionChange,
}

/// Withdrawal of one native window's contribution, not a global Release.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CloseWindowEnvelope {
    pub tick: Tick,
    pub participant: ParticipantHandle,
    pub window: WindowHandle,
}

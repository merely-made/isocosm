// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use serde::{Deserialize, Serialize};

use super::actuation::ActKey;
use crate::EntityHandle;

/// Eponym's handoff vocabulary, inhabited by ruling 232: the foreground
/// resolves each blow geometrically, its strike system reading a swept
/// volume against a body's parts, and hands the harm back in the sim's
/// terms. What comes back must pass the sim's invariants and agree with the
/// sim's own fight in distribution, checked on the bench (rulings 114, 123,
/// 154); the record's §3.8 has the sim never resolve a single blow.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EponymHandoff {
    Blow(Blow),
}

/// One blow, resolved: who struck, whom, with what act, and the harm done.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Blow {
    pub by: EntityHandle,
    pub target: EntityHandle,
    pub act: ActKey,
    pub harm: Harm,
}

/// Harm in the sim's terms (ruling 123): vigour drained first, and wounds to
/// parts of the body's tree when a blow lands hard or the vigour is gone.
/// Vigour comes back with rest; wounds heal slowly, or never. The VTT's
/// module carries the same shape for its own handoff; a core `Harm` is a
/// candidate change put to Mark, not made (ruling 197).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Harm {
    pub vigour_drained: u64,
    pub wounds: Vec<Wound>,
}

/// A wound to one part.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Wound {
    pub part: PartHandle,
    pub severity: WoundSeverity,
}

/// How badly a part is wounded, from a loss that heals to a part severed;
/// what each does to what the body affords is the sim's reading of its part
/// tree (the record's §3.3.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum WoundSeverity {
    Minor,
    Serious,
    Crippling,
    Severed,
}

/// An opaque reference to one part of a body's tree, minted by the sim: a
/// body requirement resolves to a live part address (the world conditions
/// plan), never a label. Defined here rather than in the core for the same
/// reason as [`Harm`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PartHandle(pub u64);

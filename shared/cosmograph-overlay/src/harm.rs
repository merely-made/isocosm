// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use serde::{Deserialize, Serialize};

/// Harm in the sim's terms (wing design record ruling 123): vigour drained
/// first, and wounds to parts of the body's tree when a blow lands hard or
/// the vigour is gone. Vigour comes back with rest; wounds heal slowly, or
/// never. A game's handoff carries this when its foreground resolved the
/// blow (rulings 114, 154, 232), and a ruleset's hit points calibrate
/// against vigour more than wounds (the record's §3.3.1). Lifted to the core
/// at Mark's word on 2026-09-26 from the two game modules that had each
/// defined it.
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
/// tree.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum WoundSeverity {
    Minor,
    Serious,
    Crippling,
    Severed,
}

/// An opaque reference to one part of a body's tree, minted by the sim: a
/// body requirement resolves to a live part address (the world conditions
/// plan), never a label, and never a pointer (the record's §5.2 point 6).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PartHandle(pub u64);

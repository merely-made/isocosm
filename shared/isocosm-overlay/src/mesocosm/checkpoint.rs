// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use serde::{Deserialize, Serialize};

use crate::handle::{CandidateHandle, EntityHandle};

/// An answer to one of Mesocosm's checkpoints (overlay plan §3): at a birth,
/// keep the parent or take the offspring; at a death, the next life taken up;
/// at the epoch boundary, the review's committed revision.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckpointAnswer {
    pub entity: EntityHandle,
    pub kind: CheckpointAnswerKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CheckpointAnswerKind {
    Birth(BirthAnswer),
    Death(DeathAnswer),
    EpochReview(RevisionAnswer),
}

/// At a birth: keep the parent (the default) or take the offspring (ruling
/// 183). `entity` on the enclosing [`CheckpointAnswer`] names the parent
/// whose checkpoint this answers; the offspring is unambiguous from the
/// birth it followed, so it needs no handle of its own.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BirthAnswer {
    KeepParent,
    TakeOffspring,
}

/// At a death: the next of the lineage the player takes up, from the cohort
/// that is the pool of further lives (rulings 61, 178). `entity` on the
/// enclosing [`CheckpointAnswer`] names the one who died. A reading, not a
/// ruling: there is no answer that lets the line go, since none is ruled.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeathAnswer {
    pub next: EntityHandle,
}

/// At the epoch boundary: the candidate the review committed as this line's
/// revision.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RevisionAnswer {
    pub revision: CandidateHandle,
}

// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use serde::{Deserialize, Serialize};

use crate::{EntityHandle, EventHandle};

/// An ask: a proposal from the played sophont to one peer (rulings 60, 63).
/// The sim answers by the peer's own methodology weighed by its opinion of
/// the asker, and the answer, a refusal about the work, a refusal about the
/// asker, a counteroffer or an agreement, comes back as events. A proposal
/// accepted once and held is a standing agreement (ruling 67), which a later
/// ask names in `under`; even then the peer may decline when the premises
/// have changed, since an agreement is not a command.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Proposal {
    /// The played sophont (ruling 152).
    pub from: EntityHandle,
    /// One peer. A party is those who accepted the same act (ruling 63).
    pub to: EntityHandle,
    /// The act asked, by opaque key: a craft, a journey, a fight, a home
    /// offered or taken.
    pub work: WorkKey,
    pub terms: Vec<Term>,
    /// The accepted event that formed the standing agreement this ask is
    /// made under, if any.
    pub under: Option<EventHandle>,
}

/// Work named by opaque key; the vocabulary is the world's, as the sim
/// names processes by string key.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct WorkKey(pub String);

/// One term of a proposal: what is offered to the peer or asked of it, by
/// opaque key, and how much of it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Term {
    pub side: TermSide,
    pub what: String,
    pub count: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TermSide {
    Offered,
    Asked,
}

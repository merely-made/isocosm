// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Standing, folded from deeds, and the answers a peer gives with the
//! premises behind each: refusal is an outcome, never an error.

use super::{DeedKind, EndReason, Terms};
use crate::schema::{Id, Key, Tick};
use serde::{Deserialize, Serialize};

/// Where one stands with another, and the deeds that put it there.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Standing {
    pub trust: i16,
    pub affinity: i16,
    pub from_deeds: Vec<Key>,
}

impl Standing {
    pub fn stranger() -> Self {
        Self::default()
    }
    pub fn known(&self) -> bool {
        !self.from_deeds.is_empty()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Verdict {
    Accept,
    Refuse,
    Counteroffer(Terms),
}

impl Verdict {
    pub fn accepted(&self) -> bool {
        matches!(self, Self::Accept)
    }
    pub fn name(&self) -> &'static str {
        match self {
            Self::Accept => "accepts",
            Self::Refuse => "refuses",
            Self::Counteroffer(_) => "counteroffers",
        }
    }
}

/// One reason behind an answer, pointable.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Premise {
    Deed {
        deed: Key,
        at: Tick,
        doer: Id,
        kind: DeedKind,
        trust: i16,
        affinity: i16,
    },
    Standing {
        toward: Id,
        trust: i16,
        affinity: i16,
        from_deeds: Vec<Key>,
    },
    Confidence {
        craft: Key,
        demanded: u8,
        held: u8,
        margin: i16,
    },
    TrustAsked {
        danger: u8,
        asked: i16,
        held: i16,
        met: bool,
    },
    DangerWeighed {
        danger: u8,
        bearable: i16,
        affinity: i16,
        caution: i16,
        borne: bool,
    },
    AgreementTerm {
        agreement: Id,
        share: u8,
        danger_cap: u8,
        covers: bool,
    },
    TermChange {
        agreement: Id,
        from: Terms,
        to: Terms,
    },
    Ending {
        agreement: Id,
        why: EndReason,
    },
}

impl Premise {
    pub fn deeds(&self) -> Vec<Key> {
        match self {
            Self::Deed { deed, .. } => vec![deed.clone()],
            Self::Standing { from_deeds, .. } => from_deeds.clone(),
            _ => Vec::new(),
        }
    }
}

/// The deeds `premises` cite, each once, in order.
pub fn cited_deeds(premises: &[Premise]) -> Vec<Key> {
    let mut cited: Vec<Key> = Vec::new();
    for deed in premises.iter().flat_map(Premise::deeds) {
        if !cited.contains(&deed) {
            cited.push(deed);
        }
    }
    cited
}

/// A peer's answer to an offer.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Response {
    pub by: Id,
    pub at: Tick,
    pub verdict: Verdict,
    pub premises: Vec<Premise>,
    pub recorded: Key,
}

impl Response {
    pub fn cited_deeds(&self) -> Vec<Key> {
        cited_deeds(&self.premises)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RulingKind {
    Formed(Id),
    Performed,
    Declined,
    OutsideTerms,
    Renegotiated { from: Terms, to: Terms },
    Ended(EndReason),
}

impl RulingKind {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Formed(_) => "forms the arrangement",
            Self::Performed => "does the agreed work",
            Self::Declined => "declines, under the arrangement",
            Self::OutsideTerms => "says that is not what we agreed",
            Self::Renegotiated { .. } => "changes the terms",
            Self::Ended(_) => "ends the arrangement",
        }
    }
}

/// What came of a step in an agreement's life.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ruling {
    pub by: Id,
    pub at: Tick,
    pub kind: RulingKind,
    pub premises: Vec<Premise>,
    pub recorded: Option<Key>,
}

impl Ruling {
    pub fn cited_deeds(&self) -> Vec<Key> {
        cited_deeds(&self.premises)
    }
}

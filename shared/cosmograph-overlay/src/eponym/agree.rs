// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use serde::{Deserialize, Serialize};

use super::ask::{AgreementHandle, Proposal, Term};
use crate::{EntityHandle, PlaceHandle};

/// A step in a standing agreement's life (rulings 63, 67): formed from a
/// proposal, its terms put again, ended, or a home offered under one
/// (ruling 54). An ask under a held agreement is a [`Proposal`] naming it.
/// Added for routing Eponym through its contract (ruling 794).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgreementAct {
    /// The proposal accepted once and held.
    Form(Proposal),
    /// One side puts new terms.
    Renegotiate {
        agreement: AgreementHandle,
        by: EntityHandle,
        terms: Vec<Term>,
    },
    End {
        agreement: AgreementHandle,
        by: EntityHandle,
        why: EndReason,
    },
    /// A dwelling offered as a home under the proposal's terms.
    Home {
        proposal: Proposal,
        dwelling: PlaceHandle,
    },
}

/// Why an agreement ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EndReason {
    Withdrawn,
    Resigned,
    WorkDone,
    PremisesChanged,
}

/// A deed the game saw done, by the key its rules pack declares (ruling
/// 778): standing is folded from deeds (rulings 51, 56).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Deed {
    pub doer: EntityHandle,
    pub toward: Option<EntityHandle>,
    pub kind: DeedKey,
    /// The agreement it was done under, if any.
    pub under: Option<AgreementHandle>,
}

/// A deed named by opaque key.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct DeedKey(pub String);

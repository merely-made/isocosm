// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Deeds (rulings 51 and 56): who did what toward whom, each an act of a
//! deed process that leaves a world event, its object the other party, so
//! a deed spreads by the reach field like any event and its doer holds a
//! note of its own act. Standing is folded from them, never kept.

use crate::{
    Result,
    rules::{Causation, Process},
    schema::*,
    simulation::Simulation,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// What a deed was. Agreement deeds carry their agreement in the event's
/// cause, `agreement:<id>`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeedKind {
    StoodBy,
    Abandoned,
    Shared,
    Took,
    OfferAccepted,
    OfferRefused,
    OfferCountered,
    AgreementFormed(Id),
    PerformedUnderAgreement(Id),
    AgreementRenegotiated(Id),
    AgreementEnded(Id),
}

const KEYS: [&str; 11] = [
    "deed:stood-by",
    "deed:abandoned",
    "deed:shared",
    "deed:took",
    "deed:offer-accepted",
    "deed:offer-refused",
    "deed:offer-countered",
    "deed:agreement-formed",
    "deed:performed-under-agreement",
    "deed:agreement-renegotiated",
    "deed:agreement-ended",
];

impl DeedKind {
    /// The deed's process.
    pub fn key(&self) -> &'static str {
        KEYS[match self {
            Self::StoodBy => 0,
            Self::Abandoned => 1,
            Self::Shared => 2,
            Self::Took => 3,
            Self::OfferAccepted => 4,
            Self::OfferRefused => 5,
            Self::OfferCountered => 6,
            Self::AgreementFormed(_) => 7,
            Self::PerformedUnderAgreement(_) => 8,
            Self::AgreementRenegotiated(_) => 9,
            Self::AgreementEnded(_) => 10,
        }]
    }

    fn agreement(&self) -> Option<Id> {
        match self {
            Self::AgreementFormed(a)
            | Self::PerformedUnderAgreement(a)
            | Self::AgreementRenegotiated(a)
            | Self::AgreementEnded(a) => Some(*a),
            _ => None,
        }
    }

    /// Read back from an event's process and cause.
    pub fn of(process: &str, cause: Option<&str>) -> Option<Self> {
        let a = || cause?.strip_prefix("agreement:")?.parse().ok();
        Some(match KEYS.iter().position(|k| *k == process)? {
            0 => Self::StoodBy,
            1 => Self::Abandoned,
            2 => Self::Shared,
            3 => Self::Took,
            4 => Self::OfferAccepted,
            5 => Self::OfferRefused,
            6 => Self::OfferCountered,
            7 => Self::AgreementFormed(a()?),
            8 => Self::PerformedUnderAgreement(a()?),
            9 => Self::AgreementRenegotiated(a()?),
            _ => Self::AgreementEnded(a()?),
        })
    }

    /// What it does to the standing of whoever it was toward: trust, then
    /// liking.
    pub fn weight(&self) -> (i16, i16) {
        match self {
            Self::StoodBy => (3, 2),
            Self::Abandoned => (-5, -3),
            Self::Shared => (1, 2),
            Self::Took => (-2, -2),
            Self::OfferAccepted => (1, 0),
            Self::OfferRefused | Self::OfferCountered => (0, 0),
            Self::AgreementFormed(_) => (1, 1),
            Self::PerformedUnderAgreement(_) => (2, 1),
            Self::AgreementRenegotiated(_) => (0, 0),
            Self::AgreementEnded(_) => (0, -1),
        }
    }

    pub fn phrase(&self) -> &'static str {
        match self {
            Self::StoodBy => "stood by me",
            Self::Abandoned => "left me",
            Self::Shared => "shared with me",
            Self::Took => "took from me",
            Self::OfferAccepted => "took up my offer",
            Self::OfferRefused => "turned my offer down",
            Self::OfferCountered => "answered my offer with terms of their own",
            Self::AgreementFormed(_) => "agreed a standing arrangement with me",
            Self::PerformedUnderAgreement(_) => "did the agreed work",
            Self::AgreementRenegotiated(_) => "changed our terms",
            Self::AgreementEnded(_) => "ended our arrangement",
        }
    }
}

/// A deed as read from its event.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Deed {
    pub id: Key,
    pub at: Tick,
    pub doer: Id,
    pub toward: Option<Id>,
    pub kind: DeedKind,
}

/// The deed processes a world declares to keep deeds: no effects, each
/// leaving its event.
pub fn processes() -> BTreeMap<Key, Process> {
    KEYS.iter()
        .map(|k| {
            let mut p = crate::generate::process(k, Causation::Choice, vec![]);
            p.note = true;
            (k.to_string(), p)
        })
        .collect()
}

impl Simulation {
    /// Records a deed; returns its event's key.
    pub(crate) fn record_deed(&mut self, doer: Id, toward: Option<Id>, kind: DeedKind) -> Result<Key> {
        let cause = kind.agreement().map(|a| format!("agreement:{a}"));
        let receipt = self.execute(doer, toward, kind.key(), cause);
        match receipt.accepted() {
            true => Ok(receipt.id),
            false => Err(format!("a deed refused: {:?}", receipt.outcome)),
        }
    }

    /// Every deed, in the order done.
    pub fn deeds(&self) -> Vec<Deed> {
        let mut deeds: Vec<Deed> = self
            .state
            .events
            .values()
            .filter_map(|e| {
                Some(Deed {
                    id: e.id.clone(),
                    at: e.tick,
                    doer: e.subject,
                    toward: e.object,
                    kind: DeedKind::of(&e.process, e.cause.as_deref())?,
                })
            })
            .collect();
        let order: BTreeMap<&str, usize> = self.deed_order();
        deeds.sort_by_key(|d| (d.at, order.get(d.id.as_str()).copied().unwrap_or(usize::MAX)));
        deeds
    }

    pub fn deed(&self, id: &str) -> Option<Deed> {
        self.deeds().into_iter().find(|d| d.id == id)
    }

    /// Each event's place in the doers' own notes of their acts, which are
    /// kept in the order done.
    fn deed_order(&self) -> BTreeMap<&str, usize> {
        let acts = self.state.notes.iter().filter(|n| n.core.kind == "sim:act");
        acts.enumerate().map(|(i, n)| (n.core.object.as_str(), i)).collect()
    }
}

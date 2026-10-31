// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Deeds (rulings 51 and 56): who did what toward whom, each an act of a
//! deed process that leaves a world event, its object the other party, so
//! a deed spreads by the reach field like any event and its doer holds a
//! note of its own act. Standing is folded from them, never kept. What
//! deeds a world has, and what each does to standing, are its rules pack's
//! keys (778); the asks and agreements name the seven they record.

use crate::{
    Result,
    rules::{Causation, Process, Rules},
    schema::*,
    simulation::Simulation,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const OFFER_ACCEPTED: &str = "deed:offer-accepted";
pub const OFFER_REFUSED: &str = "deed:offer-refused";
pub const OFFER_COUNTERED: &str = "deed:offer-countered";
pub const AGREEMENT_FORMED: &str = "deed:agreement-formed";
pub const PERFORMED: &str = "deed:performed-under-agreement";
pub const RENEGOTIATED: &str = "deed:agreement-renegotiated";
pub const AGREEMENT_ENDED: &str = "deed:agreement-ended";
/// The deeds asks and agreements record, which a vocabulary declares.
pub const RECORDED: [&str; 7] = [
    OFFER_ACCEPTED,
    OFFER_REFUSED,
    OFFER_COUNTERED,
    AGREEMENT_FORMED,
    PERFORMED,
    RENEGOTIATED,
    AGREEMENT_ENDED,
];

/// What one deed does to the standing of whoever it was toward, and how
/// it is said.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeedRule {
    pub trust: i16,
    pub affinity: i16,
    pub phrase: String,
}

/// A world's social vocabulary (778): the crafts its peers hold, each kept
/// as the skill of its key, and its deeds by key.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Vocabulary {
    pub crafts: BTreeSet<Key>,
    pub deeds: BTreeMap<Key, DeedRule>,
}

impl Vocabulary {
    /// Keys namespaced, and every recorded deed declared.
    pub(crate) fn validate(&self, key: fn(&str) -> Result<()>) -> Result<()> {
        for k in self.crafts.iter().chain(self.deeds.keys()) {
            key(k)?;
        }
        match RECORDED.iter().find(|k| !self.deeds.contains_key(**k)) {
            Some(k) => Err(format!("a social vocabulary without {k}")),
            None => Ok(()),
        }
    }
    /// What `deed` does to standing: trust, then liking.
    pub fn weight(&self, deed: &str) -> (i16, i16) {
        self.deeds
            .get(deed)
            .map_or((0, 0), |d| (d.trust, d.affinity))
    }
    pub fn phrase<'a>(&'a self, deed: &'a str) -> &'a str {
        self.deeds.get(deed).map_or(deed, |d| d.phrase.as_str())
    }
}

/// The world's vocabulary, empty where it declares none.
pub fn vocabulary(rules: &Rules) -> std::borrow::Cow<'_, Vocabulary> {
    match &rules.social {
        Some(v) => std::borrow::Cow::Borrowed(v),
        None => std::borrow::Cow::Owned(Vocabulary::default()),
    }
}

/// What a deed was: its key and the agreement its event names (808).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct DeedKind {
    pub key: Key,
    pub agreement: Option<Id>,
}

impl DeedKind {
    pub fn new(key: impl Into<Key>) -> Self {
        Self {
            key: key.into(),
            agreement: None,
        }
    }
    pub fn under(key: impl Into<Key>, agreement: Id) -> Self {
        Self {
            key: key.into(),
            agreement: Some(agreement),
        }
    }
    pub fn is(&self, key: &str) -> bool {
        self.key == key
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

/// The deed processes `vocabulary` declares: no effects, each leaving its
/// event.
pub fn processes(vocabulary: &Vocabulary) -> BTreeMap<Key, Process> {
    vocabulary
        .deeds
        .keys()
        .map(|k| {
            let mut p = crate::generate::process(k, Causation::Choice, vec![]);
            p.note = true;
            (k.clone(), p)
        })
        .collect()
}

impl Simulation {
    /// Records a deed the world declares; returns its event's key.
    pub(crate) fn record_deed(
        &mut self,
        doer: Id,
        toward: Option<Id>,
        kind: &DeedKind,
    ) -> Result<Key> {
        if kind
            .agreement
            .is_some_and(|id| !self.state.agreements.contains_key(&id))
        {
            return Err("a deed under no agreement".into());
        }
        self.do_deed(doer, toward, kind)
    }

    /// The formation event precedes the new agreement, both in one social act.
    pub(super) fn record_formation(&mut self, doer: Id, toward: Id, agreement: Id) -> Result<Key> {
        self.do_deed(
            doer,
            Some(toward),
            &DeedKind::under(AGREEMENT_FORMED, agreement),
        )
    }

    fn do_deed(&mut self, doer: Id, toward: Option<Id>, kind: &DeedKind) -> Result<Key> {
        if !vocabulary(&self.genesis.rules)
            .deeds
            .contains_key(&kind.key)
        {
            return Err(format!("{} is no deed this world declares", kind.key));
        }
        if !self
            .genesis
            .rules
            .processes
            .get(&kind.key)
            .is_some_and(|p| p.note)
        {
            return Err("a deed process must leave its event".into());
        }
        let receipt = self.execute(doer, toward, &kind.key, None);
        match receipt.accepted() {
            true => {
                self.state
                    .events
                    .get_mut(&receipt.id)
                    .expect("a deed leaves its event")
                    .agreement = kind.agreement;
                Ok(receipt.id)
            },
            false => Err(format!("a deed refused: {:?}", receipt.outcome)),
        }
    }

    /// Every deed, in the order done.
    pub fn deeds(&self) -> Vec<Deed> {
        let v = vocabulary(&self.genesis.rules);
        let mut deeds: Vec<Deed> = self
            .state
            .events
            .values()
            .filter(|e| v.deeds.contains_key(&e.process))
            .map(|e| Deed {
                id: e.id.clone(),
                at: e.tick,
                doer: e.subject,
                toward: e.object,
                kind: DeedKind {
                    key: e.process.clone(),
                    agreement: e.agreement,
                },
            })
            .collect();
        let order: BTreeMap<&str, usize> = self.deed_order();
        deeds.sort_by_key(|d| {
            (
                d.at,
                order.get(d.id.as_str()).copied().unwrap_or(usize::MAX),
            )
        });
        deeds
    }

    pub fn deed(&self, id: &str) -> Option<Deed> {
        self.deeds().into_iter().find(|d| d.id == id)
    }

    /// What `deed` does to standing in this world.
    pub fn weight(&self, deed: &DeedKind) -> (i16, i16) {
        vocabulary(&self.genesis.rules).weight(&deed.key)
    }

    /// Each event's place in the doers' own notes of their acts, which are
    /// kept in the order done.
    fn deed_order(&self) -> BTreeMap<&str, usize> {
        let acts = self.state.notes.iter().filter(|n| n.core.kind == "sim:act");
        acts.enumerate()
            .map(|(i, n)| (n.core.object.as_str(), i))
            .collect()
    }
}

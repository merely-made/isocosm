// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use crate::schema::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccountKind {
    Matter { lineage: Key },
    Energy,
    Attention,
    Time,
    Obligation,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Shape {
    Choice,
    Agentless,
    Transition,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Binding {
    Actor,
    Target,
    Place,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Query {
    Alive(Binding),
    Trait {
        who: Binding,
        key: Key,
    },
    Account {
        who: Binding,
        key: Key,
        at_least: u64,
    },
    Below {
        who: Binding,
        key: Key,
        amount: u64,
    },
    Age {
        at_least: Tick,
    },
    Condition {
        key: Key,
        at_least: i64,
    },
    Part {
        who: Binding,
        revision: u64,
        part: Id,
    },
    Related {
        kind: Key,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Effect {
    Transfer {
        from: Binding,
        to: Binding,
        account: Key,
        amount: u64,
    },
    /// An authored transform accounts for both sides, including byproducts.
    Transform {
        who: Binding,
        take: Ledger,
        give: Ledger,
    },
    Condition {
        key: Key,
        delta: i64,
    },
    Relate {
        kind: Key,
        present: bool,
    },
    Trait {
        who: Binding,
        key: Key,
        present: bool,
    },
    Practice {
        key: Key,
        amount: u64,
    },
    Move {
        destination: Id,
    },
    Note {
        kind: Key,
        text: String,
        lifetime: Option<Tick>,
    },
    Birth {
        provision: Ledger,
    },
    Death,
    Tell {
        event: Key,
    },
    FoundPolity {
        governance: Key,
        focus: BTreeSet<Key>,
        support: Key,
    },
    /// A measured account on an authored axis, judged against standing history.
    Record {
        axis: Key,
        account: Key,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Risk {
    pub per_million: u32,
    pub effects: Vec<Effect>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Target {
    pub same_place: bool,
    pub alive: Option<bool>,
    pub lineage: Option<Key>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Process {
    pub id: Key,
    pub shape: Shape,
    pub requires: Vec<Query>,
    pub commitments: Vec<Effect>,
    pub effects: Vec<Effect>,
    pub risk: Option<Risk>,
    pub target: Option<Target>,
    pub period: Option<Tick>,
    pub priority: i32,
    pub need_account: Option<Key>,
    pub need_below: u64,
    pub glyphs: BTreeSet<Key>,
    pub invariants: BTreeSet<Key>,
    pub note: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rules {
    pub version: u32,
    pub accounts: BTreeMap<Key, AccountKind>,
    pub conditions: BTreeSet<Key>,
    pub traits: BTreeSet<Key>,
    pub relations: BTreeSet<Key>,
    pub note_kinds: BTreeSet<Key>,
    pub processes: BTreeMap<Key, Process>,
    pub field: FieldPolicy,
    pub limits: Limits,
    pub epoch_ticks: Tick,
    pub collection_buffer: Tick,
}

impl Rules {
    pub fn revision(&self) -> Key {
        crate::digest(self)
    }
    pub fn validate(&self) -> crate::Result<()> {
        crate::validation::rules(self)
    }
}

impl Process {
    /// Conservative executable proof of independence. No shared writes, targets,
    /// identity draws, ancestry or public events can hide inside a batch.
    pub fn bulk_safe(&self) -> bool {
        self.risk.is_none()
            && self.target.is_none()
            && !self.note
            && self.requires.iter().all(|q| {
                matches!(
                    q,
                    Query::Alive(Binding::Actor)
                        | Query::Trait {
                            who: Binding::Actor,
                            ..
                        }
                        | Query::Account {
                            who: Binding::Actor,
                            ..
                        }
                        | Query::Condition { .. }
                        | Query::Below {
                            who: Binding::Actor,
                            ..
                        }
                        | Query::Age { .. }
                        | Query::Part {
                            who: Binding::Actor,
                            ..
                        }
                )
            })
            && self.commitments.iter().chain(&self.effects).all(|e| {
                matches!(
                    e,
                    Effect::Transform {
                        who: Binding::Actor,
                        ..
                    } | Effect::Trait {
                        who: Binding::Actor,
                        ..
                    } | Effect::Practice { .. }
                )
            })
    }
}

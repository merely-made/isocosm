// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use crate::schema::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

mod body;
mod competition;
mod mind;

pub(crate) use body::expressing;
pub use body::{Function, SHAPES, Seeding, default_functions, default_shapes};
pub use competition::{Competition, Competitor, Similitude};
pub use mind::{Mind, Need};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccountKind {
    Matter {
        lineage: Key,
    },
    Energy,
    Attention,
    Time,
    Obligation,
    /// A mind's kept strain (ruling 159).
    Strain,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Causation {
    Choice,
    Agentless,
    Transition,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Binding {
    Actor,
    Target,
    Place,
    /// The actor's part the process's `Expresses` requirement binds
    /// (ruling 338). A part has traits and life but keeps no ledger.
    Part,
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
    /// The actor's mood, read from the world's needs (ruling 227), is at
    /// least `at_least`.
    Mood {
        at_least: i64,
    },
    /// The actor's mood is below `amount`.
    MoodBelow {
        amount: i64,
    },
    /// A body holds at least `at_least` matter, over every matter account it
    /// keeps (ruling 287).
    Holds {
        who: Binding,
        at_least: u64,
    },
    /// The actor has a live part expressing `function` (ruling 338). A
    /// process requires at most one, and binds the lowest-numbered such part
    /// as `Binding::Part`; its receipt reads that part's address.
    Expresses {
        function: Key,
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
        /// The kind of conversion it declares, checked against that kind
        /// (rulings 342 and 357); undeclared transforms pass as before, and
        /// serialize as they did.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        conversion: Option<Conversion>,
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
    /// A body's kept level eases by up to `amount`, never below nothing:
    /// strain bleeding off (ruling 159).
    Ease {
        who: Binding,
        key: Key,
        amount: u64,
    },
    /// Eating (ruling 287): up to `amount` of a body's matter, drawn from
    /// all its matter accounts in proportion, largest remainders first in
    /// key order, and credited to the actor's own `into` account.
    Eat {
        from: Binding,
        amount: u64,
        into: Key,
    },
}

/// The conversions a transform may declare (rulings 342 and 357). World
/// matter is matter of a lineage of the world's kingdom (rulings 98 and 100);
/// all other matter is living.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Conversion {
    /// World matter into the body's own lineage's: whatever synthesizes is a
    /// producer.
    Synthesis,
    /// Living matter into the eater's own lineage's.
    Digestion,
    /// Living matter back into the world's.
    Mineralization,
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
    /// Any of these lineages (ruling 287); empty bounds nothing. Absent in
    /// selectors that name none, which serialize as before it existed.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub among: BTreeSet<Key>,
    /// The target is drawn, seeded and keyed by the act, weighted by the
    /// matter each eligible candidate holds, instead of taken first in
    /// identity order (ruling 287).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub weighted: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Process {
    pub id: Key,
    /// A process's causal kind (ruling 340), saved as `shape`, the name
    /// it had before part shapes took it, so old worlds load.
    #[serde(rename = "shape")]
    pub causation: Causation,
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
    /// Rulings 218 and 236: the world's competitions, by the site account
    /// each contests. Worlds without any serialize, and so hash, as before
    /// the field existed.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub competitions: BTreeMap<Key, Competition>,
    /// Ruling 218: the bounds a crowd must keep its readings within.
    /// Absent in worlds that state none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub similitude: Option<Similitude>,
    /// What the core reads of minds (rulings 221 and 227). Absent in worlds
    /// without one, which serialize and hash as before it existed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mind: Option<Mind>,
    /// The clock's unit (rulings 256 and 257): the world time one tick
    /// counts, in microseconds, in which process periods are read; anything
    /// finer is the foreground game's to resolve. Absent means a minute, and
    /// worlds without it serialize and hash as before it existed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tick_microseconds: Option<u64>,
    /// The shapes a world's parts take (rulings 264 and 276): open keys,
    /// the eight of `default_shapes` when a world adopts them. Worlds naming
    /// none serialize and hash as before the field existed.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub shapes: BTreeSet<Key>,
    /// The function catalogue (rulings 278, 338 and 339), by function.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub functions: BTreeMap<Key, Function>,
}

/// The clock's unit where a world states none: a minute (ruling 257).
pub const DEFAULT_TICK_MICROSECONDS: u64 = 60_000_000;

impl Rules {
    pub fn revision(&self) -> Key {
        crate::digest(self)
    }
    /// The world time one tick counts, in microseconds.
    pub fn tick_microseconds(&self) -> u64 {
        self.tick_microseconds.unwrap_or(DEFAULT_TICK_MICROSECONDS)
    }
    pub fn validate(&self) -> crate::Result<()> {
        crate::validation::rules(self)
    }
}

impl Process {
    /// The function whose part this process binds: its `Expresses`
    /// requirement, if it has one.
    pub fn expresses(&self) -> Option<&Key> {
        self.requires.iter().find_map(|q| match q {
            Query::Expresses { function } => Some(function),
            _ => None,
        })
    }

    /// Conservative executable proof of independence. No shared writes, targets,
    /// identity draws, ancestry or public events can hide inside a batch. The
    /// bound part is the actor's own, alike across a cohort.
    pub fn bulk_safe(&self) -> bool {
        self.risk.is_none()
            && self.target.is_none()
            && !self.note
            && self.requires.iter().all(|q| {
                matches!(
                    q,
                    Query::Alive(Binding::Actor | Binding::Part)
                        | Query::Trait {
                            who: Binding::Actor | Binding::Part,
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
                        | Query::Mood { .. }
                        | Query::MoodBelow { .. }
                        | Query::Holds {
                            who: Binding::Actor,
                            ..
                        }
                        | Query::Expresses { .. }
                )
            })
            && self.commitments.iter().chain(&self.effects).all(|e| {
                matches!(
                    e,
                    Effect::Transform {
                        who: Binding::Actor,
                        ..
                    } | Effect::Trait {
                        who: Binding::Actor | Binding::Part,
                        ..
                    } | Effect::Practice { .. }
                        | Effect::Ease {
                            who: Binding::Actor,
                            ..
                        }
                )
            })
    }
}

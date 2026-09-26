// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use crate::schema::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

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
    /// The actor's mood, read from the world's needs (ruling 227), is at
    /// least `at_least`.
    Mood {
        at_least: i64,
    },
    /// The actor's mood is below `amount`.
    MoodBelow {
        amount: i64,
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
    /// A body's kept level eases by up to `amount`, never below nothing:
    /// strain bleeding off (ruling 159).
    Ease {
        who: Binding,
        key: Key,
        amount: u64,
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

/// One competing lineage in a competition: how its members are told apart,
/// what they grow into, when they want the resource, and the acts the
/// competition executes for them.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Competitor {
    pub identity: Key,
    pub body: Key,
    pub hungry: Query,
    pub eat: Key,
    pub share: Key,
    /// One unit of reserve spent in a fight, and the strain it costs.
    pub spend: Key,
}

/// Ruling 115: members wanting one scarce thing at a site, each side's own
/// way of deciding picking contest or share, the sim resolving the choices.
/// Two contesters size each other up and only a close match escalates
/// (ruling 116), into rounds that strain both sides against their bearing
/// (rulings 221 to 223).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Competition {
    /// The site account contended for.
    pub food: Key,
    pub ration: u64,
    /// The leaning trait: members that carry it contest, the rest share.
    pub contest: Key,
    /// The widest gap in standing that still reads as a close match.
    pub margin: u64,
    /// Reserve the side losing an exchange spends, capped at what it holds.
    pub cost: u64,
    /// The act each side takes for each round: the round's strain.
    pub round: Key,
    /// Per mille, the chance an exchange goes against the side standing
    /// higher.
    pub upset: u32,
    /// How far a break up raises its side's standing, or a break down
    /// lowers it, for the rest of the fight (ruling 222).
    pub advantage: u64,
    pub kinds: Vec<Competitor>,
}

/// A need, as the core has needs now (ruling 227): members carrying every
/// one of `traits` for whom `query` holds have their mood moved by `weight`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Need {
    pub traits: BTreeSet<Key>,
    pub query: Query,
    pub weight: i64,
}

/// What the core reads of a mind now. Mood is read from needs and never
/// kept; strain is kept in its account (rulings 158, 159 and 227). A mind
/// bears strain up to its bearing, set by its traits (ruling 164); past it,
/// it breaks, up with the chance its traits and the moment give (ruling 163).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Mind {
    pub strain: Key,
    pub needs: Vec<Need>,
    pub bearing: i64,
    pub bearing_traits: BTreeMap<Key, i64>,
    /// Per mille chance that a break goes up.
    pub rise: i64,
    pub rise_traits: BTreeMap<Key, i64>,
    /// The moment: per mille added to the rise for each point of mood.
    pub stake: i64,
}

/// Ruling 113's tolerance: each reading's Kolmogorov-Smirnov distance
/// between the two ways, per mille, within its own bound.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Similitude {
    pub default_bound: u32,
    pub bounds: BTreeMap<Key, u32>,
}

impl Similitude {
    pub fn bound(&self, reading: &str) -> u32 {
        self.bounds
            .get(reading)
            .copied()
            .unwrap_or(self.default_bound)
    }
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
    /// Ruling 218: the world's competitions, by id. Worlds without any
    /// serialize, and so hash, as before the field existed.
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
                        | Query::Mood { .. }
                        | Query::MoodBelow { .. }
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
                        | Effect::Ease {
                            who: Binding::Actor,
                            ..
                        }
                )
            })
    }
}

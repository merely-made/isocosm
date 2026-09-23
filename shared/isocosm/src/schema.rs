// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub type Id = u64;
pub type Tick = u64;
pub type Key = String;
pub type Ledger = BTreeMap<Key, u64>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Storage {
    Asserted,
    Derived,
    Setting,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Provenance {
    Born(Key),
    Made(Id),
    Intrinsic(Key),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Method {
    Inert,
    Reactive,
    Deliberative,
    Normative,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Part {
    pub parent: Option<Id>,
    pub traits: BTreeSet<Key>,
    pub severed: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Tenet {
    pub need: u32,
    pub trust: u32,
    pub approval: u32,
    pub variance: u32,
}

/// Every field is asserted. Phenotype, needs, capability and standing are readings.
/// Quantity is per member; a cohort's total is quantity times multiplicity.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Entity {
    pub lineage: Key,
    pub kingdom: Key,
    pub scale: Key,
    pub provenance: Provenance,
    pub method: Method,
    pub place: Id,
    pub arrived: Tick,
    pub visits: Vec<Visit>,
    pub born: Tick,
    pub alive: bool,
    pub body_revision: u64,
    pub parts: BTreeMap<Id, Part>,
    pub traits: BTreeSet<Key>,
    pub accounts: Ledger,
    pub skills: BTreeMap<Key, u64>,
    pub tenets: BTreeMap<Key, Tenet>,
    pub disposition: [i16; 5],
}

/// A past residence. The departure tick belongs to the next place.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Visit {
    pub place: Id,
    pub from: Tick,
    pub until: Tick,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Lineage {
    pub parent: Option<Key>,
    pub revision: u64,
    pub traits: BTreeSet<Key>,
    pub kingdom: Key,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Route {
    pub to: Id,
    pub travel: Tick,
    pub transmission: u32,
}

/// Sites are asserted graph nodes; kind/biome are derived from conditions.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Site {
    pub terrain_seed: u64,
    pub conditions: BTreeMap<Key, i64>,
    pub accounts: Ledger,
    pub routes: Vec<Route>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Location {
    pub sites: BTreeSet<Id>,
    pub claims: BTreeSet<Id>,
    pub parent: Option<Id>,
    pub ratio: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Relation {
    pub subject: Id,
    pub kind: Key,
    pub object: Id,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Constitution {
    pub members: BTreeSet<Id>,
    pub governance: Key,
    pub focus: BTreeSet<Key>,
    pub support_account: Key,
    pub host: Option<Id>,
    pub founded_by: Key,
}

/// State belongs to the polity; a faction is a set of membership relations.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Polity {
    pub constitution: Constitution,
    pub accounts: Ledger,
    pub ended_by: Option<Key>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Magic {
    pub source: Key,
    pub gate: Key,
    pub orientation: Key,
    pub costs: BTreeSet<Key>,
    pub suspended_invariants: BTreeSet<Key>,
    pub rhythm: Key,
    pub manifestation: Key,
}

/// Impresa's closed causal core, with the storing sim's open envelope.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Note {
    pub core: wing_impresa::Record,
    pub djot: String,
    pub extra: BTreeMap<Key, String>,
    pub expires: Option<Tick>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorldTraits {
    pub shape: Key,
    pub scale: Key,
    pub base_unit_micrometres: u64,
    pub static_traits: BTreeSet<Key>,
    pub magic: BTreeMap<Key, Magic>,
    pub canon: wing_glyphs::CanonSpec,
    pub parent_world: Option<Key>,
    pub neighbours: BTreeMap<Key, Key>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Event {
    pub id: Key,
    pub tick: Tick,
    pub place: Id,
    pub subject: Id,
    pub process: Key,
    pub cause: Option<Key>,
    pub strength: u32,
    pub legend: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldPolicy {
    pub strength: u32,
    pub decay_per_tick: u32,
    pub legend_floor: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Limits {
    pub entities: u64,
    pub sites: usize,
    pub processes: usize,
    pub operations: usize,
    pub events_per_advance: usize,
    pub notes: usize,
    pub history: usize,
    pub advance_ticks: u64,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            entities: 1_000_000,
            sites: 4096,
            processes: 1024,
            operations: 128,
            events_per_advance: 1_000_000,
            notes: 100_000,
            history: 1_000_000,
            advance_ticks: 100_000,
        }
    }
}

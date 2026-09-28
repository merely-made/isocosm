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
    /// One of the world's shapes (ruling 276); empty in a part from before
    /// shapes, which expresses nothing. Parts without one serialize and hash
    /// as before the field existed.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub shape: Key,
    /// The catalogue functions this part expresses, each admitted by its
    /// shape (ruling 338).
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub functions: BTreeSet<Key>,
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
    /// Where the route crosses between the two sites' frames (ruling 397).
    /// Absent on worlds without geometry, which serialize and hash as before.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub border: Option<Border>,
}

/// One side of a site meeting one side of a neighbour. Side `k` runs from
/// the footprint's corner `k` to corner `k + 1`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Border {
    /// The side of this site the route leaves by.
    pub side: u8,
    /// The side of the neighbour it enters by.
    pub enters: u8,
    /// The sides meet end to end in the same direction rather than the
    /// usual opposite one; never on planes, rings or tori.
    #[serde(default, skip_serializing_if = "is_false")]
    pub flipped: bool,
}

fn is_false(value: &bool) -> bool {
    !*value
}

/// A site's outline in its own frame: its sides, and each side's length in
/// base units (ruling 395: squares first).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Footprint {
    pub sides: u8,
    pub side: u64,
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
    /// Every site's outline, on worlds with geometry. Absent worlds serialize
    /// and hash as before.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub footprint: Option<Footprint>,
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
    /// The work an advance may do: the members its evaluations stand for,
    /// the same grouped and individually (rulings 259 and 285).
    pub events_per_advance: usize,
    pub notes: usize,
    pub history: usize,
    /// Retired by ruling 284, which limits an advance's work and not its
    /// ticks. Kept so saved worlds keep their digest; nothing reads it.
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

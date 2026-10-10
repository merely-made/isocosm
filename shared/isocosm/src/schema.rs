// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub use isometer_core::{BodyDocument, PartId};

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

/// A part's physiology, keyed by its id in the body's document (rulings 674
/// and 699): its geometry, attachment, place in the plan, declared name and
/// tombstone are the document's, read through [`crate::geometry`].
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Part {
    pub traits: BTreeSet<Key>,
    /// The catalogue functions this part expresses (ruling 338), any of
    /// them on any shape (ruling 492).
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub functions: BTreeSet<Key>,
    /// The cells each function it expresses holds (X6's allocation), never
    /// more in all than its capacity.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub cells: BTreeMap<Key, u32>,
    /// The matter it holds, its own ledger (ruling 504): its tissue, and its
    /// reserve where it stores.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub matter: Ledger,
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
    /// Its body's geometry (674): isometer's document, absent in bodies
    /// whose parts have none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<BodyDocument>,
    pub parts: BTreeMap<PartId, Part>,
    pub traits: BTreeSet<Key>,
    pub accounts: Ledger,
    pub skills: BTreeMap<Key, u64>,
    pub tenets: BTreeMap<Key, Tenet>,
    pub disposition: [i16; 5],
    /// Each tagma's segments as the body drew them (478, 495), what an
    /// epimorphic body grows toward; absent in bodies without a recipe.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub soma: Vec<u8>,
    /// The systems it carries (rulings 568, 574 and 583), the world's
    /// defaults as founding realized them, riffed and passed down; absent in
    /// bodies that read none, which serialize and hash as before.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub systems: BTreeMap<Key, crate::rules::System>,
    /// The cells that varied from its recipe, its own and its forebears'
    /// (rulings 576 to 579): passed down, and grown again where a part
    /// regrows.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub varied: Vec<Varied>,
    /// The patch or room its game placed it in, inside a lifted site
    /// (rulings 422 and 740); set only on a member split out of its cohort,
    /// updated by its game as it moves within the site, and cleared when it
    /// leaves the site.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub patch: Option<isometer_space::places::PlaceId>,
    /// An authored character's fill (769); absent elsewhere, which serialize and hash as before.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authored: Option<crate::asserted::Character>,
}

/// A cell varied at birth (576): where it lies in the recipe, and the
/// function it left for the one it took.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Varied {
    pub situs: [u8; 3],
    pub from: Key,
    pub to: Key,
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
    /// What its bodies develop from (rulings 478 and 511); lineages without
    /// one serialize and hash as before.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub development: Option<crate::rules::Development>,
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
    /// An authored route's fill (769); absent elsewhere, which serialize and hash as before.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authored: Option<crate::asserted::Route>,
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
    /// An authored place's fill (769); absent elsewhere, which serialize and hash as before.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authored: Option<crate::asserted::Place>,
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
    /// Its weight, as a bond's (ruling 688): one relation per subject, kind
    /// and object whatever its value; nought in older saves, which
    /// serialize and hash as before.
    #[serde(default, skip_serializing_if = "is_nought")]
    pub value: i64,
}

impl Relation {
    pub fn new(subject: Id, kind: impl Into<Key>, object: Id) -> Self {
        Self {
            subject,
            kind: kind.into(),
            object,
            value: 0,
        }
    }
    /// Whether `other` is this relation, whatever either's value.
    pub fn same(&self, other: &Relation) -> bool {
        (self.subject, &self.kind, self.object) == (other.subject, &other.kind, other.object)
    }
}

/// The relation `subject` holds of kind `kind` to `object`, whatever its value.
pub fn related<'a>(
    relations: &'a BTreeSet<Relation>,
    subject: Id,
    kind: &str,
    object: Id,
) -> Option<&'a Relation> {
    let at = |value| Relation {
        value,
        ..Relation::new(subject, kind, object)
    };
    relations.range(at(i64::MIN)..=at(i64::MAX)).next()
}

fn is_nought(value: &i64) -> bool {
    *value == 0
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
    /// What an authored polity was asserted with (rulings 757 and 760);
    /// absent in derived polities, which serialize and hash as before.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authored: Option<Authored>,
}

/// An authored polity's own attributes, filling its place while it has no
/// members (760): its authored key, name, tags, and the authored keys of
/// what it claims.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Authored {
    pub key: Key,
    pub name: String,
    pub tags: BTreeSet<Key>,
    pub claims: BTreeSet<Key>,
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
    /// What a voxel's material id names, by position (ruling 403). Empty in
    /// worlds without terrain, which serialize and hash as before.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub materials: Vec<Material>,
}

/// A voxel material: the nis it is, by key, and the lineage it is nis of.
/// A founder may give it a density, the amount one base-unit cell holds,
/// and the matter account that amount moves through (rulings 412 and
/// 739); until both are set, only the dev source edits it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Material {
    pub key: Key,
    pub lineage: Key,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub density: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub account: Option<Key>,
}

/// An edit to a site's volume as the sim keeps it (rulings 413 and 696):
/// when, in which site's frame, and the shape operation itself. Its place
/// in `State::edits` is its global sequence.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Edited {
    pub tick: Tick,
    pub site: Id,
    pub edit: isometer_space::Edit,
    /// The member whose ledger it moved matter through, or none for the
    /// dev source.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub by: Option<Id>,
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
    /// An authored history line's fill (769); absent elsewhere, which serialize and hash as before.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authored: Option<crate::asserted::HistoryLine>,
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

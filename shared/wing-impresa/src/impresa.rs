// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! One association record, and an ordered run of them read as an impresa.
//! Data only: nothing here accepts an event, decays an association, resolves
//! an identity to a product noun or decides what a kind means. A product
//! appends records to its own accepted history and projects the readings it
//! wants; this crate owns the shape a record is written in.

use crate::{
    DEFAULT_MAX_CAUSE_BYTES, DEFAULT_MAX_ENTRIES, DEFAULT_MAX_JSON_BYTES, KindSet, SCHEMA_VERSION,
    bounded_text, identifier,
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, sync::Arc};

/// An opaque subject identity, `namespace:local`. The kernel never resolves
/// it; a product maps it to its own pointable thing. A glyph, a faction and a
/// lot of matter are one shape here, which is what lets an impresa cross
/// vessels as records rather than as a schema.
pub type SubjectId = String;

/// The other end of the same relation, under the same rule and the same
/// namespace, so the two ends of a record are interchangeable to read and
/// only the kind carries the direction.
pub type ObjectId = String;

/// Whether a record makes an association or ends one.
///
/// **A lapse is a record.** Never a deletion and never an absent row: the
/// cause of an ending is as pointable as the cause of a beginning, and a
/// later re-association has to be legible on top of both.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Stance {
    Associate,
    Lapse,
}

/// The one record type. Every field is what an accepted event already knew,
/// so writing one is transcription and never inference.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    pub subject: SubjectId,
    pub object: ObjectId,
    pub kind: String,
    /// The canon revision the citing event was accepted under, kept as an
    /// acquisition keeps one: what a kind meant then is not necessarily what
    /// it means now, and the record is not rewritten when it changes.
    pub canon_revision: u64,
    /// The cause receipt: the evidence string of the accepted event this
    /// record cites. Opaque here — the kernel neither parses nor resolves it.
    pub cause: String,
    pub tick: u64,
    pub stance: Stance,
}

impl Record {
    /// Admissible in a world configured with these kinds.
    pub fn validate(&self, kinds: &KindSet) -> Result<(), String> {
        self.shape()?;
        if !kinds.contains(&self.kind) {
            return Err(format!("unknown association kind: {}", self.kind));
        }
        Ok(())
    }

    /// What is true of a record wherever it travels, kinds aside. Separated
    /// because a record outlives the world it was written in: a vessel can
    /// hold a well-formed record whose kind it has not configured, and that
    /// is a mapping question at the boundary, not a malformed record.
    fn shape(&self) -> Result<(), String> {
        identifier(&self.subject)?;
        identifier(&self.object)?;
        identifier(&self.kind)?;
        // Self-association carries no relation and would make every subject
        // its own object in a projection.
        if self.subject == self.object {
            return Err(format!("{} cannot be associated with itself", self.subject));
        }
        bounded_text(&self.cause, "cause receipt", DEFAULT_MAX_CAUSE_BYTES)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImpresaLimits {
    pub entries: usize,
}
impl Default for ImpresaLimits {
    fn default() -> Self {
        Self {
            entries: DEFAULT_MAX_ENTRIES,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImpresaSpec {
    pub version: u32,
    pub id: String,
    /// In tick order, non-decreasing. Order is the value's only clock: it is
    /// what makes a lapse and a later re-association readable as a sequence
    /// rather than as two competing claims.
    pub entries: Vec<Record>,
    #[serde(default)]
    pub limits: ImpresaLimits,
}

/// An admitted impresa: the immutable spec plus the two indices its readings
/// need. Both directions, because a faction wants to know what is associated
/// with it as often as a character wants to know what it is associated with.
///
/// A triple's history is filtered out of the subject's run rather than
/// indexed again — the third index would pay for itself only in a store, and
/// storage is the product's.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "ImpresaSpec", into = "ImpresaSpec")]
pub struct Impresa {
    spec: Arc<ImpresaSpec>,
    by_subject: Arc<BTreeMap<SubjectId, Vec<usize>>>,
    by_object: Arc<BTreeMap<ObjectId, Vec<usize>>>,
}

impl Impresa {
    pub fn new(spec: ImpresaSpec, kinds: &KindSet) -> Result<Self, String> {
        Self::admit(spec, Some(kinds))
    }

    fn admit(spec: ImpresaSpec, kinds: Option<&KindSet>) -> Result<Self, String> {
        if spec.version != SCHEMA_VERSION {
            return Err("unsupported impresa version".into());
        }
        identifier(&spec.id)?;
        if spec.entries.len() > spec.limits.entries {
            return Err("impresa exceeds entry limit".into());
        }
        let mut by_subject: BTreeMap<SubjectId, Vec<usize>> = BTreeMap::new();
        let mut by_object: BTreeMap<ObjectId, Vec<usize>> = BTreeMap::new();
        let mut previous = 0u64;
        for (index, record) in spec.entries.iter().enumerate() {
            match kinds {
                Some(kinds) => record.validate(kinds)?,
                None => record.shape()?,
            }
            // Equal ticks are fine: two associations can be accepted in one
            // settlement. Going backwards is not, since the last record for a
            // triple would stop being its latest stance.
            if record.tick < previous {
                return Err(format!(
                    "record ticks must not decrease: {} after {}",
                    record.tick, previous
                ));
            }
            previous = record.tick;
            by_subject
                .entry(record.subject.clone())
                .or_default()
                .push(index);
            by_object
                .entry(record.object.clone())
                .or_default()
                .push(index);
        }
        Ok(Self {
            spec: Arc::new(spec),
            by_subject: Arc::new(by_subject),
            by_object: Arc::new(by_object),
        })
    }

    pub fn spec(&self) -> &ImpresaSpec {
        &self.spec
    }
    pub fn len(&self) -> usize {
        self.spec.entries.len()
    }
    pub fn is_empty(&self) -> bool {
        self.spec.entries.is_empty()
    }

    /// The latest stance for one triple, or `None` when nothing has ever been
    /// recorded for it. Absence is not a lapse: never having invoked a glyph
    /// and having stopped are different facts, and a product that conflates
    /// them loses the cause.
    pub fn stance(&self, subject: &str, object: &str, kind: &str) -> Option<Stance> {
        self.history(subject, object, kind).last().map(|r| r.stance)
    }

    /// Every record this subject is the subject of, in record order.
    pub fn associations<'a>(&'a self, subject: &str) -> impl Iterator<Item = &'a Record> + use<'a> {
        let entries = &self.spec.entries;
        indices(&self.by_subject, subject)
            .iter()
            .map(move |&i| &entries[i])
    }

    /// The inverse: every record this object is the object of, in record
    /// order.
    pub fn about<'a>(&'a self, object: &str) -> impl Iterator<Item = &'a Record> + use<'a> {
        let entries = &self.spec.entries;
        indices(&self.by_object, object)
            .iter()
            .map(move |&i| &entries[i])
    }

    /// Every record for one triple, in order, so a lapse and a later
    /// re-association are both pointable with their own causes.
    pub fn history<'a>(
        &'a self,
        subject: &str,
        object: &str,
        kind: &str,
    ) -> impl Iterator<Item = &'a Record> + use<'a> {
        let (object, kind) = (object.to_owned(), kind.to_owned());
        let entries = &self.spec.entries;
        indices(&self.by_subject, subject)
            .iter()
            .map(move |&i| &entries[i])
            .filter(move |record| record.object == object && record.kind == kind)
    }

    /// Whether a world configured with these kinds admits every record here.
    ///
    /// The counterpart of `ExpressionTable::covers(&Canon)` in the sibling
    /// kernel, and needed for the same reason: the serde door
    /// ([`TryFrom<ImpresaSpec>`]) admits shape alone, because a kind set
    /// cannot be threaded through `Deserialize`. A caller that deserialized
    /// an `Impresa` directly rather than through [`Self::from_json`] finishes
    /// the check here. Names the first unconfigured kind rather than counting
    /// them, since the fix is a mapping at the boundary.
    pub fn covers(&self, kinds: &KindSet) -> Result<(), String> {
        for record in &self.spec.entries {
            if !kinds.contains(&record.kind) {
                return Err(format!("unknown association kind: {}", record.kind));
            }
        }
        Ok(())
    }

    pub fn to_json(&self) -> Result<String, String> {
        self.to_json_with_limit(DEFAULT_MAX_JSON_BYTES)
    }
    pub fn to_json_with_limit(&self, limit: usize) -> Result<String, String> {
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        if json.len() > limit {
            return Err("impresa JSON exceeds limit".into());
        }
        Ok(json)
    }
    /// Reading takes the kind set, because the set is the world's
    /// configuration and is deliberately not carried in the file: serde alone
    /// admits the record shape, this admits the kinds with it.
    pub fn from_json(json: &str, kinds: &KindSet) -> Result<Self, String> {
        Self::from_json_with_limit(json, kinds, DEFAULT_MAX_JSON_BYTES)
    }
    pub fn from_json_with_limit(json: &str, kinds: &KindSet, limit: usize) -> Result<Self, String> {
        if json.len() > limit {
            return Err("impresa JSON exceeds limit".into());
        }
        let spec = serde_json::from_str(json).map_err(|e| e.to_string())?;
        Self::new(spec, kinds)
    }
}

fn indices<'a>(map: &'a BTreeMap<String, Vec<usize>>, key: &str) -> &'a [usize] {
    map.get(key).map(Vec::as_slice).unwrap_or_default()
}

impl TryFrom<ImpresaSpec> for Impresa {
    type Error = String;
    /// Shape only: a kind set cannot reach this door, so `from_json` is the
    /// one products read through.
    fn try_from(spec: ImpresaSpec) -> Result<Self, Self::Error> {
        Self::admit(spec, None)
    }
}
impl From<Impresa> for ImpresaSpec {
    fn from(value: Impresa) -> Self {
        Arc::try_unwrap(value.spec).unwrap_or_else(|spec| (*spec).clone())
    }
}

#[cfg(test)]
#[path = "impresa_tests.rs"]
mod tests;

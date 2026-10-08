// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The kinds an association may have, as a world configures them. Closed, so
//! a record cannot invent its own relation by spelling one; extensible by a
//! pack, exactly as the glyph canon is, so a world is not stuck with six.

use crate::{DEFAULT_MAX_KINDS, SCHEMA_VERSION, identifier};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{Arc, OnceLock},
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KindSetLimits {
    pub kinds: usize,
}
impl Default for KindSetLimits {
    fn default() -> Self {
        Self {
            kinds: DEFAULT_MAX_KINDS,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KindSetSpec {
    pub version: u32,
    pub id: String,
    /// Authored order is kept: a world lists its own kinds before the packs'
    /// additions, and a file round-trips as it was written.
    pub kinds: Vec<String>,
    #[serde(default)]
    pub limits: KindSetLimits,
}

/// An admitted kind set: the immutable spec plus a lookup set, built the way
/// every other value in the wing is built.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "KindSetSpec", into = "KindSetSpec")]
pub struct KindSet {
    spec: Arc<KindSetSpec>,
    kinds: Arc<BTreeSet<String>>,
}

impl KindSet {
    pub fn new(spec: KindSetSpec) -> Result<Self, String> {
        if spec.version != SCHEMA_VERSION {
            return Err("unsupported kind set version".into());
        }
        identifier(&spec.id)?;
        // Empty is a refusal, not a world that happens to associate nothing:
        // no record could ever be admitted against it, so the set is an
        // authoring mistake rather than a configuration.
        if spec.kinds.is_empty() {
            return Err("kind set requires at least one kind".into());
        }
        if spec.kinds.len() > spec.limits.kinds {
            return Err("kind set exceeds kind limit".into());
        }
        let mut kinds = BTreeSet::new();
        for kind in &spec.kinds {
            identifier(kind)?;
            if !kinds.insert(kind.clone()) {
                return Err(format!("duplicate association kind: {kind}"));
            }
        }
        Ok(Self {
            spec: Arc::new(spec),
            kinds: Arc::new(kinds),
        })
    }

    /// The seeded kinds, `datasheets/kinds.toml`, for a world that has
    /// authored none of its own and for tests. A pack extends this by shipping
    /// its own spec, never by mutating a set already in hand.
    pub fn seeded() -> Self {
        static SEEDED: OnceLock<KindSet> = OnceLock::new();
        SEEDED
            .get_or_init(|| {
                let spec = seeded_spec(SEEDED_SHEET).unwrap_or_else(|why| panic!("{why}"));
                Self::new(spec).expect("the seeded kinds are a valid set")
            })
            .clone()
    }

    pub fn spec(&self) -> &KindSetSpec {
        &self.spec
    }

    pub fn contains(&self, kind: &str) -> bool {
        self.kinds.contains(kind)
    }
}
impl TryFrom<KindSetSpec> for KindSet {
    type Error = String;
    fn try_from(spec: KindSetSpec) -> Result<Self, Self::Error> {
        Self::new(spec)
    }
}
impl From<KindSet> for KindSetSpec {
    fn from(value: KindSet) -> Self {
        Arc::try_unwrap(value.spec).unwrap_or_else(|spec| (*spec).clone())
    }
}

/// The seeded kinds' datasheet, compiled in (wing design record, ruling 625).
const SEEDED_SHEET: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/datasheets/kinds.toml"
));

/// Livery's header over the seeded set, in a pack's own kind-set shape.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct KindsSheet {
    schema: u32,
    owner: String,
    consumer: String,
    status: String,
    sources: BTreeMap<String, Source>,
    seeded: KindSetSpec,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Source {
    doc: String,
    note: String,
}

fn seeded_spec(text: &str) -> Result<KindSetSpec, String> {
    let sheet: KindsSheet = toml::from_str(text).map_err(|why| format!("kinds.toml: {why}"))?;
    if sheet.schema != 1 {
        return Err(format!("kinds.toml: schema {} is not 1", sheet.schema));
    }
    let said = |text: &String| !text.trim().is_empty();
    let sourced = !sheet.sources.is_empty()
        && sheet
            .sources
            .values()
            .all(|source| said(&source.doc) && said(&source.note));
    if !(said(&sheet.owner) && said(&sheet.consumer) && said(&sheet.status) && sourced) {
        return Err("kinds.toml: owner, consumer, status and sources are required".into());
    }
    Ok(sheet.seeded)
}

#[cfg(test)]
mod sheet_tests {
    use super::*;

    #[test]
    fn the_sheet_holds_the_six_ruled_kinds_in_order() {
        let spec = KindSet::seeded().spec().clone();
        assert_eq!(spec.id, "impresa:seeded");
        assert_eq!(spec.version, SCHEMA_VERSION);
        assert_eq!(spec.limits, KindSetLimits::default());
        assert_eq!(
            spec.kinds,
            [
                "impresa:claim",
                "impresa:discover",
                "impresa:experience",
                "impresa:embody",
                "impresa:invoke",
                "impresa:defeat",
            ]
        );
    }

    #[test]
    fn a_changed_sheet_is_caught() {
        let changed = SEEDED_SHEET.replacen("\"impresa:invoke\"", "\"impresa:summon\"", 1);
        assert_ne!(changed, SEEDED_SHEET);
        let kinds = seeded_spec(&changed).expect("reads").kinds;
        assert_ne!(kinds, KindSet::seeded().spec().kinds);
        let misspelt = SEEDED_SHEET.replacen("kinds = [", "kind = [", 1);
        assert!(seeded_spec(&misspelt).is_err());
    }
}

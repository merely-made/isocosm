// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The kinds an association may have, as a world configures them. Closed, so
//! a record cannot invent its own relation by spelling one; extensible by a
//! pack, exactly as the glyph canon is, so a world is not stuck with six.

use crate::{DEFAULT_MAX_KINDS, SCHEMA_VERSION, identifier};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, sync::Arc};

/// The six the ruling names. Directional, subject to object: the subject
/// claimed, discovered, experienced, embodied, invoked or defeated the
/// object, never the reverse, so an inverse reading is a query and not
/// another kind.
pub const SEEDED_KINDS: [&str; 6] = [
    "impresa:claim",
    "impresa:discover",
    "impresa:experience",
    "impresa:embody",
    "impresa:invoke",
    "impresa:defeat",
];

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

    /// The six seeded kinds, for a world that has authored none of its own
    /// and for tests. A pack extends this by shipping its own spec, never by
    /// mutating a set already in hand.
    pub fn seeded() -> Self {
        Self::new(KindSetSpec {
            version: SCHEMA_VERSION,
            id: "impresa:seeded".into(),
            kinds: SEEDED_KINDS.iter().copied().map(String::from).collect(),
            limits: KindSetLimits::default(),
        })
        .expect("the seeded kinds are a valid set")
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

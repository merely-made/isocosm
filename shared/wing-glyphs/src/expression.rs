// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Which traits express a glyph, per canon revision. Data only: nothing here
//! reads a body, resolves a trait to a product type, draws a mark or grants
//! anything. A product maps a trait id onto its own bearer and does the
//! reading; this crate owns the table the claim is written in.

use crate::{
    Canon, DEFAULT_MAX_EXPRESSIONS, DEFAULT_MAX_JSON_BYTES, DEFAULT_MAX_TRAITS_PER_GLYPH, GlyphId,
    SCHEMA_VERSION, identifier,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

/// An opaque product trait identity, `namespace:local`. The kernel never
/// resolves it; a product maps it to its own bearer type. Byte-compatible
/// with a qualified process id by construction, so nothing is converted at
/// the boundary.
pub type TraitId = String;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExpressionLimits {
    pub entries: usize,
    pub traits_per_glyph: usize,
}
impl Default for ExpressionLimits {
    fn default() -> Self {
        Self {
            entries: DEFAULT_MAX_EXPRESSIONS,
            traits_per_glyph: DEFAULT_MAX_TRAITS_PER_GLYPH,
        }
    }
}

/// One base glyph and the traits that express it, in this canon revision.
///
/// **Disjunction.** Any one of these expressing the glyph is enough; a
/// conjunction is an authored claim needing its own vocabulary and is not
/// admitted here.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GlyphExpression {
    pub glyph: GlyphId,
    pub traits: Vec<TraitId>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExpressionSpec {
    pub version: u32,
    pub id: String,
    /// The canon revision this table was authored against. `covers` requires
    /// the two to agree, so a table cannot be read against a canon it does
    /// not describe.
    pub canon_revision: u64,
    pub entries: Vec<GlyphExpression>,
    #[serde(default)]
    pub limits: ExpressionLimits,
}

/// An admitted table: the immutable spec plus both indices, built exactly as
/// [`Canon`] is built. Many-to-many in both directions.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "ExpressionSpec", into = "ExpressionSpec")]
pub struct ExpressionTable {
    spec: Arc<ExpressionSpec>,
    by_glyph: Arc<BTreeMap<String, usize>>,
    by_trait: Arc<BTreeMap<String, Vec<GlyphId>>>,
}

impl ExpressionTable {
    pub fn new(spec: ExpressionSpec) -> Result<Self, String> {
        if spec.version != SCHEMA_VERSION {
            return Err("unsupported expression table version".into());
        }
        identifier(&spec.id)?;
        if spec.entries.len() > spec.limits.entries {
            return Err("expression table exceeds entry limit".into());
        }
        let mut glyphs = BTreeSet::new();
        let mut by_trait: BTreeMap<String, Vec<GlyphId>> = BTreeMap::new();
        for entry in &spec.entries {
            identifier(&entry.glyph)?;
            if !glyphs.insert(entry.glyph.as_str()) {
                return Err("duplicate glyph in expression table".into());
            }
            // Empty is a refusal here, never a silent answer at `traits_of`:
            // a glyph nothing expresses is an authoring mistake, not a glyph
            // that happens to be unembodiable.
            if entry.traits.is_empty() || entry.traits.len() > spec.limits.traits_per_glyph {
                return Err(format!(
                    "glyph {} requires 1..{} expressing traits",
                    entry.glyph, spec.limits.traits_per_glyph
                ));
            }
            let mut seen = BTreeSet::new();
            for id in &entry.traits {
                identifier(id)?;
                if !seen.insert(id.as_str()) {
                    return Err(format!("duplicate expressing trait for {}", entry.glyph));
                }
                by_trait
                    .entry(id.clone())
                    .or_default()
                    .push(entry.glyph.clone());
            }
        }
        let by_glyph = Arc::new(
            spec.entries
                .iter()
                .enumerate()
                .map(|(i, e)| (e.glyph.clone(), i))
                .collect(),
        );
        Ok(Self {
            spec: Arc::new(spec),
            by_glyph,
            by_trait: Arc::new(by_trait),
        })
    }

    pub fn spec(&self) -> &ExpressionSpec {
        &self.spec
    }

    /// The traits that express this glyph. Empty only for a glyph the table
    /// does not carry; an admitted entry always has at least one.
    pub fn traits_of(&self, glyph: &str) -> &[TraitId] {
        self.by_glyph
            .get(glyph)
            .map(|&i| self.spec.entries[i].traits.as_slice())
            .unwrap_or_default()
    }

    /// The inverse: every glyph this trait expresses, in table order. One
    /// trait expressing several glyphs is expected, not an error.
    pub fn glyphs_of(&self, trait_id: &str) -> &[GlyphId] {
        self.by_trait
            .get(trait_id)
            .map(Vec::as_slice)
            .unwrap_or_default()
    }

    /// Every base in `canon` has an expressing trait, and the revisions
    /// agree. Pure validation, mirroring `EffectPack::covers`; it expresses
    /// nothing and changes neither side.
    pub fn covers(&self, canon: &Canon) -> Result<(), String> {
        if canon.spec().revision != self.spec.canon_revision {
            return Err(format!(
                "expression table is authored against canon revision {}, not {}",
                self.spec.canon_revision,
                canon.spec().revision
            ));
        }
        for glyph in &canon.spec().glyphs {
            if self.traits_of(&glyph.id).is_empty() {
                return Err(format!("no trait expresses base {}", glyph.id));
            }
        }
        Ok(())
    }

    /// A new revision in which each glyph's trait set has moved to another
    /// glyph.
    ///
    /// **Saved, never rerun.** A world permutes the authored table once and
    /// pins the result into its own definition, exactly as the canon
    /// correspondence is pinned; rerunning it on an inhabited world would
    /// change what its bodies already embody. Seeded SplitMix64 plus
    /// rejection-sampled Fisher-Yates, reproducible and not secure — the same
    /// draw [`Canon::shuffled`] makes, so the two permutations are comparable.
    pub fn shuffled(&self, seed: u64, canon_revision: u64) -> Result<Self, String> {
        if canon_revision <= self.spec.canon_revision {
            return Err("shuffle requires a newer canon revision".into());
        }
        let mut spec = (*self.spec).clone();
        spec.canon_revision = canon_revision;
        let mut random = seed;
        for i in (1..spec.entries.len()).rev() {
            let bound = (i + 1) as u64;
            let threshold = bound.wrapping_neg() % bound;
            let index = loop {
                random = random.wrapping_add(0x9e3779b97f4a7c15);
                let mut value = random;
                value = (value ^ (value >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
                value = (value ^ (value >> 27)).wrapping_mul(0x94d049bb133111eb);
                value ^= value >> 31;
                if value >= threshold {
                    break (value % bound) as usize;
                }
            };
            let traits = spec.entries[i].traits.clone();
            spec.entries[i].traits = spec.entries[index].traits.clone();
            spec.entries[index].traits = traits;
        }
        Self::new(spec)
    }

    pub fn to_json(&self) -> Result<String, String> {
        self.to_json_with_limit(DEFAULT_MAX_JSON_BYTES)
    }
    pub fn to_json_with_limit(&self, limit: usize) -> Result<String, String> {
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        if json.len() > limit {
            return Err("expression table JSON exceeds limit".into());
        }
        Ok(json)
    }
    pub fn from_json(json: &str) -> Result<Self, String> {
        Self::from_json_with_limit(json, DEFAULT_MAX_JSON_BYTES)
    }
    pub fn from_json_with_limit(json: &str, limit: usize) -> Result<Self, String> {
        if json.len() > limit {
            return Err("expression table JSON exceeds limit".into());
        }
        serde_json::from_str(json).map_err(|e| e.to_string())
    }
}
impl TryFrom<ExpressionSpec> for ExpressionTable {
    type Error = String;
    fn try_from(spec: ExpressionSpec) -> Result<Self, Self::Error> {
        Self::new(spec)
    }
}
impl From<ExpressionTable> for ExpressionSpec {
    fn from(value: ExpressionTable) -> Self {
        Arc::try_unwrap(value.spec).unwrap_or_else(|spec| (*spec).clone())
    }
}

#[cfg(test)]
#[path = "expression_tests.rs"]
mod tests;

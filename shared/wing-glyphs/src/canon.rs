// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use crate::{
    DEFAULT_MAX_GLYPHS, DEFAULT_MAX_JSON_BYTES, DEFAULT_MAX_VARIANTS, SCHEMA_VERSION, bounded_text,
    identifier,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

pub type GlyphId = String;
pub type EffectId = String;
pub type ModifierId = String;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CanonLimits {
    pub glyphs: usize,
    pub variants: usize,
}
impl Default for CanonLimits {
    fn default() -> Self {
        Self {
            glyphs: DEFAULT_MAX_GLYPHS,
            variants: DEFAULT_MAX_VARIANTS,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GlyphDefinition {
    pub id: GlyphId,
    /// Opaque Unicode; neither normalized nor used as identity.
    pub display: String,
    pub effect: EffectId,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VariantDefinition {
    pub id: GlyphId,
    pub display: String,
    pub base: GlyphId,
    /// Opaque modifier references. This kernel does not execute modifiers.
    pub modifiers: Vec<ModifierId>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CanonSpec {
    pub version: u32,
    pub id: String,
    pub revision: u64,
    pub glyphs: Vec<GlyphDefinition>,
    pub variants: Vec<VariantDefinition>,
    #[serde(default)]
    pub limits: CanonLimits,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "CanonSpec", into = "CanonSpec")]
pub struct Canon {
    spec: Arc<CanonSpec>,
    base_indices: Arc<BTreeMap<String, usize>>,
    variant_indices: Arc<BTreeMap<String, usize>>,
}

impl Canon {
    pub fn new(spec: CanonSpec) -> Result<Self, String> {
        if spec.version != SCHEMA_VERSION {
            return Err("unsupported canon version".into());
        }
        identifier(&spec.id)?;
        if spec.glyphs.len() > spec.limits.glyphs || spec.variants.len() > spec.limits.variants {
            return Err("canon exceeds glyph or variant limit".into());
        }
        let mut ids = BTreeSet::new();
        let mut effects = BTreeSet::new();
        for glyph in &spec.glyphs {
            identifier(&glyph.id)?;
            identifier(&glyph.effect)?;
            bounded_text(&glyph.display, "glyph display", 4096)?;
            if !ids.insert(glyph.id.as_str()) {
                return Err("duplicate glyph ID".into());
            }
            if !effects.insert(glyph.effect.as_str()) {
                return Err("duplicate effect ID".into());
            }
        }
        let bases = ids.clone();
        for variant in &spec.variants {
            identifier(&variant.id)?;
            bounded_text(&variant.display, "variant display", 4096)?;
            if !ids.insert(variant.id.as_str()) {
                return Err("duplicate glyph or variant ID".into());
            }
            if !bases.contains(variant.base.as_str()) {
                return Err("variant references unknown base".into());
            }
            if variant.modifiers.is_empty() || variant.modifiers.len() > 32 {
                return Err("variant requires 1..32 explicit modifiers".into());
            }
            let mut modifiers = BTreeSet::new();
            for modifier in &variant.modifiers {
                identifier(modifier)?;
                if !modifiers.insert(modifier) {
                    return Err("duplicate variant modifier ID".into());
                }
            }
        }
        let base_indices = Arc::new(
            spec.glyphs
                .iter()
                .enumerate()
                .map(|(i, g)| (g.id.clone(), i))
                .collect(),
        );
        let variant_indices = Arc::new(
            spec.variants
                .iter()
                .enumerate()
                .map(|(i, v)| (v.id.clone(), i))
                .collect(),
        );
        Ok(Self {
            spec: Arc::new(spec),
            base_indices,
            variant_indices,
        })
    }

    pub fn spec(&self) -> &CanonSpec {
        &self.spec
    }
    pub fn contains(&self, id: &str) -> bool {
        self.base_id(id).is_some()
    }
    pub fn is_base(&self, id: &str) -> bool {
        self.base_indices.contains_key(id)
    }
    pub fn base_id(&self, id: &str) -> Option<&str> {
        self.base_indices
            .get(id)
            .map(|&i| self.spec.glyphs[i].id.as_str())
            .or_else(|| {
                self.variant_indices
                    .get(id)
                    .map(|&i| self.spec.variants[i].base.as_str())
            })
    }
    pub fn effect(&self, id: &str) -> Option<&str> {
        let base = self.base_id(id)?;
        self.base_indices
            .get(base)
            .map(|&i| self.spec.glyphs[i].effect.as_str())
    }

    /// A new revision; source identity and mapping remain immutable. Seeded
    /// SplitMix64 + rejection-sampled Fisher-Yates is reproducible, not secure.
    pub fn shuffled(&self, seed: u64, revision: u64) -> Result<Self, String> {
        if revision <= self.spec.revision {
            return Err("shuffle requires a newer canon revision".into());
        }
        let mut spec = (*self.spec).clone();
        spec.revision = revision;
        let mut random = seed;
        for i in (1..spec.glyphs.len()).rev() {
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
            let effect = spec.glyphs[i].effect.clone();
            spec.glyphs[i].effect = spec.glyphs[index].effect.clone();
            spec.glyphs[index].effect = effect;
        }
        Self::new(spec)
    }

    pub fn to_json(&self) -> Result<String, String> {
        self.to_json_with_limit(DEFAULT_MAX_JSON_BYTES)
    }
    pub fn to_json_with_limit(&self, limit: usize) -> Result<String, String> {
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        if json.len() > limit {
            return Err("canon JSON exceeds limit".into());
        }
        Ok(json)
    }
    pub fn from_json(json: &str) -> Result<Self, String> {
        Self::from_json_with_limit(json, DEFAULT_MAX_JSON_BYTES)
    }
    pub fn from_json_with_limit(json: &str, limit: usize) -> Result<Self, String> {
        if json.len() > limit {
            return Err("canon JSON exceeds limit".into());
        }
        serde_json::from_str(json).map_err(|e| e.to_string())
    }
}
impl TryFrom<CanonSpec> for Canon {
    type Error = String;
    fn try_from(spec: CanonSpec) -> Result<Self, Self::Error> {
        Self::new(spec)
    }
}
impl From<Canon> for CanonSpec {
    fn from(value: Canon) -> Self {
        Arc::try_unwrap(value.spec).unwrap_or_else(|spec| (*spec).clone())
    }
}

#[cfg(test)]
#[path = "canon_tests.rs"]
mod tests;

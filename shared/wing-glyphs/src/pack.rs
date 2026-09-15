// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! What an effect reference *declares*. Data only: nothing here runs an
//! effect, draws a mark, grants a glyph or names a product noun. Products own
//! execution; this crate owns the vocabulary the declaration is written in.

use crate::{
    Canon, DEFAULT_MAX_DECLARATIONS, DEFAULT_MAX_JSON_BYTES, EffectId, SCHEMA_VERSION,
    bounded_text, identifier,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

/// What form an effect asks a product to give it. The kernel draws nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BehaviourKind {
    Inscribe,
    Emit,
    Trail,
    Enclose,
}

/// What class of thing an effect acts on. Products map these to their own
/// nouns; the kernel knows no terrain, body or organism.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReceiverClass {
    Terrain,
    Body,
    Bearer,
    None,
}

/// A named quantity a product must be able to meter. Kernel-opaque: this
/// crate never reads, sums or spends one.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CostUnit {
    pub id: String,
    pub label: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CostShape {
    Free,
    PerUse { unit: CostUnit },
    WhileSustained { unit: CostUnit },
}

impl CostShape {
    pub fn unit(&self) -> Option<&CostUnit> {
        match self {
            Self::Free => None,
            Self::PerUse { unit } | Self::WhileSustained { unit } => Some(unit),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EffectDeclaration {
    pub effect: EffectId,
    /// Opaque display text, as on `GlyphDefinition`. Not a stroke shape.
    pub display: String,
    pub behaviour: BehaviourKind,
    pub receiver: ReceiverClass,
    pub cost: CostShape,
    /// Named holes a product fills: "{actor} inscribes {amount} {unit}".
    /// The kernel neither formats nor executes it.
    pub explanation: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PackLimits {
    pub declarations: usize,
}
impl Default for PackLimits {
    fn default() -> Self {
        Self {
            declarations: DEFAULT_MAX_DECLARATIONS,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EffectPackSpec {
    pub version: u32,
    pub id: String,
    pub revision: u64,
    pub declarations: Vec<EffectDeclaration>,
    #[serde(default)]
    pub limits: PackLimits,
}

/// An admitted pack: the immutable spec plus an effect-id index, built exactly
/// as `Canon` is built.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "EffectPackSpec", into = "EffectPackSpec")]
pub struct EffectPack {
    spec: Arc<EffectPackSpec>,
    indices: Arc<BTreeMap<String, usize>>,
}

impl EffectPack {
    pub fn new(spec: EffectPackSpec) -> Result<Self, String> {
        if spec.version != SCHEMA_VERSION {
            return Err("unsupported effect pack version".into());
        }
        identifier(&spec.id)?;
        if spec.declarations.len() > spec.limits.declarations {
            return Err("effect pack exceeds declaration limit".into());
        }
        let mut effects = BTreeSet::new();
        for declaration in &spec.declarations {
            identifier(&declaration.effect)?;
            bounded_text(&declaration.display, "declaration display", 4096)?;
            bounded_text(&declaration.explanation, "declaration explanation", 4096)?;
            if let Some(unit) = declaration.cost.unit() {
                identifier(&unit.id)?;
                bounded_text(&unit.label, "cost unit label", 256)?;
            }
            if !effects.insert(declaration.effect.as_str()) {
                return Err("duplicate effect declaration".into());
            }
        }
        let indices = Arc::new(
            spec.declarations
                .iter()
                .enumerate()
                .map(|(i, d)| (d.effect.clone(), i))
                .collect(),
        );
        Ok(Self {
            spec: Arc::new(spec),
            indices,
        })
    }

    pub fn spec(&self) -> &EffectPackSpec {
        &self.spec
    }
    pub fn declaration(&self, effect: &str) -> Option<&EffectDeclaration> {
        self.indices
            .get(effect)
            .map(|&i| &self.spec.declarations[i])
    }

    /// Every base glyph's effect in `canon` has a declaration here. Pure
    /// validation: it executes nothing and changes neither side.
    pub fn covers(&self, canon: &Canon) -> Result<(), String> {
        for glyph in &canon.spec().glyphs {
            if self.declaration(&glyph.effect).is_none() {
                return Err(format!(
                    "effect pack declares no effect for base {}: {}",
                    glyph.id, glyph.effect
                ));
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
            return Err("effect pack JSON exceeds limit".into());
        }
        Ok(json)
    }
    pub fn from_json(json: &str) -> Result<Self, String> {
        Self::from_json_with_limit(json, DEFAULT_MAX_JSON_BYTES)
    }
    pub fn from_json_with_limit(json: &str, limit: usize) -> Result<Self, String> {
        if json.len() > limit {
            return Err("effect pack JSON exceeds limit".into());
        }
        serde_json::from_str(json).map_err(|e| e.to_string())
    }
}
impl TryFrom<EffectPackSpec> for EffectPack {
    type Error = String;
    fn try_from(spec: EffectPackSpec) -> Result<Self, Self::Error> {
        Self::new(spec)
    }
}
impl From<EffectPack> for EffectPackSpec {
    fn from(value: EffectPack) -> Self {
        Arc::try_unwrap(value.spec).unwrap_or_else(|spec| (*spec).clone())
    }
}

#[cfg(test)]
#[path = "pack_tests.rs"]
mod tests;

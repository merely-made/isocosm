//! Source-qualified effect coverage, kept beside the system plugins.
//!
//! This is an inspectable catalogue, not a universal spell evaluator. Native
//! candidates come from the sim's own effect grammar. References to an edition
//! do not execute it or translate its costs, choices, timing or exceptions.

use isocosm::rules::{Effect, EffectKind, Process, Query, Rules};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

mod validation;

pub const SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Family {
    Resources,
    Defence,
    Anatomy,
    Movement,
    Perception,
    Agency,
    Environment,
    SpatialAccess,
    LifeIdentity,
    Persistence,
}

impl Family {
    pub const ALL: [Self; 10] = [
        Self::Resources,
        Self::Defence,
        Self::Anatomy,
        Self::Movement,
        Self::Perception,
        Self::Agency,
        Self::Environment,
        Self::SpatialAccess,
        Self::LifeIdentity,
        Self::Persistence,
    ];
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
    pub id: String,
    pub edition: String,
    pub corpus: String,
    pub url: String,
}

/// Deliberately no "implemented spell" variant: a reference cannot establish
/// that the edition's full procedure is executable.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Correspondence {
    Related,
    NeedsRuling,
    Extension,
    Unmapped,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Mapping {
    pub relation: Correspondence,
    #[serde(default)]
    pub references: Vec<String>,
    #[serde(default)]
    pub differences: Vec<String>,
    /// Specific executable procedures, distinct from a related reference or
    /// a primitive candidate. Their accepted definitions are saved by casts.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub procedures: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeBinding {
    pub operation: EffectKind,
    /// Allocation candidates must bind and allocate the named function. This
    /// prevents an unrelated gland allocation being advertised as a sense.
    pub function: Option<String>,
}

impl NativeBinding {
    pub fn matches(&self, process: &Process) -> bool {
        let declares_function = self.function.as_ref().is_none_or(|function| {
            process
                .requires
                .iter()
                .any(|q| matches!(q, Query::Expresses { function: f } if f == function))
        });
        declares_function
            && process.effect_declarations().into_iter().any(|effect| {
                effect.kind() == self.operation
                    && self.function.as_ref().is_none_or(
                        |function| matches!(effect, Effect::Allocate { to, .. } if to == function),
                    )
            })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    pub id: String,
    pub family: Family,
    pub summary: String,
    /// Scope limits are mandatory even when a native primitive exists.
    pub limitation: String,
    #[serde(default)]
    pub native: Vec<NativeBinding>,
    pub mappings: BTreeMap<String, Mapping>,
}

impl Entry {
    /// Candidate declarations only. Guards, stock, anatomy, target choice and
    /// every other admission rule remain the native executor's responsibility.
    pub fn native_candidates(&self, rules: &Rules) -> Vec<String> {
        rules
            .processes
            .iter()
            .filter_map(|(id, process)| {
                self.native
                    .iter()
                    .any(|binding| binding.matches(process))
                    .then(|| id.clone())
            })
            .collect()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Catalogue {
    pub version: u32,
    pub scope: String,
    pub families: Vec<Family>,
    pub sources: Vec<Source>,
    pub entries: Vec<Entry>,
}

impl Catalogue {
    pub fn from_toml(text: &str) -> Result<Self, String> {
        let mut catalogue: Self =
            toml::from_str(text).map_err(|e| format!("effect catalogue: {e}"))?;
        catalogue.validate()?;
        catalogue.families.sort();
        catalogue.sources.sort_by(|a, b| a.id.cmp(&b.id));
        catalogue.entries.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(catalogue)
    }

    pub fn revision(&self) -> String {
        isocosm::digest(self)
    }

    pub fn entry(&self, id: &str) -> Option<&Entry> {
        self.entries.iter().find(|entry| entry.id == id)
    }

    pub fn family_counts(&self) -> BTreeMap<Family, usize> {
        let mut counts = BTreeMap::new();
        for entry in &self.entries {
            *counts.entry(entry.family).or_default() += 1;
        }
        counts
    }
}

pub fn builtin() -> Result<Catalogue, String> {
    let catalogue = Catalogue::from_toml(include_str!("../../data/effects.toml"))?;
    catalogue.validate_procedures(&crate::active_effects::Definitions::builtin()?)?;
    Ok(catalogue)
}

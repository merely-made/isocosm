// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use crate::{
    Result, repertoire,
    rules::{Grammar, Recipe, Rule, Template},
    schema::Key,
    validation,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// An authored body map for one generated lineage. The grammar's seed gives
/// the attachments and kind positions. Named regions bind those positions to
/// existing native kinds or to parts admitted by the world's repertoire.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BodyMap {
    pub grammar: Grammar,
    /// Maximum seeded segment variation either way, unless a run sets its own.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub variance: u8,
    /// Odds that a run omits an eligible segment's appendages.
    /// Feeding, sensing and fixing appendages always grow.
    #[serde(default = "never", skip_serializing_if = "is_never")]
    pub absence: [u32; 2],
    /// Region references in the grammar use the `region:` namespace.
    /// A `part:` value expresses that catalogue part's mapped traits.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub regions: BTreeMap<Key, Key>,
}

fn is_zero(value: &u8) -> bool {
    *value == 0
}

fn never() -> [u32; 2] {
    [0, 1]
}

fn is_never(odds: &[u32; 2]) -> bool {
    *odds == never()
}

impl std::str::FromStr for BodyMap {
    type Err = toml::de::Error;
    fn from_str(text: &str) -> std::result::Result<Self, Self::Err> {
        toml::from_str(text)
    }
}

impl BodyMap {
    /// Resolve the named anatomy in the seed and rules. The caller installs
    /// the native root and feeding organs before expanding the grammar.
    pub(super) fn resolve(
        &self,
        kinds: &BTreeMap<Key, Template>,
        lowered: Option<&repertoire::Lowered>,
        kingdom: &str,
    ) -> Result<Grammar> {
        if self.grammar.seed.is_empty() {
            return Err("body map has no root".into());
        }
        let [absent, of] = self.absence;
        if of == 0 || absent > of {
            return Err("body map has invalid absence odds".into());
        }
        if let Some(why) = validation::tagmata(&self.grammar.seed) {
            return Err(format!("body map {why}"));
        }
        let mut used = BTreeSet::new();
        let mut resolve = |reference: &mut Key| -> Result<()> {
            let source = if reference.starts_with("region:") {
                let source = self
                    .regions
                    .get(reference)
                    .ok_or_else(|| format!("body map has no binding for {reference}"))?;
                used.insert(reference.clone());
                source
            } else {
                &*reference
            };
            let kind = if source.starts_with("part:") {
                repertoire::kind_key(source)
            } else {
                source.clone()
            };
            if !kinds.contains_key(&kind) {
                return Err(format!(
                    "body map kind or catalogue part {source} is unavailable"
                ));
            }
            if let Some(catalogue) = lowered.filter(|r| r.kinds.contains_key(&kind))
                && !catalogue.reaches(kingdom, &kind)
            {
                return Err(format!(
                    "body map's {source} is outside the repertoire for {kingdom}"
                ));
            }
            *reference = kind;
            Ok(())
        };
        let mut grammar = self.grammar.clone();
        for run in &mut grammar.seed {
            resolve(&mut run.segment)?;
            if let Some(kind) = &mut run.bears {
                resolve(kind)?;
            }
            for link in &mut run.chain {
                resolve(&mut link.kind)?;
            }
        }
        for rule in &mut grammar.rules {
            if let Rule::Differentiate { kind, .. } = rule {
                resolve(kind)?;
            }
        }
        if let Some(unused) = self.regions.keys().find(|key| !used.contains(*key)) {
            return Err(format!("body map has an unused region {unused}"));
        }
        Ok(grammar)
    }

    pub(super) fn recipe(&self, grammar: &mut Grammar) -> Recipe {
        Recipe {
            tagmata: std::mem::take(&mut grammar.seed),
            variance: self.variance,
            absence: self.absence,
            riff: [0, 1],
            vary: [0, 1],
        }
    }
}

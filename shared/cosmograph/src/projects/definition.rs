// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};
use crate::{Result, rules::Rules, schema::{Key, Ledger}, social::Work};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Step { pub progress: u64, pub spend: Ledger }

/// Every admitted work step and its stocks are authored, never interpolated.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub recipe: Key,
    pub work: Work,
    pub total: u64,
    pub steps: Vec<Step>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_custody: Option<Key>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Catalogue { pub limit: u64, pub definitions: BTreeMap<Key, Definition> }

impl Catalogue {
    pub fn validate(&self, rules: &Rules) -> Result<()> {
        if self.limit == 0 || self.definitions.is_empty() {
            return Err("project catalogue needs an explicit limit and definitions".into());
        }
        for (key, definition) in &self.definitions {
            crate::validation::key(key)?;
            definition.validate(rules)?;
        }
        Ok(())
    }
}

impl Definition {
    pub fn validate(&self, rules: &Rules) -> Result<()> {
        crate::validation::key(&self.recipe)?;
        crate::validation::key(&self.work.craft)?;
        if let Some(key) = &self.output_custody {
            crate::validation::key(key)?;
            if !rules.relations.contains(key) { return Err("project output custody relation is undeclared".into()); }
        }
        let recipe = rules.material_recipes.get(&self.recipe).ok_or("project material recipe is absent")?;
        if self.total == 0 || self.steps.is_empty()
            || !rules.social.as_ref().is_some_and(|v| v.crafts.contains(&self.work.craft))
        { return Err("project needs authored progress and a declared native craft".into()); }
        let (mut progress, mut stocks) = (0u64, Ledger::new());
        for step in &self.steps {
            if step.progress == 0 { return Err("project work step makes no progress".into()); }
            progress = progress.checked_add(step.progress).ok_or("project progress overflow")?;
            for (key, amount) in &step.spend {
                crate::validation::key(key)?;
                if *amount == 0 { return Err("project work cost is zero".into()); }
                crate::meaning::credit(&mut stocks, key, *amount)?;
            }
        }
        if progress != self.total || stocks != recipe.stocks {
            return Err("project steps differ from its declared progress or recipe stocks".into());
        }
        recipe.validate(rules)
    }
}

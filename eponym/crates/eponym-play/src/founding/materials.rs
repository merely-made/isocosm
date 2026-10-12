// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Explicit new-world stock identities and named native item recipes (835/836).
//! Declarations do not issue site balances. Only selected initial arrivals
//! issue their exact recipe stocks through the native founding door.

use crate::ItemKind;
use isocosm::{material::Recipe, rules::{AccountKind, Process}, schema::Key, simulation::Genesis};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const GENERATOR_VERSION: u32 = 6;
pub const PROFILE_VERSION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MaterialStock {
    pub material: Key,
    pub account: Key,
    pub density_mg_per_cell: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ItemRecipe {
    pub kind: ItemKind,
    pub native: Recipe,
    /// A declared native consumption process, never inferred from ingredients.
    pub consume: Option<Key>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InitialMaterialItem {
    pub recipe: Key,
    /// Authored coordinates in the played site's Ground frame.
    pub at: [i32; 3],
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MaterialProfile {
    pub version: u32,
    pub key: Key,
    pub materials: Vec<MaterialStock>,
    pub recipes: BTreeMap<Key, ItemRecipe>,
    pub processes: BTreeMap<Key, Process>,
    /// Authored construction definitions join the same native recipe keys.
    pub projects: Option<isocosm::projects::Catalogue>,
    pub items: Vec<InitialMaterialItem>,
}

impl MaterialProfile {
    pub fn genesis(&self, seed: u64, side: u32, extent: u64) -> isocosm::Result<Genesis> {
        if self.version != PROFILE_VERSION || self.key.trim().is_empty() {
            return Err("unsupported or unnamed Eponym material profile".into());
        }
        // Retain the ordinary body/social pack; only this explicitly marked
        // new founding replaces the pooled terrain material bindings.
        let mut g = super::genesis(seed, side, extent)?;
        let mut materials = BTreeSet::new();
        let mut accounts = BTreeSet::new();
        for stock in &self.materials {
            if stock.account.trim().is_empty() || stock.density_mg_per_cell == 0
                || !materials.insert(stock.material.clone()) || !accounts.insert(stock.account.clone())
            {
                return Err("material stock declarations must be positive, named and distinct".into());
            }
            let material = g.world.materials.iter_mut().find(|m| m.key == stock.material)
                .ok_or("stock declaration names an absent material")?;
            let kind = AccountKind::Matter { lineage:material.lineage.clone(), reserve:false, provision:false };
            if g.rules.accounts.get(&stock.account).is_some_and(|held| held != &kind) {
                return Err("material stock would replace another native account kind".into());
            }
            g.rules.accounts.insert(stock.account.clone(), kind);
            material.account = Some(stock.account.clone());
            material.density = Some(stock.density_mg_per_cell);
        }
        if materials.len() != g.world.materials.len() {
            return Err("every declared material requires its own authored stock and density".into());
        }
        for (key, process) in &self.processes {
            if key != &process.id || g.rules.processes.contains_key(key) {
                return Err("material profile process is misnamed or replaces a native process".into());
            }
            g.rules.processes.insert(key.clone(), process.clone());
        }
        for (key, recipe) in &self.recipes {
            if key.trim().is_empty() || recipe.native.lineage != recipe.kind.lineage()
                || g.rules.material_recipes.contains_key(key)
            {
                return Err("item recipe must have a new name and its declared item-kind lineage".into());
            }
            recipe.native.validate(&g.rules)?;
            recipe.native.validate_lineages(&g.lineages)?;
            // Selected initial arrivals later validate their fixed scale at
            // the actual played site. A project-only recipe may name another
            // scale, which native staged output can retain directly.
            if recipe.consume.as_ref().is_some_and(|p| !g.rules.processes.contains_key(p)) {
                return Err("item consumption names an absent native process".into());
            }
            g.rules.material_recipes.insert(key.clone(), recipe.native.clone());
        }
        for item in &self.items {
            if !self.recipes.contains_key(&item.recipe) {
                return Err("initial item names an absent native recipe".into());
            }
        }
        g.rules.projects = self.projects.clone();
        g.validate()?;
        Ok(g)
    }
}

// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Complete fresh-world declarations, never runtime item issuance or weight pricing.

use isocosm::{material::Recipe,projects::{Catalogue,StockRules},rules::AccountKind,
    schema::{Key,Lineage},simulation::Genesis,social::Vocabulary};
use serde::{Deserialize,Serialize};
use std::collections::{BTreeMap,BTreeSet};

#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MaterialProfile {
    pub accounts:BTreeMap<Key,AccountKind>,
    pub lineages:BTreeMap<Key,Lineage>,
    pub recipes:BTreeMap<Key,Recipe>,
    pub projects:Option<Catalogue>,
    pub social:Option<Vocabulary>,
    pub owned_stock:Option<StockRules>,
    pub relations:BTreeSet<Key>,
}

impl MaterialProfile {
    pub(super) fn install(&self,genesis:&mut Genesis)->Result<(),String> {
        if self.accounts.keys().any(|key|genesis.rules.accounts.contains_key(key))
            || self.lineages.keys().any(|key|genesis.lineages.contains_key(key))
            || self.recipes.keys().any(|key|genesis.rules.material_recipes.contains_key(key))
            || self.projects.is_some() && genesis.rules.projects.is_some()
            || self.social.is_some() && genesis.rules.social.is_some()
            || self.owned_stock.is_some() && genesis.rules.owned_stock.is_some() {
            return Err("material profile replaces existing generated declarations".into());
        }
        genesis.rules.accounts.extend(self.accounts.clone());
        genesis.lineages.extend(self.lineages.clone());
        genesis.rules.material_recipes.extend(self.recipes.clone());
        if let Some(catalogue)=&self.projects {genesis.rules.projects=Some(catalogue.clone());}
        if let Some(vocabulary)=&self.social {genesis.rules.social=Some(vocabulary.clone());}
        if let Some(owned)=&self.owned_stock {genesis.rules.owned_stock=Some(owned.clone());}
        genesis.rules.relations.extend(self.relations.clone());
        for recipe in self.recipes.values() {
            recipe.validate(&genesis.rules)?;
            recipe.validate_lineages(&genesis.lineages)?;
        }
        Ok(())
    }
}

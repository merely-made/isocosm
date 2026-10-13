// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A separate authored founding grammar; earlier material profiles stay exact.

use super::materials::MaterialProfile;
use isocosm::{Result, material::Recipe, rules::AccountKind,
    schema::{Id, Key, Ledger, Lineage, Method}, simulation::Genesis,
    projects::StockRules, spread::{CapacityModifier, CapacityRules, Identity, PlaceLedger}};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const GENERATOR_VERSION: u32 = 8;
pub const PROFILE_VERSION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Founder {
    pub recipe: Recipe,
    pub method: Method,
    pub development_seed: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpreadFounding {
    pub traits: BTreeSet<Key>,
    pub lineages: BTreeMap<Key, Lineage>,
    pub accounts: BTreeMap<Key, AccountKind>,
    pub founders: BTreeMap<Key, Founder>,
    pub identities: BTreeSet<Identity>,
    pub sites: BTreeMap<Id, Ledger>,
    pub hosts: BTreeMap<Id, Ledger>,
    pub places: Vec<PlaceLedger>,
    pub capacity: CapacityRules,
    pub modifiers: Vec<CapacityModifier>,
    pub stock: StockRules,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpreadMaterialProfile {
    pub version: u32,
    pub key: Key,
    pub materials: MaterialProfile,
    pub native: SpreadFounding,
}

fn stocks(genesis: &Genesis, ledger: &Ledger) -> Result<()> {
    for (key, amount) in ledger {
        if *amount == 0 || !(matches!(genesis.rules.accounts.get(key), Some(AccountKind::Matter { .. }))
            || genesis.spreads.owns(key)) {
            return Err("spread founding stocks need exact positive native matter or registered identities".into());
        }
    }
    Ok(())
}

fn seed_stocks(accounts: &mut Ledger, ledger: &Ledger) -> Result<()> {
    for (key, amount) in ledger {
        let held = accounts.entry(key.clone()).or_default();
        *held = held.checked_add(*amount).ok_or("declared founding quantity exceeds native integers")?;
    }
    Ok(())
}

impl SpreadMaterialProfile {
    pub fn genesis(&self, seed: u64, side: u32, extent: u64) -> Result<Genesis> {
        if self.version != PROFILE_VERSION || self.key.trim().is_empty() {
            return Err("unsupported or unnamed Eponym spread material profile".into());
        }
        let mut g = self.materials.genesis(seed, side, extent)?;
        g.rules.traits.extend(self.native.traits.iter().cloned());
        for (key, lineage) in &self.native.lineages {
            if g.lineages.contains_key(key) { return Err("spread profile cannot replace a native lineage".into()); }
            g.lineages.insert(key.clone(), lineage.clone());
        }
        for (key, kind) in &self.native.accounts {
            if !matches!(kind, AccountKind::Matter { .. }) || g.rules.accounts.contains_key(key) {
                return Err("spread source declaration must be new ordinary native matter".into());
            }
            g.rules.accounts.insert(key.clone(), kind.clone());
        }
        for (key, founder) in &self.native.founders {
            founder.recipe.validate(&g.rules)?;
            founder.recipe.validate_lineages(&g.lineages)?;
            if founder.recipe.scale != "scale:meso" || g.rules.material_recipes.contains_key(key) {
                return Err("spread founder recipe is duplicated or outside native arrival's scale".into());
            }
            if founder.development_seed.is_some() && g.lineages[&founder.recipe.lineage].development.is_none() {
                return Err("bodily spread founder has no authored native development".into());
            }
            g.rules.material_recipes.insert(key.clone(), founder.recipe.clone());
        }
        if !g.spreads.is_empty() { return Err("spread profile cannot replace existing native spread storage".into()); }
        g.spreads.identities = self.native.identities.clone();
        g.spreads.places = self.native.places.clone();
        g.spreads.modifiers = self.native.modifiers.clone();
        g.rules.spread_capacity = Some(self.native.capacity.clone());
        g.rules.owned_stock = Some(self.native.stock.clone());
        for place in &g.spreads.places { stocks(&g, &place.accounts)?; }
        for (site, ledger) in &self.native.sites {
            stocks(&g, ledger)?;
            let accounts = &mut g.sites.get_mut(site).ok_or("spread founding stock names no actual site")?.accounts;
            seed_stocks(accounts, ledger)?;
        }
        for (host, ledger) in &self.native.hosts {
            stocks(&g, ledger)?;
            let accounts = &mut g.population.lift(*host)?.accounts;
            seed_stocks(accounts, ledger)?;
        }
        g.validate()?;
        Ok(g)
    }
}

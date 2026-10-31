// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A world's explicit global predicate, independent of regional heuristics.

use crate::{Result, rules::{AccountKind, Expr, Reading, Rules}, schema::{Id, Key, Method, Tick},
    simulation::{Genesis, Simulation}};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Lineages { All, Only(BTreeSet<Key>) }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PopulationFilter {
    pub lineages: Lineages,
    pub methods: BTreeSet<Method>,
    pub alive: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorldReading {
    Tick,
    Population { selector: PopulationFilter },
    /// Each selected body's native books, including its parts, counted once.
    Matter { selector: PopulationFilter, accounts: BTreeSet<Key> },
    SiteAccount { site: Id, account: Key },
    SiteCondition { site: Id, condition: Key },
    /// Registered territory/surface stocks retain native place/host identity.
    Spread { at: crate::spread::Substrate, identity: crate::spread::Identity },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Predicate {
    pub inputs: BTreeMap<Key, WorldReading>,
    /// Existing Query::Computed semantics: zero is false, nonzero is true.
    pub condition: Expr,
}

/// Trusted native truth; player views expose the lifecycle phase, not these inputs.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CollapseReading {
    pub tick: Tick,
    pub inputs: BTreeMap<Key, i64>,
    pub result: i64,
    pub collapsed: bool,
}

impl PopulationFilter {
    fn validate(&self) -> Result<()> {
        if self.methods.is_empty() { return Err("global predicate requires explicit population methods".into()); }
        if let Lineages::Only(keys) = &self.lineages {
            if keys.is_empty() { return Err("global predicate lineage set is empty".into()); }
            for key in keys { crate::validation::key(key)?; }
        }
        Ok(())
    }

    fn includes(&self, entity: &crate::schema::Entity) -> bool {
        entity.alive == self.alive && self.methods.contains(&entity.method)
            && match &self.lineages { Lineages::All => true, Lineages::Only(keys) => keys.contains(&entity.lineage) }
    }
}

impl Predicate {
    pub fn validate(&self, rules: &Rules) -> Result<()> {
        self.condition.validate()?;
        fn finite(expr: &Expr) -> bool {
            !matches!(expr, Expr::Draw { .. } | Expr::Parts { .. } | Expr::Carried { .. })
                && expr.children().into_iter().all(finite)
        }
        if !finite(&self.condition) { return Err("global predicate has no random draw or implicit body binding".into()); }
        for read in self.condition.reads() {
            match read.reading {
                Reading::Kept { name } if self.inputs.contains_key(name) => {},
                _ => return Err("global predicate must name its explicit native inputs".into()),
            }
        }
        for (key, input) in &self.inputs {
            crate::validation::key(key)?;
            match input {
                WorldReading::Population { selector } => selector.validate()?,
                WorldReading::Matter { selector, accounts } => {
                    selector.validate()?;
                    if accounts.is_empty() || accounts.iter().any(|key|
                        !matches!(rules.accounts.get(key), Some(AccountKind::Matter { .. }))) {
                        return Err("global matter reading requires explicit declared matter accounts".into());
                    }
                },
                WorldReading::SiteAccount { account, .. } if !rules.accounts.contains_key(account) =>
                    return Err("global predicate names an undeclared site account".into()),
                WorldReading::SiteCondition { condition, .. } if !rules.conditions.contains(condition) =>
                    return Err("global predicate names an undeclared site condition".into()),
                _ => {},
            }
        }
        Ok(())
    }

    pub fn validate_genesis(&self, genesis: &Genesis) -> Result<()> {
        self.validate(&genesis.rules)?;
        for input in self.inputs.values() {
            match input {
                WorldReading::Population { selector } | WorldReading::Matter { selector, .. } => {
                    if let Lineages::Only(keys) = &selector.lineages {
                        if keys.iter().any(|key| !genesis.lineages.contains_key(key)) {
                            return Err("global predicate names an absent founding lineage".into());
                        }
                    }
                },
                WorldReading::SiteAccount { site, .. } | WorldReading::SiteCondition { site, .. }
                    if !genesis.sites.contains_key(site) => return Err("global predicate names an absent native site".into()),
                WorldReading::Spread { at, identity } => {
                    use crate::spread::{Identity, Substrate};
                    if !genesis.spreads.identities.contains(identity) {
                        return Err("global predicate names an unregistered spread".into());
                    }
                    let present = match at {
                        Substrate::Site(site) => genesis.sites.contains_key(site),
                        Substrate::Host(host) => genesis.population.get(*host).is_some()
                            && !matches!(identity, Identity::Territory(_)),
                        Substrate::Place(place) => genesis.spreads.places.iter().any(|ledger| ledger.place == *place),
                    };
                    if !present { return Err("global predicate spread has no admitted substrate".into()); }
                },
                _ => {},
            }
        }
        Ok(())
    }
}

impl Simulation {
    pub fn global_collapse(&self) -> Result<Option<CollapseReading>> {
        let Some(predicate) = &self.genesis.rules.global_collapse else { return Ok(None); };
        let number = |value: u128| i64::try_from(value)
            .map_err(|_| "global predicate reading exceeds the native expression range".to_owned());
        let mut inputs = BTreeMap::new();
        for (key, input) in &predicate.inputs {
            let reading = match input {
                WorldReading::Tick => number(u128::from(self.state.tick))?,
                WorldReading::Population { selector } => {
                    let total = self.state.population.groups.values().filter(|group| selector.includes(&group.entity))
                        .try_fold(0u128, |sum, group| sum.checked_add(u128::from(group.count)).ok_or("global population overflow"))?;
                    number(total)?
                },
                WorldReading::Matter { selector, accounts } => {
                    let mut total = 0u128;
                    for group in self.state.population.groups.values().filter(|group| selector.includes(&group.entity)) {
                        let books = crate::anatomy::books(&group.entity);
                        for account in accounts {
                            let row = u128::from(books.get(account).copied().unwrap_or(0))
                                .checked_mul(u128::from(group.count)).ok_or("global matter overflow")?;
                            total = total.checked_add(row).ok_or("global matter overflow")?;
                        }
                    }
                    number(total)?
                },
                WorldReading::SiteAccount { site, account } => number(u128::from(self.state.sites.get(site)
                    .ok_or("global predicate site is absent")?.accounts.get(account).copied().unwrap_or(0)))?,
                WorldReading::SiteCondition { site, condition } => self.state.sites.get(site)
                    .ok_or("global predicate site is absent")?.conditions.get(condition).copied().unwrap_or(0),
                WorldReading::Spread { at, identity } => number(u128::from(crate::spread::held(&self.state, *at, identity)?))?,
            };
            inputs.insert(key.clone(), reading);
        }
        let result = predicate.condition.eval(&mut |reading| match reading {
            Reading::Kept { name } => inputs.get(name).copied().ok_or("global predicate input is absent".into()),
            _ => Err("global predicate has no such native reading".into()),
        }, &mut |_, _| Err("global predicate draws nothing".into()))?;
        Ok(Some(CollapseReading { tick: self.state.tick, inputs, result, collapsed: result != 0 }))
    }
}

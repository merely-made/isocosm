// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use crate::{
    Result,
    arrival::Arrival,
    rules::{AccountKind, Rules},
    schema::{Id, Key, Ledger, Lineage, Method},
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A named rules-catalogue recipe. Quantities are milligrams of their
/// declared source stocks, retained unchanged in the completed output.
/// No nominal item weight, conversion, yield or work duration is implied.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Recipe {
    pub lineage: Key,
    pub scale: Key,
    pub stocks: Ledger,
}

impl Recipe {
    fn shape(&self) -> Result<()> {
        crate::validation::key(&self.lineage)?;
        crate::validation::key(&self.scale)?;
        if !self.scale.starts_with("scale:") {
            return Err("material output needs an authored scale key".into());
        }
        if self.stocks.is_empty() || self.stocks.values().any(|amount| *amount == 0) {
            return Err("material recipe needs exact positive source stocks".into());
        }
        for key in self.stocks.keys() {
            crate::validation::key(key)?;
        }
        Ok(())
    }

    /// The catalogue may only name declared native matter accounts. Their
    /// source lineage stays the account's, independently of output kind.
    pub fn validate(&self, rules: &Rules) -> Result<()> {
        self.shape()?;
        for key in self.stocks.keys() {
            if !matches!(rules.accounts.get(key), Some(AccountKind::Matter { .. })) {
                return Err(format!("material recipe stock {key} is not declared matter"));
            }
        }
        Ok(())
    }

    /// The output's native kind is its actual lineage, whose declaration
    /// supplies its kingdom. Account source lineages are also checked by
    /// the world's ordinary matter-account validation.
    pub fn validate_lineages(&self, lineages: &BTreeMap<Key, Lineage>) -> Result<()> {
        if !lineages.contains_key(&self.lineage) {
            return Err(format!("material recipe has no output lineage {}", self.lineage));
        }
        Ok(())
    }

    /// Explicit fresh-founding issuance through the existing native door.
    /// Playing construction instead uses `output` and a project transfer.
    /// Arrival has one fixed scale; a different selected scale is refused.
    pub fn arrival(&self, site: Id) -> Result<Arrival> {
        self.shape()?;
        if self.scale != "scale:meso" {
            return Err("native arrival cannot retain this material recipe scale".into());
        }
        Ok(Arrival {
            lineage: self.lineage.clone(),
            site,
            method: Method::Inert,
            accounts: self.stocks.clone(),
            traits: Default::default(),
            skills: Default::default(),
            disposition: [0; 5],
            character: None,
        })
    }
}

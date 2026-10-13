// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::{Result, rules::{AccountKind, Rules}, simulation::Genesis};

impl JointSpec {
    pub fn validate(&self, rules: &Rules) -> Result<()> {
        if self.version != 1 || self.horizon == 0 {
            return Err("joint ecology witness requires version 1 and a positive authored horizon".into());
        }
        crate::validation::key(&self.key)?;
        if let Some(independence) = &self.independence { independence.validate(rules)?; }
        for role in self.selections() {
            crate::validation::key(&role.actor_lineage)?;
            if let Some(lineage) = &role.target_lineage { crate::validation::key(lineage)?; }
            if !rules.processes.contains_key(&role.process) {
                return Err("ecology role process is undeclared".into());
            }
        }
        if self.capture.at_least == 0 || self.returned.at_least == 0 || self.reproduction.at_least == 0 {
            return Err("ecology role quantities must be explicitly positive".into());
        }
        match &self.capture.stock {
            CaptureStock::EnergyAccount(key) if matches!(rules.accounts.get(key), Some(AccountKind::Energy)) => {},
            CaptureStock::OwnReserve(key) if matches!(rules.accounts.get(key),
                Some(AccountKind::Matter { lineage, reserve: true, .. })
                    if lineage == &self.capture.selection.actor_lineage) => {},
            _ => return Err("capture must name native energy or the capturing lineage's own reserve".into()),
        }
        if !matches!(rules.accounts.get(&self.returned.account), Some(AccountKind::Matter { .. })) {
            return Err("matter return must name a declared matter account".into());
        }
        if self.counterplay.selection.target_lineage.is_none() {
            return Err("counterplay requires an actual target lineage".into());
        }
        match &self.counterplay.proof {
            CounterProof::LivingCellsLost { at_least: 0 }
                | CounterProof::TargetDebited { at_least: 0, .. } =>
                return Err("counterplay quantity must be explicitly positive".into()),
            CounterProof::TargetDebited { account, .. } if !rules.accounts.contains_key(account) =>
                return Err("counterplay debit account is undeclared".into()),
            _ => {},
        }
        Ok(())
    }

    pub fn validate_genesis(&self, genesis: &Genesis) -> Result<()> {
        self.validate(&genesis.rules)?;
        if let Some(independence) = &self.independence { independence.validate_genesis(genesis)?; }
        for selection in self.selections() {
            if !genesis.lineages.contains_key(&selection.actor_lineage)
                || selection.target_lineage.as_ref().is_some_and(|key| !genesis.lineages.contains_key(key)) {
                return Err("ecology role names an absent native lineage".into());
            }
        }
        Ok(())
    }
}

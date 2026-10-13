// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Structural admission of the authored native affinity law. This does
//! not certify ecological reachability or manufacture favoured crossings.

use crate::{Result, rules::{Affinity, AccountKind}, simulation::Genesis};

impl Affinity {
    pub fn validate(&self) -> Result<()> {
        if self.domains == 0 { return Err("affinity requires at least one declared domain".into()); }
        if self.favoured.iter().any(|(from, into)| *from >= self.domains || *into >= self.domains) {
            return Err("affinity favours a crossing outside its declared domains".into());
        }
        Ok(())
    }
}

/// Priced edits transfer this exact stock without conversion (403, 739).
/// Its declared material lineage therefore cannot relabel that stock's nis.
pub(crate) fn validate_material_provenance(g: &Genesis) -> Result<()> {
    for material in &g.world.materials {
        let (Some(_), Some(account)) = (&material.density, &material.account) else { continue; };
        if !matches!(g.rules.accounts.get(account), Some(AccountKind::Matter { lineage, .. })
            if lineage == &material.lineage) {
            return Err(format!("priced material {} and account {account} have different native matter provenance",
                material.key));
        }
    }
    Ok(())
}

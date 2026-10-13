// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Native-derived coverage and consumed mechanical effects (875, 879).

mod types;
mod inventory;
mod references;
mod project;
mod consumers;
mod scalars;
mod admission;
mod structural;
pub use types::*;
pub use inventory::GeneratedBasis;
pub use admission::{Admission, admit};

use crate::{Result, rules::Rules, simulation::Genesis};
use std::collections::BTreeSet;

impl IndependenceSpec {
    pub fn validate(&self, rules: &Rules) -> Result<()> {
        if self.version != 1 { return Err("unsupported native ecological independence declaration".into()); }
        let mut nouns = BTreeSet::new();
        for extra in &self.extras {
            if !nouns.insert(&extra.noun) || !rules.processes.contains_key(&extra.consumer.process)
                || extra.effect.process != extra.consumer.process || !rules.processes.contains_key(&extra.compare.process) {
                return Err("independence declaration has duplicate or unbound native consumers".into());
            }
        }
        let (mut names, mut sources) = (BTreeSet::new(), BTreeSet::new());
        for scalar in &self.scalars {
            crate::validation::key(&scalar.name)?;
            if !names.insert(&scalar.name) || !sources.insert(&scalar.source) {
                return Err("independence scalar names and consumed sources must be unique".into());
            }
        }
        Ok(())
    }

    pub fn validate_genesis(&self, genesis: &Genesis) -> Result<()> {
        self.validate(&genesis.rules)?;
        let inventory = inventory::inventory(genesis)?;
        for extra in &self.extras {
            if !inventory.contains_key(&extra.noun) { return Err("claimed extra noun is absent from native source inventory".into()); }
            consumers::consumed(genesis, extra)?;
            let references = references::References::new(genesis, &extra.equivalences)?;
            project::projection(genesis, &extra.effect, &references, true)?;
            project::projection(genesis, &extra.compare, &references, false)?;
        }
        scalars::consumed(genesis, self)?;
        Ok(())
    }
}

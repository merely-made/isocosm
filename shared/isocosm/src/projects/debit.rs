// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::simulation::State;

/// Refuses a missing disposition vocabulary; it neither allocates nor repairs lots.
pub fn guard_debit(state: &State, source: StockSource, account: &str, amount: u64) -> Result<()> {
    if amount > 0 && state.projects.ownership.values().any(|held|
        held.current_source() == Some(source) && held.lot.account == account) {
        return Err("owned-account debit needs explicit native stock-lot disposition (846)".into());
    }
    Ok(())
}

impl Simulation {
    pub(crate) fn project_identity_roots(&self) -> BTreeSet<Id> {
        let mut roots = BTreeSet::new();
        for held in self.state.projects.ownership.values() {
            if let Some(StockSource::Entity(id)) = held.current_source() { roots.insert(id); }
            for owner in &held.lot.owners { if let Owner::Entity(id) = owner { roots.insert(*id); } }
        }
        for project in self.state.projects.records.values() {
            roots.insert(project.maker);
            roots.extend(project.carrier);
            roots.extend(project.output);
            roots.extend(project.permissions.keys().copied());
        }
        roots.extend(self.state.projects.bindings.keys().copied());
        roots
    }
    pub(crate) fn guard_entity_debits(&self, id:Id, before:&Ledger, after:&Entity) -> Result<()> {
        let now = crate::anatomy::books(after);
        for (key,amount) in before {
            guard_debit(&self.state,StockSource::Entity(id),key,
                amount.saturating_sub(now.get(key).copied().unwrap_or(0)))?;
        }
        Ok(())
    }
}

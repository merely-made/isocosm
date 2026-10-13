// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::simulation::State;

mod types;
mod validate;
mod spread;
pub(crate) use spread::{substrate_source, transit_amount};
pub use types::*;
mod produced;
pub use produced::{ProducedOrigin, ProducedSpread, ProducedOperation, ProducedOutput,
    OutputReference, ProducedQuantity, ProducedClaim, ResolvedOutput, ClaimInput,
    ClaimQuantity, ReferencedOutputClaim, ReferencedRetainedOutput};
mod references;
mod capture;
pub(crate) use capture::{Capture, Context, Mode, Balances};
mod disposition;
pub(crate) use disposition::state_balances;
mod commands;
mod runner;
pub(crate) use runner::Report;

/// A slot in the actual physical ledgers, rather than a rendered position.
pub(crate) type Slot = (StockSource, Key);

pub fn guard_debit(state: &State, source: StockSource, account: &str, amount: u64) -> Result<()> {
    guard_debit_at(state,source,account,amount,capture::actual(state,source,account))
}

pub(crate) fn guard_debit_at(state: &State, source: StockSource, account: &str, amount:u64, actual:u64) -> Result<()> {
    if state.projects.debit_capture.record(state, source, account, amount, actual)? { return Ok(()); }
    if amount > 0 && state.projects.ownership.values().any(|held|
        held.current_source() == Some(source) && held.lot.account == account) {
        return Err("owned-account debit needs exact selected lots and an authored disposition (846/849/850)".into());
    }
    Ok(())
}

impl Simulation {
    pub(crate) fn project_identity_roots(&self) -> BTreeSet<Id> {
        let mut roots = BTreeSet::new();
        for held in self.state.projects.ownership.values() {
            if held.at == LotAt::Closed { continue; }
            if let Some(StockSource::Entity(id)) = held.current_source() { roots.insert(id); }
            if let Some(crate::spread::Identity::Territory(id)) = self.state.spreads.identities.iter()
                .find(|identity| identity.key() == held.lot.account) { roots.insert(*id); }
            for owner in &held.lot.owners { if let Owner::Entity(id) = owner { roots.insert(*id); } }
        }
        for mark in &self.state.spreads.transit_marks {
            if let crate::spread::Substrate::Host(id) = mark.source { roots.insert(id); }
            if let Some(crate::spread::Transfer { identity:crate::spread::Identity::Territory(id), .. })
                = self.state.spreads.transit.get(mark.index) { roots.insert(*id); }
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
            guard_debit_at(&self.state,StockSource::Entity(id),key,
                amount.saturating_sub(now.get(key).copied().unwrap_or(0)),*amount)?;
        }
        Ok(())
    }
}

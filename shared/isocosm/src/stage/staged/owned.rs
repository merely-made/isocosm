// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::projects::StockSource;

impl Staged<'_> {
    pub(super) fn guard_owned_debit(&self, who:Binding, key:&str, amount:u64) -> Result<()> {
        let source = match self.holder(who).ok_or("owned debit source is absent")? {
            Holder::Entity(id) | Holder::Part(id,_) => StockSource::Entity(id),
            Holder::Site(id) => StockSource::Site(id),
            _ => return Err("owned debit needs an addressed native source".into()),
        };
        if self.stage.project_debit.as_ref().is_some_and(|(at,account,n)|
            *at == source && account == key && *n == amount) { return Ok(()); }
        crate::projects::guard_debit(&self.sim.state,source,key,amount)
    }
    pub(super) fn guard_owned_body_change(&self, who:Binding, before:&Ledger) -> Result<()> {
        let id = self.sim.bound(self.stage.actor,self.stage.target,who)?;
        self.sim.guard_entity_debits(id,before,self.stage.bodies.get(&id).ok_or("owned debit body is absent")?)
    }
}

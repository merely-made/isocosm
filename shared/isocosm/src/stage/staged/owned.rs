// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::projects::StockSource;

impl Staged<'_> {
    pub(super) fn guard_owned_debit(&self, who:Binding, key:&str, amount:u64) -> Result<()> {
        let holder = if who == Binding::Part { Some(Holder::Entity(self.stage.actor)) } else { self.holder(who) };
        let source = match holder.ok_or("owned debit source is absent")? {
            Holder::Entity(id) | Holder::Part(id,_) => StockSource::Entity(id),
            Holder::Site(id) => StockSource::Site(id),
            _ => return Err("owned debit needs an addressed native source".into()),
        };
        let actual = match source {
            StockSource::Entity(id) => self.stage.bodies.get(&id).map(|body| crate::anatomy::books(body))
                .and_then(|books| books.get(key).copied()).unwrap_or(0),
            StockSource::Site(id) => self.stage.site.as_ref().filter(|_| id == self.stage.place)
                .or_else(|| self.sim.site_at_start(id)).and_then(|site| site.accounts.get(key).copied()).unwrap_or(0),
            StockSource::Place(_) | StockSource::Transit(_) => return Err("body binding cannot invent a finer or transit holder".into()),
        };
        if self.stage.project_debit.as_ref().is_some_and(|(at,account,n)|
            *at == source && account == key && *n == amount) {
            self.sim.state.projects.debit_capture.with(|context| context.exempt(source,key,amount,actual))?;
            return Ok(());
        }
        crate::projects::debit::guard_debit_at(&self.sim.state,source,key,amount,actual)
    }
    pub(super) fn guard_owned_body_change(&self, who:Binding, before:&Ledger) -> Result<()> {
        let id = self.sim.bound(self.stage.actor,self.stage.target,who)?;
        self.sim.guard_entity_debits(id,before,self.stage.bodies.get(&id).ok_or("owned debit body is absent")?)
    }
    pub(super) fn guard_owned_severed(&self, removed:&Ledger) -> Result<()> {
        let actor = self.stage.actor;
        let now = crate::anatomy::books(self.stage.bodies.get(&actor).ok_or("severed stock actor is absent")?);
        for (key, amount) in removed {
            let before = now.get(key).copied().unwrap_or(0).checked_add(*amount).ok_or("severed stock overflow")?;
            crate::projects::debit::guard_debit_at(&self.sim.state,StockSource::Entity(actor),key,*amount,before)?;
        }
        Ok(())
    }
    pub(super) fn owned_balances(&self) -> crate::projects::debit::Balances {
        let mut rows = crate::projects::debit::state_balances(&self.sim.state);
        for (id, body) in &self.stage.bodies {
            rows.retain(|(source, _), _| *source != StockSource::Entity(*id));
            for (key, amount) in crate::anatomy::books(body) { rows.insert((StockSource::Entity(*id),key),amount); }
        }
        if let Some(site) = &self.stage.site {
            rows.retain(|(source, _), _| *source != StockSource::Site(self.stage.place));
            for (key, amount) in &site.accounts { rows.insert((StockSource::Site(self.stage.place),key.clone()),*amount); }
        }
        for (index, body) in self.stage.births.iter().enumerate() {
            let id = self.sim.state.population.next_id + index as u64;
            for (key, amount) in crate::anatomy::books(body) { rows.insert((StockSource::Entity(id),key),amount); }
        }
        rows
    }
    pub(super) fn begin_owned_operation(&self) -> Result<bool> {
        let capture = &self.sim.state.projects.debit_capture;
        let begin = capture.with(|context| Ok(!context.active))?.unwrap_or(false);
        if !begin { return Ok(false); }
        let mut bindings = vec![(Binding::Actor,StockSource::Entity(self.stage.actor)),
            (Binding::Part,StockSource::Entity(self.stage.actor))];
        if let Some(target) = self.stage.target { bindings.push((Binding::Target,StockSource::Entity(target))); }
        if let Some(site) = self.stage.bodies[&self.stage.actor].resident_site() {
            bindings.push((Binding::Place,StockSource::Site(site)));
        }
        capture.with(|context| {
            context.staged_titles = self.stage.projects.ownership.clone();
            context.shadow.extend(context.staged_titles.clone());
            context.produced.extend(self.stage.projects.produced.clone());
            context.begin(self.owned_balances(),bindings,None)
        })?;
        Ok(true)
    }
    pub(super) fn owned_transfer(&self, effect:&Effect) -> Result<()> {
        let preserve = match effect {
            Effect::Transfer { to, account, .. } => {
                let holder = if *to == Binding::Part { Some(Holder::Entity(self.stage.actor)) } else { self.holder(*to) };
                holder.and_then(|holder| match holder {
                    Holder::Entity(id) | Holder::Part(id,_) => Some((StockSource::Entity(id),account.clone())),
                    Holder::Site(id) => Some((StockSource::Site(id),account.clone())), _ => None,
                })
            }, _ => None,
        };
        self.sim.state.projects.debit_capture.with(|context| { context.preserve = preserve; Ok(()) })?;
        Ok(())
    }
    pub(super) fn finish_owned_operation(&mut self) -> Result<()> {
        let after = self.owned_balances();
        let (changes,produced) = self.sim.state.projects.debit_capture.with(|context| {
            for (key, held) in &self.stage.projects.ownership {
                if context.staged_titles.get(key) != Some(held) { context.shadow.insert(key.clone(),held.clone()); }
            }
            if let Err(why) = context.finish(after) {
                context.failed = true;
                return Err(why);
            }
            Ok((context.changes(),context.produced_changes()))
        })?.unwrap_or_default();
        for (key, held) in changes {
            self.stage.projects.ownership_before.entry(key.clone())
                .or_insert_with(|| self.sim.stock_lot_at_start(&key).cloned());
            self.stage.projects.ownership.insert(key,held);
        }
        for (key,record) in produced {
            self.stage.projects.produced_before.entry(key.clone())
                .or_insert_with(|| self.sim.state.projects.produced.get(&key).cloned());
            self.stage.projects.produced.insert(key,record);
        }
        Ok(())
    }
}

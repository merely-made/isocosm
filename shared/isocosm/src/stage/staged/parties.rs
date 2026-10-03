// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The individual runner's parties for the shared effect meanings: the
//! staged bodies and site, a body's own matter routed through its parts.

use super::*;

impl Staged<'_> {
    /// A take or give of a body's own matter, routed through its parts and
    /// logged for the flow record; `None` where the key lives elsewhere.
    fn routed(&mut self, who: Binding, key: &str, amount: u64, give: bool) -> Option<Result<()>> {
        let Some(Holder::Entity(id)) = self.holder(who) else {
            return None;
        };
        let rules = &self.sim.genesis.rules;
        let body = self.stage.bodies.get_mut(&id)?;
        let split = match give {
            true => crate::anatomy::give(body, rules, key, amount)?,
            false => crate::anatomy::take(body, rules, key, amount)?,
        };
        Some(split.map(|parts| {
            let (body, key) = (id, key.into());
            self.stage.routed.push(flows::Routed {
                body,
                key,
                give,
                parts,
            });
        }))
    }
}

impl Parties for Staged<'_> {
    fn development(&mut self, lineage: &str) -> Result<crate::rules::Development> {
        let l = self.sim.state.lineages.get(lineage);
        l.and_then(|l| l.development.clone())
            .ok_or_else(|| format!("{lineage} develops from no recipe"))
    }

    fn held(&mut self, who: Binding, key: &str) -> Result<u64> {
        let rules = &self.sim.genesis.rules;
        match who {
            Binding::Place => Ok(meaning::value(self.ledger(who)?, key)),
            Binding::Part => {
                let (body, id) = self.part()?;
                let part = body.parts.get(&id).ok_or("bound part missing")?;
                Ok(meaning::value(&part.matter, key))
            },
            other => Ok(crate::anatomy::held(self.body(other)?, rules, key)),
        }
    }
    fn reach(&mut self, who: Binding) -> Result<()> {
        if who == Binding::Place {
            let site = self.sim.state.sites.get(&self.stage.place);
            return site.map(|_| ()).ok_or_else(|| "site missing".into());
        }
        self.ledger(who).map(|_| ())
    }
    fn take(&mut self, who: Binding, key: &str, amount: u64) -> Result<()> {
        match self.routed(who, key, amount, false) {
            Some(done) => done,
            None => debit(self.ledger(who)?, key, amount),
        }
    }
    fn give(&mut self, who: Binding, key: &str, amount: u64) -> Result<()> {
        match self.routed(who, key, amount, true) {
            Some(done) => done,
            None => credit(self.ledger(who)?, key, amount),
        }
    }
    fn body(&mut self, who: Binding) -> Result<&mut Entity> {
        let id = self.sim.bound(self.stage.actor, self.stage.target, who)?;
        Ok(self.stage.bodies.get_mut(&id).ok_or("body missing")?)
    }
    fn part(&mut self) -> Result<(&mut Entity, Id)> {
        let part = self.stage.part.ok_or("no part is bound")?;
        Ok((self.actor(), part))
    }
    fn shift(&mut self, key: &str, delta: i64) -> Result<()> {
        meaning::shift(&mut self.site()?.conditions, key, delta, 1)
    }
}

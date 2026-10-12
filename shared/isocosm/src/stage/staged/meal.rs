// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Native meals: the unchanged bite draw and pass quotas, narrowed only
//! when the process declares an addressed port feeding contract.

use super::*;

impl Staged<'_> {
    pub(super) fn meal(
        &mut self, from: Binding, amount: &Amount, into: &Key, of: &[Key], whole: bool,
    ) -> Result<()> {
        if self.body(from)?.journey.is_some() {
            return Err("travelling prey has no resident site".into());
        }
        let sim = self.sim;
        let rules = &sim.genesis.rules;
        let actor = self.stage.actor;
        let portion = self.stage.shares.as_mut().and_then(|s| s.meal.take());
        // The original 459 draw remains over the accounts the meal names.
        let draw = crate::draw(sim.genesis.seed, "bite", &[self.stage.act]);
        let accounts = if of.is_empty() {
            rules.accounts.iter().filter_map(|(k, a)| {
                matches!(a, AccountKind::Matter { .. }).then_some(k.clone())
            }).collect::<Vec<_>>()
        } else { of.to_vec() };
        let bitten = crate::anatomy::bitten(self.body(from)?, &accounts, draw);
        self.stage.bitten = bitten;
        if let (Some(part), Some(Holder::Entity(body))) = (bitten, self.holder(from)) {
            if let Some(d) = &mut self.stage.demands {
                if d.meal.as_ref().is_some_and(|(holder, ..)| *holder == Holder::Entity(body)) {
                    d.meal_part = Some(Holder::Part(body, part));
                }
            }
        }
        let (holdings, alive) = {
            let prey = self.body(from)?;
            let held = match bitten {
                Some(part) => &prey.parts.get(&part).ok_or("bitten part missing")?.matter,
                None => &prey.accounts,
            };
            (held.clone(), prey.alive)
        };
        let mut offered = edible(&holdings, of);
        let mut whole_admitted = true;
        if let Some(feeding) = &self.stage.port_feeding {
            let id = self.stage.intake.ok_or("no intake organ is bound")?;
            let intake = feeding.intake(&self.stage.bodies[&actor], id)?;
            offered = feeding.offered(intake, &offered, alive, rules, &sim.state.lineages)?;
            if offered.is_empty() {
                return Err("intake port admits no matter in the bitten offering".into());
            }
            // Incorporation carries the entire part, not merely `of`. Refused
            // provenance stays in the prey; its admitted share can be digested.
            if whole && bitten.is_some() {
                let all = feeding.offered(intake, &holdings, alive, rules, &sim.state.lineages)?;
                whole_admitted = holdings.iter().filter(|(_, n)| **n > 0)
                    .all(|(k, n)| all.get(k) == Some(n));
            }
        }
        if let (true, Some(part)) = (whole && whole_admitted, bitten) {
            let asked: u64 = match &portion {
                Some(p) => p.values().sum(),
                None => amount.resolved()?,
            };
            let all: u64 = offered.values().sum();
            if asked >= all && self.incorporate(from, part)? { return Ok(()); }
        }
        let taken: Ledger = match portion {
            // Do not redistribute rejected account quotas to other accounts.
            Some(portion) => portion.into_iter().map(|(k, v)| {
                let held = offered.get(&k).copied().unwrap_or(0);
                (k, v.min(held))
            }).filter(|(_, v)| *v > 0).collect(),
            None => meaning::share(&offered, rules, amount.resolved()?),
        };
        let prey = self.holder(from);
        let mut total = 0u64;
        for (key, value) in &taken {
            self.guard_owned_debit(from,key,*value)?;
            match (bitten, prey) {
                (Some(part), Some(Holder::Entity(id))) => {
                    let body = self.body(from)?;
                    let p = body.parts.get_mut(&part).ok_or("bitten part missing")?;
                    debit(&mut p.matter, key, *value)?;
                    self.stage.routed.push(flows::Routed {
                        body: id, key: key.clone(), give: false, parts: vec![(part, *value)],
                    });
                },
                _ => debit(self.ledger(from)?, key, *value)?,
            }
            total = total.checked_add(*value).ok_or("meal overflow")?;
        }
        self.give(Binding::Actor, into, total)?;
        if let Some(prey) = self.holder(from).filter(|_| sim.flowing()) {
            self.stage.legs.extend(taken.into_iter().map(|(key, amount)| Leg {
                from: (prey, key), to: (Holder::Entity(actor), into.clone()), amount,
            }));
        }
        Ok(())
    }
}

// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Native meals: the unchanged bite draw and pass quotas, narrowed only
//! when the process declares an addressed port feeding contract.

use super::*;

mod bite;
mod guard;
mod partition;

impl Staged<'_> {
    /// What a meal asks: its amount, times how often its eater's lineage
    /// decides where a choice takes it (D20), so a slower decider eats as
    /// much over the ticks it lets pass.
    pub(super) fn asked(&self, amount: &Amount) -> Result<u64> {
        let sim = self.sim;
        let choice = sim
            .genesis
            .rules
            .processes
            .get(&self.stage.process)
            .is_some_and(|p| p.causation == Causation::Choice);
        let lineage = self
            .stage
            .bodies
            .get(&self.stage.actor)
            .map(|e| &e.lineage)
            .filter(|_| choice);
        let pace = lineage
            .and_then(|l| sim.state.lineages.get(l))
            .and_then(|l| l.decides)
            .unwrap_or(1);
        amount
            .resolved()?
            .checked_mul(pace)
            .ok_or_else(|| "a meal overflows".into())
    }

    /// The meal itself: what its eater's mouth does not tolerate of what it
    /// took, for `bite` to charge.
    fn eat(
        &mut self,
        from: Binding,
        amount: &Amount,
        into: &Key,
        of: &[Key],
        whole: bool,
    ) -> Result<bite::Harms> {
        self.sim.diagnostic_stage("meal.request");
        if self.body(from)?.journey.is_some() {
            return Err("travelling prey has no resident site".into());
        }
        let asked_amount = self.asked(amount)?;
        let sim = self.sim;
        let rules = &sim.genesis.rules;
        let actor = self.stage.actor;
        // The original 459 draw remains over the accounts the meal names.
        let draw = crate::draw(sim.genesis.seed, "bite", &[self.stage.act]);
        let accounts = if of.is_empty() {
            rules
                .accounts
                .iter()
                .filter_map(|(k, a)| matches!(a, AccountKind::Matter { .. }).then_some(k.clone()))
                .collect::<Vec<_>>()
        } else {
            of.to_vec()
        };
        let partition = self
            .stage
            .port_feeding
            .as_ref()
            .and_then(|f| f.meal.clone());
        let actual = if partition.is_some() {
            let p = rules
                .processes
                .get(&self.stage.process)
                .ok_or("partitioned feed definition missing")?;
            let eater = self.stage.bodies[&actor].clone();
            let offered = p.food_offerings(&eater, self.body(from)?, rules, &sim.state.lineages)?;
            let total: u128 = offered
                .iter()
                .flat_map(|p| p.offered.values())
                .map(|n| u128::from(*n))
                .sum();
            if total == 0 {
                return Err("no admitted exposed offering".into());
            }
            let mut pick = u128::from(draw) % total;
            offered.into_iter().find(|p| {
                let weight: u128 = p.offered.values().map(|n| u128::from(*n)).sum();
                if pick < weight {
                    true
                } else {
                    pick -= weight;
                    false
                }
            })
        } else {
            None
        };
        let bitten = match &actual {
            Some(offering) => offering.part,
            None => crate::anatomy::bitten_with_rules(self.body(from)?, rules, &accounts, draw),
        };
        self.stage.bitten = bitten;
        let source = self.holder(from).ok_or("meal source is missing")?;
        let part = match (bitten, source) {
            (Some(part), Holder::Entity(body)) => Some(Holder::Part(body, part)),
            _ => None,
        };
        let (holdings, alive) = {
            let prey = self.body(from)?;
            let held = match bitten {
                Some(part) => &prey.parts.get(&part).ok_or("bitten part missing")?.matter,
                None => &prey.accounts,
            };
            (held.clone(), crate::mosaic::ports::admission::Eaten::part(prey, bitten, rules))
        };
        if sim.diagnostic_active() {
            sim.diagnostic_meal(crate::watch::diagnostic::MealObservation {
                at: self.stage.effect_at.clone(),
                part: bitten,
                into: into.clone(),
                asked: asked_amount,
                holdings: holdings.clone(),
                offered: Ledger::new(),
                reserved: None,
                planned_at: vec![],
                planned_part: None,
                quota: None,
                taken: Ledger::new(),
                given: Ledger::new(),
                ingesta_before: None,
                ingesta_after: None,
            });
        }
        let mut offered = actual.map_or_else(|| edible(&holdings, of), |p| p.offered);
        let mut whole_admitted = true;
        sim.diagnostic_stage("meal.port");
        if let Some(feeding) = &self.stage.port_feeding {
            let id = self.stage.intake.ok_or("no intake organ is bound")?;
            let intake = feeding.intake(&self.stage.bodies[&actor], id)?;
            offered = feeding.offered(intake, &offered, alive, rules, &sim.state.lineages)?;
            sim.diagnostic_meal_update(Some(&offered), None, None);
            if offered.is_empty() {
                return Err("intake port admits no matter in the bitten offering".into());
            }
            // Incorporation carries the entire part, not merely `of`. Refused
            // provenance stays in the prey; its admitted share can be digested.
            if whole && bitten.is_some() {
                let all = feeding.offered(intake, &holdings, alive, rules, &sim.state.lineages)?;
                whole_admitted = holdings
                    .iter()
                    .filter(|(_, n)| **n > 0)
                    .all(|(k, n)| all.get(k) == Some(n));
            }
        }
        sim.diagnostic_meal_update(Some(&offered), None, None);
        let spec = crate::schedule::MealSpec {
            actual_part: self
                .stage
                .port_feeding
                .as_ref()
                .is_some_and(|f| f.meal.is_some()),
            at: self.stage.effect_at.clone(),
            from: source,
            part,
            asked: asked_amount,
            into: into.clone(),
            of: of.to_vec(),
            eligible: offered
                .iter()
                .filter(|(key, n)| **n > 0 && meaning::matter(rules, key))
                .map(|(key, _)| key.clone())
                .collect(),
            whole,
        };
        sim.diagnostic_stage("meal.quota");
        let portion = if let Some(shares) = &mut self.stage.shares {
            let grant = shares
                .meals
                .pop_front()
                .ok_or("scheduled meal has no planned quota")?;
            sim.diagnostic_reservation(&grant.spec, &grant.portion);
            if grant.spec != spec {
                return Err("scheduled meal does not match its planned declaration or part".into());
            }
            // Account quotas may never enlarge this Eat's authored ask.
            // Earlier authored effects may also have spent this offering.
            // Defence and incorporation read the actual granted bite.
            Some(
                meaning::share(&grant.portion, rules, asked_amount)
                    .into_iter()
                    .filter_map(|(key, n)| {
                        let granted = n.min(offered.get(&key).copied().unwrap_or(0));
                        (granted > 0).then_some((key, granted))
                    })
                    .collect::<Ledger>(),
            )
        } else {
            None
        };
        sim.diagnostic_meal_update(None, Some(portion.as_ref()), None);
        if let Some(demands) = &mut self.stage.demands {
            demands.meals.push(spec);
        }
        // Demand planning must not test the requested bite: other takers and
        // the selected part's shortage determine its final pass-granted quota.
        sim.diagnostic_stage("meal.defence");
        if rules.defence.is_some()
            && self.stage.demands.is_none()
            && let Some(part) = bitten
        {
            let granted = match &portion {
                Some(portion) => portion.values().try_fold(0u64, |sum, value| {
                    sum.checked_add(*value).ok_or("granted bite overflow")
                })?,
                None => meaning::share(&offered, rules, asked_amount)
                    .values()
                    .try_fold(0u64, |sum, value| {
                        sum.checked_add(*value).ok_or("granted bite overflow")
                    })?,
            };
            if let Some(threshold) = crate::defence::threshold(rules, self.body(from)?, part)?
                && granted < threshold
            {
                return Err(format!(
                    "protected part {part:?}: final granted bite {granted} is below {threshold}"
                ));
            }
        }
        // The guard rule reads the bite before it changes anything: armour
        // resists the whole of a part's bite too, so a resisted one is not
        // taken whole.
        sim.diagnostic_stage("meal.guard");
        let guarded = self.guarded(from)?;
        let resisted = guarded.map_or(0, |g| g.resisted);
        sim.diagnostic_stage("meal.whole");
        if let (true, Some(part)) = (whole && whole_admitted && resisted == 0, bitten) {
            let asked: u64 = match &portion {
                Some(p) => p.values().sum(),
                None => asked_amount,
            };
            let all: u64 = offered.values().sum();
            if asked >= all && self.incorporate(from, part)? {
                sim.diagnostic_meal_update(None, None, Some(&holdings));
                sim.diagnostic_given(&holdings);
                let eater = self.untolerated(from, offered.keys())?;
                return Ok(bite::Harms { eater, wards: Self::wards(guarded, 0) });
            }
        }
        let mut taken: Ledger = match portion {
            // Do not redistribute rejected account quotas to other accounts.
            Some(portion) => portion
                .into_iter()
                .map(|(k, v)| {
                    let held = offered.get(&k).copied().unwrap_or(0);
                    (k, v.min(held))
                })
                .filter(|(_, v)| *v > 0)
                .collect(),
            None => meaning::share(&offered, rules, asked_amount),
        };
        let withheld = crate::ecology::Guard::withhold(&mut taken, resisted, draw);
        let harmed = bite::Harms { eater: self.untolerated(from, taken.keys())?, wards: Self::wards(guarded, withheld) };
        sim.diagnostic_stage("meal.transfer");
        let prey = self.holder(from);
        let mut total = 0u64;
        for (key, value) in &taken {
            self.guard_owned_debit(from, key, *value)?;
            match (bitten, prey) {
                (Some(part), Some(Holder::Entity(id))) => {
                    let body = self.body(from)?;
                    let p = body.part_mut(&part).ok_or("bitten part missing")?;
                    debit(&mut p.matter, key, *value)?;
                    self.stage.routed.push(flows::Routed {
                        body: id,
                        key: key.clone(),
                        give: false,
                        parts: vec![(part, *value)],
                    });
                },
                _ => debit(self.ledger(from)?, key, *value)?,
            }
            total = total.checked_add(*value).ok_or("meal overflow")?;
        }
        sim.diagnostic_meal_update(None, None, Some(&taken));
        if let Some(meal) = partition {
            if sim.flowing() {
                let intake = self.stage.intake.ok_or("intake part missing")?;
                if let Some(prey) = prey {
                    self.stage
                        .legs
                        .extend(taken.into_iter().map(|(key, amount)| Leg {
                            from: (prey, key),
                            to: (Holder::Part(actor, intake), meal.ingesta.clone()),
                            amount,
                        }));
                }
            }
            self.partition_meal(&meal, into, total)?;
            return Ok(harmed);
        }
        // The detritus rule: an animal leaves part of what it took at its site as dung.
        let resident = self.stage.bodies[&actor].resident_site() == Some(self.stage.place);
        let dung = if resident {
            crate::ecology::left_by(rules.detritus, &self.stage.bodies[&actor], total, Some(sim.genesis.seed), sim.state.tick, self.stage.place)
        } else {
            0
        };
        self.give(Binding::Actor, into, total - dung)?;
        if dung > 0 {
            self.give(Binding::Place, crate::ecology::DUNG, dung)?;
        }
        sim.diagnostic_given(&Ledger::from([(into.clone(), total - dung)]));
        if let Some(prey) = self.holder(from).filter(|_| sim.flowing()) {
            let mut left = dung;
            for (key, amount) in taken {
                let deposited = amount.min(left);
                left -= deposited;
                for (to, give, n) in [
                    (Holder::Site(self.stage.place), crate::ecology::DUNG.to_owned(), deposited),
                    (Holder::Entity(actor), into.clone(), amount - deposited),
                ] {
                    if n > 0 {
                        self.stage.legs.push(Leg { from: (prey, key.clone()), to: (to, give), amount: n });
                    }
                }
            }
        }
        Ok(harmed)
    }
}

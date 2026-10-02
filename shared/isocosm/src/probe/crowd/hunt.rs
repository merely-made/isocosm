// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Feeding by a weighted draw (ruling 287) as a crowd, under ruling 454.
//! The core has every hunter draw its prey from the pass's start, among the
//! members its selector accepts, each weighted by what it holds; the crowd
//! draws each hunter's prey state, weighted by its members times what each
//! holds, and which of the state's alike members it lands on, uniformly.
//! Hunters landing on one prey share it: each takes its bite where the prey
//! holds enough for all, or else the same fraction of its bite, floored, and
//! the prey gives what they take as one share of its accounts. Neither
//! runner's order of hunters changes anything, so nothing is refused.

use super::{
    super::{aggregate::Seen, draws::Stream},
    Crowd,
};
use crate::{
    Result,
    meaning::{credit, debit, mass, share},
    rules::{Binding, Effect, Process, Query, Rules, Target},
    schedule::edible,
    schema::*,
};
use std::collections::BTreeMap;

/// A feeding process as the crowd runs it.
struct Meal<'a> {
    selector: &'a Target,
    bite: u64,
    into: &'a Key,
    of: &'a [Key],
    /// The least a prey holds to be taken.
    least: u64,
}

/// Whether a query reads the prey, or anything else only identity settles.
fn binds_target(q: &Query) -> bool {
    match q {
        Query::Related { .. } => true,
        Query::Alive(who) => *who == Binding::Target,
        Query::Trait { who, .. }
        | Query::Account { who, .. }
        | Query::Below { who, .. }
        | Query::Part { who, .. }
        | Query::Holds { who, .. } => *who == Binding::Target,
        _ => false,
    }
}

fn meal(p: &Process) -> Result<Meal<'_>> {
    let refuse = |why: &str| Err(format!("{}: {why}", p.id));
    let Some(selector) = p.target.as_ref().filter(|t| t.weighted) else {
        return refuse("the crowd draws only a weighted target");
    };
    if !selector.same_place {
        return refuse("the crowd hunts only at a hunter's site");
    }
    if p.risk.is_some() || p.note || !p.commitments.is_empty() {
        return refuse("depends on identity; it runs individually");
    }
    if p.expresses().is_some() {
        return refuse("binds a part; it runs individually");
    }
    let [
        Effect::Eat {
            from,
            amount,
            into,
            of,
        },
    ] = p.effects.as_slice()
    else {
        return refuse("the crowd feeds only by eating");
    };
    if *from != Binding::Target {
        return refuse("the crowd eats only its target");
    }
    let Ok(bite) = amount.resolved() else {
        return refuse("bites a computed amount; it runs individually");
    };
    let mut least = 0;
    for q in &p.requires {
        match q {
            Query::Holds {
                who: Binding::Target,
                at_least,
            } => least = least.max(*at_least),
            q if binds_target(q) => return refuse("reads its prey beyond what it holds"),
            _ => {},
        }
    }
    Ok(Meal {
        selector,
        bite,
        into,
        of,
        least,
    })
}

impl Meal<'_> {
    fn accepts(&self, e: &Entity, site: Id, rules: &Rules) -> bool {
        let t = self.selector;
        e.place == site
            && t.alive.is_none_or(|alive| alive == e.alive)
            && t.lineage.as_ref().is_none_or(|l| *l == e.lineage)
            && (t.among.is_empty() || t.among.contains(&e.lineage))
            && mass(&e.accounts, rules) >= u128::from(self.least)
    }
}

/// One prey member the pass's hunters landed on: its state, and each of
/// its hunters by state.
struct Landed {
    prey: usize,
    hunters: Vec<usize>,
}

impl Crowd<'_> {
    /// One feeding process's pass over `hunters`, the states its gates let
    /// through as the pass began.
    pub(super) fn hunt(&mut self, p: &Process, hunters: Vec<(Entity, u64)>) -> Result<()> {
        let m = meal(p)?;
        let rules = &self.world.genesis.rules;
        let mut at: BTreeMap<Id, Vec<(Entity, u64)>> = BTreeMap::new();
        for (e, n) in hunters {
            let site = self.sites.get(&e.place).ok_or("a bin at an unknown site")?;
            if m.accepts(&e, e.place, rules) {
                return Err(format!("{}: a hunter is its own prey", p.id));
            }
            self.work.evaluations += 1;
            self.work.represented += n;
            let seen = Seen {
                member: Some(&e),
                site,
                tick: self.tick,
                rules,
            };
            let mut own = p.requires.iter().filter(|q| !binds_target(q));
            if own.all(|q| seen.holds(q).unwrap_or(false)) {
                at.entry(e.place).or_default().push((e, n));
            } else {
                self.work.blocked += n;
            }
        }
        for (site, hunters) in at {
            self.hunt_at(p, &m, site, hunters)?;
        }
        Ok(())
    }

    fn hunt_at(
        &mut self,
        p: &Process,
        m: &Meal,
        site: Id,
        hunters: Vec<(Entity, u64)>,
    ) -> Result<()> {
        let rules = &self.world.genesis.rules;
        let prey: Vec<(Entity, u64)> = self
            .bins
            .iter()
            .filter(|(e, _)| m.accepts(e, site, rules))
            .map(|(e, &n)| (e.clone(), n))
            .collect();
        let weighted = self.variant != super::Variant::Unweighted;
        let weights: Vec<u128> = prey
            .iter()
            .map(|(e, n)| {
                u128::from(*n)
                    * if weighted {
                        mass(&e.accounts, rules)
                    } else {
                        1
                    }
            })
            .collect();
        let total = u64::try_from(weights.iter().sum::<u128>())
            .map_err(|_| "prey weigh too much to draw")?;
        if total == 0 {
            // No prey: every hunter is blocked, as the core finds no target.
            self.work.blocked += hunters.iter().map(|h| h.1).sum::<u64>();
            return Ok(());
        }
        let domain = format!("probe-hunt:{}", p.id);
        let mut s = Stream::new(crate::draw(self.dynamics, &domain, &[self.tick, site]));
        // Each hunter's prey state, by weight, then which of its members.
        let mut landed: BTreeMap<(usize, u64), Vec<usize>> = BTreeMap::new();
        for (h, (_, n)) in hunters.iter().enumerate() {
            for _ in 0..*n {
                let mut pick = u128::from(s.below(total));
                let state = weights
                    .iter()
                    .position(|w| {
                        let here = pick < *w;
                        if !here {
                            pick -= w;
                        }
                        here
                    })
                    .expect("the pick falls within the total weight");
                let member = s.below(prey[state].1);
                landed.entry((state, member)).or_default().push(h);
            }
            self.work.accepted += n;
        }
        let landed: Vec<Landed> = landed
            .into_iter()
            .map(|((prey, _), hunters)| Landed { prey, hunters })
            .collect();
        let mut moves: Vec<(Entity, Entity)> = Vec::new();
        for l in landed {
            let (was, _) = &prey[l.prey];
            let offered = edible(&was.accounts, m.of);
            let held: u128 = offered
                .iter()
                .filter(|(k, _)| crate::meaning::matter(rules, k))
                .map(|(_, v)| u128::from(*v))
                .sum();
            let asked = u128::from(m.bite) * l.hunters.len() as u128;
            // Each hunter's bite, or the same fraction of it, floored.
            let given = if asked <= held {
                m.bite
            } else {
                (u128::from(m.bite) * held / asked) as u64
            };
            let taken = share(&offered, rules, given * l.hunters.len() as u64);
            if given < m.bite {
                self.shortfalls += 1;
            }
            let mut after = was.clone();
            for (key, value) in &taken {
                debit(&mut after.accounts, key, *value)?;
            }
            moves.push((was.clone(), super::normalize(after)));
            let log = self.meals.entry(p.id.clone()).or_default();
            log.count += l.hunters.len() as u64;
            log.held += mass(&was.accounts, rules) * l.hunters.len() as u128;
            for h in l.hunters {
                let hunter = &hunters[h].0;
                let mut fed = hunter.clone();
                credit(&mut fed.accounts, m.into, given)?;
                moves.push((hunter.clone(), super::normalize(fed)));
            }
        }
        for (from, to) in moves {
            self.moved(&from, to, 1);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;

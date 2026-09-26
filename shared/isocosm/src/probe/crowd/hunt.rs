// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Feeding by a weighted draw (ruling 287) as a crowd. The core draws each
//! hunter's prey among the members its selector accepts, each weighted by
//! what it holds; the crowd draws a prey state, weighted by its members
//! times what each holds, which is the chance the core's draw lands on one
//! of them. Each meal moves one member to its eaten state before the next
//! hunter draws, as the core's pass leaves it. Only prey holding a whole
//! bite are accepted, so every meal is a bite: a hunter's own state never
//! bears on what its meal leaves, and the order the hunters come in changes
//! nothing, until the prey run out part way through a site's hunters. The
//! core then feeds the first in identity order. When those hunters share
//! one state, which ones eat changes nothing either, since they are alike;
//! when they do not, the crowd cannot know which come first, so it refuses,
//! as it refuses a contended site debit.

use super::{
    super::{aggregate::Seen, draws::Stream},
    Crowd,
};
use crate::{
    Result,
    meaning::{credit, mass, share},
    rules::{Binding, Effect, Process, Query, Target},
    schema::*,
};
use std::collections::BTreeMap;

/// A feeding process as the crowd runs it.
struct Meal<'a> {
    selector: &'a Target,
    bite: u64,
    into: &'a Key,
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
    let [Effect::Eat { from, amount, into }] = p.effects.as_slice() else {
        return refuse("the crowd feeds only by eating");
    };
    if *from != Binding::Target {
        return refuse("the crowd eats only its target");
    }
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
    if least < *amount {
        return refuse("a prey may hold less than a bite");
    }
    Ok(Meal {
        selector,
        bite: *amount,
        into,
        least,
    })
}

impl Meal<'_> {
    fn accepts(&self, e: &Entity, site: Id, rules: &crate::rules::Rules) -> bool {
        let t = self.selector;
        e.place == site
            && t.alive.is_none_or(|alive| alive == e.alive)
            && t.lineage.as_ref().is_none_or(|l| *l == e.lineage)
            && (t.among.is_empty() || t.among.contains(&e.lineage))
            && mass(&e.accounts, rules) >= u128::from(self.least)
    }

    /// The meals a prey holding `held` gives before it holds too little.
    fn meals(&self, held: u128) -> u128 {
        let least = u128::from(self.least);
        if held < least {
            0
        } else {
            (held - least) / u128::from(self.bite) + 1
        }
    }
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
        let demand: u64 = hunters.iter().map(|h| h.1).sum();
        let mut prey: BTreeMap<Entity, u64> = self
            .bins
            .iter()
            .filter(|(e, _)| m.accepts(e, site, rules))
            .map(|(e, &n)| (e.clone(), n))
            .collect();
        let capacity: u128 = prey
            .iter()
            .map(|(e, &n)| m.meals(mass(&e.accounts, rules)) * u128::from(n))
            .sum();
        let meals = u64::try_from(capacity.min(u128::from(demand))).expect("at most the demand");
        if 0 < meals && meals < demand && hunters.len() > 1 {
            return Err(format!(
                "{}: prey run out part way through hunters in more than one state at site \
                 {site}, tick {}; the core feeds the first in identity order, which the crowd \
                 cannot know",
                p.id, self.tick
            ));
        }
        if 0 < meals && meals < demand {
            self.shortfalls += 1;
        }
        let domain = format!("probe-hunt:{}", p.id);
        let mut s = Stream::new(crate::draw(self.dynamics, &domain, &[self.tick, site]));
        let weighted = self.variant != super::Variant::Unweighted;
        let weight = |e: &Entity, n: u64| {
            u128::from(n)
                * if weighted {
                    mass(&e.accounts, rules)
                } else {
                    1
                }
        };
        for _ in 0..meals {
            let total: u128 = prey.iter().map(|(e, &n)| weight(e, n)).sum();
            let total = u64::try_from(total).map_err(|_| "prey weigh too much to draw")?;
            let mut pick = u128::from(s.below(total));
            let mut eaten = None;
            for (e, &n) in &prey {
                if pick < weight(e, n) {
                    eaten = Some(e.clone());
                    break;
                }
                pick -= weight(e, n);
            }
            let eaten = eaten.expect("the pick falls within the total weight");
            let mut after = eaten.clone();
            for (key, value) in share(&eaten.accounts, rules, m.bite) {
                crate::meaning::debit(&mut after.accounts, &key, value)?;
            }
            let after = super::normalize(after);
            let slot = prey.get_mut(&eaten).expect("the eaten state is prey");
            *slot -= 1;
            if *slot == 0 {
                prey.remove(&eaten);
            }
            if m.accepts(&after, site, rules) {
                *prey.entry(after.clone()).or_default() += 1;
            }
            self.moved(&eaten, after, 1);
        }
        // Every hunter eats, or the one state they share is split between
        // those that eat and those left without.
        let mut left = meals;
        for (e, n) in hunters {
            let eat = n.min(left);
            left -= eat;
            self.work.accepted += eat;
            self.work.blocked += n - eat;
            if eat > 0 {
                let mut fed = e.clone();
                credit(&mut fed.accounts, m.into, m.bite)?;
                self.moved(&e, super::normalize(fed), eat);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;

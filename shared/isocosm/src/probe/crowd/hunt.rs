// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Feeding by a weighted draw (ruling 287) as a crowd, under ruling 454.
//! The core has every hunter draw its prey from the pass's start, among the
//! members its selector accepts, each weighted by what it holds; the crowd
//! draws each hunter's prey state, weighted by its members times what each
//! holds, and which of the state's alike members it lands on, uniformly.
//! Hunters landing on one prey share it: each takes its mouthful, computed
//! of its own state, where the prey holds enough for all, or else the same
//! fraction of it, floored, and the prey gives what they take as one share
//! of the accounts the meal names. Each hunter's whole act then runs
//! against its prey as the pass began, as the core's does: its bite lands on
//! one part, drawn by what each holds, and takes what that part held of its
//! share, or the part whole (459, 516). What it took lands on the prey as
//! the hunters before it left it only where it fits, as a pass's acts land
//! in the core: a bite of what the part still holds, a part taken whole
//! only as the pass found it; otherwise the act is blocked.

use super::{
    super::{
        aggregate::{self, Seen, Took, binds_target},
        draws::Stream,
    },
    Crowd,
};
use crate::{
    Result,
    anatomy::{self, books},
    meaning::{debit, mass, share},
    rules::{Binding, Effect, Process, Query, Rules, Target},
    schedule::edible,
    schema::*,
};
use std::collections::BTreeMap;

/// A feeding process as the crowd runs it: its meal comes first.
struct Meal<'a> {
    selector: &'a Target,
    of: &'a [Key],
    /// The least a prey holds to be taken.
    least: u64,
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
    let Some(Effect::Eat {
        from: Binding::Target,
        amount,
        of,
        ..
    }) = aggregate::meal_of(p)
    else {
        return refuse("the crowd feeds by eating its target first");
    };
    if !aggregate::slots(p)?.is_empty() {
        return refuse("a hunt that draws runs individually");
    }
    if let crate::rules::Amount::Computed(x) = amount
        && x.reads().iter().any(|u| u.body() == Binding::Target)
    {
        return refuse("a mouthful read of its prey runs individually");
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
    Ok(Meal {
        selector,
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
            && mass(&books(e), rules) >= u128::from(self.least)
    }
}

impl Crowd<'_> {
    /// One feeding process's pass over `hunters`, the states its gates let
    /// through as the pass began.
    pub(super) fn hunt(&mut self, p: &Process, hunters: Vec<(Entity, u64)>) -> Result<()> {
        let m = meal(p)?;
        let rules = &self.world.genesis.rules;
        let start = self.sites.clone();
        let mut at: BTreeMap<Id, Vec<(Entity, u64, u64)>> = BTreeMap::new();
        for (e, n) in hunters {
            let site = start.get(&e.place).ok_or("a bin at an unknown site")?;
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
                lineages: Some(&self.lineages),
            };
            let mut own = p.requires.iter().filter(|q| !binds_target(q));
            if !own.all(|q| seen.holds_for(p, q).unwrap_or(false)) {
                self.work.blocked += n;
                continue;
            }
            // What each of this state's hunters asks of its prey.
            let mouthful = aggregate::mouthful(p, &e, site, (rules, Some(&self.lineages)))?;
            at.entry(e.place).or_default().push((e, n, mouthful));
        }
        for (site, hunters) in at {
            self.hunt_at(p, &m, (site, &start[&site]), hunters)?;
        }
        Ok(())
    }

    fn hunt_at(
        &mut self,
        p: &Process,
        m: &Meal,
        (site, began): (Id, &Site),
        hunters: Vec<(Entity, u64, u64)>,
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
            .map(|(e, n)| u128::from(*n) * if weighted { mass(&books(e), rules) } else { 1 })
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
        for (h, (_, n, _)) in hunters.iter().enumerate() {
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
        }
        let mut moves: Vec<(Entity, Entity)> = Vec::new();
        for ((state, _), eaters) in landed {
            let was = &prey[state].0;
            let mut offered = edible(&books(was), m.of);
            offered.retain(|k, _| crate::meaning::matter(rules, k));
            let held: u128 = offered.values().map(|v| u128::from(*v)).sum();
            let asked: u128 = eaters.iter().map(|&h| u128::from(hunters[h].2)).sum();
            // Each hunter's mouthful, or the same fraction of it, floored.
            let given: Vec<u64> = eaters
                .iter()
                .map(|&h| {
                    let take = hunters[h].2;
                    match asked <= held {
                        true => take,
                        false => (u128::from(take) * held / asked) as u64,
                    }
                })
                .collect();
            if asked > held {
                self.shortfalls += 1;
            }
            let mut left = share(&offered, rules, given.iter().sum());
            let mut after = was.clone();
            for (&h, g) in eaters.iter().zip(given) {
                let portion = share(&left, rules, g);
                for (k, v) in &portion {
                    *left.get_mut(k).expect("a portion of what is left") -= v;
                }
                // A prey with no parts to hold matter draws nothing.
                let bitten = match bodied(was) {
                    true => anatomy::bitten(was, m.of, s.below(u64::MAX)),
                    false => None,
                };
                let hunter = &hunters[h].0;
                let live = self.sites.get(&site).ok_or("a bin at an unknown site")?;
                let mut scratch = live.clone();
                let draws = BTreeMap::new();
                let a = aggregate::Act {
                    start: began,
                    count: 1,
                    tick: self.tick,
                    rules,
                    lineages: Some(&self.lineages),
                    draws: &draws,
                    meal: Some((was, &portion, bitten)),
                };
                let run = aggregate::Run::Free;
                let fed = aggregate::act(p, hunter, &mut scratch, run, a)?;
                let Some(fed) = fed.filter(|f| lands(&f.took, was, &mut after, bitten)) else {
                    self.work.blocked += 1;
                    continue;
                };
                self.sites.insert(site, scratch);
                self.learn(&fed.lessons);
                self.work.accepted += 1;
                let log = self.meals.entry(p.id.clone()).or_default();
                log.count += 1;
                log.held += mass(&books(was), rules);
                moves.push((hunter.clone(), fed.member));
            }
            moves.push((was.clone(), super::normalize(after)));
        }
        for (from, to) in moves {
            self.moved(&from, to, 1);
        }
        Ok(())
    }
}

/// Lands what a meal took on the prey as the hunt has left it, where it
/// fits: a bite of what the part, or the ledger, still holds; a part taken
/// whole only as the pass found it. Returns whether it landed.
fn lands(took: &Option<Took>, was: &Entity, after: &mut Entity, bitten: Option<Id>) -> bool {
    let holds =
        |l: &Ledger, t: &Ledger| t.iter().all(|(k, v)| l.get(k).copied().unwrap_or(0) >= *v);
    match took {
        None => true,
        Some(Took::Whole(id)) => {
            let held = |e: &Entity| {
                let p = e
                    .parts
                    .get(id)
                    .map(|p| p.matter.clone())
                    .unwrap_or_default();
                p.into_iter().filter(|(_, v)| *v > 0).collect::<Ledger>()
            };
            if !after.parts.contains_key(id) || held(after) != held(was) {
                return false;
            }
            after.parts.remove(id);
            if !after.parts.values().any(|q| !q.severed) {
                after.alive = false;
            }
            true
        },
        Some(Took::Bite(taken)) => {
            let ledger = match bitten {
                Some(id) => match after.parts.get_mut(&id) {
                    Some(part) => &mut part.matter,
                    None => return false,
                },
                None => &mut after.accounts,
            };
            if !holds(ledger, taken) {
                return false;
            }
            for (k, v) in taken {
                debit(ledger, k, *v).expect("checked above");
            }
            true
        },
    }
}

/// Whether a prey keeps its matter in parts, so a bite draws one.
fn bodied(e: &Entity) -> bool {
    e.parts.values().any(|p| !p.severed && p.bodied())
}

#[cfg(test)]
mod tests;

// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The fungible as a crowd: counts of members per exact state, with no
//! identities (ruling 113: "who" is undefined until someone looks). Each
//! process applies once per state. The round's randomness is drawn as counts
//! with the distribution the member-by-member round has: which members land
//! in each segment, then the pair types of a uniform random matching, then
//! each fight between two states.

mod hunt;
mod round;

use super::{
    Mind, ProbeWorld,
    aggregate::{self, normalize},
    draws::Stream,
};
use crate::{
    Result,
    meaning::mass,
    rules::{AccountKind, Causation, Process},
    schema::*,
    simulation::Work,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Variant {
    /// The exact-state histogram of ruling 2A.
    Histogram,
    /// The negative control: after each round every lineage's reserves at a
    /// site are replaced by their average, conserving the total.
    Averaged,
    /// The histogram with ruling 220's approximate pairing draw: each count
    /// of the round taken in one step near its mean and variance.
    Approximate,
    /// The draw's negative control (ruling 287): each hunter's prey drawn
    /// by its members alone, as if every prey held alike.
    Unweighted,
}

pub struct Crowd<'w> {
    world: &'w ProbeWorld,
    /// The mind a world's fights read; none in a world without them.
    mind: Option<&'w Mind>,
    dynamics: u64,
    variant: Variant,
    /// Draw the competitions in reverse order, to show it changes nothing.
    reversed: bool,
    matter: u128,
    pub tick: Tick,
    pub sites: BTreeMap<Id, Site>,
    pub bins: BTreeMap<Entity, u64>,
    /// What its lineages develop from, their lexicons as their bodies
    /// have taught them (468).
    pub lineages: BTreeMap<Key, Lineage>,
    pub work: Work,
    /// Prey a hunting pass shared out among eaters it could not cover.
    pub shortfalls: u64,
    /// Site accounts a pass shared out among takers they could not cover
    /// (ruling 454).
    pub site_shortfalls: u64,
    /// Each hunt's meals.
    pub meals: super::MealLog,
}

impl<'w> Crowd<'w> {
    pub fn new(world: &'w ProbeWorld, dynamics: u64, variant: Variant) -> Result<Self> {
        let mut bins = BTreeMap::new();
        for group in world.genesis.population.groups.values() {
            *bins.entry(normalize(group.entity.clone())).or_default() += group.count;
        }
        let mut crowd = Self {
            world,
            mind: world.genesis.rules.mind.as_ref(),
            dynamics,
            variant,
            reversed: false,
            matter: 0,
            tick: 0,
            sites: world.genesis.sites.clone(),
            bins,
            lineages: world.genesis.lineages.clone(),
            work: Work::default(),
            shortfalls: 0,
            site_shortfalls: 0,
            meals: Default::default(),
        };
        crowd.matter = crowd.total_matter();
        Ok(crowd)
    }

    pub fn run(mut self) -> Result<Self> {
        for _ in 0..self.world.ticks {
            self.step()?;
        }
        Ok(self)
    }

    #[cfg(test)]
    pub(super) fn reversed(mut self) -> Self {
        self.reversed = true;
        self
    }

    fn total_matter(&self) -> u128 {
        let rules = &self.world.genesis.rules;
        let members: u128 = self
            .bins
            .iter()
            .map(|(e, &n)| mass(&crate::anatomy::books(e), rules) * u128::from(n))
            .sum();
        members
            + self
                .sites
                .values()
                .map(|s| mass(&s.accounts, rules))
                .sum::<u128>()
    }

    fn step(&mut self) -> Result<()> {
        self.tick += 1;
        let world = self.world;
        let mut due: Vec<&Process> = world
            .genesis
            .rules
            .processes
            .values()
            .filter(|p| p.period.is_some_and(|q| self.tick.is_multiple_of(q)))
            .collect();
        due.sort_by(|a, b| (a.priority, &a.id).cmp(&(b.priority, &b.id)));
        for p in due {
            self.scheduled(p)?;
        }
        self.round()?;
        if self.variant == Variant::Averaged {
            self.average();
        }
        if self.total_matter() != self.matter {
            return Err(format!(
                "the crowd changed total matter at tick {}",
                self.tick
            ));
        }
        Ok(())
    }

    /// Kinds its lineages learned by what their bodies took in (468).
    fn learn(&mut self, lessons: &[(Key, Key)]) {
        for (lineage, kind) in lessons {
            let d = self
                .lineages
                .get_mut(lineage)
                .and_then(|l| l.development.as_mut());
            if let Some(d) = d {
                d.lexicon.insert(kind.clone());
            }
        }
    }

    fn moved(&mut self, from: &Entity, to: Entity, n: u64) {
        let slot = self.bins.get_mut(from).expect("source bin exists");
        *slot -= n;
        if *slot == 0 {
            self.bins.remove(from);
        }
        *self.bins.entry(to).or_default() += n;
    }

    /// The core scheduler's pass, one evaluation per state instead of per
    /// member. States reached during the pass are not evaluated again, and a
    /// state the process's gates keep out, as the core files them, is not
    /// evaluated at all. A process drawing its target by weight hunts.
    fn scheduled(&mut self, p: &Process) -> Result<()> {
        let rules = &self.world.genesis.rules;
        let gates = crate::schedule::Gates::of(p, &self.world.genesis.rules);
        let snapshot: Vec<(Entity, u64)> = self.bins.iter().map(|(e, &n)| (e.clone(), n)).collect();
        let hunting = p.target.as_ref().is_some_and(|t| t.weighted);
        let mut acting = Vec::new();
        for (e, n) in snapshot {
            if !e.alive || !gates.open(&e, self.tick) {
                continue;
            }
            if p.causation == Causation::Agentless {
                if e.kingdom != "kingdom:world" {
                    continue;
                }
            } else if e.method == Method::Inert {
                continue;
            }
            if p.need_account
                .as_ref()
                .is_some_and(|a| crate::anatomy::held(&e, rules, a) >= p.need_below)
            {
                continue;
            }
            acting.push((e, n));
        }
        if hunting {
            return self.hunt(p, acting);
        }
        self.pass(p, acting)
    }

    /// One pass over the states acting in it (ruling 454): each reads its
    /// site as the pass began, and the pass's takes of a site are planned
    /// and shared out, each member the same fraction of its take, floored,
    /// where they would take more than the site held.
    fn pass(&mut self, p: &Process, acting: Vec<(Entity, u64)>) -> Result<()> {
        let rules = &self.world.genesis.rules;
        let start = self.sites.clone();
        let site_of = |e: &Entity| start.get(&e.place).ok_or("a bin at an unknown site");
        // A state's members each draw their own (X3): they are split by
        // what they draw, member by member, before any acts.
        let slots = aggregate::slots(p)?;
        let domain = format!("probe-amount:{}", p.id);
        let mut s = Stream::new(crate::draw(self.dynamics, &domain, &[self.tick]));
        let mut split: Vec<(Entity, u64, BTreeMap<u8, u64>)> = Vec::new();
        for (e, n) in acting {
            if slots.is_empty() {
                split.push((e, n, BTreeMap::new()));
                continue;
            }
            let mut drawn: BTreeMap<BTreeMap<u8, u64>, u64> = BTreeMap::new();
            for _ in 0..n {
                let one = slots.iter().map(|(&slot, &below)| (slot, s.below(below)));
                *drawn.entry(one.collect()).or_default() += 1;
            }
            split.extend(drawn.into_iter().map(|(d, k)| (e.clone(), k, d)));
        }

        // Each state's takes of its site, per member, or none if blocked.
        let mut planned: Vec<Option<Ledger>> = Vec::with_capacity(split.len());
        let mut wanted: BTreeMap<(Id, Key), u128> = BTreeMap::new();
        // A pass that takes nothing it shares is not planned, as the core
        // does not plan it.
        let planning = p.takes_shared();
        for (e, n, draws) in split.iter().filter(|_| planning) {
            let mut takes = Ledger::new();
            let site = site_of(e)?;
            let mut scratch = site.clone();
            let run = aggregate::Run::Plan(&mut takes);
            let a = aggregate::Act {
                start: site,
                count: 1,
                tick: self.tick,
                rules,
                lineages: Some(&self.lineages),
                draws,
                meal: None,
            };
            let blocked = aggregate::act(p, e, &mut scratch, run, a)?;
            if blocked.is_none() {
                planned.push(None);
                continue;
            }
            for (k, v) in &takes {
                *wanted.entry((e.place, k.clone())).or_default() += u128::from(*v) * u128::from(*n);
            }
            planned.push(Some(takes));
        }
        let short = |((site, k), asked): (&(Id, Key), &u128)| {
            *asked > u128::from(start[site].accounts.get(k).copied().unwrap_or(0))
        };
        self.site_shortfalls += wanted.iter().filter(|w| short(*w)).count() as u64;
        let planned = planned.into_iter().map(Some).chain(std::iter::repeat(None));
        for ((e, n, draws), plan) in split.into_iter().zip(planned) {
            self.work.evaluations += 1;
            self.work.represented += n;
            let takes = match plan {
                Some(Some(takes)) => takes,
                Some(None) => {
                    self.work.blocked += n;
                    continue;
                },
                None => Ledger::new(),
            };
            let held = |k: &Key| start[&e.place].accounts.get(k).copied().unwrap_or(0);
            let shares: Ledger = takes
                .into_iter()
                .map(|(k, take)| {
                    let asked = wanted[&(e.place, k.clone())];
                    let given = if asked <= u128::from(held(&k)) {
                        take
                    } else {
                        (u128::from(take) * u128::from(held(&k)) / asked) as u64
                    };
                    (k, given)
                })
                .collect();
            let site = self
                .sites
                .get_mut(&e.place)
                .ok_or("a bin at an unknown site")?;
            let run = match planning {
                true => aggregate::Run::Act(&shares),
                false => aggregate::Run::Free,
            };
            let a = aggregate::Act {
                start: &start[&e.place],
                count: n,
                tick: self.tick,
                rules,
                lineages: Some(&self.lineages),
                draws: &draws,
                meal: None,
            };
            match aggregate::act(p, &e, site, run, a)? {
                Some(next) => {
                    self.work.accepted += n;
                    self.learn(&next.lessons);
                    self.moved(&e, next.member, n);
                },
                None => self.work.blocked += n,
            }
        }
        Ok(())
    }

    fn act(&mut self, e: &Entity, process: &str, n: u64) -> Result<Entity> {
        let world = self.world;
        let p = &world.genesis.rules.processes[process];
        let site = self
            .sites
            .get_mut(&e.place)
            .ok_or("a bin at an unknown site")?;
        self.work.evaluations += 1;
        self.work.represented += n;
        let start = site.clone();
        let rules = &world.genesis.rules;
        let run = aggregate::Run::Free;
        let bodies = (rules, Some(&self.lineages));
        let next = aggregate::apply(p, e, &start, site, n, self.tick, bodies, run)?
            .ok_or_else(|| format!("{process} was blocked in the round"))?;
        self.work.accepted += n;
        Ok(next)
    }

    /// The negative control (ruling 209): every class of living members at
    /// a site has what it holds of an account replaced by the class's
    /// average. The account is the body a competition sizes a kind up by,
    /// or, in a world without competitions, each of a lineage's own matter
    /// accounts in turn, tissue and reserve alike (ruling 507).
    fn average(&mut self) {
        let kinds = self.world.kinds();
        let rules = &self.world.genesis.rules;
        let averaged = |e: &Entity| -> Vec<Key> {
            if kinds.is_empty() {
                let own = |k: &AccountKind| matches!(k, AccountKind::Matter { lineage, .. } if *lineage == e.lineage);
                let found = rules.accounts.iter().filter(|(_, k)| own(k));
                found.map(|(key, _)| key.clone()).collect()
            } else {
                let kind = kinds.iter().find(|k| e.traits.contains(&k.identity));
                kind.map(|k| k.body.clone()).into_iter().collect()
            }
        };
        let accounts = self.bins.keys().map(|e| averaged(e).len()).max();
        for which in 0..accounts.unwrap_or(0) {
            let mut classes: BTreeMap<(Key, Id), Vec<(Entity, u64)>> = BTreeMap::new();
            for (e, &n) in &self.bins {
                if let Some(key) = averaged(e).into_iter().nth(which).filter(|_| e.alive) {
                    let class = classes.entry((key, e.place)).or_default();
                    class.push((e.clone(), n));
                }
            }
            for ((key, _), members) in classes {
                average_class(&mut self.bins, rules, &key, &members);
            }
        }
    }
}

/// One class's average of `key` set into `bins`. Each member keeps
/// everything else; a body keeping matter in parts holds the average only up
/// to their room, the rest going on to members with room so that nothing is
/// lost.
fn average_class(
    bins: &mut BTreeMap<Entity, u64>,
    rules: &crate::rules::Rules,
    key: &Key,
    members: &[(Entity, u64)],
) {
    let n: u64 = members.iter().map(|m| m.1).sum();
    let total: u64 = members
        .iter()
        .map(|(e, m)| crate::anatomy::held(e, rules, key) * m)
        .sum();
    for (e, m) in members {
        let slot = bins.get_mut(e).expect("member bin exists");
        *slot -= m;
        if *slot == 0 {
            bins.remove(e);
        }
    }
    let (base, mut extra) = (total / n, total % n);
    let mut over = 0u64;
    let mut placed: Vec<(Entity, u64)> = vec![];
    for (e, m) in members {
        let high = extra.min(*m);
        extra -= high;
        for (amount, count) in [(base + 1, high), (base, m - high)] {
            if count > 0 {
                let (e, left) = averaged_into(e, rules, key, amount);
                over += left * count;
                placed.push((e, count));
            }
        }
    }
    for (e, mut count) in placed {
        while over > 0 && count > 0 {
            let room = crate::anatomy::room(&e, rules, key);
            if room == 0 || !crate::anatomy::anatomical(&e, rules, key) {
                break;
            }
            let mut one = e.clone();
            let given = room.min(over);
            crate::anatomy::give(&mut one, rules, key, given)
                .expect("given within room")
                .expect("given within room");
            over -= given;
            count -= 1;
            *bins.entry(normalize(one)).or_default() += 1;
        }
        if count > 0 {
            *bins.entry(normalize(e)).or_default() += count;
        }
    }
    debug_assert_eq!(over, 0, "an average fits its class");
}

/// `e` holding `amount` of `key`: in its ledger, or for a body keeping
/// matter in parts emptied from them and given back up to their room.
/// Returns it and what did not fit.
fn averaged_into(e: &Entity, rules: &crate::rules::Rules, key: &Key, amount: u64) -> (Entity, u64) {
    let mut e = e.clone();
    if !crate::anatomy::anatomical(&e, rules, key) {
        e.accounts.insert(key.clone(), amount);
        return (e, 0);
    }
    for part in e.parts.values_mut() {
        part.matter.remove(key);
    }
    let fits = amount.min(crate::anatomy::room(&e, rules, key));
    crate::anatomy::give(&mut e, rules, key, fits)
        .expect("the key lives in parts")
        .expect("given within room");
    (e, amount - fits)
}

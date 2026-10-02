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
};
use crate::{
    Result,
    meaning::{mass, value},
    rules::{Causation, Process},
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
    mind: &'w Mind,
    dynamics: u64,
    variant: Variant,
    /// Draw the competitions in reverse order, to show it changes nothing.
    reversed: bool,
    matter: u128,
    pub tick: Tick,
    pub sites: BTreeMap<Id, Site>,
    pub bins: BTreeMap<Entity, u64>,
    pub work: Work,
    /// Hunting passes at a site where the prey ran out part way through
    /// hunters sharing one state.
    pub shortfalls: u64,
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
            mind: world.mind()?,
            dynamics,
            variant,
            reversed: false,
            matter: 0,
            tick: 0,
            sites: world.genesis.sites.clone(),
            bins,
            work: Work::default(),
            shortfalls: 0,
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
            .map(|(e, &n)| mass(&e.accounts, rules) * u128::from(n))
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
                .is_some_and(|a| value(&e.accounts, a) >= p.need_below)
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
        // Each state's takes of its site, per member, or none if blocked.
        let mut planned: Vec<Option<Ledger>> = Vec::with_capacity(acting.len());
        let mut wanted: BTreeMap<(Id, Key), u128> = BTreeMap::new();
        for (e, n) in &acting {
            let mut takes = Ledger::new();
            let site = site_of(e)?;
            let mut scratch = site.clone();
            let run = aggregate::Run::Plan(&mut takes);
            let blocked = aggregate::apply(p, e, site, &mut scratch, 1, self.tick, rules, run)?;
            if blocked.is_none() {
                planned.push(None);
                continue;
            }
            for (k, v) in &takes {
                *wanted.entry((e.place, k.clone())).or_default() += u128::from(*v) * u128::from(*n);
            }
            planned.push(Some(takes));
        }
        for ((e, n), takes) in acting.into_iter().zip(planned) {
            self.work.evaluations += 1;
            self.work.represented += n;
            let Some(takes) = takes else {
                self.work.blocked += n;
                continue;
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
            let run = aggregate::Run::Act(&shares);
            let from = &start[&e.place];
            match aggregate::apply(p, &e, from, site, n, self.tick, rules, run)? {
                Some(next) => {
                    self.work.accepted += n;
                    self.moved(&e, next, n);
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
        let next = aggregate::apply(p, e, &start, site, n, self.tick, rules, run)?
            .ok_or_else(|| format!("{process} was blocked in the round"))?;
        self.work.accepted += n;
        Ok(next)
    }

    fn average(&mut self) {
        let kinds = self.world.kinds();
        let mut classes: BTreeMap<(usize, Id), Vec<(Entity, u64)>> = BTreeMap::new();
        for (e, &n) in &self.bins {
            let Some(k) = kinds.iter().position(|k| e.traits.contains(&k.identity)) else {
                continue;
            };
            if e.alive {
                classes
                    .entry((k, e.place))
                    .or_default()
                    .push((e.clone(), n));
            }
        }
        for ((k, _), members) in classes {
            let body = &kinds[k].body;
            let n: u64 = members.iter().map(|m| m.1).sum();
            let total: u64 = members
                .iter()
                .map(|(e, m)| value(&e.accounts, body) * m)
                .sum();
            for (e, m) in &members {
                let slot = self.bins.get_mut(e).expect("member bin exists");
                *slot -= m;
                if *slot == 0 {
                    self.bins.remove(e);
                }
            }
            // Each member keeps everything but its reserve.
            let (base, mut extra) = (total / n, total % n);
            for (e, m) in members {
                let high = extra.min(m);
                extra -= high;
                for (reserve, count) in [(base + 1, high), (base, m - high)] {
                    if count > 0 {
                        let mut e = e.clone();
                        e.accounts.insert(body.clone(), reserve);
                        *self.bins.entry(normalize(e)).or_default() += count;
                    }
                }
            }
        }
    }
}

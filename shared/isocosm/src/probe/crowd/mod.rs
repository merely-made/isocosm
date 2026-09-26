// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The fungible as a crowd: counts of members per exact state, with no
//! identities (ruling 113: "who" is undefined until someone looks). Each
//! process applies once per state. The round's randomness is drawn as counts
//! with the distribution the member-by-member round has: which members land
//! in each segment, then the pair types of a uniform random matching, then
//! each fight between two states.

mod round;

use super::{
    Mind, ProbeWorld,
    aggregate::{self, normalize},
};
use crate::{
    Result,
    meaning::{mass, value},
    rules::{Process, Shape},
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
    /// state without a trait the process requires is not evaluated at all.
    fn scheduled(&mut self, p: &Process) -> Result<()> {
        let traits = crate::schedule::required(p);
        let snapshot: Vec<(Entity, u64)> = self.bins.iter().map(|(e, &n)| (e.clone(), n)).collect();
        for (e, n) in snapshot {
            if !e.alive || !crate::schedule::carries(&e, &traits) {
                continue;
            }
            if p.shape == Shape::Agentless {
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
            self.work.evaluations += 1;
            self.work.represented += n;
            let site = self
                .sites
                .get_mut(&e.place)
                .ok_or("a bin at an unknown site")?;
            match aggregate::apply(p, &e, site, n, self.tick, &self.mind.needs)? {
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
        let next = aggregate::apply(p, e, site, n, self.tick, &self.mind.needs)?
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

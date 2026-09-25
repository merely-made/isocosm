// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The fungible as a crowd: counts of members per exact state, with no
//! identities (ruling 113: "who" is undefined until someone looks). Each
//! process applies once per state. The round's randomness is drawn as counts
//! with the distribution the member-by-member round has: which members land
//! in each segment, then the pair types of a uniform random matching.

use super::{
    Competition, ProbeWorld, Side,
    aggregate::{self, normalize, value},
    allocate,
    draws::Stream,
    resolve,
};
use crate::{
    Result,
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
}

pub struct Crowd<'w> {
    world: &'w ProbeWorld,
    dynamics: u64,
    variant: Variant,
    matter: u128,
    pub tick: Tick,
    pub sites: BTreeMap<Id, Site>,
    pub bins: BTreeMap<Entity, u64>,
    pub work: Work,
}

/// Members of one bin that meet one outcome in the round.
struct Fate {
    bin: usize,
    count: u64,
    pay: u64,
    gain: u64,
}

impl<'w> Crowd<'w> {
    pub fn new(world: &'w ProbeWorld, dynamics: u64, variant: Variant) -> Self {
        let mut bins = BTreeMap::new();
        for group in world.genesis.population.groups.values() {
            *bins.entry(normalize(group.entity.clone())).or_default() += group.count;
        }
        let mut crowd = Self {
            world,
            dynamics,
            variant,
            matter: 0,
            tick: 0,
            sites: world.genesis.sites.clone(),
            bins,
            work: Work::default(),
        };
        crowd.matter = crowd.total_matter();
        crowd
    }

    pub fn run(mut self) -> Result<Self> {
        for _ in 0..self.world.ticks {
            self.step()?;
        }
        Ok(self)
    }

    fn total_matter(&self) -> u128 {
        let rules = &self.world.genesis.rules;
        let members: u128 = self
            .bins
            .iter()
            .map(|(e, &n)| aggregate::matter(&e.accounts, rules) * u128::from(n))
            .sum();
        members
            + self
                .sites
                .values()
                .map(|s| aggregate::matter(&s.accounts, rules))
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
    /// member. States reached during the pass are not evaluated again.
    fn scheduled(&mut self, p: &Process) -> Result<()> {
        let snapshot: Vec<(Entity, u64)> = self.bins.iter().map(|(e, &n)| (e.clone(), n)).collect();
        for (e, n) in snapshot {
            if !e.alive {
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
            match aggregate::apply(p, &e, site, n, self.tick)? {
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
        let next = aggregate::apply(p, e, site, n, self.tick)?
            .ok_or_else(|| format!("{process} was blocked in the round"))?;
        self.work.accepted += n;
        Ok(next)
    }

    fn round(&mut self) -> Result<()> {
        let world = self.world;
        let c: &Competition = &world.competition;
        let sites: Vec<Id> = self.sites.keys().copied().collect();
        for site in sites {
            let mut hungry: Vec<(Entity, u64, usize)> = Vec::new();
            for (e, &n) in &self.bins {
                if !e.alive || e.place != site {
                    continue;
                }
                let Some(k) = c.kinds.iter().position(|k| e.traits.contains(&k.identity)) else {
                    continue;
                };
                if aggregate::holds(&c.kinds[k].hungry, e, &self.sites[&site], self.tick)? {
                    hungry.push((e.clone(), n, k));
                }
            }
            let counts: Vec<u64> = hungry.iter().map(|h| h.1).collect();
            let food = value(&self.sites[&site].accounts, &c.food);
            let fates = match allocate(counts.iter().sum(), food / c.ration) {
                None => (0..hungry.len())
                    .map(|bin| Fate {
                        bin,
                        count: counts[bin],
                        pay: 0,
                        gain: c.ration,
                    })
                    .collect(),
                Some(a) => {
                    let seed = crate::draw(self.dynamics, "probe-crowd", &[self.tick, site]);
                    fates(c, &hungry, &counts, a, &mut Stream::new(seed))
                },
            };
            for fate in fates {
                let (e, _, k) = &hungry[fate.bin];
                let kind = &c.kinds[*k];
                let mut state = e.clone();
                for _ in 0..fate.pay {
                    state = self.act(&state, &kind.strain, fate.count)?;
                }
                if fate.gain == c.ration {
                    state = self.act(&state, &kind.eat, fate.count)?;
                } else if fate.gain > 0 {
                    state = self.act(&state, &kind.share, fate.count)?;
                }
                self.moved(e, state, fate.count);
            }
        }
        Ok(())
    }

    fn average(&mut self) {
        let world = self.world;
        let c = &world.competition;
        let mut classes: BTreeMap<(usize, Id), Vec<(Entity, u64)>> = BTreeMap::new();
        for (e, &n) in &self.bins {
            let Some(k) = c.kinds.iter().position(|k| e.traits.contains(&k.identity)) else {
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
            let body = &c.kinds[k].body;
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
            let (base, extra) = (total / n, total % n);
            for (reserve, count) in [(base + 1, extra), (base, n - extra)] {
                if count > 0 {
                    let mut e = members[0].0.clone();
                    e.accounts.insert(body.clone(), reserve);
                    *self.bins.entry(normalize(e)).or_default() += count;
                }
            }
        }
    }
}

/// Draws the round's outcomes by count: the members of each bin that land in
/// the doubles and contested segments, the pair types among the contested,
/// and a coin per exact tie between different bins.
fn fates(
    c: &Competition,
    hungry: &[(Entity, u64, usize)],
    counts: &[u64],
    a: super::Allocation,
    s: &mut Stream,
) -> Vec<Fate> {
    let doubles = s.split(counts, 2 * a.doubles);
    let rest: Vec<u64> = counts.iter().zip(&doubles).map(|(n, d)| n - d).collect();
    let contested = s.split(&rest, 2 * a.contested);
    let mut out: Vec<Fate> = doubles
        .iter()
        .enumerate()
        .filter(|(_, d)| **d > 0)
        .map(|(bin, &count)| Fate {
            bin,
            count,
            pay: 0,
            gain: c.ration,
        })
        .collect();
    let side = |bin: usize| {
        let (e, _, k) = &hungry[bin];
        Side {
            contest: e.traits.contains(&c.contest),
            body: value(&e.accounts, &c.kinds[*k].body),
        }
    };
    for ((i, j), n) in s.matching(&contested) {
        let r = resolve(c, side(i), side(j));
        let wins = if !r.tie {
            None
        } else if i == j {
            Some(n)
        } else {
            Some((0..n).filter(|_| s.coin()).count() as u64)
        };
        let mut push = |bin, count, pay, gain| {
            if count > 0 {
                out.push(Fate {
                    bin,
                    count,
                    pay,
                    gain,
                });
            }
        };
        match wins {
            None => {
                push(i, n, r.pay[0], r.gain[0]);
                push(j, n, r.pay[1], r.gain[1]);
            },
            // A pair within one bin has a winner and a loser from it.
            Some(w) if i == j => {
                push(i, w, r.pay[0], c.ration);
                push(i, n, r.pay[0], 0);
            },
            Some(w) => {
                push(i, w, r.pay[0], c.ration);
                push(i, n - w, r.pay[0], 0);
                push(j, n - w, r.pay[1], c.ration);
                push(j, w, r.pay[1], 0);
            },
        }
    }
    out
}

// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The crowd's competition round. Every competition runs against the
//! members' state at the tick's start (ruling 240). In each, which members
//! land in each segment and how the contested pair up are drawn by count,
//! and each fight between two states is fought through the exact runner's
//! code on copies, what an act makes of a state remembered within the
//! round. A state's members then take from every competition at once: how
//! their takes from one combine with their takes from another is a uniform
//! cross-classification, drawn by count, as independent shuffles make it.
//! Members sharing a state and a set of takes settle together.

use super::{
    super::{
        Allocation, Competition, Meeting,
        aggregate::Seen,
        allocate,
        draws::{Counts, Stream},
        fight::{Copies, Ground, Known, fight},
        meet,
        settle::{Take, settlement},
    },
    Crowd, Variant,
};
use crate::{
    Result,
    meaning::value,
    rules::{Mind, Rules},
    schema::*,
    simulation::Work,
};
use std::collections::BTreeMap;

/// Each hungry state's members, by what they take.
type Takes = BTreeMap<Entity, BTreeMap<Take, u64>>;

/// Everything one competition's draws at one site are made against.
struct Contest<'a> {
    c: &'a Competition,
    mind: &'a Mind,
    rules: &'a Rules,
    hungry: &'a [(Entity, u64, usize)],
    ground: Ground<'a>,
    how: Counts,
}

impl Contest<'_> {
    /// Draws the outcomes by count: the members of each state that land in
    /// the doubles and contested segments, the pair types among the
    /// contested, and each fight.
    fn takes(
        &self,
        a: Allocation,
        s: &mut Stream,
        known: &mut Known,
        work: &mut Work,
    ) -> Result<Takes> {
        let counts: Vec<u64> = self.hungry.iter().map(|h| h.1).collect();
        let doubles = s.split_by(self.how, &counts, 2 * a.doubles);
        let rest: Vec<u64> = counts.iter().zip(&doubles).map(|(n, d)| n - d).collect();
        let contested = s.split_by(self.how, &rest, 2 * a.contested);
        let mut out = Takes::new();
        let mut push = |bin: usize, take: Take, count: u64| {
            if count > 0 && !take.is_nothing() {
                let state = self.hungry[bin].0.clone();
                *out.entry(state).or_default().entry(take).or_default() += count;
            }
        };
        let ration = self.c.ration;
        for (bin, &count) in doubles.iter().enumerate() {
            push(bin, Take::gained(ration), count);
        }
        for ((i, j), n) in s.matching_by(self.how, &contested) {
            let bins = [i, j];
            let kinds = bins.map(|b| &self.c.kinds[self.hungry[b].2]);
            let states = bins.map(|b| &self.hungry[b].0);
            let contest = [0, 1].map(|x| states[x].traits.contains(&self.c.contest));
            let reserve = [0, 1].map(|x| value(&states[x].accounts, &kinds[x].body));
            match meet(self.c, contest, reserve) {
                Meeting::Settled(gain) => {
                    push(i, Take::gained(gain[0]), n);
                    push(j, Take::gained(gain[1]), n);
                },
                Meeting::Fight => {
                    for _ in 0..n {
                        let mut sides = Copies {
                            states: [states[0].clone(), states[1].clone()],
                            known: Some(&mut *known),
                            rules: self.rules,
                            ground: &self.ground,
                            work: &mut *work,
                        };
                        let f = fight(self.c, self.mind, kinds, &self.ground, &mut sides, s)?;
                        push(i, Take::fought(&f, 0, ration), 1);
                        push(j, Take::fought(&f, 1, ration), 1);
                    }
                },
            }
        }
        Ok(out)
    }
}

/// Splits each group of a state's members among `labels` uniformly at
/// random, as a shuffle independent of the one that formed the groups does.
fn cross(
    cells: Vec<(BTreeMap<Key, Take>, u64)>,
    key: &Key,
    labels: &[(Take, u64)],
    s: &mut Stream,
    how: Counts,
) -> Vec<(BTreeMap<Key, Take>, u64)> {
    let mut pool: Vec<u64> = labels.iter().map(|l| l.1).collect();
    let mut out = Vec::new();
    for (takes, m) in cells {
        let left: u64 = pool.iter().sum();
        let split = if m == left {
            pool.clone()
        } else {
            s.split_by(how, &pool, m)
        };
        for (j, &k) in split.iter().enumerate() {
            if k == 0 {
                continue;
            }
            pool[j] -= k;
            let mut takes = takes.clone();
            if !labels[j].0.is_nothing() {
                takes.insert(key.clone(), labels[j].0);
            }
            out.push((takes, k));
        }
    }
    out
}

impl Crowd<'_> {
    /// The approximate crowd draws its counts near; the others exactly.
    fn counts(&self) -> Counts {
        match self.variant {
            Variant::Approximate => Counts::Near,
            Variant::Histogram | Variant::Averaged => Counts::Exact,
        }
    }

    pub(super) fn round(&mut self) -> Result<()> {
        let world = self.world;
        let mut order: Vec<&Key> = world.competitions().keys().collect();
        if self.reversed {
            order.reverse();
        }
        self.round_ordered(&order)
    }

    /// The round with the competitions drawn in `order`; each draws from its
    /// own stream against the tick's start, and takes combine in the order
    /// of what they contest, so any order gives the same crowd.
    fn round_ordered(&mut self, order: &[&Key]) -> Result<()> {
        let (mind, world) = (self.mind, self.world);
        let rules = &world.genesis.rules;
        let sites: Vec<Id> = self.sites.keys().copied().collect();
        for site in sites {
            let ground = self.sites[&site].clone();
            let mut all: BTreeMap<&Key, Takes> = BTreeMap::new();
            for &key in order {
                let c = &world.competitions()[key];
                let mut hungry: Vec<(Entity, u64, usize)> = Vec::new();
                for (e, &n) in &self.bins {
                    if !e.alive || e.place != site {
                        continue;
                    }
                    let Some(k) = c.kinds.iter().position(|k| e.traits.contains(&k.identity))
                    else {
                        continue;
                    };
                    let seen = Seen {
                        member: Some(e),
                        site: &ground,
                        tick: self.tick,
                        needs: &mind.needs,
                    };
                    if seen.holds(&c.kinds[k].hungry)? {
                        hungry.push((e.clone(), n, k));
                    }
                }
                let total: u64 = hungry.iter().map(|h| h.1).sum();
                let food = value(&ground.accounts, key);
                let takes = match allocate(total, food / c.ration) {
                    None => hungry
                        .iter()
                        .map(|(e, n, _)| {
                            (e.clone(), BTreeMap::from([(Take::gained(c.ration), *n)]))
                        })
                        .collect(),
                    Some(a) => {
                        let domain = format!("probe-crowd:{key}");
                        let seed = crate::draw(self.dynamics, &domain, &[self.tick, site]);
                        let contest = Contest {
                            c,
                            mind,
                            rules,
                            hungry: &hungry,
                            ground: Ground {
                                site: &ground,
                                tick: self.tick,
                            },
                            how: self.counts(),
                        };
                        let mut known = Known::new();
                        let mut s = Stream::new(seed);
                        contest.takes(a, &mut s, &mut known, &mut self.work)?
                    },
                };
                all.insert(key, takes);
            }
            self.settle(site, all)?;
        }
        Ok(())
    }

    /// The tick's end at one site: each state's members cross-classified
    /// over the competitions in the order of what they contest, then each
    /// cell settled together.
    fn settle(&mut self, site: Id, all: BTreeMap<&Key, Takes>) -> Result<()> {
        let world = self.world;
        // Counted before any member moves: a settled state can equal one
        // still to settle.
        let mut states: Vec<(Entity, u64)> = all
            .values()
            .flat_map(|t| t.keys())
            .map(|e| (e.clone(), self.bins[e]))
            .collect();
        states.sort();
        states.dedup();
        let seed = crate::draw(self.dynamics, "probe-settle", &[self.tick, site]);
        let mut s = Stream::new(seed);
        for (e, n) in states {
            let mut cells = vec![(BTreeMap::new(), n)];
            for (key, takes) in &all {
                let mut labels: Vec<(Take, u64)> = match takes.get(&e) {
                    Some(t) => t.iter().map(|(t, c)| (*t, *c)).collect(),
                    None => vec![],
                };
                let taking: u64 = labels.iter().map(|l| l.1).sum();
                labels.push((Take::default(), n - taking));
                cells = cross(cells, key, &labels, &mut s, self.counts());
            }
            for (takes, m) in cells {
                let mut state = e.clone();
                for process in settlement(world.competitions(), &e, &takes) {
                    state = self.act(&state, &process, m)?;
                }
                self.moved(&e, state, m);
            }
        }
        Ok(())
    }
}

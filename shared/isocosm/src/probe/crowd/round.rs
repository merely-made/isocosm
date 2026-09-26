// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The crowd's competition round. Which members land in each segment and
//! how the contested pair up are drawn by count. Each fight between two
//! states is then fought through the same code the exact runner uses, on
//! copies of the states; what an act makes of a state is the interpreter's
//! meaning, remembered per state within the round. Members sharing a state,
//! a record of acts and a gain move together.

use super::{
    super::{
        Allocation, Competition, Competitor, Meeting,
        aggregate::{self, Seen},
        allocate,
        draws::Stream,
        fight::{Act, Ground, Sides, fight},
        meet,
    },
    Crowd,
};
use crate::{
    Result,
    meaning::value,
    rules::{Mind, Need, Rules},
    schema::*,
    simulation::Work,
};
use std::collections::BTreeMap;

/// Members of one bin that meet one outcome in the round.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Fate {
    bin: usize,
    acts: Vec<Act>,
    gain: u64,
}

/// What an act makes of a state, as far as the round has needed to know.
type Known = BTreeMap<(Entity, Key), Entity>;

/// Two members fighting as copies of their states.
struct Copies<'a> {
    states: [Entity; 2],
    known: &'a mut Known,
    rules: &'a Rules,
    needs: &'a [Need],
    ground: &'a Ground<'a>,
    work: &'a mut Work,
}

impl Sides for Copies<'_> {
    fn act(&mut self, side: usize, process: &str) -> Result<()> {
        let key = (self.states[side].clone(), process.to_string());
        if let Some(next) = self.known.get(&key) {
            self.states[side] = next.clone();
            return Ok(());
        }
        let p = &self.rules.processes[process];
        // Only an act whose meaning does not depend on the site can be
        // taken apart from it and remembered.
        if !aggregate::site_free(p) {
            return Err(format!("{process} reads the site; a fight cannot take it"));
        }
        self.work.evaluations += 1;
        self.work.represented += 1;
        let mut site = self.ground.site.clone();
        let next = aggregate::apply(p, &key.0, &mut site, 1, self.ground.tick, self.needs)?
            .ok_or_else(|| format!("{process} was blocked in a fight"))?;
        self.work.accepted += 1;
        self.states[side] = next.clone();
        self.known.insert(key, next);
        Ok(())
    }
    fn member(&self, side: usize) -> &Entity {
        &self.states[side]
    }
}

/// Everything the round's draws are made against.
struct Round<'a> {
    c: &'a Competition,
    mind: &'a Mind,
    rules: &'a Rules,
    hungry: &'a [(Entity, u64, usize)],
    ground: Ground<'a>,
}

impl Round<'_> {
    /// Draws the round's outcomes by count: the members of each bin that land
    /// in the doubles and contested segments, the pair types among the
    /// contested, and each fight.
    fn fates(
        &self,
        a: Allocation,
        s: &mut Stream,
        known: &mut Known,
        work: &mut Work,
    ) -> Result<BTreeMap<Fate, u64>> {
        let counts: Vec<u64> = self.hungry.iter().map(|h| h.1).collect();
        let doubles = s.split(&counts, 2 * a.doubles);
        let rest: Vec<u64> = counts.iter().zip(&doubles).map(|(n, d)| n - d).collect();
        let contested = s.split(&rest, 2 * a.contested);
        let mut out: BTreeMap<Fate, u64> = BTreeMap::new();
        let mut push = |bin, acts: Vec<Act>, gain, count| {
            if count > 0 && (gain > 0 || !acts.is_empty()) {
                *out.entry(Fate { bin, acts, gain }).or_default() += count;
            }
        };
        for (bin, &count) in doubles.iter().enumerate() {
            push(bin, vec![], self.c.ration, count);
        }
        for ((i, j), n) in s.matching(&contested) {
            let bins = [i, j];
            let kind = |side: usize| &self.c.kinds[self.hungry[bins[side]].2];
            let state = |side: usize| &self.hungry[bins[side]].0;
            let contest = [0, 1].map(|x| state(x).traits.contains(&self.c.contest));
            let reserve = [0, 1].map(|x| value(&state(x).accounts, &kind(x).body));
            match meet(self.c, contest, reserve) {
                Meeting::Settled(gain) => {
                    push(i, vec![], gain[0], n);
                    push(j, vec![], gain[1], n);
                },
                Meeting::Fight => {
                    let kinds: [&Competitor; 2] = [kind(0), kind(1)];
                    for _ in 0..n {
                        let mut sides = Copies {
                            states: [state(0).clone(), state(1).clone()],
                            known: &mut *known,
                            rules: self.rules,
                            needs: &self.mind.needs,
                            ground: &self.ground,
                            work: &mut *work,
                        };
                        let f = fight(self.c, self.mind, kinds, &self.ground, &mut sides, s)?;
                        let [first, second] = f.acts;
                        let gain = |side| if f.winner == side { self.c.ration } else { 0 };
                        push(i, first, gain(0), 1);
                        push(j, second, gain(1), 1);
                    }
                },
            }
        }
        Ok(out)
    }
}

impl Crowd<'_> {
    pub(super) fn round(&mut self) -> Result<()> {
        let (c, mind, world) = (self.competition, self.mind, self.world);
        let rules = &world.genesis.rules;
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
                let seen = Seen {
                    member: Some(e),
                    site: &self.sites[&site],
                    tick: self.tick,
                    needs: &mind.needs,
                };
                if seen.holds(&c.kinds[k].hungry)? {
                    hungry.push((e.clone(), n, k));
                }
            }
            let ground = self.sites[&site].clone();
            let total: u64 = hungry.iter().map(|h| h.1).sum();
            let food = value(&ground.accounts, &c.food);
            let fates = match allocate(total, food / c.ration) {
                None => (0..hungry.len())
                    .map(|bin| {
                        let fate = Fate {
                            bin,
                            acts: vec![],
                            gain: c.ration,
                        };
                        (fate, hungry[bin].1)
                    })
                    .collect(),
                Some(a) => {
                    let seed = crate::draw(self.dynamics, "probe-crowd", &[self.tick, site]);
                    let round = Round {
                        c,
                        mind,
                        rules,
                        hungry: &hungry,
                        ground: Ground {
                            site: &ground,
                            tick: self.tick,
                        },
                    };
                    let mut known = Known::new();
                    let mut s = Stream::new(seed);
                    round.fates(a, &mut s, &mut known, &mut self.work)?
                },
            };
            for (fate, count) in fates {
                let (e, _, k) = &hungry[fate.bin];
                let kind = &c.kinds[*k];
                let mut state = e.clone();
                for act in &fate.acts {
                    let process = match act {
                        Act::Round => &c.round,
                        Act::Spend => &kind.spend,
                    };
                    state = self.act(&state, process, count)?;
                }
                if fate.gain == c.ration {
                    state = self.act(&state, &kind.eat, count)?;
                } else if fate.gain > 0 {
                    state = self.act(&state, &kind.share, count)?;
                }
                self.moved(e, state, count);
            }
        }
        Ok(())
    }
}

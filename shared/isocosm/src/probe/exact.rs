// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The reference: the core's exact individual runner. Its own scheduler runs
//! the periodic processes member by member; the competition round then
//! executes every member's act through the same interpreter, a fight's
//! rounds included. Collecting after each tick regroups equal neighbours in
//! storage and changes no outcome.

use super::{
    Competition, Meeting, ProbeWorld, allocate,
    draws::Stream,
    fight::{Ground, Sides, fight},
    meet,
};
use crate::meaning::value;
use crate::{
    Execution, Result, Simulation,
    rules::Mind,
    schema::*,
    simulation::{Outcome, Work},
};

pub struct ExactRun {
    pub sim: Simulation,
    pub work: Work,
}

pub fn run_exact(world: &ProbeWorld, dynamics: u64, collect: bool) -> Result<ExactRun> {
    let mut genesis = world.genesis.clone();
    genesis.dynamics = Some(dynamics);
    let mut sim = Simulation::new(genesis, Execution::Individuals)?;
    let mut work = Work::default();
    let (c, mind) = (world.competition()?, world.mind()?);
    for _ in 0..world.ticks {
        let scheduled = sim.advance(1)?;
        work.evaluations += scheduled.evaluations;
        work.represented += scheduled.represented;
        work.accepted += scheduled.accepted;
        work.blocked += scheduled.blocked;
        round(&mut sim, c, mind, dynamics, &mut work)?;
        if collect {
            sim.collect();
        }
    }
    Ok(ExactRun { sim, work })
}

/// Every act the round decides must be accepted: the allocation never
/// promises food the site lacks, nor a fight a cost a member cannot pay.
fn act(sim: &mut Simulation, id: Id, process: &str, work: &mut Work) -> Result<()> {
    let receipt = sim.execute(id, None, process, None);
    work.evaluations += 1;
    work.represented += 1;
    if receipt.outcome != Outcome::Accepted {
        return Err(format!("{process} for {id}: {:?}", receipt.outcome));
    }
    work.accepted += 1;
    Ok(())
}

/// Two members fighting in the world itself.
struct Pair<'a> {
    sim: &'a mut Simulation,
    ids: [Id; 2],
    work: &'a mut Work,
}

impl Sides for Pair<'_> {
    fn act(&mut self, side: usize, process: &str) -> Result<()> {
        act(self.sim, self.ids[side], process, self.work)
    }
    fn member(&self, side: usize) -> &Entity {
        let e = self.sim.state().population.get(self.ids[side]);
        e.expect("fighting members exist")
    }
}

fn round(
    sim: &mut Simulation,
    c: &Competition,
    mind: &Mind,
    dynamics: u64,
    work: &mut Work,
) -> Result<()> {
    let tick = sim.state().tick;
    let sites: Vec<Id> = sim.state().sites.keys().copied().collect();
    for site in sites {
        let mut hungry = Vec::new();
        for (&first, group) in &sim.state().population.groups {
            let e = &group.entity;
            if !e.alive || e.place != site {
                continue;
            }
            let Some(k) = c.kinds.iter().position(|k| e.traits.contains(&k.identity)) else {
                continue;
            };
            // Equal members read alike, so the definition's query is asked
            // once for the group.
            if sim.query(first, None, site, &c.kinds[k].hungry).is_ok() {
                hungry.extend((first..first + group.count).map(|id| (id, k)));
            }
        }
        let ground = sim.state().sites[&site].clone();
        let food = value(&ground.accounts, &c.food);
        let Some(a) = allocate(hungry.len() as u64, food / c.ration) else {
            for &(id, k) in &hungry {
                act(sim, id, &c.kinds[k].eat, work)?;
            }
            continue;
        };
        Stream::new(crate::draw(dynamics, "probe-shuffle", &[tick, site])).shuffle(&mut hungry);
        let doubles = 2 * a.doubles as usize;
        for &(id, k) in &hungry[..doubles] {
            act(sim, id, &c.kinds[k].eat, work)?;
        }
        let contested = &hungry[doubles..doubles + 2 * a.contested as usize];
        for pair in contested.chunks_exact(2) {
            let ids = [pair[0].0, pair[1].0];
            let kinds = [&c.kinds[pair[0].1], &c.kinds[pair[1].1]];
            let member = |sim: &Simulation, side: usize| {
                let e = sim.state().population.get(ids[side]);
                e.expect("hungry member exists").clone()
            };
            let (a, b) = (member(sim, 0), member(sim, 1));
            let contest = [a.traits.contains(&c.contest), b.traits.contains(&c.contest)];
            let reserve = [
                value(&a.accounts, &kinds[0].body),
                value(&b.accounts, &kinds[1].body),
            ];
            let gain = match meet(c, contest, reserve) {
                Meeting::Settled(gain) => gain,
                Meeting::Fight => {
                    let seed = crate::draw(dynamics, "probe-fight", &[tick, ids[0], ids[1]]);
                    let ground = Ground {
                        site: &ground,
                        tick,
                    };
                    let mut sides = Pair { sim, ids, work };
                    let f = fight(c, mind, kinds, &ground, &mut sides, &mut Stream::new(seed))?;
                    let mut gain = [0; 2];
                    gain[f.winner] = c.ration;
                    gain
                },
            };
            for side in 0..2 {
                if gain[side] == c.ration {
                    act(sim, ids[side], &kinds[side].eat, work)?;
                } else if gain[side] > 0 {
                    act(sim, ids[side], &kinds[side].share, work)?;
                }
            }
        }
    }
    Ok(())
}

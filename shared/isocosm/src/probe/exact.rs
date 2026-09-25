// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The reference: the core's exact individual runner. Its own scheduler runs
//! the periodic processes member by member; the competition round then
//! executes every member's act through the same interpreter. Collecting
//! after each tick regroups equal neighbours in storage and changes no
//! outcome.

use super::{Competition, ProbeWorld, Side, aggregate::value, allocate, draws::Stream, resolve};
use crate::{
    Execution, Result, Simulation,
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
    for _ in 0..world.ticks {
        let scheduled = sim.advance(1)?;
        work.evaluations += scheduled.evaluations;
        work.represented += scheduled.represented;
        work.accepted += scheduled.accepted;
        work.blocked += scheduled.blocked;
        round(&mut sim, &world.competition, dynamics, &mut work)?;
        if collect {
            sim.collect();
        }
    }
    Ok(ExactRun { sim, work })
}

/// Every act the round decides must be accepted: the allocation never
/// promises food the site lacks, nor a cost a member cannot pay.
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

fn round(sim: &mut Simulation, c: &Competition, dynamics: u64, work: &mut Work) -> Result<()> {
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
        let food = value(&sim.state().sites[&site].accounts, &c.food);
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
            let side = |(id, k): (Id, usize)| {
                let e = sim
                    .state()
                    .population
                    .get(id)
                    .expect("hungry member exists");
                Side {
                    contest: e.traits.contains(&c.contest),
                    body: value(&e.accounts, &c.kinds[k].body),
                }
            };
            let result = resolve(c, side(pair[0]), side(pair[1]));
            for (j, &(id, k)) in pair.iter().enumerate() {
                for _ in 0..result.pay[j] {
                    act(sim, id, &c.kinds[k].strain, work)?;
                }
            }
            let gain = if result.tie {
                let coin = crate::draw(dynamics, "probe-tie", &[tick, pair[0].0, pair[1].0]);
                if coin.is_multiple_of(2) {
                    [c.ration, 0]
                } else {
                    [0, c.ration]
                }
            } else {
                result.gain
            };
            for (j, &(id, k)) in pair.iter().enumerate() {
                if gain[j] == c.ration {
                    act(sim, id, &c.kinds[k].eat, work)?;
                } else if gain[j] > 0 {
                    act(sim, id, &c.kinds[k].share, work)?;
                }
            }
        }
    }
    Ok(())
}

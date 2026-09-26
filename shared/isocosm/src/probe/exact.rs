// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The reference: the core's exact individual runner. Its own scheduler runs
//! the periodic processes member by member. Then every competition runs
//! against the members' state at the tick's start, member by member, its
//! fights on copies of the two states; at the tick's end each member's
//! takes settle through the interpreter (ruling 240). Collecting after each
//! tick regroups equal neighbours in storage and changes no outcome.

use super::{
    Meeting, ProbeWorld, allocate,
    draws::Stream,
    fight::{Copies, Ground, fight},
    meet,
    settle::{Take, settlement},
};
use crate::meaning::value;
use crate::{
    Execution, Result, Simulation,
    schema::*,
    simulation::{Outcome, Work},
};
use std::collections::BTreeMap;

pub struct ExactRun {
    pub sim: Simulation,
    pub work: Work,
}

pub fn run_exact(world: &ProbeWorld, dynamics: u64, collect: bool) -> Result<ExactRun> {
    let order: Vec<&Key> = world.competitions().keys().collect();
    run_ordered(world, dynamics, collect, &order)
}

/// Runs with the competitions resolved in `order`; any order gives the same
/// world, since each resolves only against the tick's start.
pub(super) fn run_ordered(
    world: &ProbeWorld,
    dynamics: u64,
    collect: bool,
    order: &[&Key],
) -> Result<ExactRun> {
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
        round(&mut sim, world, order, dynamics, &mut work)?;
        if collect {
            sim.collect();
        }
    }
    Ok(ExactRun { sim, work })
}

/// Every act a round settles must be accepted: the allocation never
/// promises food the site lacks, nor settlement a reserve a member lacks.
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

fn round(
    sim: &mut Simulation,
    world: &ProbeWorld,
    order: &[&Key],
    dynamics: u64,
    work: &mut Work,
) -> Result<()> {
    let tick = sim.state().tick;
    let mind = world.mind()?;
    let rules = &world.genesis.rules;
    let sites: Vec<Id> = sim.state().sites.keys().copied().collect();
    let mut takes: BTreeMap<Id, BTreeMap<Key, Take>> = BTreeMap::new();
    let mut take = |id: Id, key: &Key, t: Take| {
        if !t.is_nothing() {
            takes.entry(id).or_default().insert(key.clone(), t);
        }
    };
    let state = sim.state();
    for &key in order {
        let c = &world.competitions()[key];
        for &site in &sites {
            let mut hungry = Vec::new();
            for (&first, group) in &state.population.groups {
                let e = &group.entity;
                if !e.alive || e.place != site {
                    continue;
                }
                let Some(k) = c.kinds.iter().position(|k| e.traits.contains(&k.identity)) else {
                    continue;
                };
                // Equal members read alike, so the definition's query is
                // asked once for the group.
                if sim.query(first, None, site, &c.kinds[k].hungry).is_ok() {
                    hungry.extend((first..first + group.count).map(|id| (id, k)));
                }
            }
            let ground = &state.sites[&site];
            let Some(a) = allocate(hungry.len() as u64, value(&ground.accounts, key) / c.ration)
            else {
                for &(id, _) in &hungry {
                    take(id, key, Take::gained(c.ration));
                }
                continue;
            };
            let shuffle = crate::draw(dynamics, &format!("probe-shuffle:{key}"), &[tick, site]);
            Stream::new(shuffle).shuffle(&mut hungry);
            let doubles = 2 * a.doubles as usize;
            for &(id, _) in &hungry[..doubles] {
                take(id, key, Take::gained(c.ration));
            }
            let contested = &hungry[doubles..doubles + 2 * a.contested as usize];
            for pair in contested.chunks_exact(2) {
                let ids = [pair[0].0, pair[1].0];
                let kinds = [&c.kinds[pair[0].1], &c.kinds[pair[1].1]];
                let states = [0, 1].map(|x| {
                    let e = state.population.get(ids[x]);
                    e.expect("hungry member exists").clone()
                });
                let contest = [0, 1].map(|x| states[x].traits.contains(&c.contest));
                let reserve = [0, 1].map(|x| value(&states[x].accounts, &kinds[x].body));
                match meet(c, contest, reserve) {
                    Meeting::Settled(gain) => {
                        take(ids[0], key, Take::gained(gain[0]));
                        take(ids[1], key, Take::gained(gain[1]));
                    },
                    Meeting::Fight => {
                        let domain = format!("probe-fight:{key}");
                        let seed = crate::draw(dynamics, &domain, &[tick, ids[0], ids[1]]);
                        let ground = Ground { site: ground, tick };
                        let mut sides = Copies {
                            states,
                            known: None,
                            rules,
                            ground: &ground,
                            work: &mut *work,
                        };
                        let f = fight(c, mind, kinds, &ground, &mut sides, &mut Stream::new(seed))?;
                        take(ids[0], key, Take::fought(&f, 0, c.ration));
                        take(ids[1], key, Take::fought(&f, 1, c.ration));
                    },
                }
            }
        }
    }
    // The tick's end: each member's takes settle, in identity order.
    for (id, taken) in takes {
        let member = sim
            .state()
            .population
            .get(id)
            .expect("taking members exist");
        let acts = settlement(world.competitions(), member, &taken);
        for process in acts {
            act(sim, id, &process, work)?;
        }
    }
    Ok(())
}

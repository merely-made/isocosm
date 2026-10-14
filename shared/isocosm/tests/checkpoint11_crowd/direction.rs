// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A territory directing its overflow (498): the crowd's own chooser against
//! actual native choice, for autonomous territories only (881).

use super::fixtures::*;
use isocosm::{rules::{Binding,Causation,Effect,Process,Query},schema::{Id,Method},
    spread::Substrate};
use std::collections::BTreeSet;

pub const DIRECTOR:&str = "c11:director";

fn pick(seed:u64, what:&str, i:u64, below:u64) -> u64 { isocosm::draw(seed,what,&[i]) % below }

fn direction(id:&str, from:Id, destinations:BTreeSet<Id>, priority:i32, period:u64) -> Process {
    Process {id:id.into(),causation:Causation::Choice,
        requires:vec![Query::Alive(Binding::Actor),Query::Trait {who:Binding::Actor,key:DIRECTOR.into()}],
        commitments:vec![],effects:vec![Effect::DirectOverflow {from,destinations}],risk:None,target:None,
        period:Some(period),priority,need_account:None,need_below:0,glyphs:BTreeSet::new(),
        invariants:BTreeSet::from(["sim:matter".into(),"sim:nonnegative".into()]),
        note:true,port_feeding:None,goal_directives:Vec::new()}
}

/// One lifted territory with one to four drawn directions: destination sets
/// over the two transmitting routes and the zero-transmission one, priorities
/// that tie, periods of one to three ticks, and a founding stock over capacity.
pub fn directed(seed:u64, method:Method) -> World {
    let mut w = world(seed);
    let g = &mut w.probe.genesis;
    g.rules.traits.insert(DIRECTOR.into());
    let body = g.population.lift(w.host).unwrap();
    body.method = method;
    body.traits.insert(DIRECTOR.into());
    let routes = [w.to[0],w.to[1],w.to[2]];
    for i in 0..1+pick(seed,"directions",0,4) {
        let mut destinations = BTreeSet::new();
        while destinations.is_empty() {
            for (r,to) in routes.iter().enumerate() {
                if pick(seed,"destination",i*3+r as u64,3) == 0 { destinations.insert(*to); }
            }
            if destinations.is_empty() { destinations.insert(routes[pick(seed,"fallback",i,2) as usize]); }
        }
        let id = format!("c11:direct-{i}");
        let p = direction(&id,w.from,destinations,pick(seed,"priority",i,3) as i32,1+pick(seed,"period",i,3));
        g.rules.processes.insert(id,p);
    }
    g.sites.get_mut(&w.from).unwrap().accounts.insert(w.territory.key(),40+pick(seed,"stock",0,90));
    g.validate().unwrap();
    w
}

/// Directed processes that ran natively, by process.
pub fn ran(p:&Pair) -> Vec<String> {
    p.native.sim.state().events.values().filter(|e| e.process.starts_with("c11:direct-"))
        .map(|e| e.process.clone()).collect()
}

/// Drawn worlds tick by tick: every stock, queue and named body equal, with
/// mid-run placements whose spill remainders the territory then directs.
pub fn timeline(seed:u64, method:Method) -> Vec<String> {
    let w = directed(seed,method);
    let mut p = Pair::new(&w,&BTreeSet::new());
    for tick in 0..7 {
        if tick == 2 || tick == 4 { p.place(&w,&w.territory,Substrate::Site(w.from),7+pick(seed,"placed",tick,40)); }
        p.advance(1);
    }
    p.replay();
    ran(&p)
}

/// A fixed territory with the named directions, its site holding `stock`.
pub fn fixed(method:Method, directions:&[(&str,usize,i32)], stock:u64) -> World {
    let mut w = world(17);
    let g = &mut w.probe.genesis;
    g.rules.traits.insert(DIRECTOR.into());
    let body = g.population.lift(w.host).unwrap();
    body.method = method;
    body.traits.insert(DIRECTOR.into());
    for &(id,route,priority) in directions {
        g.rules.processes.insert(id.into(),direction(id,w.from,BTreeSet::from([w.to[route]]),priority,1));
    }
    g.sites.get_mut(&w.from).unwrap().accounts.insert(w.territory.key(),stock);
    g.validate().unwrap();
    w
}

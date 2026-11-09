// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The heirs of a lost critter: its lineage's other living deliberative
//! members in the order the player is offered them (cases 1204, 1217). A
//! pure function of the state, never kept.

use super::scopes::route_steps;
use crate::{schema::*, simulation::Simulation};
use std::collections::{BTreeMap, VecDeque};

const CHILD: &str = "sim:child";
const PARENT: &str = "sim:parent";

/// Who takes up from `lost`, first to last: its descendants by generation,
/// then the rest by route steps from the site it resided at (members at
/// unreachable sites last), ties by fewer steps to a common ancestor
/// (unrelated last), then by id. The first is the default; any may be taken.
pub fn heirs(sim: &Simulation, lost: Id) -> Vec<Id> {
    let state = sim.state();
    let pop = &state.population;
    let here = pop.get(lost);
    let mut up: BTreeMap<Id, Vec<Id>> = BTreeMap::new();
    let mut down: BTreeMap<Id, Vec<Id>> = BTreeMap::new();
    for r in &state.relations {
        let (parent, child) = match r.kind.as_str() {
            PARENT => (r.object, r.subject),
            CHILD => (r.subject, r.object),
            _ => continue,
        };
        up.entry(child).or_default().push(parent);
        down.entry(parent).or_default().push(child);
    }
    let generations = walk(&down, lost);
    let ancestry = walk(&up, lost);
    let steps = here.and_then(Entity::resident_site).map(|site| route_steps(sim, site));
    let mut found: Vec<_> = pop
        .groups
        .iter()
        .filter(|(first, g)| {
            let e = &g.entity;
            e.alive
                && e.method == Method::Deliberative
                && **first != lost
                && here.is_none_or(|h| h.lineage == e.lineage)
        })
        .map(|(&first, g)| {
            let generation = generations.get(&first).copied();
            let distance = steps.as_ref().and_then(|s| g.entity.resident_site().and_then(|site| s.get(&site)));
            let kin = walk(&up, first)
                .into_iter()
                .filter_map(|(a, n)| ancestry.get(&a).map(|m| n + m))
                .min();
            (generation.is_none(), generation, distance.is_none(), distance.copied(), kin.is_none(), kin, first)
        })
        .collect();
    found.sort();
    found.into_iter().map(|key| key.6).collect()
}

/// Steps from `start` to every id reachable along `edges`, itself at 0.
fn walk(edges: &BTreeMap<Id, Vec<Id>>, start: Id) -> BTreeMap<Id, u32> {
    let mut steps = BTreeMap::from([(start, 0)]);
    let mut queue = VecDeque::from([start]);
    while let Some(at) = queue.pop_front() {
        for &next in edges.get(&at).into_iter().flatten() {
            if !steps.contains_key(&next) {
                steps.insert(next, steps[&at] + 1);
                queue.push_back(next);
            }
        }
    }
    steps
}

// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Habitability for a critter (rulings 179 and 751): a site is habitable
//! for a lineage when it meets every condition the lineage's processes
//! require and a member of the lineage lives there.

use crate::{
    rules::{Binding, Process, Query},
    schema::*,
    simulation::Simulation,
};
use std::collections::BTreeSet;

/// The traits a process requires of its actor.
fn required_traits(p: &Process) -> impl Iterator<Item = &Key> {
    p.requires.iter().filter_map(|q| match q {
        Query::Trait {
            who: Binding::Actor,
            key,
        } => Some(key),
        _ => None,
    })
}

/// The site conditions `lineage`'s processes require: those of every
/// process that asks a trait of its actor, all of which the lineage carries.
pub fn conditions(sim: &Simulation, lineage: &str) -> Vec<(Key, i64)> {
    let Some(l) = sim.state().lineages.get(lineage) else {
        return vec![];
    };
    let ours = |p: &&Process| {
        let mut asked = required_traits(p).peekable();
        asked.peek().is_some() && asked.all(|t| l.traits.contains(t))
    };
    let processes = sim.genesis().rules.processes.values().filter(ours);
    let mut out: Vec<(Key, i64)> = processes
        .flat_map(|p| &p.requires)
        .filter_map(|q| match q {
            Query::Condition { key, at_least } => Some((key.clone(), *at_least)),
            _ => None,
        })
        .collect();
    out.sort();
    out.dedup();
    out
}

/// The sites habitable for `lineage` now.
pub fn habitable(sim: &Simulation, lineage: &str) -> BTreeSet<Id> {
    let wanted = conditions(sim, lineage);
    let s = sim.state();
    let meets = |site: &Site| {
        let at = |key: &Key| site.conditions.get(key).copied();
        wanted
            .iter()
            .all(|(key, least)| at(key).is_some_and(|v| v >= *least))
    };
    let living = s.population.groups.values().map(|g| &g.entity);
    let living = living.filter(|e| e.alive && e.lineage == lineage);
    living
        .map(|e| e.place)
        .filter(|place| s.sites.get(place).is_some_and(meets))
        .collect()
}

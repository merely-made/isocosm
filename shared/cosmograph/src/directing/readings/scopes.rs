// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Per-scope population readings and source availability (sim plan S10.3
//! step 3, S10.4): a patch (one site, the sim's well-mixed compartment), a
//! fixed geographic footprint declared by its sites, and the whole world,
//! each naming its member sites. Pure functions of the state, never kept.
//! A loss at a smaller scope stays a loss even where a wider one holds the
//! lineage. A source is a living resident population with a route path and
//! a native means in the rules; a catalogue entry or a remote, unconnected
//! population is never one.

use crate::{rules::*, schema::*, simulation::Simulation};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

/// What a reading covers.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Scope {
    /// One site.
    Patch(Id),
    /// A fixed set of sites, declared before the run.
    Footprint(BTreeSet<Id>),
    World,
}

/// Living residents of a scope's sites. Travellers belong to no site until
/// they arrive, so they are counted apart, by their destination.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScopeReading {
    pub sites: BTreeSet<Id>,
    /// Living resident members by lineage, cohort multiplicity included.
    pub living: BTreeMap<Key, u64>,
    /// Living resident members by trophic level, the founding's three and
    /// any slotted.
    pub levels: BTreeMap<Key, u64>,
    /// Each level's living matter, as regions read it.
    pub biomass: BTreeMap<Key, u128>,
    /// Living members travelling to a member site, by lineage.
    pub arriving: BTreeMap<Key, u64>,
}

impl ScopeReading {
    pub fn holds(&self, lineage: &str) -> bool {
        self.living.get(lineage).is_some_and(|&n| n > 0)
    }
}

/// The smallest scope at which a lineage is absent, read over nested scopes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Absence {
    Present,
    /// Gone from the patch, held in the footprint.
    Patch,
    /// Gone from the footprint, held elsewhere in the world.
    Regional,
    Global,
}

/// The sites `scope` names.
///
/// # Errors
/// An empty scope, or one naming a site the world lacks.
pub fn sites(sim: &Simulation, scope: &Scope) -> crate::Result<BTreeSet<Id>> {
    let all = &sim.state().sites;
    let named: BTreeSet<Id> = match scope {
        Scope::Patch(site) => BTreeSet::from([*site]),
        Scope::Footprint(sites) => sites.clone(),
        Scope::World => return Ok(all.keys().copied().collect()),
    };
    if named.is_empty() {
        return Err("a scope names at least one site".into());
    }
    match named.iter().find(|s| !all.contains_key(s)) {
        Some(s) => Err(format!("scope names absent site {s}")),
        None => Ok(named),
    }
}

/// The trophic levels read: the founding's three and any slotted level.
fn levels(sim: &Simulation) -> BTreeSet<Key> {
    let slots = sim.genesis().rules.directing().slots;
    crate::directing::found::LEVELS.iter().map(|l| l.to_string()).chain(slots.into_keys()).collect()
}

/// The reading of `scope` now.
///
/// # Errors
/// As [`sites`].
pub fn reading(sim: &Simulation, scope: &Scope) -> crate::Result<ScopeReading> {
    let sites = sites(sim, scope)?;
    let levels = levels(sim);
    let mut r = ScopeReading {
        levels: levels.iter().map(|k| (k.clone(), 0)).collect(),
        biomass: levels.iter().map(|k| (k.clone(), 0)).collect(),
        ..Default::default()
    };
    for g in sim.state().population.groups.values() {
        let e = &g.entity;
        if !e.alive || e.method == Method::Inert {
            continue;
        }
        if let Some(site) = e.resident_site() {
            if !sites.contains(&site) {
                continue;
            }
            *r.living.entry(e.lineage.clone()).or_default() += g.count;
            for (level, n) in r.levels.iter_mut() {
                if e.traits.contains(level) {
                    *n += g.count;
                }
            }
        } else if e.journey.is_some_and(|j| sites.contains(&j.to)) {
            *r.arriving.entry(e.lineage.clone()).or_default() += g.count;
        }
    }
    for (site, by) in super::regions::site_mass(sim, levels.iter()) {
        if sites.contains(&site) {
            for (level, mass) in by {
                *r.biomass.entry(level).or_default() += mass;
            }
        }
    }
    r.sites = sites;
    Ok(r)
}

/// Where `lineage` is absent among a patch, a footprint holding it, and the
/// world.
///
/// # Errors
/// A footprint without its patch, or a site the world lacks.
pub fn absence(sim: &Simulation, lineage: &str, patch: Id, footprint: &BTreeSet<Id>) -> crate::Result<Absence> {
    if !footprint.contains(&patch) {
        return Err("a footprint holds its patch".into());
    }
    let holds = |scope| reading(sim, &scope).map(|r| r.holds(lineage));
    Ok(if holds(Scope::Patch(patch))? {
        Absence::Present
    } else if holds(Scope::Footprint(footprint.clone()))? {
        Absence::Patch
    } else if holds(Scope::World)? {
        Absence::Regional
    } else {
        Absence::Global
    })
}

/// A native way a member reaches the next site on its path.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Means {
    /// A process whose effect or risk moves the actor to `to`.
    Walk { process: Key, to: Id, travel: Tick },
    /// A birth scattered by `medium`, carried with the source site's
    /// strength out of a hundred to one of its routes, drawn.
    Scatter { process: Key, medium: Key, strength: i64 },
}

/// A surviving population that could replenish a target site.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Source {
    pub site: Id,
    /// Its living resident members of the lineage.
    pub members: u64,
    /// Route hops to the target; None where no route path reaches it.
    pub hops: Option<u32>,
    /// The means its members hold for the first hop toward the target.
    pub means: Vec<Means>,
}

impl Source {
    /// Connected and able to go: a path and at least one native means.
    pub fn available(&self) -> bool {
        self.hops.is_some() && !self.means.is_empty()
    }
}

/// Route hops from every site to `target`, over routes as they stand.
fn hops_to(sim: &Simulation, target: Id) -> BTreeMap<Id, u32> {
    let mut back: BTreeMap<Id, Vec<Id>> = BTreeMap::new();
    for (&id, site) in &sim.state().sites {
        for r in &site.routes {
            back.entry(r.to).or_default().push(id);
        }
    }
    let mut hops = BTreeMap::from([(target, 0)]);
    let mut queue = VecDeque::from([target]);
    while let Some(at) = queue.pop_front() {
        for &from in back.get(&at).into_iter().flatten() {
            if !hops.contains_key(&from) {
                hops.insert(from, hops[&at] + 1);
                queue.push_back(from);
            }
        }
    }
    hops
}

/// Whether `p`'s static preconditions admit a member with `traits` at
/// `site`: its actor traits and site conditions. Counts, ages and stores
/// are read when the process runs, not here.
fn admits(p: &Process, traits: &BTreeSet<Key>, site: &Site) -> bool {
    p.requires.iter().all(|q| match q {
        Query::Trait { who: Binding::Actor, key } => traits.contains(key),
        Query::Condition { key, at_least } => site.conditions.get(key).copied().unwrap_or(0) >= *at_least,
        _ => true,
    })
}

fn effects(p: &Process) -> impl Iterator<Item = &Effect> {
    let risk = p.risk.iter().flat_map(|r| &r.effects);
    p.effects.iter().chain(risk).flat_map(|e| std::iter::once(e).chain(e.branches()))
}

/// The surviving populations of `lineage` outside `target`, with their
/// route distance and native means toward it, in site order.
///
/// # Errors
/// A target the world lacks.
pub fn sources(sim: &Simulation, lineage: &str, target: Id) -> crate::Result<Vec<Source>> {
    let state = sim.state();
    if !state.sites.contains_key(&target) {
        return Err(format!("source target {target} is absent"));
    }
    let hops = hops_to(sim, target);
    let mut members: BTreeMap<Id, (u64, BTreeSet<Key>)> = BTreeMap::new();
    for g in state.population.groups.values() {
        let e = &g.entity;
        let Some(site) = e.resident_site().filter(|&s| e.alive && e.lineage == lineage && s != target) else {
            continue;
        };
        let entry = members.entry(site).or_default();
        entry.0 += g.count;
        entry.1.extend(e.traits.iter().cloned());
    }
    let rules = &sim.genesis().rules;
    Ok(members
        .into_iter()
        .map(|(site, (count, traits))| {
            let here = &state.sites[&site];
            let next: BTreeSet<Id> = here
                .routes
                .iter()
                .filter(|r| hops.get(&r.to).is_some_and(|&h| Some(h + 1) == hops.get(&site).copied()))
                .map(|r| r.to)
                .collect();
            let mut means = Vec::new();
            for (key, p) in rules.processes.iter().filter(|(_, p)| admits(p, &traits, here)) {
                for e in effects(p) {
                    match e {
                        Effect::Move { destination, via } | Effect::MovePaced { destination, via, .. }
                            if next.contains(destination) =>
                        {
                            let travel = crate::journey::route(here, *destination, *via).map_or(0, |r| r.travel);
                            means.push(Means::Walk { process: key.clone(), to: *destination, travel });
                        }
                        Effect::Birth { scatter: Some(medium), .. } if !next.is_empty() => {
                            let strength = here.conditions.get(medium).copied().unwrap_or(0);
                            if strength > 0 {
                                means.push(Means::Scatter { process: key.clone(), medium: medium.clone(), strength });
                            }
                        }
                        _ => {}
                    }
                }
            }
            means.dedup();
            Source { site, members: count, hops: hops.get(&site).copied(), means }
        })
        .collect())
}

#[cfg(test)]
#[path = "scopes_tests.rs"]
mod tests;

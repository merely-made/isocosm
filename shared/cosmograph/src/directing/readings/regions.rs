// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Regions at site grain (rulings 225, 226, 687, 689): derived each round,
//! never kept. A region grows from its lowest unclaimed site along routes,
//! taking a neighbour while every trophic level still fits its slot, the
//! world rule's capacity; so regions merge as biomass falls, and geography
//! counts. A region has collapsed when a level it once held is gone (225,
//! 754): a site has held a level where a member of it was founded, lives,
//! lay dead or passed through.

use crate::{schema::*, simulation::Simulation};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Region {
    pub sites: BTreeSet<Id>,
    /// Each slotted level's living matter here.
    pub biomass: BTreeMap<Key, u128>,
    /// The slotted levels its sites have ever held.
    pub held: BTreeSet<Key>,
    /// A level it once held is gone here (225, 754).
    pub collapsed: bool,
}

/// Each site's living matter by slotted level.
pub fn site_biomass(sim: &Simulation) -> BTreeMap<Id, BTreeMap<Key, u128>> {
    let slots = sim.genesis().rules.directing().slots;
    site_mass(sim, slots.keys())
}

/// Each site's living matter by each of `levels`, a member's trait.
pub(crate) fn site_mass<'a>(
    sim: &Simulation,
    levels: impl Iterator<Item = &'a Key> + Clone,
) -> BTreeMap<Id, BTreeMap<Key, u128>> {
    let rules = &sim.genesis().rules;
    let mut by: BTreeMap<Id, BTreeMap<Key, u128>> = sim
        .state()
        .sites
        .keys()
        .map(|&s| (s, levels.clone().map(|k| (k.clone(), 0)).collect()))
        .collect();
    for g in sim.state().population.groups.values() {
        let e = &g.entity;
        let Some(site) = e
            .resident_site()
            .and_then(|site| by.get_mut(&site))
            .filter(|_| e.alive && !rules.dormant(e))
        else {
            continue;
        };
        let pool: i128 = g
            .pool
            .iter()
            .filter(|(key, _)| crate::meaning::matter(rules, key))
            .map(|(_, amount)| *amount)
            .sum();
        let mass = (crate::meaning::mass(&crate::anatomy::books(e), rules) * u128::from(g.count))
            .saturating_add_signed(pool);
        for (level, held) in site.iter_mut() {
            if e.traits.contains(level) {
                *held += mass;
            }
        }
    }
    by
}

/// The slotted levels each site has ever held, read from the founding
/// and from every body's place and visits.
pub fn held(sim: &Simulation) -> BTreeMap<Id, BTreeSet<Key>> {
    let slots = sim.genesis().rules.directing().slots;
    let mut by: BTreeMap<Id, BTreeSet<Key>> = BTreeMap::new();
    let founded = sim.genesis().population.groups.values();
    for g in founded.chain(sim.state().population.groups.values()) {
        let e = &g.entity;
        let levels = slots.keys().filter(|l| e.traits.contains(*l));
        for level in levels {
            let places = e
                .resident_site()
                .into_iter()
                .chain(e.visits.iter().map(|v| v.place));
            for place in places {
                by.entry(place).or_default().insert(level.clone());
            }
        }
    }
    by
}

/// The world's regions now, in order of their lowest site.
pub fn regions(sim: &Simulation) -> Vec<Region> {
    grow(sim, &site_biomass(sim))
}

/// Regions grown over the given per-site biomass.
pub(crate) fn grow(sim: &Simulation, biomass: &BTreeMap<Id, BTreeMap<Key, u128>>) -> Vec<Region> {
    let slots = sim.genesis().rules.directing().slots;
    let sites = &sim.state().sites;
    let ever = held(sim);
    let mut claimed = BTreeSet::new();
    let mut regions = Vec::new();
    for &seed in biomass.keys() {
        if claimed.contains(&seed) {
            continue;
        }
        let mut region = Region {
            sites: BTreeSet::new(),
            biomass: slots.keys().map(|k| (k.clone(), 0)).collect(),
            held: BTreeSet::new(),
            collapsed: false,
        };
        let mut queue = VecDeque::from([seed]);
        while let Some(site) = queue.pop_front() {
            if claimed.contains(&site) {
                continue;
            }
            let here = &biomass[&site];
            let fits = region.sites.is_empty()
                || slots
                    .iter()
                    .all(|(level, slot)| region.biomass[level] + here[level] <= *slot);
            if !fits {
                continue;
            }
            claimed.insert(site);
            region.sites.insert(site);
            region
                .held
                .extend(ever.get(&site).into_iter().flatten().cloned());
            for (level, held) in here {
                *region.biomass.get_mut(level).unwrap() += held;
            }
            let mut next: Vec<Id> = sites[&site].routes.iter().map(|r| r.to).collect();
            next.sort_unstable();
            queue.extend(next.into_iter().filter(|s| !claimed.contains(s)));
        }
        region.collapsed = region.held.iter().any(|l| region.biomass[l] == 0);
        regions.push(region);
    }
    regions
}

/// The region holding `site`.
pub fn region_of(regions: &[Region], site: Id) -> Option<&Region> {
    regions.iter().find(|r| r.sites.contains(&site))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Execution, Founding, Session, directing::Played};

    #[test]
    fn site_biomass_includes_signed_matter_in_a_band_pool() {
        for gap in [-3i128, 5] {
            let g = Founding {
                seed: 3,
                sites: 2,
                population: 30,
                cohort_size: 2,
                lineages: 3,
                ecology: true,
                played: Some(Played {
                    lineage: 1,
                    region_sites: 2,
                }),
                ..Default::default()
            }
            .generate()
            .unwrap();
            let mut s = Session::new(g, Execution::Individuals).unwrap();
            let (&first, group) = s
                .sim
                .state
                .population
                .groups
                .iter()
                .find(|(_, group)| {
                    group.entity.alive && group.entity.traits.contains("life:producer")
                })
                .unwrap();
            let site = group.entity.place;
            let before = site_biomass(&s.sim)[&site]["life:producer"];
            let pool = &mut s.sim.state.population.groups.get_mut(&first).unwrap().pool;
            pool.insert("matter:0-0".into(), gap);
            // A non-matter account cannot alter the trophic reading.
            pool.insert("sim:strain".into(), 700);
            assert_eq!(
                site_biomass(&s.sim)[&site]["life:producer"],
                before.saturating_add_signed(gap)
            );
        }
    }
}

// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The population model's bands (the execution plan's E4; D24, D27 to D29).
//! Outside the near tier, a site and lineage's members fold into bands by
//! age band and the needs they feel: one representative body with a count,
//! and a pool of what its members hold beyond count times the representative.
//! Nothing the game points at folds: attention's roots, relations, nudged
//! or released members, travellers, deliberative members, authored or
//! placed characters, members holding a pose.

use crate::{
    anatomy::books,
    meaning::{Named, Scene},
    population::Pool,
    schema::*,
    simulation::Simulation,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// A world's banding rule (D29): the width of an age band, in ticks.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Banding {
    pub age_band: Tick,
}

/// What puts members in one band.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Band {
    site: Id,
    lineage: Key,
    age: Tick,
    felt: Vec<bool>,
}

/// What the population model's bands hold in their pools (E4, D28): each
/// pool's matter, signed, summed over the world. A pool is below nought
/// where its representative holds more than its members' mean.
pub(crate) fn pooled(state: &crate::simulation::State, rules: &crate::rules::Rules) -> i128 {
    let mut gained = Ledger::new();
    let mut lost = Ledger::new();
    for g in state.population.groups.values() {
        for (k, v) in &g.pool {
            let side = if *v > 0 { &mut gained } else { &mut lost };
            *side.entry(k.clone()).or_default() += v.unsigned_abs() as u64;
        }
    }
    let mass = |a: &Ledger| crate::spread::ledger_mass(a, rules, &state.spreads);
    mass(&gained) as i128 - mass(&lost) as i128
}

impl Simulation {
    /// Folds the far members of each band into one cohort, conserving every
    /// account through its pool.
    pub(crate) fn fold_far(&mut self) {
        if let Some(banding) = self.genesis.rules.banding {
            self.fold_bands(banding);
        }
    }

    /// The same under `banding`.
    pub(crate) fn fold_bands(&mut self, banding: Banding) {
        let width = banding.age_band.max(1);
        let s = &self.state;
        let mut pointed: BTreeSet<Id> = s.roots.iter().copied().collect();
        pointed.extend(s.released.keys());
        pointed.extend(s.nudges.iter().map(|n| n.critter));
        pointed.extend(s.relations.iter().flat_map(|r| [r.subject, r.object]));
        let mut bands: BTreeMap<Band, Vec<Id>> = BTreeMap::new();
        for (&first, g) in &s.population.groups {
            let e = &g.entity;
            let held = e.journey.is_some() || e.authored.is_some() || e.patch.is_some();
            if !e.alive || held || e.method == Method::Deliberative {
                continue;
            }
            let posed = s.spatial_poses.range(first..first + g.count).next().is_some();
            if posed || s.attention.foreground(first, g.count) || pointed.range(first..first + g.count).next().is_some() {
                continue;
            }
            let Some(site) = e.resident_site() else {
                continue;
            };
            let age = s.tick.saturating_sub(e.born) / width;
            let felt = self.felt(e, site);
            bands.entry(Band { site, lineage: e.lineage.clone(), age, felt }).or_default().push(first);
        }
        for firsts in bands.into_values().filter(|f| f.len() > 1) {
            self.fold(&firsts);
        }
    }

    /// Which of the mind's needs a body feels, as mood reads them.
    fn felt(&self, e: &Entity, site: Id) -> Vec<bool> {
        let rules = &self.genesis.rules;
        let Some(mind) = &rules.mind else {
            return vec![];
        };
        let related = |_: &Key| -> crate::Result<bool> { Err("no target".into()) };
        let scene = Scene {
            actor: Some(e),
            target: Named::Unnamed,
            part: None,
            site: self.state.sites.get(&site),
            tick: self.state.tick,
            related: &related,
            rules,
            lineages: Some(&self.state.lineages),
        };
        mind.needs
            .iter()
            .map(|n| n.traits.is_subset(&e.traits) && crate::meaning::read(&n.query, &scene).is_ok_and(|(yes, _)| yes))
            .collect()
    }

    /// Folds `firsts` into one band under the first's body.
    fn fold(&mut self, firsts: &[Id]) {
        let population = &self.state.population;
        let representative = population.groups[&firsts[0]].entity.clone();
        let own = books(&representative);
        let mut pool = Pool::new();
        let mut count = 0;
        for first in firsts {
            let g = &population.groups[first];
            count += g.count;
            let theirs = books(&g.entity);
            let keys: BTreeSet<&Key> = theirs.keys().chain(own.keys()).collect();
            for key in keys {
                let gap = i128::from(*theirs.get(key).unwrap_or(&0)) - i128::from(*own.get(key).unwrap_or(&0));
                *pool.entry(key.clone()).or_default() += gap * i128::from(g.count);
            }
            for (key, v) in &g.pool {
                *pool.entry(key.clone()).or_default() += v;
            }
        }
        pool.retain(|_, v| *v != 0);
        let next = population.next_id;
        if let Some(j) = &mut self.journal {
            for first in firsts {
                j.group(*first, self.state.population.groups.get(first));
            }
            j.group(next, None);
        }
        let population = &mut self.state.population;
        for first in firsts {
            population.groups.remove(first);
        }
        let band = population.insert(representative, count).expect("the band's members existed");
        population.groups.get_mut(&band).expect("inserted above").pool = pool;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Execution, Founding, Session};

    #[test]
    fn far_members_fold_into_bands_conserving_every_account() {
        let g = Founding {
            seed: 3,
            sites: 2,
            population: 60,
            cohort_size: 1,
            lineages: 3,
            ecology: true,
            ..Default::default()
        }
        .generate()
        .unwrap();
        let mut s = Session::new(g, Execution::Individuals).unwrap();
        s.advance(30).unwrap();
        let state = |s: &Session| {
            let p = &s.sim.state().population;
            (s.sim.matter(), p.totals().unwrap(), p.count())
        };
        let before = state(&s);
        let groups = s.sim.state().population.groups.len();
        s.sim.fold_bands(Banding { age_band: 20 });
        assert_eq!(state(&s), before, "folding conserves matter, every account and every member");
        let p = &s.sim.state().population;
        assert!(p.groups.len() < groups, "some members folded");
        let (&first, band) = p.groups.iter().find(|(_, g)| !g.pool.is_empty()).expect("a band whose members differed");
        assert!(band.count > 1);
        s.sim.state.population.lift_shared(first + 1).unwrap();
        assert_eq!(state(&s), before, "lifting a band's member keeps the totals");
    }
}

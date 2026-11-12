// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The matter budget: a read-only count of the world's matter by where it
//! sits, at one tick. It reads the state and writes nothing, is never
//! stored, serialized or hashed, and adds no rule. Its parts sum to
//! [`Simulation::matter`], which counts the same matter in one number.
//!
//! The reservoirs: living bodies and dormant seeds, each by lineage; dead
//! bodies by lineage (matter a body still holds after it dies, for the
//! decomposers and the rot to take); the bands' pools; each site's accounts
//! by account (the soil, and any lineage's matter spilled on the ground);
//! and the rest: polities, finer places, spreads in transit, and projects.

use crate::{anatomy::books, schema::Key, simulation::Simulation, spread::ledger_mass};
use std::collections::BTreeMap;

/// Matter by reservoir, in the milligram-like units the accounts count.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Budget {
    /// Awake, living bodies' matter, by lineage; a band's count times its
    /// representative's.
    pub living: BTreeMap<Key, u128>,
    /// Living bodies that carry the world's dormancy trait, by lineage.
    pub seeds: BTreeMap<Key, u128>,
    /// Dead bodies' matter, by lineage.
    pub dead: BTreeMap<Key, u128>,
    /// What the bands' pools hold beyond their representatives, signed.
    pub pools: i128,
    /// Each site's matter, summed over sites, by account.
    pub sites: BTreeMap<Key, u128>,
    /// Polities' accounts.
    pub polities: u128,
    /// Finer places' accounts.
    pub places: u128,
    /// Spread matter in transit between sites.
    pub transit: u128,
    /// Projects' reserved and constructed matter.
    pub projects: u128,
}

impl Budget {
    /// Every reservoir by name, signed, in a fixed order. The names are
    /// `living <lineage>`, `seeds <lineage>`, `dead <lineage>`, `pools`,
    /// `site <account>`, `polities`, `places`, `transit` and `projects`.
    pub fn reservoirs(&self) -> Vec<(String, i128)> {
        let held = |kind: &str, map: &BTreeMap<Key, u128>| -> Vec<(String, i128)> {
            map.iter().map(|(k, v)| (format!("{kind} {k}"), *v as i128)).collect()
        };
        let mut all = held("living", &self.living);
        all.extend(held("seeds", &self.seeds));
        all.extend(held("dead", &self.dead));
        all.push(("pools".into(), self.pools));
        all.extend(held("site", &self.sites));
        all.extend([
            ("polities".into(), self.polities as i128),
            ("places".into(), self.places as i128),
            ("transit".into(), self.transit as i128),
            ("projects".into(), self.projects as i128),
        ]);
        all
    }

    /// The world's matter: the reservoirs summed. A pool below nought is
    /// matter its band's representative holds beyond the mean, so the sum
    /// of the signed reservoirs is never below nought.
    pub fn total(&self) -> u128 {
        let sum = |map: &BTreeMap<Key, u128>| map.values().sum::<u128>();
        let bodies = sum(&self.living) + sum(&self.seeds) + sum(&self.dead);
        let rest = sum(&self.sites) + self.polities + self.places + self.transit + self.projects;
        u128::try_from(bodies as i128 + self.pools).unwrap_or_default() + rest
    }
}

impl Simulation {
    /// The matter budget at this tick, read-only. Costs one pass over the
    /// bodies and sites, and only when asked.
    pub fn budget(&self) -> Budget {
        let (state, rules) = (&self.state, &self.genesis.rules);
        let mass = |ledger: &crate::schema::Ledger| ledger_mass(ledger, rules, &state.spreads);
        let seed = rules.dormancy.as_ref().map(|d| &d.key);
        let mut budget = Budget::default();
        for g in state.population.groups.values() {
            let e = &g.entity;
            let held = mass(&books(e)) * u128::from(g.count);
            let side = match (e.alive, seed.is_some_and(|key| e.traits.contains(key))) {
                (false, _) => &mut budget.dead,
                (true, true) => &mut budget.seeds,
                (true, false) => &mut budget.living,
            };
            *side.entry(e.lineage.clone()).or_default() += held;
        }
        budget.pools = crate::band::pooled(state, rules);
        for site in state.sites.values() {
            for (key, value) in &site.accounts {
                let one = crate::schema::Ledger::from([(key.clone(), *value)]);
                *budget.sites.entry(key.clone()).or_default() += mass(&one);
            }
        }
        budget.sites.retain(|_, v| *v > 0);
        budget.polities = state.polities.values().map(|p| mass(&p.accounts)).sum();
        budget.places = state.spreads.places.iter().map(|p| mass(&p.accounts)).sum();
        budget.transit = state.spreads.transit.iter().map(|t| u128::from(t.amount)).sum();
        budget.projects = state.projects.records.values().map(|p| mass(&p.reserved) + mass(&p.construction)).sum();
        budget
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Execution, Founding, Session, bodied::Bodies, directing::Played, ecology::Rates, rules::Dormancy};

    fn session() -> Session {
        let grid = crate::map::Grid { width: 3, height: 2, ..crate::map::Grid::drawn(9) };
        let founding = Founding {
            seed: 9,
            population: 256,
            sites: grid.width * grid.height,
            map: Some(crate::map::Layout::Grid(grid)),
            ecology: true,
            ecology_rates: Rates::A1,
            bodies: Some(Bodies::default()),
            played: Some(Played { lineage: 1, region_sites: 2 }),
            dormancy: Some(Dormancy::flat()),
            limits: Some(crate::ecology::Limits::default()),
            ..Founding::default()
        };
        Session::new(founding.generate().unwrap(), Execution::Individuals).unwrap()
    }

    #[test]
    fn the_budget_sums_to_the_world_and_changes_nothing() {
        let mut s = session();
        for ticks in [0, 40] {
            s.advance(ticks).unwrap();
            let hash = s.sim.state_hash();
            let budget = s.sim.budget();
            assert_eq!(s.sim.state_hash(), hash, "reading the budget leaves the world as it was");
            let parts: i128 = budget.reservoirs().iter().map(|(_, v)| v).sum();
            assert_eq!(u128::try_from(parts).unwrap(), budget.total(), "the total is the parts");
            assert_eq!(budget.total(), s.sim.matter(), "the budget counts what the world counts");
            assert!(budget.total() > 0);
            assert_eq!(budget, s.sim.budget(), "reading twice reads the same");
        }
    }
}

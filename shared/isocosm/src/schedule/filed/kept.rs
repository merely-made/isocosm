// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The index kept across advances (the execution plan's E1c, D13). Bodies
//! are checked by the handle each group was filed from, so any write made
//! between advances, by a command, a restriction or a journal putting the
//! world back, files that group again. What gates read beyond bodies is
//! noted: the rules and movement mode the index was built for and each
//! site's routes, any change rebuilding it, and each site's conditions,
//! whose residents are filed again when they change.

use super::*;
use crate::simulation::Genesis;

/// What the index was built against beyond bodies.
#[derive(Clone, Debug, Default)]
pub(super) struct Basis {
    genesis: Weak<Genesis>,
    immediate: bool,
    routes: BTreeMap<Id, Vec<Route>>,
    conditions: BTreeMap<Id, BTreeMap<Key, i64>>,
}

impl Basis {
    fn of(genesis: &Arc<Genesis>, state: &State) -> Self {
        let sites = &state.sites;
        Self {
            genesis: Arc::downgrade(genesis),
            immediate: state.movement.immediate(),
            routes: sites.iter().map(|(id, s)| (*id, s.routes.clone())).collect(),
            conditions: sites.iter().map(|(id, s)| (*id, s.conditions.clone())).collect(),
        }
    }
}

impl Filed {
    /// The index for every process on a period whose gates read anything,
    /// none where no process has such gates.
    pub(crate) fn build(genesis: &Arc<Genesis>, state: &State) -> Option<Self> {
        let rules = &genesis.rules;
        let immediate = state.movement.immediate();
        let gated: BTreeMap<Key, Gates> = rules
            .processes
            .values()
            .filter(|p| p.period.is_some())
            .map(|p| (p.id.clone(), Gates::of(p, rules).residing(p, &state.sites, immediate).reading(p, rules)))
            .filter(|(_, g)| !g.none())
            .collect();
        if gated.is_empty() {
            return None;
        }
        let mut filed = Self::new(&state.population, gated, &World::of(state, rules));
        filed.basis = Basis::of(genesis, state);
        Some(filed)
    }

    /// Whether the index was built for these rules, this movement mode and
    /// these routes, so that reconciling it answers the world.
    pub(crate) fn answers(&self, genesis: &Arc<Genesis>, state: &State) -> bool {
        let b = &self.basis;
        std::ptr::eq(b.genesis.as_ptr(), Arc::as_ptr(genesis))
            && b.immediate == state.movement.immediate()
            && b.routes.len() == state.sites.len()
            && state.sites.iter().all(|(id, s)| b.routes.get(id) == Some(&s.routes))
    }

    /// Files again every group whose body is not the one it was filed from,
    /// or that is new or gone, then the residents of every site whose
    /// conditions changed since the index last saw them.
    pub(crate) fn reconcile(&mut self, state: &State, rules: &Rules) {
        let world = World::of(state, rules);
        let population = &state.population;
        let same = |first: &Id, e: &Arc<Entity>| {
            self.filings.get(first).is_some_and(|f| std::ptr::eq(f.body.as_ptr(), Arc::as_ptr(e)))
        };
        let mut firsts: Vec<Id> = population.groups.iter().filter(|(f, g)| !same(f, &g.entity)).map(|(f, _)| *f).collect();
        firsts.extend(self.filings.keys().filter(|f| !population.groups.contains_key(f)));
        #[cfg(debug_assertions)]
        for (first, g) in population.groups.iter().filter(|(f, g)| same(f, &g.entity)) {
            let mut candidates = self.untraited.clone();
            for bits in g.entity.traits.iter().filter_map(|t| self.by_trait.get(t)) {
                candidates.union(bits);
            }
            let whole = passing(&self.gates, &candidates, &mut Reading::new(&g.entity, &world));
            debug_assert_eq!(self.filings[first].passed, whole, "group {first}'s kept filing differs from a whole reading");
        }
        self.refile(population, firsts, &BTreeMap::new(), &world, false);
        let moved: Vec<Id> = state
            .sites
            .iter()
            .filter(|(id, s)| self.basis.conditions.get(id) != Some(&s.conditions))
            .map(|(id, _)| *id)
            .collect();
        for site in moved {
            self.touch_site(population, site, &world);
        }
        self.settle(state);
    }

    /// Notes each site's conditions as the world now stands, so the next
    /// reconciling files again only the residents of sites changed since.
    pub(crate) fn settle(&mut self, state: &State) {
        for (id, s) in &state.sites {
            if self.basis.conditions.get(id) != Some(&s.conditions) {
                self.basis.conditions.insert(*id, s.conditions.clone());
            }
        }
        self.basis.conditions.retain(|id, _| state.sites.contains_key(id));
    }
}

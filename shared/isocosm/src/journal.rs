// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! What an advance changed, kept so a refused advance can put the world back
//! as it was (ruling 237). An advance used to run on a copy of the whole
//! simulation and keep the copy only on success; now it runs on the world
//! itself and notes, before each first change, what stood there.

use crate::{
    population::{Cohort, Population},
    schema::*,
    simulation::State,
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug)]
pub(crate) struct Journal {
    tick: Tick,
    next_action: u64,
    next_id: Id,
    notes: usize,
    /// Each stored group as it stood before its first change, by first
    /// identity; `None` where no group started.
    groups: BTreeMap<Id, Option<Cohort>>,
    sites: BTreeMap<Id, Site>,
    /// Whether each relation held before its first change.
    relations: BTreeMap<Relation, bool>,
    polities: BTreeSet<Id>,
    record: Option<hagiograph::Record<Key, Id>>,
    events: BTreeSet<Key>,
    /// Debug builds keep the whole world too, to check each rollback.
    #[cfg(debug_assertions)]
    before: Box<State>,
}

impl Journal {
    pub(crate) fn new(state: &State) -> Self {
        Self {
            tick: state.tick,
            next_action: state.next_action,
            next_id: state.population.next_id,
            notes: state.notes.len(),
            groups: BTreeMap::new(),
            sites: BTreeMap::new(),
            relations: BTreeMap::new(),
            polities: BTreeSet::new(),
            record: None,
            events: BTreeSet::new(),
            #[cfg(debug_assertions)]
            before: Box::new(state.clone()),
        }
    }

    pub(crate) fn group(&mut self, first: Id, was: Option<&Cohort>) {
        self.groups.entry(first).or_insert_with(|| was.cloned());
    }
    pub(crate) fn groups(&mut self, population: &Population, firsts: &[Id]) {
        for &first in firsts {
            self.group(first, population.groups.get(&first));
        }
    }
    pub(crate) fn site(&mut self, id: Id, was: &Site) {
        self.sites.entry(id).or_insert_with(|| was.clone());
    }
    pub(crate) fn relation(&mut self, relation: &Relation, held: bool) {
        self.relations.entry(relation.clone()).or_insert(held);
    }
    pub(crate) fn polity(&mut self, id: Id) {
        self.polities.insert(id);
    }
    pub(crate) fn record(&mut self, was: &hagiograph::Record<Key, Id>) {
        self.record.get_or_insert_with(|| was.clone());
    }
    pub(crate) fn event(&mut self, id: &Key) {
        self.events.insert(id.clone());
    }

    /// Puts back everything the advance changed.
    pub(crate) fn rollback(self, s: &mut State) {
        s.tick = self.tick;
        s.next_action = self.next_action;
        s.population.next_id = self.next_id;
        for (first, was) in self.groups {
            match was {
                Some(group) => s.population.groups.insert(first, group),
                None => s.population.groups.remove(&first),
            };
        }
        s.sites.extend(self.sites);
        for (relation, held) in self.relations {
            if held {
                s.relations.insert(relation);
            } else {
                s.relations.remove(&relation);
            }
        }
        for id in self.polities {
            s.polities.remove(&id);
        }
        if let Some(record) = self.record {
            s.record = record;
        }
        s.notes.truncate(self.notes);
        for id in self.events {
            s.events.remove(&id);
            s.reach.arrivals.remove(&id);
        }
        #[cfg(debug_assertions)]
        assert!(*s == *self.before, "a refused advance was not put back");
    }
}

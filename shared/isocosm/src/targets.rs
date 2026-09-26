// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Where a scheduled act with a target finds it: the first member in
//! identity order the process's selector and requirements accept, one per
//! stored group, as it always was. During an advance the groups are filed by
//! place and lineage, so a selector that names either looks only where it
//! could match instead of across the whole population (ruling 237).

use crate::{population::Population, rules::*, schema::*, simulation::Simulation};
use std::collections::{BTreeMap, BTreeSet};

/// Stored groups by their first identity, filed by place and lineage.
#[derive(Clone, Debug, Default)]
pub(crate) struct Targets {
    filed: BTreeMap<(Id, Key), BTreeSet<Id>>,
    at: BTreeMap<Id, (Id, Key)>,
}

impl Targets {
    pub(crate) fn new(population: &Population) -> Self {
        let mut t = Self::default();
        t.touch(
            population,
            population.groups.keys().copied().collect::<Vec<_>>(),
        );
        t
    }

    /// Files again every group that may start at one of `firsts`, after the
    /// population changed there.
    pub(crate) fn touch(&mut self, population: &Population, firsts: impl IntoIterator<Item = Id>) {
        for first in firsts {
            if let Some(key) = self.at.remove(&first)
                && let Some(set) = self.filed.get_mut(&key)
            {
                set.remove(&first);
                if set.is_empty() {
                    self.filed.remove(&key);
                }
            }
            if let Some(group) = population.groups.get(&first) {
                let key = (group.entity.place, group.entity.lineage.clone());
                self.filed.entry(key.clone()).or_default().insert(first);
                self.at.insert(first, key);
            }
        }
    }

    /// The filed groups a selector could accept from `place`, each list in
    /// identity order.
    fn lists<'a>(
        &'a self,
        selector: &'a Target,
        place: Id,
    ) -> Box<dyn Iterator<Item = &'a BTreeSet<Id>> + 'a> {
        let lineage = |key: &(Id, Key)| {
            selector
                .lineage
                .as_ref()
                .is_none_or(|lineage| *lineage == key.1)
        };
        if selector.same_place {
            let at = (place, Key::new())..;
            let here = self.filed.range(at).take_while(move |(k, _)| k.0 == place);
            Box::new(here.filter(move |(k, _)| lineage(k)).map(|(_, v)| v))
        } else {
            Box::new(
                self.filed
                    .iter()
                    .filter(move |(k, _)| lineage(k))
                    .map(|(_, v)| v),
            )
        }
    }
}

impl Simulation {
    pub(crate) fn choose_target(&self, actor: Id, p: &Process) -> Option<Id> {
        let selector = p.target.as_ref()?;
        let entity = self.state.population.get(actor)?;
        // Reject an ineligible actor before searching the population for food.
        // This is only a read shortcut; apply still checks every requirement.
        if p.requires.iter().any(|q| {
            matches!(q, Query::Trait {who:Binding::Actor,key}
            if !entity.traits.contains(key))
        }) {
            return None;
        }
        let place = entity.place;
        let groups = &self.state.population.groups;
        // A group offers its first member, or its second when the first is
        // the actor; identities never reach the top of their range, so the
        // second always has a number.
        let offered = |first: Id| -> Option<Id> {
            let count = groups.get(&first)?.count;
            let candidate = if first == actor { first + 1 } else { first };
            let accepted = candidate < first + count
                && self.target_matches(actor, Some(candidate), p)
                && p.requires
                    .iter()
                    .all(|q| self.query(actor, Some(candidate), place, q).is_ok());
            accepted.then_some(candidate)
        };
        let everywhere = || groups.keys().find_map(|&first| offered(first));
        match &self.targets {
            // Each list is in identity order, so its first accepted member is
            // its least; groups outside the lists fail the selector anyway.
            Some(t) => {
                let found = t
                    .lists(selector, place)
                    .filter_map(|list| list.iter().find_map(|&first| offered(first)))
                    .min();
                debug_assert_eq!(found, everywhere(), "the filed search missed a group");
                found
            },
            None => everywhere(),
        }
    }
}

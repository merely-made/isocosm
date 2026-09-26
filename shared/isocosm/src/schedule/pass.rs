// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A pass visits the groups stored when it began. An act that splits one
//! leaves pieces, so the pass keeps each split group's span and still
//! visits it whole, as a scan over the groups it began with would.

use crate::{population::Population, schema::*};
use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub(crate) struct Pass {
    /// Identities from here on were born during the pass.
    pub(super) bound: Id,
    split: BTreeMap<Id, u64>,
    /// Debug builds keep every span, to check each one the pass reads.
    #[cfg(debug_assertions)]
    began: BTreeMap<Id, u64>,
}

impl Pass {
    pub(super) fn new(population: &Population) -> Self {
        Self {
            bound: population.next_id,
            split: BTreeMap::new(),
            #[cfg(debug_assertions)]
            began: population
                .groups
                .iter()
                .map(|(&first, g)| (first, g.count))
                .collect(),
        }
    }

    fn span(&self, id: Id) -> Option<(Id, u64)> {
        let (&first, &count) = self.split.range(..=id).next_back()?;
        (id < first + count).then_some((first, count))
    }

    /// Keeps the span of the group holding `id`, before an act lifts `id`
    /// out of it.
    pub(crate) fn lifting(&mut self, population: &Population, id: Id) {
        if id >= self.bound || self.span(id).is_some() {
            return;
        }
        if let Some((&first, group)) = population.groups.range(..=id).next_back()
            && id < first + group.count
            && group.count > 1
        {
            self.split.insert(first, group.count);
        }
    }

    /// The span, when the pass began, of the group now starting at `first`.
    pub(super) fn origin(&self, population: &Population, first: Id) -> (Id, u64) {
        let span = self
            .span(first)
            .unwrap_or_else(|| (first, population.groups[&first].count));
        #[cfg(debug_assertions)]
        {
            let began = self.began.range(..=first).next_back();
            let began = began.map(|(&f, &c)| (f, c));
            debug_assert_eq!(Some(span), began, "the pass lost a group's span");
        }
        span
    }
}

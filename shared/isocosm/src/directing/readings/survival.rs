// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! What a player sees is a mode (rulings 180, 184, 691): creative shows the
//! truth and edits nothing; survival filters it through what the critter
//! knows and the reach field, until senses refine the filter.

use super::Suggestion;
use crate::{schema::*, simulation::Simulation};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mode {
    Survival,
    Creative,
}

/// What a mode shows: sites, living groups by first identity and count,
/// and events.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct View {
    pub sites: BTreeSet<Id>,
    pub things: BTreeMap<Id, u64>,
    pub events: BTreeSet<Key>,
}

/// What `critter`'s player sees in `mode`.
pub fn view(sim: &Simulation, critter: Id, mode: Mode) -> View {
    let s = sim.state();
    let living = s
        .population
        .groups
        .iter()
        .filter(|(_, g)| g.entity.alive && !crate::directing::is_participant(&g.entity));
    if mode == Mode::Creative {
        return View {
            sites: s.sites.keys().copied().collect(),
            things: living.map(|(f, g)| (*f, g.count)).collect(),
            events: s.events.keys().cloned().collect(),
        };
    }
    let Some(e) = s.population.get(critter) else {
        return View::default();
    };
    let mut v = View::default();
    v.sites.insert(e.place);
    v.sites.extend(e.visits.iter().map(|visit| visit.place));
    let mut subjects = BTreeSet::new();
    for (key, event) in &s.events {
        if sim.knows(critter, key).unwrap_or(false) {
            v.events.insert(key.clone());
            v.sites.insert(event.place);
            subjects.insert(event.subject);
        }
    }
    for (first, g) in living {
        let here = g.entity.place == e.place;
        let told = subjects.range(*first..*first + g.count).next().is_some();
        if here || told {
            v.things.insert(*first, g.count);
        }
    }
    v
}

impl View {
    /// Whether the view shows member `id`.
    pub fn shows(&self, id: Id) -> bool {
        let group = self.things.range(..=id).next_back();
        group.is_some_and(|(first, count)| id < first + count)
    }
    /// The suggestions whose target the view shows.
    pub fn filter(&self, suggestions: Vec<Suggestion>) -> Vec<Suggestion> {
        let shown = |s: &Suggestion| s.target.is_none_or(|t| self.shows(t));
        suggestions.into_iter().filter(shown).collect()
    }
}

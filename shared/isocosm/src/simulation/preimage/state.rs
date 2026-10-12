// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Kept v2 whole-state postcard layout; events retain numeric site preimages.
use super::{event::Events, *};
use crate::simulation;

#[derive(Serialize)]
pub(in crate::simulation) struct State<'a> {
    tick: Tick,
    next_action: u64,
    population: &'a population::Population,
    sites: &'a BTreeMap<Id, Site>,
    lineages: &'a BTreeMap<Key, Lineage>,
    locations: &'a BTreeMap<Id, Location>,
    polities: &'a BTreeMap<Id, Polity>,
    relations: &'a BTreeSet<Relation>,
    notes: &'a Vec<Note>,
    roots: &'a BTreeSet<Id>,
    released: &'a BTreeMap<Id, Tick>,
    record: &'a hagiograph::Record<Key, Id>,
    events: Events<'a>,
    reach: &'a crate::reach::Reach,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    edits: &'a Vec<Edited>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    nudges: &'a Vec<crate::directing::Nudge>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    laws: &'a BTreeMap<Key, crate::asserted::Law>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    agreements: &'a BTreeMap<Id, crate::social::Agreement>,
    #[serde(skip_serializing_if = "crate::asserted::Records::is_empty")]
    asserted: &'a crate::asserted::Records,
    #[serde(skip_serializing_if = "crate::attention::Windows::is_empty")]
    attention: &'a crate::attention::Windows,
    #[serde(skip_serializing_if = "Movement::immediate")]
    movement: &'a Movement,
}
impl<'a> From<&'a simulation::State> for State<'a> {
    fn from(value: &'a simulation::State) -> Self {
        let simulation::State { tick, next_action, population, sites, lineages,
            locations, polities, relations, notes, roots, released, record, events,
            reach, edits, nudges, laws, agreements, asserted, attention, movement, spreads: _ } = value;
        Self { tick:*tick, next_action:*next_action, population, sites, lineages,
            locations, polities, relations, notes, roots, released, record,
            events:Events(events), reach, edits, nudges, laws, agreements, asserted,
            attention, movement }
    }
}

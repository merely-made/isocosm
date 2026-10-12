// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Geography disclosed by explicit native viewers, without table discovery state.

use super::Reading;
use isocosm::{asserted::Place, schema::{Id, Route}};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Geography<'a> {
    pub places: BTreeMap<Id, Option<&'a Place>>,
    pub routes: Vec<(Id, &'a Route)>,
    /// Travel has no current resident site; it never borrows its departure.
    pub residents: BTreeMap<Id, Option<Id>>,
}

impl<'a> Reading<'a> {
    /// A viewer's residence and the site locations of events it natively knows.
    /// Journey events grant no site reach, and no table alias selects a viewer.
    pub fn geography(&self, viewers: &BTreeSet<Id>) -> isocosm::Result<Geography<'a>> {
        let state = self.state();
        let mut known = BTreeSet::new();
        let mut residents = BTreeMap::new();
        for &viewer in viewers {
            let entity = state.population.get(viewer).ok_or("unknown entity")?;
            let resident = entity.resident_site().filter(|id| state.sites.contains_key(id));
            known.extend(resident);
            residents.insert(viewer, resident);
            for (key, event) in &state.events {
                if let Some(site) = event.place.site() {
                    if state.sites.contains_key(&site) && self.knows(viewer, key)? {
                        known.insert(site);
                    }
                }
            }
        }
        let places = known.iter().map(|&id| (id, state.sites[&id].authored.as_ref())).collect();
        let mut routes = Vec::new();
        for &from in &known {
            for route in &state.sites[&from].routes {
                if known.contains(&route.to) { routes.push((from, route)); }
            }
        }
        Ok(Geography { places, routes, residents })
    }
}

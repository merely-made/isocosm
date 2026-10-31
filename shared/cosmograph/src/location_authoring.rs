// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Explicit Location vocabulary admitted by the native author/history door.
//! Creation grants no body knowing, party disclosure, control or geometry.

use crate::{Result,schema::Location,simulation::Simulation};

impl Simulation {
    pub(crate) fn author_location(&mut self, location: &Location) -> Result<String> {
        if location.sites.is_empty() || location.sites.len()>self.genesis.rules.limits.sites
            || location.sites.iter().any(|site|!self.state.sites.contains_key(site)) {
            return Err("Location needs explicitly named existing native sites".into());
        }
        if location.claims.iter().any(|polity|!self.state.polities.contains_key(polity))
            || location.parent.is_some_and(|parent|!self.state.locations.contains_key(&parent)) {
            return Err("Location names an absent native polity or parent".into());
        }
        let mut id=self.state.population.next_id;
        while self.state.sites.contains_key(&id) || self.state.polities.contains_key(&id)
            || self.state.locations.contains_key(&id) {
            id=id.checked_add(1).ok_or("Location identity exhausted")?;
        }
        let next=id.checked_add(1).ok_or("Location identity exhausted")?;
        // The supplied ratio and claim set retain their native meaning;
        // no relative scale, region boundaries or undisclosed sites are inferred.
        self.state.locations.insert(id,location.clone());
        self.state.population.next_id=next;
        Ok(format!("location:{id}"))
    }
}

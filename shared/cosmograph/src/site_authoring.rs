// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Authored place attachment admitted and replayed by the native Session.

use crate::{Result, schema::{Id, Site}, asserted::Place, simulation::Simulation};
use serde::{Deserialize, Serialize};

/// Explicit site identity or supplied native geometry, never a table cell conversion.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Attachment {
    Existing(Id),
    New { site: Id, geometry: Site },
}

impl Simulation {
    pub(crate) fn attach_site(&mut self, place: &Place, attachment: &Attachment) -> Result<String> {
        if place.key.trim().is_empty() { return Err("place attachment has no key".into()); }
        let id = match attachment { Attachment::Existing(id) => *id,
            Attachment::New { site, .. } => *site };
        let aliases: Vec<_> = self.state.sites.iter().filter(|(_, s)|
            s.authored.as_ref().is_some_and(|p| p.key == place.key)).collect();
        if aliases.len() > 1 || aliases.first().is_some_and(|(known, _)| **known != id) {
            return Err("ambiguous authored place attachment".into());
        }
        let mut sites = self.state.sites.clone();
        let fresh = match attachment {
            Attachment::Existing(_) => {
                let site = sites.get_mut(&id).ok_or("attachment names an unknown native site")?;
                if site.authored.as_ref().is_some_and(|was| was != place) {
                    return Err("native site already has another authored place".into());
                }
                site.authored = Some(place.clone());
                None
            },
            Attachment::New { geometry, .. } => {
                if geometry.authored.is_some() || !geometry.accounts.is_empty()
                    || geometry.routes.iter().any(|route| route.authored.is_some())
                {
                    return Err("new site geometry carries authored identity or stocks".into());
                }
                let mut site = geometry.clone();
                site.authored = Some(place.clone());
                if let Some(was) = sites.get(&id) {
                    if was != &site { return Err("new site identity is already occupied".into()); }
                    None
                } else {
                    if self.state.population.get(id).is_some() || self.state.polities.contains_key(&id) {
                        return Err("new site identity names another native noun".into());
                    }
                    if sites.len() >= self.genesis.rules.limits.sites { return Err("site limit".into()); }
                    sites.insert(id, site);
                    Some(id.checked_add(1).ok_or("native site identity exhausted")?)
                }
            },
        };
        if self.genesis.world.footprint.is_none() || self.genesis.rules.skeleton.is_none() {
            return Err("native attachment requires a founded footprint and skeleton".into());
        }
        for site in sites.values() {
            if site.conditions.keys().any(|key| !self.genesis.rules.conditions.contains(key)) {
                return Err("attachment contains an undeclared native condition".into());
            }
            if site.routes.iter().any(|route| !sites.contains_key(&route.to)
                || route.travel == 0 || route.transmission > 1_000_000)
            {
                return Err("attachment contains an invalid native route".into());
            }
        }
        crate::map::validate_sites(&self.genesis, &sites)?;
        let view = crate::terrain::View::over(&self.genesis, &sites)?;
        use isometer_space::Atlas;
        view.lattice(id)?;
        self.state.sites = sites;
        if let Some(next) = fresh { self.state.population.next_id = self.state.population.next_id.max(next); }
        Ok(format!("site:{id}"))
    }
}

#[cfg(test)]
mod tests;

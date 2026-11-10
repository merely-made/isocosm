// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Where a round may start for a lineage: its range and the fringe one route
//! step beyond it (case 1211, lineage-rounds D5). A pure function of the
//! state, never kept.

use super::scopes::{Means, route_steps, sources};
use crate::{Result, schema::*, simulation::Simulation};
use std::collections::{BTreeMap, BTreeSet};

/// One possible start: the site, the body taken there, and whether that body
/// must first journey to the site (a fringe start).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StartAt {
    pub site: Id,
    pub critter: Id,
    pub journey: bool,
}

/// Every valid start for `lineage`: range sites first, then fringe sites,
/// each in site order. The range is where a living member resides; the
/// fringe is the sites one route step from it, outside it, that a member can
/// walk to: a body that cannot move starts where it is. A range
/// site takes its lowest-id playable member; a fringe site the lowest-id
/// playable member at the first range site whose members can walk there.
/// Sites with no playable body, or none that can walk there, are left out.
pub fn starts(sim: &Simulation, lineage: &str) -> Vec<StartAt> {
    let mut range: BTreeMap<Id, Vec<(Id, bool)>> = BTreeMap::new();
    for (&first, g) in &sim.state().population.groups {
        let e = &g.entity;
        if e.alive && e.lineage == lineage
            && let Some(site) = e.resident_site()
        {
            range.entry(site).or_default().push((first, e.method == Method::Deliberative));
        }
    }
    let mut out: Vec<StartAt> = range
        .iter()
        .filter_map(|(&site, members)| {
            let (critter, _) = members.iter().find(|(_, playable)| *playable)?;
            Some(StartAt { site, critter: *critter, journey: false })
        })
        .collect();
    // A fringe site is one a playable member can walk to in a single route
    // step: a body that cannot move starts where it is.
    let fringe: BTreeSet<Id> = range
        .keys()
        .flat_map(|&s| route_steps(sim, s).into_iter().filter(|(_, n)| *n == 1).map(|(site, _)| site))
        .filter(|site| !range.contains_key(site))
        .collect();
    for site in fringe {
        let walkers = sources(sim, lineage, site).unwrap_or_default();
        let critter = walkers
            .iter()
            .filter(|from| from.hops == Some(1) && from.means.iter().any(|m| matches!(m, Means::Walk { .. })))
            .find_map(|from| range[&from.site].iter().find(|(_, playable)| *playable).map(|(id, _)| *id));
        if let Some(critter) = critter {
            out.push(StartAt { site, critter, journey: true });
        }
    }
    out
}

/// The start at `site` for `lineage`, or why there is none.
///
/// # Errors
/// Refuses, naming the lineage and the site, where the lineage has no living
/// member, has members there but none playable, or the site is outside its
/// range and fringe.
pub fn start_at(sim: &Simulation, lineage: &str, site: Id) -> Result<StartAt> {
    if let Some(found) = starts(sim, lineage).into_iter().find(|s| s.site == site) {
        return Ok(found);
    }
    let living = sim
        .state()
        .population
        .groups
        .values()
        .map(|g| &g.entity)
        .filter(|e| e.alive && e.lineage == lineage)
        .collect::<Vec<_>>();
    if living.is_empty() {
        return Err(format!("{lineage} has no living member to start {site} with"));
    }
    if living.iter().any(|e| e.resident_site() == Some(site)) {
        return Err(format!("{lineage} has members at site {site} but none is playable"));
    }
    let near = route_steps(sim, site);
    let reaches = living.iter().any(|e| e.resident_site().is_some_and(|s| near.get(&s) == Some(&1)));
    if reaches {
        return Err(format!("{lineage} has no playable member that can walk to site {site}"));
    }
    Err(format!("site {site} is outside the range of {lineage} and its fringe"))
}

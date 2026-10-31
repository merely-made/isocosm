// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

//! A place read off its residents for the GM (doctrine World 10): the sim's
//! reading, whose record carries the founding door's events (case 1032),
//! with the names it mentions resolved.

use isocosm::{Session, schema::Id, social::ladder::PlaceReading};
use isocosm_overlay::{OrdinaryAccess, Seat};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlaceView {
    pub reading: PlaceReading,
    /// The name of everyone and every group the reading mentions that has one.
    pub names: BTreeMap<Id, String>,
}

fn view(session: &Session, reading: PlaceReading) -> PlaceView {
    let mut people: BTreeSet<Id> = reading.residents.iter().copied().collect();
    people.extend(reading.households.iter().map(|h| h.with));
    people.extend(reading.record.iter().flat_map(|e| std::iter::once(e.subject).chain(e.object)));
    let mut names: BTreeMap<Id, String> = people.into_iter()
        .filter_map(|id| session.sim.name_of(id).map(|n| (id, n.to_owned()))).collect();
    names.extend(reading.groups.iter().filter_map(|g| g.name.clone().map(|n| (g.id, n))));
    let sites = &session.sim.state().sites;
    names.extend(reading.sites.iter().filter_map(|s| sites.get(s)?.authored.as_ref().map(|p| (*s, p.name.clone()))));
    PlaceView { reading, names }
}

/// Live place readings for a participant holding GM authority. Case 1030
/// makes GM an authority people hold and share, and a table may have none
/// or several; this table carries it as the DM seat, so the gate is that.
pub fn place_readings(session: &Session, access: OrdinaryAccess) -> Result<Vec<PlaceView>, String> {
    super::super::movement::current_access(session, access)?;
    if access.seat != Seat::Dm { return Err("live place readings require GM authority".into()); }
    let readings = session.sim.place_readings()?;
    Ok(readings.into_iter().map(|r| view(session, r)).collect())
}

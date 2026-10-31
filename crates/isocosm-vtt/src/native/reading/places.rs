// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

//! A place read off its residents for the GM (doctrine World 10): the sim's
//! reading with names resolved, and the history entries that made it, read
//! from the log that is the record (344).

use isocosm::{Command, Session, asserted::Assertion, schema::{Id, Tick},
    social::{Social, ladder::PlaceReading}};
use isocosm_overlay::{OrdinaryAccess, Seat};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// One history entry naming the place, its dwellings or its residents.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntryLine { pub order: u64, pub tick: Tick, pub what: String, pub outcome: String }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlaceView {
    pub reading: PlaceReading,
    /// The name of everyone the reading mentions who has one.
    pub names: BTreeMap<Id, String>,
    /// Why this place exists, as the history log has it: earliest first.
    pub entries: Vec<EntryLine>,
}

fn what(command: &Command) -> String {
    match command {
        Command::Arrive(a) => format!("arrived{}", a.character.as_ref().map(|c| format!(" as {c}")).unwrap_or_default()),
        Command::Assert(Assertion::Place(p)) => format!("asserted the place {}", p.name),
        Command::Assert(a) => format!("asserted {}", a.key()),
        Command::Location { .. } => "authored the location".into(),
        Command::Social(Social::OfferHome { .. }) => "offered a home".into(),
        _ => "acted".into(),
    }
}

fn view(session: &Session, reading: PlaceReading) -> PlaceView {
    let mut people: BTreeSet<Id> = reading.residents.iter().copied().collect();
    people.extend(reading.households.iter().map(|h| h.with));
    let names = people.into_iter()
        .filter_map(|id| session.sim.name_of(id).map(|n| (id, n.to_owned()))).collect();
    let named: BTreeSet<String> = reading.residents.iter().map(|id| format!("entity:{id}"))
        .chain(reading.sites.iter().map(|id| format!("site:{id}")))
        .chain(reading.claims.iter().map(|id| format!("polity:{id}")))
        .chain([format!("location:{}", reading.location)]).collect();
    let entries = session.entries.iter().filter(|e| named.contains(&e.outcome))
        .map(|e| EntryLine { order: e.order, tick: e.tick, what: what(&e.command), outcome: e.outcome.clone() })
        .collect();
    PlaceView { reading, names, entries }
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

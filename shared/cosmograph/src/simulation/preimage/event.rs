// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Kept native event witness layout, separate from v4 binary wire.
use crate::schema::{self, EventPlace, Id, Key, Tick};
use serde::{Serialize, Serializer, ser::SerializeMap};
use std::collections::BTreeMap;

#[derive(Clone, Copy)]
struct Place(EventPlace);
impl Serialize for Place {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self.0 { EventPlace::Site(id) => id.serialize(s),
            EventPlace::Journey(j) => j.serialize(s) }
    }
}

#[derive(Serialize)]
pub(crate) struct Event<'a> {
    id: &'a Key,
    tick: Tick,
    place: Place,
    subject: Id,
    #[serde(skip_serializing_if = "Option::is_none")]
    object: Option<Id>,
    process: &'a Key,
    cause: &'a Option<Key>,
    #[serde(skip_serializing_if = "Option::is_none")]
    agreement: Option<Id>,
    strength: u32,
    legend: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    authored: &'a Option<crate::asserted::HistoryLine>,
}
impl<'a> From<&'a schema::Event> for Event<'a> {
    fn from(value: &'a schema::Event) -> Self {
        let schema::Event { id, tick, place, subject, object, process, cause,
            agreement, strength, legend, authored } = value;
        Self { id, tick:*tick, place:Place(*place), subject:*subject, object:*object,
            process, cause, agreement:*agreement, strength:*strength, legend:*legend, authored }
    }
}
pub(crate) struct Events<'a>(pub &'a BTreeMap<Key, schema::Event>);
impl Serialize for Events<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut map = s.serialize_map(Some(self.0.len()))?;
        for (key, event) in self.0 { map.serialize_entry(key, &Event::from(event))?; }
        map.end()
    }
}

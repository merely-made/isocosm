// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use cosmograph::{Session, directing::readings::Mode};
use cosmograph_overlay::{EventRecord, EventTopic, Seat, Tick};
use crate::window::Projection;

/// The access cut selects visibility; a product Mode never selects it.
pub fn records(session: &Session, cut: &Projection) -> Result<Vec<EventRecord>, String> {
    let subject = cut.access.subject.map(|s| s.0);
    if session.sim.observer(cut.access.participant.0, subject).ok() != Some(subject) {
        return Err("event stream access names a stale subject".into());
    }
    let visibility = if cut.access.seat == Seat::Player { Mode::Survival } else { Mode::Creative };
    session.sim.attended_events_as(cut.access.participant.0, visibility, subject)?.into_iter()
        .filter(|event| cut.visible.events.contains(&event.id))
        .map(|event| Ok(EventRecord { tick: Tick(event.tick), topic: EventTopic(event.process.clone()),
            payload: serde_json::to_vec(&event).map_err(|e| e.to_string())? })).collect()
}

pub fn lines(session: &Session, cut: &Projection) -> Result<Vec<String>, String> {
    records(session, cut)?.into_iter().map(|entry| {
        let event: cosmograph::schema::Event = serde_json::from_slice(&entry.payload).map_err(|e| e.to_string())?;
        Ok(format!("{}: {} ({})", entry.tick.0, entry.topic.0, event.id))
    }).collect()
}

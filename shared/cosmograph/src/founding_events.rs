// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! What enters by the founding door leaves a world event where the world's
//! rules ask for it (case 1032), so every product reads why a place exists
//! from the record. Each event is keyed by its act and what it named, so a
//! held, repeated assertion leaves none twice.

use crate::{Result, history::Command, schema::*, simulation::Simulation};

impl Simulation {
    /// Records `outcome`, what `command` produced, as a founding event.
    pub(crate) fn founding_event(&mut self, command: &Command, outcome: String) -> Result<String> {
        if !self.genesis.rules.founding_events {
            return Ok(outcome);
        }
        let placeless = crate::directing::PLACELESS;
        let named = |kind: &str| outcome.strip_prefix(kind).and_then(|s| s.strip_prefix(':'))
            .and_then(|s| s.parse::<Id>().ok());
        let (act, subject, place) = match command {
            Command::Arrive(a) => ("arrive", named("entity"), Some(a.site)),
            Command::Assert(_) => match (named("entity"), named("polity"), named("site")) {
                (Some(e), _, _) => ("assert", Some(e), None),
                (_, Some(p), _) => ("assert", Some(p), None),
                (_, _, Some(s)) => ("assert", None, Some(s)),
                _ => return Ok(outcome),
            },
            Command::Location { location } => ("location", None, location.sites.first().copied()),
            _ => return Ok(outcome),
        };
        let id = format!("founding:{act}:{outcome}");
        if self.state.events.contains_key(&id) {
            return Ok(outcome);
        }
        if self.state.events.len() >= self.genesis.rules.limits.history {
            return Err("history limit".into());
        }
        let event = Event {
            id: id.clone(), tick: self.state.tick, place: EventPlace::Site(place.unwrap_or(placeless)),
            subject: subject.unwrap_or(placeless), object: None, process: format!("founding:{act}"),
            cause: None, agreement: None, strength: 0, legend: false, authored: None,
        };
        self.state.events.insert(id, event);
        Ok(outcome)
    }
}

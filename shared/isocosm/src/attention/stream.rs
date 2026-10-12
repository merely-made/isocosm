// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A player's locally derived retained event stream (204, 213, 815).
//! This reads existing native events and their reach. It creates no second
//! event ledger, subscriptions, replay entries, or transmission protocol.

use super::{PlayerAttention, Region, Target};
use crate::{Result, directing::readings::{self, Mode}, schema::{Entity, Event, Id},
    simulation::Simulation};

impl Simulation {
    /// Events touching the player's current union, plus retained record
    /// events whose native field reaches the body they actually play.
    /// Survival is the existing native knowledge cut, independent of profile.
    pub fn attended_events(&self, participant: Id, visibility: Mode) -> Result<Vec<Event>> {
        let attention = self.player_attention(participant)?;
        let subject = self.plays(participant);
        let known = readings::view(self, subject.unwrap_or(Id::MAX), visibility);
        let mut events: Vec<_> = self.state.events.values()
            .filter(|event| known.events.contains(&event.id))
            .filter(|event| self.attended(event, &attention)
                || subject.is_some_and(|id| self.event_reaches(id, event)))
            .cloned().collect();
        events.sort_by(|a, b| (a.tick, &a.id).cmp(&(b.tick, &b.id)));
        Ok(events)
    }

    fn attended(&self, event: &Event, attention: &PlayerAttention) -> bool {
        attention.played.iter().any(|id| party(event, *id))
            || attention.pinned.iter().chain(&attention.care).any(|target| self.touches(event, target))
            || attention.examined.iter().any(|region| event.place.site()
                .is_some_and(|site| self.in_region(site, region)))
    }

    fn in_region(&self, site: Id, region: &Region) -> bool {
        match region {
            Region::Site(id) => site == *id,
            Region::Location(id) => self.state.locations.get(id).is_some_and(|l| l.sites.contains(&site)),
        }
    }

    fn touches(&self, event: &Event, target: &Target) -> bool {
        let parties = [Some(event.subject), event.object];
        match target {
            Target::Entity(id) => party(event, *id),
            Target::Site(id) => event.place.site() == Some(*id),
            Target::Location(id) => event.place.site().is_some_and(|site|
                self.in_region(site, &Region::Location(*id))),
            Target::Lineage(key) => parties.into_iter().flatten().any(|id|
                self.state.population.get(id).is_some_and(|e| &e.lineage == key)),
            Target::Faction(id) => self.state.polities.get(id).is_some_and(|p|
                parties.into_iter().flatten().any(|id| p.constitution.members.contains(&id))),
            Target::Event(key) => &event.id == key || event.cause.as_ref() == Some(key),
        }
    }

    fn event_reaches(&self, subject: Id, event: &Event) -> bool {
        let Some(body) = self.state.population.get(subject) else { return false; };
        let field = &self.genesis.rules.field;
        let exposure = |body: &Entity| {
            body.resident_site().is_some_and(|site| self.state.reach.exposure(event, site,
                body.arrived, self.state.tick.saturating_add(1), field) > 0)
                || body.visits.iter().any(|v| self.state.reach.exposure(event, v.place,
                    v.from, v.until, field) > 0)
        };
        exposure(&body)
    }
}

fn party(event: &Event, id: Id) -> bool { event.subject == id || event.object == Some(id) }

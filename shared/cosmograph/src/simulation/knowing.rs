// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! What an entity knows and the notes that record it: native knowing
//! (the reach of an event's field, or a note kept) and the note budget.

use super::*;

impl Simulation {
    pub fn knows(&self, entity: Id, event: &str) -> Result<bool> {
        let e = self.state.population.get(entity).ok_or("unknown entity")?;
        self.known(entity, e, &[], event)
    }
    /// Whether member `entity`, in state `e`, knows `event`, counting notes
    /// an act has staged but not yet committed.
    pub(crate) fn known(
        &self,
        entity: Id,
        e: &Entity,
        staged: &[Note],
        event: &str,
    ) -> Result<bool> {
        self.known_in(entity,e,&self.state.notes,staged,event)
    }
    /// Same native knowing, with the pass's retained note prefix when requested.
    pub(crate) fn known_in(&self, entity: Id, e: &Entity, notes: &[Note], staged: &[Note], event: &str) -> Result<bool> {
        let event = self.state.events.get(event).ok_or("unknown event")?;
        let subject = format!("entity:{entity}");
        if notes.iter().chain(staged).any(|n| {
            n.core.subject == subject
                && n.core.object == event.id
                && (n.core.kind == "sim:discover"
                    || (event.place.site().is_none() && n.core.kind == "sim:act"))
        }) {
            return Ok(true);
        }
        Ok(self.exposed(entity, e, event))
    }
    /// The reach half of knowing: whether `event`'s field reached `entity`.
    pub(crate) fn exposed(&self, entity: Id, e: &Entity, event: &Event) -> bool {
        let field = &self.genesis.rules.field;
        let mut exposure = e.resident_site().map_or(0, |site| self.state.reach.exposure(
            event, site, e.arrived, self.state.tick.saturating_add(1), field,
        ));
        for visit in &e.visits {
            exposure = exposure.saturating_add(self.state.reach.exposure(
                event,
                visit.place,
                visit.from,
                visit.until,
                field,
            ));
        }
        crate::draw(
            self.genesis.dynamics_seed(),
            &format!("knowing:{}", event.id),
            &[entity],
        ) % 1_000_000
            < exposure.min(1_000_000)
    }
    pub fn learn(&mut self, entity: Id, event: &str) -> Result<bool> {
        if !self.knows(entity, event)? {
            return Ok(false);
        }
        if self.state.notes.iter().any(|n| {
            n.core.subject == format!("entity:{entity}")
                && n.core.object == event
                && n.core.kind == "sim:discover"
        }) {
            return Ok(true);
        }
        let mut candidate = self.clone();
        candidate.inspect(entity)?;
        candidate.add_note(
            entity,
            event.into(),
            "sim:discover",
            String::new(),
            None,
            event.into(),
        )?;
        self.state = candidate.state;
        Ok(true)
    }
    pub(crate) fn add_note(
        &mut self,
        subject: Id,
        object: Key,
        kind: &str,
        djot: String,
        expires: Option<Tick>,
        cause: Key,
    ) -> Result<()> {
        self.room(self.state.notes.len())?;
        let note = self.note(subject, object, kind, djot, expires, cause)?;
        self.state.notes.push(note);
        Ok(())
    }
    /// Whether the note budget holds one more beside the `held` already kept.
    pub(crate) fn room(&self, held: usize) -> Result<()> {
        if held >= self.genesis.rules.limits.notes {
            return Err("note budget exhausted".into());
        }
        Ok(())
    }
    /// A note as the world records it now.
    pub(crate) fn note(
        &self,
        subject: Id,
        object: Key,
        kind: &str,
        djot: String,
        expires: Option<Tick>,
        cause: Key,
    ) -> Result<Note> {
        if !self.genesis.rules.note_kinds.contains(kind) {
            return Err("unknown note kind".into());
        }
        Ok(Note {
            core: wing_impresa::Record {
                subject: format!("entity:{subject}"),
                object,
                kind: kind.into(),
                canon_revision: self.genesis.world.canon.revision,
                cause,
                tick: self.state.tick,
                stance: wing_impresa::Stance::Associate,
            },
            djot,
            extra: BTreeMap::new(),
            expires,
        })
    }
}

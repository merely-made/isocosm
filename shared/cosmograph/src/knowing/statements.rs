// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Explicit held statements (117, 871, 876), not live world truth or sensing.

use crate::{Result, asserted::Fact, schema::*, simulation::Simulation};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const SOURCE: &str = "know:statement-source";
pub const HELD: &str = "know:statement-held";
pub const NOTE_KINDS: [&str; 2] = [SOURCE, HELD];

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Statement {
    LocationIdentity { location: Id },
    LocationMembers { location: Id, sites: BTreeSet<Id> },
    LocationExtent { location: Id, sites: BTreeSet<Id> },
    Condition { site: Id, key: Key, value: i64 },
    Claim { site: Id, polity: Id, present: bool },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Source { pub note: Note, pub hash: Key, pub revision: Key }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Meaning { Observed, Reported { reporter: Id, held_note: Key } }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Held {
    pub id: Key, pub holder: Id, pub statement: Statement, pub source: Source, pub meaning: Meaning,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocationKnowledge {
    pub identity: Vec<Held>, pub members: Vec<Held>, pub extent: Vec<Held>,
}

impl Statement {
    /// Only names and declared vocabulary are checked. A statement may be wrong.
    pub fn validate(&self, sim: &Simulation) -> Result<()> {
        let site = |id| if sim.state().sites.contains_key(&id) { Ok(()) }
            else { Err("statement names an absent native site".to_string()) };
        match self {
            Self::LocationIdentity { location } | Self::LocationMembers { location, .. }
                | Self::LocationExtent { location, .. } => {
                if !sim.state().locations.contains_key(location) { return Err("statement names an absent native Location".into()); }
                if let Self::LocationMembers { sites, .. } | Self::LocationExtent { sites, .. } = self {
                    for id in sites { site(*id)?; }
                }
            },
            Self::Condition { site: id, key, .. } => {
                site(*id)?;
                if !sim.genesis().rules.conditions.contains(key) { return Err("statement condition is undeclared".into()); }
            },
            Self::Claim { site: id, polity, .. } => {
                site(*id)?;
                if !sim.state().polities.contains_key(polity) { return Err("statement names an absent native polity".into()); }
            },
        }
        Ok(())
    }

    /// Exact text for an authored selected-process Note; no producer is inferred.
    pub fn text(&self) -> Result<String> { serde_json::to_string(self).map_err(|e| e.to_string()) }

    /// The existing author admission door gives this source to one explicit body.
    pub fn fact(&self, holder: Id, key: Key) -> Result<Fact> {
        crate::validation::key(&key)?;
        Ok(Fact { about: format!("entity:{holder}"), key, kind: SOURCE.into(),
            text: self.text()?, tags: BTreeSet::new() })
    }
}

impl Held {
    pub fn validate(&self) -> Result<()> {
        let statement: Statement = serde_json::from_str(&self.source.note.djot).map_err(|e| e.to_string())?;
        if self.source.note.core.kind != SOURCE || self.statement != statement
            || self.source.hash != crate::digest(&self.source.note)
            || self.source.revision != revision(&self.source.note) || self.id.is_empty() {
            return Err("held statement differs from its exact admitted source".into());
        }
        Ok(())
    }
}

fn revision(note: &Note) -> Key {
    crate::digest(&(&note.core.canon_revision, note.core.tick, &note.core.cause))
}

/// Retained records establish identity separately from any claimed extent.
pub fn locations_of(records: &[Held]) -> BTreeMap<Id, LocationKnowledge> {
    let mut locations = BTreeMap::<Id, LocationKnowledge>::new();
    for record in records {
        if let Statement::LocationIdentity { location } = &record.statement {
            locations.entry(*location).or_default().identity.push(record.clone());
        }
    }
    for record in records {
        match &record.statement {
            Statement::LocationMembers { location, .. } => {
                if let Some(known) = locations.get_mut(location) { known.members.push(record.clone()); }
            },
            Statement::LocationExtent { location, .. } => {
                if let Some(known) = locations.get_mut(location) { known.extent.push(record.clone()); }
            },
            _ => {},
        }
    }
    locations
}

impl Simulation {
    fn holds_source(&self, observer: Id, note: &Note) -> Result<bool> {
        let actor = self.state.population.get(observer).ok_or("statement observer is absent")?;
        if note.core.subject == format!("entity:{observer}") { return Ok(true); }
        let Some(authored) = &actor.authored else { return Ok(false); };
        if note.core.subject != authored.key { return Ok(false); }
        let identities: u128 = self.state.population.groups.values().filter(|group|
            group.entity.authored.as_ref().is_some_and(|fill| fill.key == authored.key)).map(|group| u128::from(group.count)).sum();
        if identities != 1 { return Err("statement's admitted authored body identity is ambiguous".into()); }
        Ok(true)
    }

    pub fn statement_sources(&self, observer: Id) -> Result<Vec<(Key, Statement, Source)>> {
        let mut sources = Vec::new();
        for note in self.state.notes.iter().filter(|note| note.core.kind == SOURCE) {
            if !self.holds_source(observer, note)? { continue; }
            let statement: Statement = serde_json::from_str(&note.djot).map_err(|e| e.to_string())?;
            sources.push((note.core.cause.clone(), statement,
                Source { note: note.clone(), hash: crate::digest(note), revision: revision(note) }));
        }
        Ok(sources)
    }

    /// Identity is learned separately. Membership/extent never enumerate Locations.
    pub fn known_locations(&self, holder: Id) -> Result<BTreeMap<Id, LocationKnowledge>> {
        let records = self.held_statements(holder)?;
        Ok(locations_of(&records))
    }

    pub fn held_statements(&self, holder: Id) -> Result<Vec<Held>> {
        if self.state.population.get(holder).is_none() { return Err("statement holder is absent".into()); }
        self.state.notes.iter().filter(|note| note.core.kind == HELD
            && note.core.subject == format!("entity:{holder}")).map(|note| {
            let record: Held = serde_json::from_str(note.extra.get("statement")
                .ok_or("held statement has no typed record")?).map_err(|e| e.to_string())?;
            record.validate()?;
            if record.id != note.core.cause || record.holder != holder || record.source.hash != note.core.object {
                return Err("held statement differs from its native note identity".into());
            }
            Ok(record)
        }).collect()
    }

    pub fn held_statement(&self, holder: Id, id: &str) -> Result<Held> {
        let mut records = self.held_statements(holder)?.into_iter().filter(|r| r.id == id);
        let record = records.next().ok_or("body does not hold that statement")?;
        if records.next().is_some() { return Err("held statement identity is ambiguous".into()); }
        Ok(record)
    }

    pub(crate) fn observe_statement(&mut self, observer: Id, source_note: &str) -> Result<Key> {
        if self.state.population.get(observer).is_none() { return Err("statement observer is absent".into()); }
        let mut sources = self.state.notes.iter().filter(|note| note.core.cause == source_note && note.core.kind == SOURCE);
        let note = sources.next().ok_or("observer does not hold that admitted statement source")?.clone();
        if sources.next().is_some() { return Err("statement source identity is ambiguous".into()); }
        if !self.holds_source(observer, &note)? { return Err("observer does not hold that admitted statement source".into()); }
        let statement: Statement = serde_json::from_str(&note.djot).map_err(|e| e.to_string())?;
        statement.validate(self)?;
        let source = Source { hash: crate::digest(&note), revision: revision(&note), note };
        self.keep_statement(observer, statement, source, Meaning::Observed)
    }

    pub(crate) fn tell_statement(&mut self, reporter: Id, hearer: Id, held_note: &str) -> Result<Key> {
        if self.state.population.get(hearer).is_none() { return Err("statement hearer is absent".into()); }
        let held = self.held_statement(reporter, held_note)?;
        self.keep_statement(hearer, held.statement, held.source,
            Meaning::Reported { reporter, held_note: held_note.into() })
    }

    fn keep_statement(&mut self, holder: Id, statement: Statement, source: Source, meaning: Meaning) -> Result<Key> {
        self.room(self.state.notes.len())?;
        let id = format!("statement:{}", crate::digest(&(self.state.tick, self.state.notes.len(), holder,
            &statement, &source, &meaning)));
        if self.state.notes.iter().any(|note| note.core.cause == id) { return Err("statement identity collision".into()); }
        let record = Held { id: id.clone(), holder, statement, source, meaning };
        let mut note = self.note(holder, record.source.hash.clone(), HELD, String::new(), None, id.clone())?;
        note.extra.insert("statement".into(), serde_json::to_string(&record).map_err(|e| e.to_string())?);
        self.state.notes.push(note);
        Ok(id)
    }
}

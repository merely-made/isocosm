// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Authored content entering the world (ruling 757). Each assertion is an
//! entry in the history log, which is the record (344), and lands on the
//! native noun the rulings give it: an authored faction is a polity with no
//! members yet, carrying the attributes that fill its place (760); a fact
//! is a note (80), written whether or not the sim runs (248).

use crate::{Result, schema::*, simulation::Simulation};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// What a table or an author asserts.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Assertion {
    Faction(Faction),
    Fact(Fact),
}

/// An authored faction: its authored attributes, and its constitution's
/// governance and focus.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Faction {
    pub authored: Authored,
    pub governance: Key,
    pub focus: BTreeSet<Key>,
}

/// An authored fact: what it is about, its authored key, a note kind the
/// world's rules declare, its text and tags.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fact {
    pub about: Key,
    pub key: Key,
    pub kind: Key,
    pub text: String,
    pub tags: BTreeSet<Key>,
}

/// The cause an assertion's records cite.
fn cause(key: &str) -> Key {
    format!("assert:{key}")
}

impl Simulation {
    /// Asserts authored content. Asserting the same thing again changes
    /// nothing; asserting something else under its key is refused.
    pub(crate) fn assert(&mut self, assertion: &Assertion) -> Result<String> {
        match assertion {
            Assertion::Faction(f) => self.assert_faction(f).map(|id| format!("polity:{id}")),
            Assertion::Fact(f) => self.assert_fact(f).map(|()| format!("fact:{}", f.key)),
        }
    }

    /// The polity asserted under an authored key.
    pub fn authored_polity(&self, key: &str) -> Option<(Id, &Polity)> {
        let named = |p: &Polity| p.authored.as_ref().is_some_and(|a| a.key == key);
        let mut found = self.state.polities.iter().filter(|(_, p)| named(p));
        found.next().map(|(id, p)| (*id, p))
    }

    fn assert_faction(&mut self, f: &Faction) -> Result<Id> {
        let polity = Polity {
            constitution: Constitution {
                members: BTreeSet::new(),
                governance: f.governance.clone(),
                focus: f.focus.clone(),
                support_account: Key::new(),
                host: None,
                founded_by: cause(&f.authored.key),
            },
            accounts: BTreeMap::new(),
            ended_by: None,
            authored: Some(f.authored.clone()),
        };
        if let Some((id, held)) = self.authored_polity(&f.authored.key) {
            return match *held == polity {
                true => Ok(id),
                false => Err(format!("faction {} is asserted otherwise", f.authored.key)),
            };
        }
        // Polities share the entity id space; an authored one takes a fresh id.
        let id = self.state.population.next_id;
        self.state.population.next_id += 1;
        self.state.polities.insert(id, polity);
        Ok(id)
    }

    fn assert_fact(&mut self, f: &Fact) -> Result<()> {
        if !self.genesis.rules.note_kinds.contains(&f.kind) {
            return Err("unknown note kind".into());
        }
        let mut extra = BTreeMap::new();
        if !f.tags.is_empty() {
            extra.insert(
                "tags".into(),
                f.tags.iter().cloned().collect::<Vec<_>>().join(" "),
            );
        }
        let note = Note {
            core: wing_impresa::Record {
                subject: f.about.clone(),
                object: f.key.clone(),
                kind: f.kind.clone(),
                canon_revision: self.genesis.world.canon.revision,
                cause: cause(&f.key),
                tick: self.state.tick,
                stance: wing_impresa::Stance::Associate,
            },
            djot: f.text.clone(),
            extra,
            expires: None,
        };
        let held = self
            .state
            .notes
            .iter()
            .find(|n| n.core.cause == note.core.cause);
        if let Some(held) = held {
            let same = (&held.core.subject, &held.core.kind, &held.djot, &held.extra)
                == (&note.core.subject, &note.core.kind, &note.djot, &note.extra);
            return match same {
                true => Ok(()),
                false => Err(format!("fact {} is asserted otherwise", f.key)),
            };
        }
        self.room(self.state.notes.len())?;
        self.state.notes.push(note);
        Ok(())
    }
}

#[cfg(test)]
mod tests;

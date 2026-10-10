// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Authored content entering the world (ruling 757). Each assertion is an
//! entry in the history log, which is the record (344), and lands on its
//! native noun with the authored attributes filling its place: a faction a
//! polity with no members yet (760), a place a site, a route a site's
//! route, a character a placeless entity, a law an asserted rule record, a
//! history line an event (769), a fact a note (80), written whether or not
//! the sim runs (248). A campaign carries its assertions and folds them
//! into its world; a native session replays them when the sim switches on
//! (768).

use crate::{Result, schema::*, simulation::Simulation};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub use crate::schema::Authored;

mod apply;
mod fold;
pub use fold::{Asserted, Refused};

/// What a table or an author asserts.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Assertion {
    Faction(Faction),
    Fact(Fact),
    Place(Place),
    Route(Route),
    Character(Character),
    Law(Law),
    History(HistoryLine),
}

impl Assertion {
    /// The authored key it is asserted under.
    pub fn key(&self) -> &str {
        match self {
            Self::Faction(f) => &f.authored.key,
            Self::Fact(f) => &f.key,
            Self::Place(p) => &p.key,
            Self::Route(r) => &r.key,
            Self::Character(c) => &c.key,
            Self::Law(l) => &l.key,
            Self::History(h) => &h.key,
        }
    }
}

/// An authored faction: its authored attributes, and its constitution's
/// governance and focus.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Faction {
    pub authored: Authored,
    pub governance: Key,
    pub focus: BTreeSet<Key>,
}

/// An authored fact: what it is about, its authored key, a note kind the
/// world's rules declare, its text and tags.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Fact {
    pub about: Key,
    pub key: Key,
    pub kind: Key,
    pub text: String,
    pub tags: BTreeSet<Key>,
}

/// An authored place: a site's fill, with the map and the overmap
/// position its table gives it.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Place {
    pub key: Key,
    pub name: String,
    pub tags: BTreeSet<Key>,
    pub map: Option<Key>,
    pub position: Option<(i32, i32)>,
}

/// An authored route between two authored places, its weight the travel.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Route {
    pub key: Key,
    pub from: Key,
    pub to: Key,
    pub tags: BTreeSet<Key>,
    pub weight: u32,
}

/// An authored character: a placeless entity's fill, naming its faction
/// and place by authored key.
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(default)]
pub struct Character {
    pub key: Key,
    pub name: String,
    pub tags: BTreeSet<Key>,
    pub faction: Option<Key>,
    pub place: Option<Key>,
}

/// An authored law: a world-scope rule asserted as a record (41).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Law {
    pub key: Key,
    pub name: String,
    pub text: String,
    pub tags: BTreeSet<Key>,
    pub parameters: BTreeMap<Key, String>,
}

/// An authored history line: an event's fill, at the table's own time,
/// naming its participants and place by authored key.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct HistoryLine {
    pub key: Key,
    pub time: i64,
    pub kind: Key,
    pub text: String,
    pub participants: Vec<Key>,
    pub place: Option<Key>,
    pub tags: BTreeSet<Key>,
}

/// The cause an assertion's records cite.
fn cause(key: &str) -> Key {
    format!("assert:{key}")
}

/// Same content again is nothing new; other content under the key is refused.
fn held<T: PartialEq>(held: &T, new: &T, what: &str, key: &str) -> Result<bool> {
    match held == new {
        true => Ok(false),
        false => Err(format!("{what} {key} is asserted otherwise")),
    }
}

impl Simulation {
    /// Asserts authored content onto its native noun.
    pub(crate) fn assert(&mut self, assertion: &Assertion) -> Result<String> {
        let at = |kind: &str, id: Id| format!("{kind}:{id}");
        match assertion {
            Assertion::Faction(f) => self.assert_faction(f).map(|id| at("polity", id)),
            Assertion::Fact(f) => self.assert_fact(f).map(|()| format!("fact:{}", f.key)),
            Assertion::Place(p) => self.assert_place(p).map(|id| at("site", id)),
            Assertion::Route(r) => self.assert_route(r).map(|()| format!("route:{}", r.key)),
            Assertion::Character(c) => self.assert_character(c).map(|id| at("entity", id)),
            Assertion::Law(l) => self.assert_law(l).map(|()| format!("law:{}", l.key)),
            Assertion::History(h) => self.assert_line(h),
        }
    }
}

#[cfg(test)]
mod tests;

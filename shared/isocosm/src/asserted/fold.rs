// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A run of assertions folded into what they assert, by authored key, with
//! no world behind them: how a campaign with the sim off reads its world
//! (768). The same entries replay onto a native session when it switches on.

use super::*;

/// What a run of assertions holds.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Asserted {
    pub factions: BTreeMap<Key, Faction>,
    pub places: BTreeMap<Key, Place>,
    pub routes: BTreeMap<Key, Route>,
    pub characters: BTreeMap<Key, Character>,
    pub laws: BTreeMap<Key, Law>,
    /// In the table's time order, the order asserted breaking ties.
    pub history: Vec<HistoryLine>,
    pub facts: BTreeMap<Key, Fact>,
}

/// Why a fold refused an assertion.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Refused {
    /// It has no key.
    Keyless,
    /// Something else is asserted under its key.
    Otherwise(Key),
    /// A route names a place not yet asserted.
    Unplaced(Key),
}

type Folded = std::result::Result<bool, Refused>;

/// Holds `value` under `key` once: the same again is nothing new.
fn once<T: Clone + PartialEq>(map: &mut BTreeMap<Key, T>, key: &str, value: &T) -> Folded {
    match map.get(key) {
        Some(was) if was == value => Ok(false),
        Some(_) => Err(Refused::Otherwise(key.into())),
        None => {
            map.insert(key.into(), value.clone());
            Ok(true)
        },
    }
}

impl Asserted {
    /// Folds entries in order; the first refused one stops the fold.
    pub fn fold<'a>(
        entries: impl IntoIterator<Item = &'a Assertion>,
    ) -> std::result::Result<Self, Refused> {
        let mut folded = Self::default();
        for entry in entries {
            folded.apply(entry)?;
        }
        Ok(folded)
    }

    /// Folds one assertion in; true when it changed what is held.
    pub fn apply(&mut self, assertion: &Assertion) -> Folded {
        let key = assertion.key();
        if key.trim().is_empty() {
            return Err(Refused::Keyless);
        }
        match assertion {
            Assertion::Faction(f) => once(&mut self.factions, key, f),
            Assertion::Fact(f) => once(&mut self.facts, key, f),
            Assertion::Place(p) => once(&mut self.places, key, p),
            Assertion::Route(r) => {
                if !self.places.contains_key(&r.from) || !self.places.contains_key(&r.to) {
                    return Err(Refused::Unplaced(key.into()));
                }
                once(&mut self.routes, key, r)
            },
            Assertion::Character(c) => once(&mut self.characters, key, c),
            Assertion::Law(l) => once(&mut self.laws, key, l),
            Assertion::History(h) => {
                if let Some(was) = self.history.iter().find(|w| w.key == h.key) {
                    return match was == h {
                        true => Ok(false),
                        false => Err(Refused::Otherwise(key.into())),
                    };
                }
                let at = self.history.partition_point(|w| w.time <= h.time);
                self.history.insert(at, h.clone());
                Ok(true)
            },
        }
    }
}

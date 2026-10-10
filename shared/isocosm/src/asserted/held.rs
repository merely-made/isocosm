// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The table's records beside the native nouns (795): a map edit held on
//! its place, and a storylet applied or a pack forced, its assertions
//! landing on their own nouns all together or not at all.

use super::*;

/// What a world holds of the table's map edits, storylets and packs.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Records {
    pub maps: BTreeMap<Key, MapEdit>,
    pub storylets: BTreeMap<Key, Applied>,
    pub packs: BTreeMap<Key, Applied>,
}

impl Records {
    pub fn is_empty(&self) -> bool {
        self.maps.is_empty() && self.storylets.is_empty() && self.packs.is_empty()
    }
}

impl Simulation {
    /// The site a map names: an authored place, or a drawn site as
    /// `site:<id>`.
    fn map_site(&self, place: &str) -> Option<Id> {
        let drawn = place.strip_prefix("site:").and_then(|s| s.parse().ok());
        let drawn = drawn.filter(|id| self.state.sites.contains_key(id));
        drawn.or_else(|| self.authored_site(place).map(|(id, _)| id))
    }

    pub(super) fn assert_map(&mut self, m: &MapEdit) -> Result<()> {
        if let Some(was) = self.state.asserted.maps.get(&m.key) {
            return held(was, m, "map", &m.key).map(|_| ());
        }
        if self.map_site(&m.place).is_none() {
            return Err(format!("map {} names an unasserted place", m.key));
        }
        self.state.asserted.maps.insert(m.key.clone(), m.clone());
        Ok(())
    }

    /// Applies a storylet's or a forced pack's assertions, restoring the
    /// state if any is refused.
    pub(super) fn assert_group(&mut self, a: &Applied, pack: bool) -> Result<String> {
        let kind = if pack { "pack" } else { "storylet" };
        let held_now = match pack {
            true => self.state.asserted.packs.get(&a.key),
            false => self.state.asserted.storylets.get(&a.key),
        };
        if let Some(was) = held_now {
            return held(was, a, kind, &a.key).map(|_| format!("{kind}:{}", a.key));
        }
        let before = self.state.clone();
        for assertion in &a.asserts {
            if let Err(why) = self.assert(assertion) {
                self.state = before;
                return Err(why);
            }
        }
        let records = &mut self.state.asserted;
        let map = if pack { &mut records.packs } else { &mut records.storylets };
        map.insert(a.key.clone(), a.clone());
        Ok(format!("{kind}:{}", a.key))
    }
}

// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Native attention values and active-window contribution ownership (815).
//! Native history admits each lifecycle change before the shared union
//! drives roots and foreground. The core adapter borrows that same Session.

use crate::schema::{Id, Key, Tick};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

mod admission;
mod care;
mod stream;
pub mod spatial;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Change {
    Attach { participant: Id, window: WindowId, profile: Key, attention: WindowAttention },
    Replace { participant: Id, window: WindowId, profile: Key, attention: WindowAttention },
    Close { participant: Id, window: WindowId },
}

/// Issued by native admission; serialized values still require owner checks.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct WindowId(u64);

impl WindowId {
    pub fn id(self) -> u64 {
        self.0
    }
}

/// Resolved native namespaces; adapter handles are not interchangeable IDs.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Target {
    Entity(Id),
    Site(Id),
    Location(Id),
    Lineage(Key),
    Faction(Id),
    Event(Key),
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Region {
    Site(Id),
    Location(Id),
    /// The part of a site a camera shows (the execution plan's D32): its
    /// horizontal voxel bounds, `min` inclusive and `max` exclusive, as a
    /// terrain window's.
    Frame { site: Id, min: [i64; 2], max: [i64; 2] },
}

impl Region {
    /// Whether a pose at `at` in `site` lies in a frame.
    pub(crate) fn frames(&self, site: Id, at: [i64; 3]) -> bool {
        match self {
            Region::Frame { site: s, min, max } => *s == site
                && (0..2).all(|k| (min[k]..max[k]).contains(&at[2 * k])),
            _ => false,
        }
    }
}

/// A close-up region is local to one active window, never a far camera.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WindowAttention {
    pub pinned: BTreeSet<Target>,
    pub examined: Option<Region>,
}

/// The single effective player set, with every active close-up region (815).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerAttention {
    pub played: BTreeSet<Id>,
    pub pinned: BTreeSet<Target>,
    pub examined: BTreeSet<Region>,
    pub care: BTreeSet<Target>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Contribution {
    participant: Id,
    attention: WindowAttention,
    // Admission resolves a registered native profile. Care is derived from
    // current native facts rather than saved as caller-provided pin data.
    profile: Key,
}

/// Native-owned values for State, not an accepted host-side ledger.
/// The crate-private mutators require native validation before admission.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Windows {
    next: u64,
    active: BTreeMap<WindowId, Contribution>,
    issued: BTreeSet<WindowId>,
    root_cache: BTreeSet<Id>,
    released: BTreeMap<Id, Tick>,
}

impl Windows {
    pub fn is_empty(&self) -> bool {
        self.next == 0 && self.active.is_empty() && self.issued.is_empty()
            && self.root_cache.is_empty() && self.released.is_empty()
    }

    pub(crate) fn attach(
        &mut self,
        participant: Id,
        profile: Key,
        attention: WindowAttention,
    ) -> Result<WindowId, String> {
        let next = self.next.checked_add(1).ok_or("attention window identity exhausted")?;
        let window = WindowId(self.next);
        self.attach_at(participant, window, profile, attention)?;
        self.next = next;
        Ok(window)
    }

    fn attach_at(&mut self, participant: Id, window: WindowId, profile: Key,
        attention: WindowAttention) -> Result<(), String> {
        if self.issued.contains(&window) { return Err("attention window identity already issued".into()); }
        self.issued.insert(window);
        self.active.insert(window, Contribution { participant, attention, profile });
        Ok(())
    }

    pub fn buffered(&self, tick: Tick) -> BTreeSet<Id> {
        self.released.iter().filter(|(_, until)| **until > tick).map(|(id, _)| *id).collect()
    }

    /// Every camera frame an active window examines (D32).
    pub(crate) fn frames(&self) -> Vec<Region> {
        self.active.values().filter_map(|c| c.attention.examined.clone())
            .filter(|r| matches!(r, Region::Frame { .. })).collect()
    }

    pub(crate) fn foreground(&self, first: Id, count: u64) -> bool {
        self.root_cache.range(first..first + count).next().is_some()
    }

    pub(crate) fn replace(
        &mut self,
        participant: Id,
        window: WindowId,
        profile: Key,
        attention: WindowAttention,
    ) -> Result<bool, String> {
        let before = self.owned(participant, window)?;
        let next = Contribution { participant, attention, profile };
        if *before == next {
            return Ok(false);
        }
        self.active.insert(window, next);
        Ok(true)
    }

    pub(crate) fn close(&mut self, participant: Id, window: WindowId) -> Result<(), String> {
        self.owned(participant, window)?;
        self.active.remove(&window);
        Ok(())
    }

    pub(crate) fn resolve(&self, participant: Id, handle: u64) -> Result<WindowId, String> {
        let window = WindowId(handle);
        self.owned(participant, window)?;
        Ok(window)
    }

    fn owned(&self, participant: Id, window: WindowId) -> Result<&Contribution, String> {
        let contribution = self.active.get(&window).ok_or("unknown active attention window")?;
        if contribution.participant != participant {
            return Err("attention window belongs to another participant".into());
        }
        Ok(contribution)
    }

    /// Only the native reader supplies played identity and native profile
    /// derivation. A game cannot add played or care through WindowAttention.
    pub(crate) fn for_player(
        &self,
        participant: Id,
        played: impl IntoIterator<Item = Id>,
        mut native_care: impl FnMut(&str) -> BTreeSet<Target>,
    ) -> PlayerAttention {
        let mut attention = PlayerAttention {
            played: played.into_iter().collect(),
            ..Default::default()
        };
        for contribution in self.active.values().filter(|c| c.participant == participant) {
            attention.pinned.extend(contribution.attention.pinned.iter().cloned());
            attention.examined.extend(contribution.attention.examined.iter().cloned());
            attention.care.extend(native_care(&contribution.profile));
        }
        attention
    }
}

#[cfg(test)]
#[path = "attention/windows_tests.rs"]
mod tests;

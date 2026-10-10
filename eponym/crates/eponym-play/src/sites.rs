// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Eponym's meanings over the world's native sites (wing rulings 72 and 147:
//! a slot is the record's site). A slot keeps its address while what fills
//! it changes; an inherited site is an asserted fact about it (89), which
//! the world also notes natively. *Reading, not ruled:* underground slots
//! wait for rooms read from isometer's places, so a generated map has none.

use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

use isocosm::schema::Id;
use isocosm::simulation::Simulation;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Layer {
    Surface,
    Underground,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SlotId {
    pub site: Id,
    pub layer: Layer,
}

impl SlotId {
    pub const fn surface(site: Id) -> Self {
        Self { site, layer: Layer::Surface }
    }
    pub const fn underground(site: Id) -> Self {
        Self { site, layer: Layer::Underground }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct HistoryFactId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SiteKind {
    Wilds,
    Settlement,
    Ruin,
    Encounter,
    Dungeon,
}

impl SiteKind {
    pub fn name(self) -> &'static str {
        match self {
            Self::Wilds => "wilds",
            Self::Settlement => "settlement",
            Self::Ruin => "ruin",
            Self::Encounter => "encounter",
            Self::Dungeon => "dungeon",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SiteSource {
    Generated,
    Inherited(HistoryFactId),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Site {
    pub slot: SlotId,
    pub kind: SiteKind,
    pub source: SiteSource,
    pub parent: Option<SlotId>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorldMap {
    sites: BTreeMap<SlotId, Site>,
    routes: BTreeMap<SlotId, BTreeSet<SlotId>>,
}

impl WorldMap {
    /// The settlement is the least site, the ruin the farthest from it and
    /// the encounter the best joined of the rest; every other site is wilds.
    pub(crate) fn generate(sim: &Simulation) -> Self {
        let sites = &sim.state().sites;
        let neighbours = |id: Id| sites[&id].routes.iter().map(|r| r.to).collect::<BTreeSet<_>>();
        let ids: Vec<Id> = sites.keys().copied().collect();
        let settlement = ids[0];
        let hops = hops_from(&neighbours, settlement);
        let others = || ids.iter().copied().filter(|s| *s != settlement);
        let ruin = others().max_by_key(|s| (hops.get(s).copied().unwrap_or(0), *s)).unwrap_or(settlement);
        let encounter = others()
            .filter(|s| *s != ruin)
            .min_by_key(|s| (Reverse(neighbours(*s).len()), *s))
            .unwrap_or(settlement);
        let mut map = Self { sites: BTreeMap::new(), routes: BTreeMap::new() };
        for &id in &ids {
            let kind = match id {
                s if s == settlement => SiteKind::Settlement,
                s if s == ruin => SiteKind::Ruin,
                s if s == encounter => SiteKind::Encounter,
                _ => SiteKind::Wilds,
            };
            let slot = SlotId::surface(id);
            map.sites.insert(slot, Site { slot, kind, source: SiteSource::Generated, parent: None });
            map.routes.entry(slot).or_default();
        }
        for &id in &ids {
            for to in neighbours(id) {
                map.link(SlotId::surface(id), SlotId::surface(to));
            }
        }
        map
    }

    fn link(&mut self, left: SlotId, right: SlotId) {
        self.routes.entry(left).or_default().insert(right);
        self.routes.entry(right).or_default().insert(left);
    }

    pub fn sites(&self) -> impl Iterator<Item = &Site> {
        self.sites.values()
    }

    pub fn site(&self, slot: SlotId) -> Option<&Site> {
        self.sites.get(&slot)
    }

    pub fn neighbours(&self, slot: SlotId) -> Option<&BTreeSet<SlotId>> {
        self.routes.get(&slot)
    }

    pub fn slots_of_kind(&self, kind: SiteKind) -> impl Iterator<Item = SlotId> + '_ {
        self.sites.values().filter(move |s| s.kind == kind).map(|s| s.slot)
    }

    pub(crate) fn inherit(&mut self, slot: SlotId, kind: SiteKind, fact: HistoryFactId) -> bool {
        let Some(site) = self.sites.get_mut(&slot) else {
            return false;
        };
        site.kind = kind;
        site.source = SiteSource::Inherited(fact);
        true
    }

    pub fn route(&self, from: SlotId, to: SlotId) -> Option<Vec<SlotId>> {
        if !self.sites.contains_key(&from) || !self.sites.contains_key(&to) {
            return None;
        }
        let mut previous = BTreeMap::new();
        let mut seen = BTreeSet::from([from]);
        let mut queue = VecDeque::from([from]);
        while let Some(at) = queue.pop_front() {
            if at == to {
                let mut path = vec![to];
                while let Some(parent) = previous.get(path.last()?) {
                    path.push(*parent);
                }
                path.reverse();
                return Some(path);
            }
            for next in self.routes.get(&at).into_iter().flatten() {
                if seen.insert(*next) {
                    previous.insert(*next, at);
                    queue.push_back(*next);
                }
            }
        }
        None
    }
}

/// Hops from `from` to every site reached.
fn hops_from(neighbours: &impl Fn(Id) -> BTreeSet<Id>, from: Id) -> BTreeMap<Id, u32> {
    let mut hops = BTreeMap::from([(from, 0)]);
    let mut queue = VecDeque::from([from]);
    while let Some(at) = queue.pop_front() {
        let next = hops[&at] + 1;
        for n in neighbours(at) {
            if !hops.contains_key(&n) {
                hops.insert(n, next);
                queue.push_back(n);
            }
        }
    }
    hops
}

// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Items as the sim holds them (Eponym's world move, wing ruling 755): each
//! an inert body of its kind's lineage, food holding the matter a sophont
//! eats; carrying a relation from its carrier, a spent item one no longer
//! living. Where a lying item rests in the site and which part wears one
//! are the game's, as the sim never sees a member move within its site
//! (740). *Reading, not ruled:* a worn item is a carried one, its part the
//! game's.

use std::collections::BTreeMap;

use isocosm::schema::Id;
use isometer_core::PartId;
use serde::{Deserialize, Serialize};

use crate::founding::{CARRIES, DRESSING, FOOD, SCRAP};
use crate::identity::SubjectId;

/// An item, by its entity's id.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ItemId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ItemKind {
    Food,
    Dressing,
    Scrap,
}

impl ItemKind {
    pub const fn mass_mg(self) -> u32 {
        match self {
            Self::Food => 250,
            Self::Dressing => 100,
            Self::Scrap => 900,
        }
    }
    /// Its lineage in the sim.
    pub const fn lineage(self) -> &'static str {
        match self {
            Self::Food => FOOD,
            Self::Dressing => DRESSING,
            Self::Scrap => SCRAP,
        }
    }
    fn of(lineage: &str) -> Option<Self> {
        [Self::Food, Self::Dressing, Self::Scrap].into_iter().find(|k| k.lineage() == lineage)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ItemLocation {
    At([i32; 3]),
    Carried(SubjectId),
    Attached { subject: SubjectId, part: PartId },
    Consumed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Item {
    pub id: ItemId,
    pub kind: ItemKind,
    pub location: ItemLocation,
}

/// What the game keeps of items: where each lying one rests and the part
/// each worn one is on.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemPlaces {
    pub at: BTreeMap<ItemId, [i32; 3]>,
    pub worn: BTreeMap<ItemId, PartId>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Items {
    items: BTreeMap<ItemId, Item>,
}

impl Items {
    /// Every item the game placed, read from the sim.
    pub(crate) fn read(world: &crate::World, places: &ItemPlaces) -> Self {
        let sim = world.sim();
        let subject_of: BTreeMap<Id, SubjectId> = world.subjects().map(|(s, e)| (e, s)).collect();
        let carrier = |item: Id| {
            let rs = &sim.state().relations;
            rs.iter().find(|r| r.kind == CARRIES && r.object == item).and_then(|r| subject_of.get(&r.subject))
        };
        let mut items = BTreeMap::new();
        for (&id, &at) in &places.at {
            let Some(e) = sim.state().population.get(id.0) else { continue };
            let Some(kind) = ItemKind::of(&e.lineage) else { continue };
            let location = match (e.alive, carrier(id.0), places.worn.get(&id)) {
                (false, ..) => ItemLocation::Consumed,
                (true, Some(s), Some(part)) => ItemLocation::Attached { subject: *s, part: *part },
                (true, Some(s), None) => ItemLocation::Carried(*s),
                (true, None, _) => ItemLocation::At(at),
            };
            items.insert(id, Item { id, kind, location });
        }
        Self { items }
    }

    pub fn get(&self, id: ItemId) -> Option<&Item> {
        self.items.get(&id)
    }
    pub fn all(&self) -> impl Iterator<Item = &Item> {
        self.items.values()
    }
    pub fn at(&self, position: [i32; 3]) -> impl Iterator<Item = &Item> {
        self.items.values().filter(move |item| item.location == ItemLocation::At(position))
    }
    pub fn carried_by(&self, subject: SubjectId) -> impl Iterator<Item = &Item> {
        self.items.values().filter(move |item| {
            matches!(
                item.location,
                ItemLocation::Carried(owner) | ItemLocation::Attached { subject: owner, .. } if owner == subject
            )
        })
    }
    pub fn carried_mass_mg(&self, subject: SubjectId) -> u32 {
        self.carried_by(subject).map(|item| item.kind.mass_mg()).sum()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ItemError {
    Missing(ItemId),
    NotHere(ItemId),
    NotCarried(ItemId, SubjectId),
    AlreadyAttached(ItemId),
    WrongKind(ItemId, ItemKind),
    OverCapacity { capacity_mg: u32, attempted_mg: u32 },
}

// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Read native spatial facts through one owned window and native awareness.
//! These values are derived presentations, never another world or event log.

use super::{Region, WindowAttention};
use crate::{Result, schema::{Id, Key, Part, PartId, BodyDocument}, simulation::Simulation,
    directing::readings::{self, Mode, View}};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Far/overmap/minimap content deliberately has no individual body identity.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Crowd { pub site: Id, pub lineage: Key, pub count: u64 }

/// Geometry from an explicitly watched native site, at native identity grain.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VisibleBody {
    pub id: Id, pub lineage: Key, pub kingdom: Key, pub revision: u64,
    pub alive: bool,
    /// Functional tissue summaries admitted by the same local close-up as geometry.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub allocations: BTreeMap<PartId, PartAllocation>,
    pub document: BodyDocument, pub patch: Option<isometer_space::places::PlaceId>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PartAllocation {
    pub capacity: u32,
    pub cells: BTreeMap<Key, u32>,
}

/// What the creature skin reads from an entity (case 1049): each part's
/// matter ledger and its cells per function out of its capacity. A derived
/// presentation input, like the allocations it is read beside.
pub fn skin_parts(entity: &crate::schema::Entity) -> Vec<isometer_core::PartSkin> {
    let allocations = allocations(entity);
    entity
        .parts
        .iter()
        .map(|(&part, held)| {
            let allocation = allocations.get(&part);
            isometer_core::PartSkin {
                matter: held.matter.clone(),
                functions: allocation.map(|a| a.cells.clone()).unwrap_or_default(),
                capacity: allocation.map_or(0, |a| a.capacity),
                ..isometer_core::PartSkin::new(part)
            }
        })
        .collect()
}

pub fn allocations(entity: &crate::schema::Entity) -> BTreeMap<PartId, PartAllocation> {
    entity
        .living()
        .filter_map(|(id, p)| {
            let capacity = crate::anatomy::living_cells(entity.extent(id), p);
            (capacity > 0).then(|| {
                (
                    id,
                    PartAllocation {
                        capacity,
                        cells: p.cells.clone(),
                    },
                )
            })
        })
        .collect()
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegionReading { pub site: Id, pub played: Option<Id>, pub bodies: Vec<VisibleBody> }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InspectedBody { pub body: VisibleBody, pub parts: BTreeMap<PartId, Part> }

impl Simulation {
    fn spatial_window(&self, participant: Id, window: u64, mode: Mode, observer: Option<Id>)
        -> Result<(WindowAttention, View)> {
        let (_, _, held) = self.attention_window(participant, window)?;
        let subject = self.observer(participant, observer)?.unwrap_or(Id::MAX);
        Ok((held, readings::view(self, subject, mode)))
    }

    /// Camera distance never causes a native watch or materializes members.
    pub fn far_crowds(&self, participant: Id, window: u64, mode: Mode) -> Result<Vec<Crowd>> {
        self.far_crowds_as(participant, window, mode, None)
    }

    /// [`Self::far_crowds`] viewed as one of the participant's bodies (1065).
    pub fn far_crowds_as(&self, participant: Id, window: u64, mode: Mode, observer: Option<Id>)
        -> Result<Vec<Crowd>> {
        let (_, visible) = self.spatial_window(participant, window, mode, observer)?;
        let mut crowds = BTreeMap::<(Id, Key), u64>::new();
        for (first, count) in visible.things {
            let Some(entity) = self.state.population.get(first) else { continue; };
            let Some(site) = entity.resident_site() else { continue; };
            if !entity.alive || !visible.sites.contains(&site) { continue; }
            let total = crowds.entry((site, entity.lineage.clone())).or_default();
            *total = total.checked_add(count).ok_or("crowd multiplicity overflow")?;
        }
        Ok(crowds.into_iter().map(|((site, lineage), count)| Crowd { site, lineage, count }).collect())
    }

    /// A close-up reads only its own admitted region, even when another active
    /// window keeps a different region in the player's effective attention.
    pub fn region_reading(&self, participant: Id, window: u64, mode: Mode, site: Id)
        -> Result<RegionReading> {
        self.region_reading_as(participant, window, mode, site, None)
    }

    /// [`Self::region_reading`] viewed as one of the participant's bodies (1065).
    pub fn region_reading_as(&self, participant: Id, window: u64, mode: Mode, site: Id, observer: Option<Id>)
        -> Result<RegionReading> {
        let observer = self.observer(participant, observer)?;
        let (held, visible) = self.spatial_window(participant, window, mode, observer)?;
        if !visible.sites.contains(&site) { return Err("close-up site is outside native awareness".into()); }
        let watched = match held.examined {
            Some(Region::Site(id) | Region::Frame { site: id, .. }) => id == site,
            Some(Region::Location(id)) => {
                let actual = self.state.locations.get(&id).is_some_and(|l| l.sites.contains(&site));
                if mode == Mode::Creative { actual } else {
                    let observer = observer.ok_or("Location close-up has no actual played observer")?;
                    let known = self.known_locations(observer)?;
                    let location = known.get(&id).ok_or("Location identity has not been observed or told to this body")?;
                    actual && location.members.iter().chain(&location.extent).any(|record| match &record.statement {
                        crate::knowing::statements::Statement::LocationMembers { sites, .. }
                            | crate::knowing::statements::Statement::LocationExtent { sites, .. } => sites.contains(&site),
                        _ => false,
                    })
                }
            },
            None => false,
        };
        if !watched { return Err("close-up site is not this window's admitted region".into()); }
        let mut bodies = Vec::new();
        for (first, count) in visible.things {
            let Some(entity) = self.state.population.get(first) else { continue; };
            if entity.resident_site() != Some(site) { continue; }
            let Some(document) = &entity.body else { continue; };
            let end = first.checked_add(count).ok_or("native identity interval overflow")?;
            for id in first..end {
                bodies.push(VisibleBody { id, lineage: entity.lineage.clone(), kingdom: entity.kingdom.clone(),
                    revision: entity.body_revision, alive:entity.alive,
                    allocations: allocations(entity),
                    document: document.clone(), patch: entity.patch });
            }
        }
        Ok(RegionReading { site, played: observer.filter(|id|
            self.state.population.get(*id).is_some_and(|e| e.resident_site() == Some(site))), bodies })
    }

    /// An examiner may inspect only a body supplied by that same local
    /// close-up and native awareness. A raw pointable ID grants no access.
    pub fn inspected_body(&self, participant: Id, window: u64, mode: Mode, site: Id, id: Id)
        -> Result<InspectedBody> {
        self.inspected_body_as(participant, window, mode, site, id, None)
    }

    /// [`Self::inspected_body`] viewed as one of the participant's bodies (1065).
    pub fn inspected_body_as(&self, participant: Id, window: u64, mode: Mode, site: Id, id: Id,
        observer: Option<Id>) -> Result<InspectedBody> {
        let region = self.region_reading_as(participant, window, mode, site, observer)?;
        let body = region.bodies.into_iter().find(|b| b.id == id)
            .ok_or("examiner target is outside this window's visible close-up")?;
        let entity = self.state.population.get(id).ok_or("examiner target is no longer native")?;
        let parts = entity.parts.iter().map(|(id, p)| (*id, Part::clone(p))).collect();
        Ok(InspectedBody { body, parts })
    }
}

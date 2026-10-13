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
    pub document: BodyDocument, pub patch: Option<isometer_space::places::PlaceId>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegionReading { pub site: Id, pub played: Option<Id>, pub bodies: Vec<VisibleBody> }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InspectedBody { pub body: VisibleBody, pub parts: BTreeMap<PartId, Part> }

impl Simulation {
    fn spatial_window(&self, participant: Id, window: u64, mode: Mode)
        -> Result<(WindowAttention, View)> {
        let (_, _, held) = self.attention_window(participant, window)?;
        let subject = self.plays(participant).unwrap_or(Id::MAX);
        Ok((held, readings::view(self, subject, mode)))
    }

    /// Camera distance never causes a native watch or materializes members.
    pub fn far_crowds(&self, participant: Id, window: u64, mode: Mode) -> Result<Vec<Crowd>> {
        let (_, visible) = self.spatial_window(participant, window, mode)?;
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
        let (held, visible) = self.spatial_window(participant, window, mode)?;
        if !visible.sites.contains(&site) { return Err("close-up site is outside native awareness".into()); }
        let watched = match held.examined {
            Some(Region::Site(id)) => id == site,
            Some(Region::Location(id)) => {
                let actual = self.state.locations.get(&id).is_some_and(|l| l.sites.contains(&site));
                if mode == Mode::Creative { actual } else {
                    let observer = self.plays(participant).ok_or("Location close-up has no actual played observer")?;
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
                    document: document.clone(), patch: entity.patch });
            }
        }
        Ok(RegionReading { site, played: self.plays(participant).filter(|id|
            self.state.population.get(*id).is_some_and(|e| e.resident_site() == Some(site))), bodies })
    }

    /// An examiner may inspect only a body supplied by that same local
    /// close-up and native awareness. A raw pointable ID grants no access.
    pub fn inspected_body(&self, participant: Id, window: u64, mode: Mode, site: Id, id: Id)
        -> Result<InspectedBody> {
        let region = self.region_reading(participant, window, mode, site)?;
        let body = region.bodies.into_iter().find(|b| b.id == id)
            .ok_or("examiner target is outside this window's visible close-up")?;
        let entity = self.state.population.get(id).ok_or("examiner target is no longer native")?;
        Ok(InspectedBody { body, parts: entity.parts.clone() })
    }
}

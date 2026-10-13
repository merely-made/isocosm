// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Named terrain evidence from an actual game-owned native eye (741, 871).

use crate::{Result, schema::*, simulation::Simulation, operations::Window};
use isometer_space::places::{PlaceId, Places, Rules};
use serde::{Deserialize, Serialize};

pub const HELD: &str = "know:terrain-held";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NamedWindow { pub name: Key, pub site: Id, pub min: [i64; 3], pub max: [i64; 3] }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Eye {
    pub window: Window, pub places: Rules, pub eye: [i64; 3], pub range: u64, pub patch: PlaceId,
    pub body_revision: u64, pub resident_since: Tick, pub terrain_revision: Key,
    pub position_entry: Key, pub game_revision: Key,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Observation { pub named: NamedWindow, pub eye: Eye }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cell { pub at: [i64; 3], pub material: Key }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Source {
    pub observer: Id, pub tick: Tick, pub state: u64, pub observation: Observation,
    /// Omitted cells are unknown, including cells the existing place-scoped ray refuses.
    pub cells: Vec<Cell>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Held {
    pub id: Key, pub holder: Id, pub source: Source, pub hash: Key,
    pub meaning: super::statements::Meaning,
}

impl Simulation {
    pub fn held_terrain(&self, holder: Id) -> Result<Vec<Held>> {
        if self.state.population.get(holder).is_none() { return Err("terrain holder is absent".into()); }
        self.state.notes.iter().filter(|note| note.core.kind == HELD
            && note.core.subject == format!("entity:{holder}")).map(|note| {
            let record: Held = serde_json::from_str(note.extra.get("terrain")
                .ok_or("held terrain has no typed native record")?).map_err(|e| e.to_string())?;
            if record.id != note.core.cause || record.holder != holder || record.hash != note.core.object
                || record.hash != crate::digest(&record.source) {
                return Err("held terrain differs from its exact native source".into());
            }
            Ok(record)
        }).collect()
    }

    pub(crate) fn observe_terrain(&mut self, observer: Id, observation: &Observation) -> Result<Key> {
        let named = &observation.named;
        let eye = &observation.eye;
        crate::validation::key(&named.name)?;
        let actor = self.state.population.get(observer).ok_or("terrain observer is absent")?;
        if !actor.alive || actor.resident_site() != Some(named.site) || actor.patch != Some(eye.patch)
            || eye.patch.site != named.site || actor.body_revision != eye.body_revision
            || actor.arrived != eye.resident_since || self.terrain_revision() != eye.terrain_revision
            || eye.position_entry.is_empty() || eye.game_revision.is_empty() || eye.range > i64::MAX as u64
            || eye.window.depth < 0 || (0..3).any(|k| named.min[k] >= named.max[k]) {
            return Err("terrain observation has stale or incomplete native eye geometry".into());
        }
        let volume = self.volume(named.site, eye.window.min, eye.window.max, eye.window.depth)?;
        if named.min[0] < volume.min[0] || named.min[2] < volume.min[1] || named.max[0] > volume.max[0]
            || named.max[2] > volume.max[1] || named.min[1] < volume.datum || named.max[1] > volume.ceiling() {
            return Err("named terrain window exceeds its actual admitted volume".into());
        }
        let places = Places::derive(&volume, eye.places)?;
        let range = eye.range as i64;
        if places.places.values().any(|place| (0..3).any(|axis| place.bounds[0][axis].checked_sub(range).is_none()
            || place.bounds[1][axis].checked_add(range).is_none()))
            || !volume.holds(eye.eye[0], eye.eye[2]) || eye.eye[1] < volume.datum || eye.eye[1] >= volume.ceiling()
            || places.place_at(eye.eye) != Some(eye.patch)
            || !volume.material(eye.eye).is_some_and(|m| m == volume.materials.air || m == volume.materials.water) {
            return Err("terrain source eye is outside its admitted clear patch".into());
        }
        let mut cells = Vec::new();
        for x in named.min[0]..named.max[0] { for y in named.min[1]..named.max[1] { for z in named.min[2]..named.max[2] {
            let at = [x, y, z];
            let span = (0..3).map(|k| eye.eye[k].abs_diff(at[k])).max().unwrap_or(0);
            if span.checked_mul(span).and_then(|v| v.checked_mul(2)).is_none_or(|v| v > i64::MAX as u64) {
                return Err("terrain ray exceeds exact integer bounds".into());
            }
            if places.sees(&volume, eye.eye, at, eye.range) {
                let material = volume.material(at).and_then(|m| self.genesis.world.materials.get(m as usize))
                    .ok_or("observed terrain has no declared native material")?;
                cells.push(Cell { at, material: material.key.clone() });
            }
        } } }
        if cells.is_empty() { return Err("named terrain window has no actually observed cells".into()); }
        let source = Source { observer, tick: self.state.tick, state: self.state_hash(), observation: observation.clone(), cells };
        self.keep_terrain(observer, source, super::statements::Meaning::Observed)
    }

    pub(crate) fn tell_terrain(&mut self, reporter: Id, hearer: Id, held_note: &str) -> Result<Key> {
        if self.state.population.get(hearer).is_none() { return Err("terrain hearer is absent".into()); }
        let mut held = self.held_terrain(reporter)?.into_iter().filter(|record| record.id == held_note);
        let record = held.next().ok_or("reporter does not hold that terrain source")?;
        if held.next().is_some() { return Err("terrain source identity is ambiguous".into()); }
        self.keep_terrain(hearer, record.source,
            super::statements::Meaning::Reported { reporter, held_note: held_note.into() })
    }

    fn keep_terrain(&mut self, holder: Id, source: Source, meaning: super::statements::Meaning) -> Result<Key> {
        self.room(self.state.notes.len())?;
        let hash = crate::digest(&source);
        let id = format!("terrain:{}", crate::digest(&(self.state.tick, self.state.notes.len(), holder, &hash, &meaning)));
        if self.state.notes.iter().any(|note| note.core.cause == id) { return Err("terrain knowledge identity collision".into()); }
        let record = Held { id: id.clone(), holder, source, hash: hash.clone(), meaning };
        let mut note = self.note(holder, hash, HELD, String::new(), None, id.clone())?;
        note.extra.insert("terrain".into(), serde_json::to_string(&record).map_err(|e| e.to_string())?);
        self.state.notes.push(note);
        Ok(id)
    }
}

// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Read-only voxel sight from the clock's actual admitted eye (741, 847).
//! Its result only removes bodies from the caller's existing native cut.

use super::{clock, ModeState, PictureBody, SubjectId};
use cosmograph::{Session, schema::Id};
use isometer_space::{places::Places, volume::Volume};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SightConfig {
    pub version: u32,
    /// Base-cell range per explicit Eponym subject, including declared birth
    /// slots if their future observer range is supplied at authoring.
    pub ranges: BTreeMap<SubjectId, u64>,
}

impl SightConfig {
    pub fn validate(&self, mode: &ModeState, clock: &clock::State) -> Result<(), String> {
        if self.version != 1 || self.ranges.is_empty()
            || self.ranges.values().any(|range| *range > i64::MAX as u64) {
            return Err("Eponym sight needs explicit bounded subject ranges".into());
        }
        for subject in self.ranges.keys() {
            if !mode.bindings.iter().any(|binding| binding.subject == *subject)
                && !clock.profile.births.iter().any(|input| input.slots.iter().any(|slot| slot.subject == *subject)) {
                return Err("Eponym sight range has no admitted subject or declared birth slot".into());
            }
        }
        Ok(())
    }
}

pub(super) fn refine(config: &SightConfig, session: &Session, state: &ModeState,
    volume: &Volume, observer: Id, bodies: &[PictureBody]) -> Result<BTreeSet<Id>, String> {
    let clock = state.clock.as_ref().ok_or("Eponym sight has no actual clock eye")?;
    config.validate(state, clock)?;
    let binding = state.bindings.iter().find(|binding| binding.entity == observer)
        .ok_or("Eponym sight observer has no explicit subject binding")?;
    let body = session.sim.state().population.get(observer).ok_or("Eponym sight observer is absent")?;
    if !body.alive || body.resident_site() != Some(volume.site)
        || !bodies.iter().any(|body| body.entity == observer) {
        return Err("Eponym sight requires its actual living resident played body".into());
    }
    let physical = clock.current(session, binding)?;
    let range = *config.ranges.get(&binding.subject).ok_or("Eponym played subject has no authored sight range")?;
    let places = Places::derive(volume, physical.places)?;
    let radius = range as i64;
    if places.places.values().any(|place| (0..3).any(|axis|
        place.bounds[0][axis].checked_sub(radius).is_none()
            || place.bounds[1][axis].checked_add(radius).is_none())) {
        return Err("Eponym sight candidate bounds overflow".into());
    }
    cell(session, volume, &places, observer, physical.eye)?;
    let mut seen = BTreeSet::from([observer]);
    for body in bodies {
        let at = crate::walking::to_site(volume, body.pose.cell()
            .map_err(|error| format!("Eponym sight target pose: {error:?}"))?);
        cell(session, volume, &places, body.entity, at)?;
        let span = (0..3).map(|axis| physical.eye[axis].abs_diff(at[axis])).max().unwrap_or(0);
        if span.checked_mul(span).and_then(|n| n.checked_mul(2)).is_none_or(|n| n > i64::MAX as u64) {
            return Err("Eponym sight ray exceeds integer base-cell bounds".into());
        }
        if places.sees(volume, physical.eye, at, range) { seen.insert(body.entity); }
    }
    Ok(seen)
}

fn cell(session: &Session, volume: &Volume, places: &Places, id: Id, at: [i64; 3]) -> Result<(), String> {
    let body = session.sim.state().population.get(id).ok_or("Eponym sight target is absent")?;
    let patch = body.patch.ok_or("Eponym sight target has no admitted native patch")?;
    if body.resident_site() != Some(volume.site) || patch.site != volume.site
        || !volume.holds(at[0], at[2]) || at[1] < volume.datum || at[1] >= volume.ceiling()
        || !places.places.contains_key(&patch) || places.place_at(at) != Some(patch)
        || !volume.material(at).is_some_and(|material|
            material == volume.materials.air || material == volume.materials.water) {
        return Err("Eponym sight cell is outside the target's actual clear patch/window".into());
    }
    Ok(())
}

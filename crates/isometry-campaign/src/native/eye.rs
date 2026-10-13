// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

//! VTT sight refines native awareness using its saved physical eye, not a camera.

use super::GameBindings;
use isocosm::{Session,schema::Id,directing::readings::{Mode,View}};
use isometer_space::{places::{Places,PlaceId},volume::Volume};
use std::collections::{BTreeMap,BTreeSet};

fn cell(volume:&Volume,places:&Places,at:[i64;3],patch:Option<PlaceId>)->Result<bool,String> {
    if !volume.holds(at[0],at[2]) || at[1]<volume.datum || at[1]>=volume.ceiling() {return Ok(false);}
    let realized=places.place_at(at).ok_or("sight target has no realized native place")?;
    if patch.is_some_and(|patch|patch!=realized)
        || !volume.material(at).is_some_and(|material|material==volume.materials.air || material==volume.materials.water) {
        return Err("sight target differs from its admitted clear native patch".into());
    }
    Ok(true)
}

/// None is the original wholly unmarked VTT sight path. A marked survival
/// picture requires this actual observer's admitted current eye and range.
/// Events and remembered remote subjects stay native knowledge; this adds none.
pub fn refine(session:&Session,game:&GameBindings,observer:Id,mode:Mode,site:Id)
    ->Result<Option<View>,String> {
    if mode==Mode::Creative || !game.positions().values().any(|position|position.spatial.is_some()) {
        return Ok(None);
    }
    game.validate(session)?;
    let native=session.sim.state().population.get(observer).ok_or("sight observer is absent")?;
    if !native.alive || native.resident_site()!=Some(site) {return Err("sight observer is dead or outside this native site".into());}
    let (key,_)=game.characters().iter().find(|(_,character)|character.entity==observer)
        .ok_or("sight observer has no admitted native character")?;
    let position=game.positions().get(key).ok_or("sight observer has no supplied current position")?;
    let (eye,range)=position.current_eye(session)?.ok_or("marked VTT sight needs this observer's authored eye and range")?;
    let window=&position.placement.window;
    let volume=session.sim.volume(site,window.min,window.max,window.depth)?;
    let places=Places::derive(&volume,position.placement.places)?;
    let mut view=isocosm::directing::readings::view(&session.sim,observer,mode);
    let mut targets=BTreeMap::new();
    for (key,character) in game.characters() {
        if !view.shows(character.entity) || !session.sim.state().population.get(character.entity)
            .is_some_and(|body|body.resident_site()==Some(site)) {continue;}
        let binding=game.positions().get(key).ok_or("visible sight target has no admitted physical position")?;
        let at=binding.current_pose(session)?.at;
        if cell(&volume,&places,at,Some(binding.placement.patch))? {targets.insert(character.entity,at);}
    }
    let mut items=BTreeSet::new();
    for binding in game.item_records().iter().rev().filter_map(|record|record.binding.as_ref()) {
        if !items.insert(binding.entity) || !view.shows(binding.entity) || !session.sim.state().population.get(binding.entity)
            .is_some_and(|body|body.resident_site()==Some(site)) {continue;}
        let pose=super::items::current_item(session,game,binding.entity)?;
        let patch=session.sim.state().population.get(binding.entity).and_then(|body|body.patch);
        // Held items intentionally have no fixed patch. Their actual native
        // custody/mount and clearance provide the supplied physical point.
        if cell(&volume,&places,pose.at,patch)? {
            if targets.get(&binding.entity).is_some_and(|at|at!=&pose.at) {
                return Err("character and item sight points disagree on one native body".into());
            }
            targets.insert(binding.entity,pose.at);
        }
    }
    let mut seen=BTreeSet::from([observer]);
    for (&id,&at) in &targets {
        let span=(0..3).map(|axis|eye[axis].abs_diff(at[axis])).max().unwrap_or(0);
        if span.checked_mul(span).and_then(|value|value.checked_mul(2)).is_none_or(|value|value>i64::MAX as u64) {
            return Err("sight ray exceeds exact integer base-cell bounds".into());
        }
        if places.sees(&volume,eye,at,range) {seen.insert(id);}
    }
    let mut things=BTreeMap::new();
    for (&first,&count) in &view.things {
        let body=session.sim.state().population.get(first).ok_or("sight view names an absent member")?;
        if body.resident_site()!=Some(site) {things.insert(first,count);continue;}
        let end=first.checked_add(count).ok_or("sight identity interval overflows")?;
        for id in seen.range(first..end) {things.insert(*id,1);}
    }
    view.things=things;
    Ok(Some(view))
}

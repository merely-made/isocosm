// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Explicit physical edits; the old table edit grammar remains metadata.

use isocosm::{Command, Session, schema::{Id, Key}};
use isocosm_overlay::{EntityHandle, OrdinaryAccess, Seat, vtt::{Assertion, CellEdit, MapEdit}};
use isometer_space::{Edit, Op, edit::Shape};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

mod geometry;
mod records;
mod types;
pub use types::{PhysicalMapInput, GeometryProvider, CellGeometry, Operation, MaterialBinding,
    MaterialQuantity, Step, StepPreview, PhysicalMapPreview, PhysicalMapRecord};
pub use records::validate_records;

pub fn physical_source(session:&Session) -> Key {
    isocosm::digest(&(session.sim.genesis(),&session.branch,&session.entries,session.sim.state()))
}

fn check_access(session:&Session,game:&[u8],access:OrdinaryAccess,input:&PhysicalMapInput)
    -> Result<super::GameBindings,String> {
    super::movement::current_access(session,access)?;
    if access.seat != Seat::Dm || access.subject != Some(EntityHandle(input.actor))
        || session.sim.plays(access.participant.0) != Some(input.actor) {
        return Err("physical map authoring needs the actual DM's currently played body".into());
    }
    if input.version != 1 || input.expected_source != physical_source(session)
        || input.expected_game != isocosm::digest(&game)
        || input.expected_terrain != session.sim.terrain_revision() {
        return Err("physical map input names a changed native or product source".into());
    }
    let bindings=super::GameBindings::decode(game)?;
    bindings.validate(session)?;
    if bindings.physical_edits().iter().any(|record| record.input.metadata.key==input.metadata.key) {
        return Err("physical map edit key is already recorded".into());
    }
    let body=session.sim.state().population.get(input.actor).ok_or("physical edit body is absent")?;
    if !body.alive || body.body.is_none() || body.body_revision != input.body_revision
        || body.resident_site() != Some(input.geometry.site) || input.metadata.site.0 != input.geometry.site {
        return Err("physical map input names a stale body revision or another actual site".into());
    }
    let (character,_) = bindings.characters().iter().find(|(_,recipe)| recipe.entity == input.actor)
        .ok_or("physical edit body has no admitted character recipe")?;
    bindings.positions().get(character).ok_or("physical edit body has no supplied native position")?.current(session)?;
    geometry::check(input)?;
    Ok(bindings)
}

fn quoted(session:&Session,input:&PhysicalMapInput,index:usize) -> Result<StepPreview,String> {
    let command=geometry::command(session,input,index)?;
    let movements=geometry::movements(session,&command)?;
    let debits=session.prepare_owned_command(&command)?;
    let step=&input.steps[index];
    let supplied = step.terrain_before == session.sim.terrain_revision() && step.materials == movements
        && geometry::selections(&debits,&step.selections);
    Ok(StepPreview { index,command,terrain_before:session.sim.terrain_revision(),materials:movements,
        supplied,refusal:debits.refusal.clone(),debits })
}

/// Read-only native measurement; no missing quantity or owned selection is filled in.
pub fn prepare_physical_map(session:&Session,game:&[u8],access:OrdinaryAccess,input:&PhysicalMapInput)
    -> Result<PhysicalMapPreview,String> {
    let mut bindings=check_access(session,game,access,input)?;
    let mut candidate=session.clone();
    let mut preview=PhysicalMapPreview { steps:Vec::new(),complete:false,refusal:None };
    let mut edit_entries=Vec::new();
    for index in 0..input.steps.len() {
        let quote=quoted(&candidate,input,index)?;
        let ready=quote.supplied && quote.refusal.is_none();
        let command=quote.command.clone();
        preview.refusal=quote.refusal.clone();
        preview.steps.push(quote);
        if !ready { return Ok(preview); }
        if let Err(why)=candidate.command(Command::Owned { command:Box::new(command),
            selections:input.steps[index].selections.clone() }) {
            preview.refusal=Some(why);
            return Ok(preview);
        }
        edit_entries.push(candidate.entries.last().ok_or("physical edit quote has no accepted native entry")?.id.clone());
    }
    if let Err(why)=finish(&mut candidate,&mut bindings,access,input,edit_entries) {
        preview.refusal=Some(why);
        return Ok(preview);
    }
    preview.complete=true;
    Ok(preview)
}

/// Native commands and the explicit product marker accept as one candidate.
pub fn submit_physical_map(session:&mut Session,game:Vec<u8>,access:OrdinaryAccess,input:&PhysicalMapInput)
    -> Result<(String,Vec<u8>),String> {
    let mut bindings=check_access(session,&game,access,input)?;
    let mut candidate=session.clone();
    let mut edit_entries=Vec::new();
    for index in 0..input.steps.len() {
        let quote=quoted(&candidate,input,index)?;
        if !quote.supplied || quote.refusal.is_some() {
            return Err(format!("physical cell {index} needs its exact measured material quantities and native debit selections"));
        }
        candidate.command(Command::Owned { command:Box::new(quote.command),
            selections:input.steps[index].selections.clone() })?;
        edit_entries.push(candidate.entries.last().ok_or("physical edit has no accepted native entry")?.id.clone());
    }
    let next=finish(&mut candidate,&mut bindings,access,input,edit_entries)?;
    *session=candidate;
    Ok((format!("physically edited:{}",input.metadata.key),next))
}

fn finish(candidate:&mut Session,bindings:&mut super::GameBindings,access:OrdinaryAccess,
    input:&PhysicalMapInput,edit_entries:Vec<Key>) -> Result<Vec<u8>,String> {
    let window=&input.geometry.window;
    candidate.sim.volume(input.geometry.site,window.min,window.max,window.depth)?;
    super::submit_contract_assertions(candidate,&[Assertion::Edit(input.metadata.clone())])
        .map_err(|error|error.to_string())?;
    let metadata_entry=candidate.entries.last().ok_or("physical metadata has no native entry")?.id.clone();
    let record=PhysicalMapRecord { input:input.clone(),participant:access.participant.0,edit_entries,metadata_entry,
        terrain_after:candidate.sim.terrain_revision() };
    bindings.record_physical_edit(record);
    bindings.validate(candidate)?;
    bindings.encode()
}

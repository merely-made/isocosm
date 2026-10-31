// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;
use cosmograph::projects::ProjectId;
use std::collections::{BTreeMap,BTreeSet};

pub const CONFIG_KIND:&str="vtt:project-carry-config";

pub fn validate_policies(session:&Session,game:&GameBindings)->Result<(),String> {
    let mut entities=BTreeSet::new();
    for (key,policy) in game.project_carry() {
        if key!=&policy.character || !entities.insert(policy.entity)
            || game.characters().get(key).is_none_or(|recipe|recipe.entity!=policy.entity) {
            return Err("project carry policy differs from its unique admitted native character".into());
        }
        session.sim.custody_provider(&policy.provider)?;
    }
    for record in game.item_records() {
        if let Some(witness)=&record.project_custody {
            if game.project_carry().values().filter(|policy|policy.entity==record.input.actor
                && policy.provider==witness.provider).count()!=1 {
                return Err("project custody witness has no exact saved authored capacity policy".into());
            }
        }
    }
    Ok(())
}

pub(super) fn configure_rows(session:&Session,game:&mut GameBindings,dm:Id,input:&ProjectCarryConfig)
    ->Result<(),String> {
    if input.kind!=CONFIG_KIND || input.version!=1 || input.expected_native!=source(session)
        || !session.sim.state().population.get(dm).is_some_and(cosmograph::directing::is_participant) {
        return Err("project carry configuration differs from its actual native/game/DM cut".into());
    }
    let mut rows:BTreeMap<Key,ProjectCarryPolicy>=game.project_carry().clone();
    let mut supplied=BTreeSet::new();
    for policy in &input.policies {
        if !supplied.insert(&policy.character) {return Err("duplicate project carry policy".into());}
        if rows.get(&policy.character).is_some_and(|old|old!=policy) {
            return Err("project carry configuration cannot reassign a saved provider or character".into());
        }
        rows.insert(policy.character.clone(),policy.clone());
    }
    game.record_project_carry(rows);
    validate_policies(session,game)
}

pub fn submit_project_carry_config(session:&Session,game:Vec<u8>,access:OrdinaryAccess,input:&ProjectCarryConfig)
    ->Result<Vec<u8>,String> {
    super::super::movement::current_access(session,access)?;
    if access.seat!=Seat::Dm {return Err("project carry configuration requires the actual DM seat".into());}
    if input.expected_game!=cosmograph::digest(&game) {return Err("project carry configuration has a stale game cut".into());}
    let mut next=GameBindings::decode(&game)?;
    next.validate(session)?;
    // Opaque legacy payloads are authenticated before their first wrapper is added.
    configure_rows(session,&mut next,access.participant.0,input)?;
    next.validate(session)?;
    next.encode()
}

pub(super) fn provider<'a>(game:&'a GameBindings,actor:Id)->Result<&'a str,String> {
    let mut rows=game.project_carry().values().filter(|policy|policy.entity==actor);
    let row=rows.next().ok_or("project carrier needs an explicit DM-authored native capacity provider")?;
    if rows.next().is_some() {return Err("project carrier has ambiguous capacity policies".into());}
    Ok(&row.provider)
}

pub(super) fn perform(session:&mut Session,records:&[PhysicalItemRecord],position:&PositionBinding,
    input:&ItemInput,participant:Id,provider:&str)->Result<(ProjectPosition,ProjectCustodyWitness),String> {
    let ItemAction::ProjectCarry {project,carried}=&input.action else {return Err("project carry input is absent".into());};
    let native=session.sim.state().projects.get(*project).ok_or("selected project is absent")?;
    if native.output.is_some() {return Err("completed project has no unfinished ledger to carry".into());}
    let place=location(session,records,*project,&|actor| {
        if actor!=input.actor {return Err("project is held by another native carrier".into());}
        Ok(position.clone())
    })?;
    if place.site!=position.placement.site || place.at!=position.current(session)?
        || *carried && place.carrier.is_some() || !*carried && place.carrier!=Some(input.actor) {
        return Err("project pickup/drop differs from the actor's current native custody and point".into());
    }
    session.sim.custody_provider(provider)?;
    if *carried {
        let burden=session.sim.item_burden(input.actor)?.checked_add(session.sim.project_matter(*project)?)
            .ok_or("project carry burden overflows")?;
        if burden>u128::from(session.sim.item_capacity(input.actor,provider)?) {
            return Err("project exceeds the actual authored native carry capacity".into());
        }
    }
    let outcome=session.command(Command::Project(Change::Carry {participant,actor:input.actor,project:*project,carried:*carried}))?;
    let receipt:cosmograph::projects::Custody=serde_json::from_str(&outcome).map_err(|error|error.to_string())?;
    if receipt.project!=*project || receipt.carrier!=carried.then_some(input.actor)
        || receipt.site!=place.site || receipt.matter!=session.sim.project_matter(*project)? {
        return Err("project carry receipt differs from the actual conserved ledger and carrier".into());
    }
    Ok((ProjectPosition {project:*project,site:place.site,at:place.at,
        created_site:place.created_site,created_at:place.created_at},
        ProjectCustodyWitness {provider:provider.into(),receipt}))
}

pub fn current_project(session:&Session,game:&GameBindings,id:ProjectId)->Result<ProjectLocation,String> {
    game.validate(session)?;
    location(session,game.item_records(),id,&|carrier|actor_position(session,game,carrier))
}

pub(super) fn location(session:&Session,records:&[PhysicalItemRecord],id:ProjectId,
    position:&impl Fn(Id)->Result<PositionBinding,String>)->Result<ProjectLocation,String> {
    let native=session.sim.state().projects.get(id).ok_or("selected project is absent")?;
    if native.output.is_some() {return Err("completed project is represented by its actual bound output".into());}
    let record=records.iter().rev().find(|record|record.project.as_ref().is_some_and(|place|place.project==id))
        .ok_or("project has no accepted physical creation position")?;
    let place=record.project.as_ref().ok_or("project physical position is absent")?;
    let last_carry=session.entries.iter().rev().find(|entry|matches!(&entry.command,
        Command::Project(Change::Carry {project,..}) if *project==id));
    let accepted_carry=records.iter().rev().find(|record|record.project_custody.as_ref()
        .is_some_and(|witness|witness.receipt.project==id));
    if last_carry.is_some_and(|entry|accepted_carry.is_none_or(|record|!record.entries.contains(&entry.id))) {
        return Err("project's latest native custody change has no admitted physical witness".into());
    }
    let (site,at,window,places)=if let Some(carrier)=native.carrier {
        let carry=accepted_carry.and_then(|record|record.project_custody.as_ref())
            .ok_or("native carried project has no admitted physical custody witness")?;
        if carry.receipt.carrier!=Some(carrier) {return Err("project carrier differs from its accepted custody witness".into());}
        let held=position(carrier)?;
        historical_position(session,&held)?;
        (held.placement.site,held.current(session)?,held.placement.window.clone(),held.placement.places)
    } else {
        if native.site!=place.site {return Err("project fixed site differs from its last accepted physical position".into());}
        let window=record.actor_position.placement.window.clone();
        let rules=record.actor_position.placement.places;
        let volume=session.sim.volume(place.site,window.min,window.max,window.depth)?;
        let places=isometer_space::places::Places::derive(&volume,rules)?;
        if places.place_at(place.at).is_none() || !volume.material(place.at).is_some_and(|material|
            material==volume.materials.air || material==volume.materials.water) {
            return Err("project point is outside its current realized clear native window".into());
        }
        (place.site,place.at,window,rules)
    };
    Ok(ProjectLocation {project:id,site,at,window,places,carrier:native.carrier,
        created_site:place.created_site,created_at:place.created_at})
}

pub(super) fn historical_position(session:&Session,position:&PositionBinding)->Result<(),String> {
    let entry=session.entries.iter().find(|entry|entry.id==position.patch_entry)
        .ok_or("project carrier placement entry is absent")?;
    if !matches!(&entry.command,Command::Patch {entity,patch} if *entity==position.entity
        && *patch==Some(position.placement.patch)) || entry.outcome!="placed" {
        return Err("project carrier position differs from actual accepted patch history".into());
    }
    position.current(session)?;
    Ok(())
}

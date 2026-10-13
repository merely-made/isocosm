// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Resident physical items and conserved work beside the caller's native history.

use super::{GameBindings,PositionBinding};
use isocosm::{Command,Session,projects::{Acted,Change,DebitSelection,OwnedActed,OwnedProjectActed},schema::{Id,Key}};
use isocosm_overlay::{EntityHandle,OrdinaryAccess,Seat};
mod types;
mod placement;
mod records;
pub use types::{EntityRef,ItemPlacement,ItemBindingInput,ItemAction,ItemInput,BoundItem,
    ProjectPosition,PhysicalItemRecord,ItemPreview};
pub use records::validate_item_records;

pub fn source(session:&Session)->Key {super::physical_map::physical_source(session)}

fn actor_position(session:&Session,game:&GameBindings,actor:Id)->Result<PositionBinding,String> {
    let (key,_)=game.characters().iter().find(|(_,binding)|binding.entity==actor)
        .ok_or("item actor has no admitted native character recipe")?;
    let position=game.positions().get(key).ok_or("item actor has no supplied current native position")?;
    position.current(session)?;
    Ok(position.clone())
}

fn check(session:&Session,game:&[u8],access:OrdinaryAccess,input:&ItemInput)
    ->Result<(GameBindings,PositionBinding),String> {
    super::movement::current_access(session,access)?;
    if input.version!=1 || input.expected_native!=source(session) || input.expected_game!=isocosm::digest(&game)
        || access.subject!=Some(EntityHandle(input.actor)) {
        return Err("item input differs from its actual native, game or controlled body cut".into());
    }
    let body=session.sim.state().population.get(input.actor).ok_or("item actor is absent")?;
    if !body.alive || body.body_revision!=input.body_revision || body.resident_site().is_none() {
        return Err("item actor is dead, travelling or has a changed body revision".into());
    }
    if matches!(input.action,ItemAction::AuthorBind(_)|ItemAction::AuthorOwn(_)) && access.seat!=Seat::Dm {
        return Err("item physical binding requires the actual DM author".into());
    }
    if let ItemAction::Act {process,..}=&input.action {
        let process=session.sim.genesis().rules.processes.get(process).ok_or("item process is absent")?;
        if process.requires.iter().any(|query|matches!(query,
            isocosm::rules::Query::Project {..}|isocosm::rules::Query::ProjectDecision)) {
            return Err("native project phases and consent use their exact typed item/project doors".into());
        }
    }
    let bindings=GameBindings::decode(game)?;
    bindings.validate(session)?;
    let position=actor_position(session,&bindings,input.actor)?;
    target_here(session,&bindings,access,&position,input)?;
    Ok((bindings,position))
}

fn target_here(session:&Session,game:&GameBindings,access:OrdinaryAccess,position:&PositionBinding,input:&ItemInput)
    ->Result<(),String> {
    let target=match &input.action {ItemAction::Project {target,..}|ItemAction::Act {target,..}=>*target,_=>None};
    let Some(target)=target.filter(|target|*target!=input.actor) else {return Ok(());};
    if access.seat==Seat::Player && !isocosm::directing::readings::view(&session.sim,input.actor,
        isocosm::directing::readings::Mode::Survival).shows(target) {
        return Err("item target is outside actual native awareness".into());
    }
    let at=position.current(session)?;
    let target_at=game.item_records().iter().rev().filter_map(|record|record.binding.as_ref())
        .find(|binding|binding.entity==target).ok_or("item target has no accepted physical item binding")?.current(session)?;
    let site=session.sim.state().population.get(target).and_then(|body|body.resident_site());
    if target_at!=at || site!=Some(position.placement.site) {
        return Err("item target is not at the actor's actual resident position".into());
    }
    Ok(())
}

fn selections(quote:&isocosm::projects::DebitPreview,selected:&[DebitSelection])->bool {
    quote.refusal.is_none() && selected.len()==quote.requests.len() && quote.requests.iter()
        .all(|request|selected.iter().filter(|selection|&selection.request==request).count()==1)
}

fn debit_quote(session:&Session,access:OrdinaryAccess,input:&ItemInput)->Result<Option<isocosm::projects::DebitPreview>,String> {
    Ok(match &input.action {
        ItemAction::Project {process,target,project,lots,..}=>Some(session.sim.prepare_owned_project(
            access.participant.0,input.actor,*target,process,*project,lots,None)?),
        ItemAction::Act {process,target,cause,..}=>Some(session.sim.prepare_owned_act(input.actor,*target,process,cause.clone())?),
        _=>None,
    })
}

fn selected(input:&ItemInput)->&[DebitSelection] {
    match &input.action {ItemAction::Project {selections,..}|ItemAction::Act {selections,..}=>selections,_=>&[]}
}

fn project_position(session:&Session,game:&GameBindings,position:&PositionBinding,id:isocosm::projects::ProjectId,
    existing:bool)->Result<ProjectPosition,String> {
    let project=session.sim.state().projects.get(id).ok_or("selected native project is absent")?;
    if project.carrier.is_some() {return Err("carried VTT projects need a complete custody and capacity provider".into());}
    let at=position.current(session)?;
    let place=if existing {game.item_records().iter().rev().filter_map(|record|record.project.as_ref())
        .find(|held|held.project==id).cloned().ok_or("project has no accepted physical creation position")?}
        else {ProjectPosition {project:id,site:position.placement.site,at,
            created_site:position.placement.site,created_at:at}};
    if project.site!=place.site || place.site!=position.placement.site || place.at!=at {
        return Err("project work is not at its fixed actual creation position".into());
    }
    Ok(place)
}

fn perform(session:&mut Session,game:&GameBindings,access:OrdinaryAccess,input:&ItemInput,position:&PositionBinding,
    preview:&mut ItemPreview)->Result<(Option<BoundItem>,Option<ProjectPosition>),String> {
    let participant=access.participant.0;
    match &input.action {
        ItemAction::AuthorBind(binding)=>Ok((Some(placement::bind(session,game,binding,None)?),None)),
        ItemAction::AuthorOwn(lot)=>{
            session.command(Command::Assert(isocosm::asserted::Assertion::StockOwnership(lot.clone())))?;
            Ok((None,None))
        },
        ItemAction::Project {process,target,project,lots,selections,output}=>{
            let prior=project.map(|id|project_position(session,game,position,id,true)).transpose()?;
            let outcome=session.command(Command::Project(Change::ActOwned {participant,actor:input.actor,
                target:*target,process:process.clone(),project:*project,cause:None,lots:lots.clone(),selections:selections.clone()}))?;
            let owned:OwnedProjectActed=serde_json::from_str(&outcome).map_err(|error|error.to_string())?;
            let acted=owned.acted;
            preview.project=Some(acted.clone());
            let id=acted.project.ok_or("accepted project process produced no native project")?;
            let place=if let Some(place)=prior {if place.project!=id {return Err("work selected a different project".into());} place}
                else {project_position(session,game,position,id,false)?};
            let binding=match (acted.output,output) {
                (Some(entity),Some(binding))=>Some(placement::bind(session,game,binding,Some(entity))?),
                (None,None)=>None,
                _=>return Err("completed output needs its exact supplied physical binding; unfinished work binds no output".into()),
            };
            if binding.as_ref().is_some_and(|binding|binding.placement.site!=place.site || binding.placement.at!=place.at) {
                return Err("completed item leaves the project's actual fixed position".into());
            }
            Ok((binding,Some(place)))
        },
        ItemAction::Consent {project,owner,worker,grant,process,cause}=>{
            session.command(Command::Project(Change::Consent {participant,actor:input.actor,project:*project,
                owner:*owner,worker:*worker,grant:*grant,process:process.clone(),cause:cause.clone()}))?;
            Ok((None,None))
        },
        ItemAction::Act {process,target,cause,selections}=>{
            let outcome=session.command(Command::ActOwned {participant,actor:input.actor,target:*target,
                process:process.clone(),cause:cause.clone(),selections:selections.clone()})?;
            let acted:OwnedActed=serde_json::from_str(&outcome).map_err(|error|error.to_string())?;
            if !acted.receipt.accepted() {return Err(format!("native item act was not admitted: {:?}",acted.receipt.outcome));}
            Ok((None,None))
        },
    }
}

fn candidate(session:&Session,mut game:GameBindings,access:OrdinaryAccess,input:&ItemInput,position:PositionBinding,
    preview:&mut ItemPreview)->Result<(Session,Vec<u8>),String> {
    let mut next=session.clone();
    let start=next.entries.len();
    let (binding,project)=perform(&mut next,&game,access,input,&position,preview)?;
    preview.binding=binding.clone();
    let record=PhysicalItemRecord {input:input.clone(),participant:access.participant.0,actor_position:position,
        entries:next.entries[start..].iter().map(|entry|entry.id.clone()).collect(),
        outcomes:next.entries[start..].iter().map(|entry|entry.outcome.clone()).collect(),binding,project};
    game.record_item(record);
    game.validate(&next)?;
    let bytes=game.encode()?;
    Ok((next,bytes))
}

pub fn prepare_item(session:&Session,game:&[u8],access:OrdinaryAccess,input:&ItemInput)->Result<ItemPreview,String> {
    let (bindings,position)=check(session,game,access,input)?;
    let debits=debit_quote(session,access,input)?;
    let mut preview=ItemPreview {refusal:debits.as_ref().and_then(|quote|quote.refusal.clone()),
        debits,complete:false,project:None,binding:None};
    if preview.debits.as_ref().is_some_and(|quote|!selections(quote,selected(input))) {return Ok(preview);}
    match candidate(session,bindings,access,input,position,&mut preview) {
        Ok(_)=>preview.complete=true,Err(why)=>preview.refusal=Some(why),
    }
    Ok(preview)
}

pub fn submit_item(session:&mut Session,game:Vec<u8>,access:OrdinaryAccess,input:&ItemInput)->Result<(String,Vec<u8>),String> {
    let (bindings,position)=check(session,&game,access,input)?;
    let debits=debit_quote(session,access,input)?;
    if debits.as_ref().is_some_and(|quote|!selections(quote,selected(input))) {
        return Err("item action needs its exact native debit requests and explicit quantity-lot selections".into());
    }
    let mut preview=ItemPreview {debits,complete:false,refusal:None,project:None,binding:None};
    let (next,game)=candidate(session,bindings,access,input,position,&mut preview)?;
    *session=next;
    Ok(("accepted native item/project input".into(),game))
}

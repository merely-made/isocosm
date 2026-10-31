// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;
use std::collections::{BTreeMap,BTreeSet};

fn native_binding(session:&mut Session,input:&ItemBindingInput,output:Option<Id>,bound:&BoundItem)
    ->Result<(),String> {
    let entity=match input.entity {EntityRef::Existing(id)=>id,
        EntityRef::SelectedProjectOutput=>output.ok_or("historical item has no selected project output")?};
    let body=session.sim.state().population.get(entity).ok_or("historical item entity is absent")?;
    if session.sim.item_carrier(entity)?.is_some() {return Err("ground binding replaces an actually held item".into());}
    if bound.item!=input.item || bound.entity!=entity || bound.placement!=input.placement
        || body.lineage!=input.lineage || cosmograph::anatomy::books(body)!=input.stocks
        || output.is_some_and(|output|output!=entity) {
        return Err("physical item binding differs from actual entity, output, lineage or stocks".into());
    }
    if let Some(seed)=input.development_seed {session.command(Command::Develop {entity,seed})?;}
    placement::check(session,entity,&input.placement,false)?;
    session.command(Command::Patch {entity,patch:Some(input.placement.patch)})?;
    if session.entries.last().is_none_or(|entry|entry.id!=bound.patch_entry) {
        return Err("physical item patch witness differs from native history".into());
    }
    bound.current(session)?;
    Ok(())
}

/// Historical native entry cuts prove quantities and placement; expected_game
/// remains the outer live product publication precondition.
pub fn validate_item_records(session:&Session,records:&[PhysicalItemRecord])->Result<(),String> {
    let mut used=BTreeSet::new();
    let mut items:BTreeMap<Id,BoundItem>=BTreeMap::new();
    let mut projects:BTreeMap<cosmograph::projects::ProjectId,ProjectPosition>=BTreeMap::new();
    let mut last_order=None;
    for (index,record) in records.iter().enumerate() {
        let input=&record.input;
        let first=record.entries.first().ok_or("physical item record has no native entry")?;
        let mut before=session.before_entry(first)?;
        let body=before.sim.state().population.get(input.actor).ok_or("historical item actor is absent")?;
        let version=input_version(&input.action);
        if input.version!=version || record.entries.len()!=record.outcomes.len() || input.expected_native!=source(&before)
            || before.sim.plays(record.participant)!=Some(input.actor) || !body.alive
            || body.body_revision!=input.body_revision || record.actor_position.entity!=input.actor {
            return Err("physical item record differs from its actual native source, actor or control".into());
        }
        let actor_at=record.actor_position.current(&before)?;
        if !matches!(&input.action,ItemAction::ProjectCarry {..}) && record.project_custody.is_some()
            || !matches!(&input.action,ItemAction::Project {..}) && record.project_carrier_position.is_some() {
            return Err("item record invents unrelated project custody or carrier geometry".into());
        }
        let target=match &input.action {ItemAction::Project {target,..}|ItemAction::Act {target,..}=>*target,_=>None};
        if let Some(target)=target.filter(|target|*target!=input.actor) {
            let pose=current::from_records(&before,&records[..index],target,&|actor| {
                if actor!=input.actor {return Err("historical target has another actual carrier".into());}
                Ok(record.actor_position.clone())
            })?;
            if (pose.at!=actor_at && pose.carrier!=Some(input.actor)) || pose.site!=record.actor_position.placement.site {
                return Err("historical item target differs from the actual resident contact".into());
            }
        }
        let start=before.entries.len();
        match &input.action {
            ItemAction::AuthorBind(binding)=>{
                if record.project.is_some() {return Err("item binding invents a project placement".into());}
                native_binding(&mut before,binding,None,record.binding.as_ref().ok_or("bound item witness is absent")?)?;
            },
            ItemAction::AuthorOwn(lot)=>{
                before.command(Command::Assert(cosmograph::asserted::Assertion::StockOwnership(lot.clone())))?;
                if record.binding.is_some() || record.project.is_some() {
                    return Err("stock ownership assertion changes unrelated physical bindings".into());
                }
            },
            ItemAction::Project {process,target,project,lots,selections,output}=>{
                let prior=project.map(|id| {
                    let native=before.sim.state().projects.get(id).ok_or("historical project is absent")?;
                    if native.carrier.is_none() && record.project_carrier_position.is_some() {
                        return Err("fixed project invents carrier geometry".into());
                    }
                    project_carry::location(&before,&records[..index],id,&|carrier| {
                        let position=record.project_carrier_position.as_ref()
                            .ok_or("carried work lacks its actual historical carrier position")?;
                        if position.entity!=carrier {return Err("work names a different project carrier".into());}
                        project_carry::historical_position(&before,position)?;
                        Ok(position.clone())
                    })
                }).transpose()?;
                if project.is_none() && record.project_carrier_position.is_some() {
                    return Err("new project invents existing carrier geometry".into());
                }
                let outcome=before.command(Command::Project(Change::ActOwned {participant:record.participant,
                    actor:input.actor,target:*target,process:process.clone(),project:*project,cause:None,
                    lots:lots.clone(),selections:selections.clone()}))?;
                let acted:OwnedProjectActed=serde_json::from_str(&outcome).map_err(|error|error.to_string())?;
                let id=acted.acted.project.ok_or("historical project act names no project")?;
                let place=record.project.as_ref().ok_or("historical project position is absent")?;
                let native=before.sim.state().projects.get(id).ok_or("accepted project is absent")?;
                if place.project!=id || before.sim.project_site(native)!=Some(place.site)
                    || place.site!=record.actor_position.placement.site || place.at!=actor_at
                    || prior.as_ref().is_some_and(|prior|prior.project!=id || prior.site!=place.site || prior.at!=place.at
                        || prior.created_site!=place.created_site || prior.created_at!=place.created_at)
                    || project.is_none() && (projects.contains_key(&id)
                        || place.created_site!=place.site || place.created_at!=place.at) {
                    return Err("project position differs from actual creation, carrier or resumed work".into());
                }
                match (acted.acted.output,output,&record.binding) {
                    (Some(entity),Some(input),Some(bound))=>{
                        if bound.placement.site!=place.site || bound.placement.at!=place.at {
                            return Err("completed output changes the project's physical position".into());
                        }
                        native_binding(&mut before,input,Some(entity),bound)?;
                    },
                    (None,None,None)=>{},
                    _=>return Err("historical output differs from its exact supplied binding".into()),
                }
                projects.insert(id,place.clone());
            },
            ItemAction::Consent {project,owner,worker,grant,process,cause}=>{
                before.command(Command::Project(Change::Consent {participant:record.participant,actor:input.actor,
                    project:*project,owner:*owner,worker:*worker,grant:*grant,process:process.clone(),cause:cause.clone()}))?;
                if record.binding.is_some() || record.project.is_some() {return Err("consent changes physical item bindings".into());}
            },
            ItemAction::Act {process,target,cause,selections}=>{
                let outcome=before.command(Command::ActOwned {participant:record.participant,actor:input.actor,
                    target:*target,process:process.clone(),cause:cause.clone(),selections:selections.clone()})?;
                let acted:OwnedActed=serde_json::from_str(&outcome).map_err(|error|error.to_string())?;
                if !acted.receipt.accepted() || record.binding.is_some() || record.project.is_some() {
                    return Err("historical item act was refused or changes unrelated placement".into());
                }
            },
            ItemAction::Custody(_)=>{
                let (binding,_)=custody::perform(&mut before,&records[..index],&record.actor_position,
                    input,record.participant)?;
                if binding!=record.binding || record.project.is_some() {
                    return Err("custody record changes its exact native drop binding or unrelated project position".into());
                }
            },
            ItemAction::ProjectCarry {project,..}=>{
                let witness=record.project_custody.as_ref().ok_or("project carry witness is absent")?;
                let (place,actual)=project_carry::perform(&mut before,&records[..index],&record.actor_position,
                    input,record.participant,&witness.provider)?;
                if &actual!=witness || record.project.as_ref()!=Some(&place) || record.binding.is_some() {
                    return Err("project carry record differs from actual custody, capacity or physical location".into());
                }
                projects.insert(*project,place);
            },
        }
        let generated=&before.entries[start..];
        if generated.len()!=record.entries.len() {return Err("item record changes its actual native command count".into());}
        for ((entry,key),outcome) in generated.iter().zip(&record.entries).zip(&record.outcomes) {
            let actual=session.entries.iter().find(|native|&native.id==key).ok_or("item native entry is absent")?;
            if !used.insert(key) || entry!=actual || &entry.outcome!=outcome
                || last_order.is_some_and(|prior|entry.order<=prior) {
                return Err("item record changes native entries, outcomes or accepted order".into());
            }
            last_order=Some(entry.order);
        }
        if let Some(binding)=&record.binding {
            if items.values().any(|old|old.item==binding.item && old.entity!=binding.entity)
                || items.get(&binding.entity).is_some_and(|old|old.item!=binding.item) {
                return Err("physical item identity names two native entities or items".into());
            }
            items.insert(binding.entity,binding.clone());
        }
    }
    Ok(())
}

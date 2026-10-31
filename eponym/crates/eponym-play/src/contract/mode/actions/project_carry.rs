// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Conserved native project custody with actual saved game positions (840).

use super::*;
use super::projects::ProjectOrigin;
use cosmograph::schema::Method;
use std::collections::BTreeMap;

#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
pub enum ProjectCapacity { Imported, Native {provider:Key} }

#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectCarryPolicy { pub subject:SubjectId, pub capacity:ProjectCapacity }

#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
pub struct ProjectLocation {
    pub project:ProjectId, pub site:Id, pub at:[i64;3], pub carrier:Option<Id>,
    pub created_site:Id, pub created_at:[i64;3],
}

fn actor_at(session:&Session,actor:&PoseBinding)->Result<[i64;3],String> {
    let volume=session.sim.volume(actor.site,actor.window.min,actor.window.max,actor.window.depth)?;
    Ok(crate::walking::to_site(&volume,actor.current(session)?.cell().map_err(debug)?))
}

fn clear(session:&Session,site:Id,at:[i64;3],window:&Window)->Result<(),String> {
    let volume=session.sim.volume(site,window.min,window.max,window.depth)?;
    if !volume.holds(at[0],at[2]) || at[1]<volume.datum || at[1]>=volume.ceiling()
        || !volume.material(at).is_some_and(|material|material==volume.materials.air || material==volume.materials.water) {
        return Err("Eponym project position is outside its actual clear native window".into());
    }
    Ok(())
}

pub fn project_location(session:&Session,mode:&ModeState,actions:&ActionState,id:ProjectId)
    ->Result<ProjectLocation,String> {
    let project=session.sim.state().projects.get(id).ok_or("Eponym project is absent")?;
    if project.output.is_some() {return Err("Eponym project is already complete".into());}
    let pose=actions.projects.iter().find(|pose|pose.project==id)
        .ok_or("Eponym project has no accepted physical position")?;
    let origin=pose.origin.clone().unwrap_or(ProjectOrigin {site:pose.site,at:pose.at});
    if !session.sim.state().sites.contains_key(&origin.site) || !session.entries.iter().any(|entry|
        entry.id==pose.entry && match &entry.command {
            Command::Project(Change::Act {..})=>serde_json::from_str::<Acted>(&entry.outcome)
                .is_ok_and(|acted|acted.project==Some(id) && acted.receipt.accepted()),
            Command::Project(Change::ActOwned {..})=>serde_json::from_str::<OwnedProjectActed>(&entry.outcome)
                .is_ok_and(|acted|acted.acted.project==Some(id) && acted.acted.receipt.accepted()),
            _=>false,
        }) {
        return Err("Eponym project lacks its actual accepted native creation witness".into());
    }
    let (site,at)=if let Some(carrier)=project.carrier {
        let body=session.sim.state().population.get(carrier).ok_or("Eponym project carrier is absent")?;
        let site=body.resident_site().ok_or("Eponym project carrier is in transit")?;
        let binding=mode.bindings.iter().find(|binding|binding.entity==carrier)
            .ok_or("Eponym project carrier has no explicit current physical binding")?;
        if binding.site!=site {return Err("Eponym project carrier binding names another site".into());}
        (site,actor_at(session,binding)?)
    } else {
        if project.site!=pose.site {return Err("fixed Eponym project differs from its accepted native site".into());}
        clear(session,pose.site,pose.at,&pose.window)?;
        (pose.site,pose.at)
    };
    Ok(ProjectLocation {project:id,site,at,carrier:project.carrier,created_site:origin.site,created_at:origin.at})
}

pub(super) fn validate_shape(mode:&ModeState,actions:&ActionState)->Result<(),String> {
    let mut subjects=BTreeSet::new();
    for policy in &actions.project_carry {
        if !subjects.insert(policy.subject) {return Err("duplicate Eponym project carrying policy".into());}
        binding(mode,policy.subject)?;
        if matches!(&policy.capacity,ProjectCapacity::Native {provider} if provider.trim().is_empty()) {
            return Err("Eponym project capacity names no explicit native provider".into());
        }
    }
    if let Some(imported)=&actions.imported {
        let mut bodies=BTreeSet::new();
        for capacity in &imported.products.carry_capacities {
            if !bodies.insert(capacity.subject) || binding(mode,capacity.subject)?.entity!=capacity.entity {
                return Err("imported capacity has duplicate or replaced original body identity".into());
            }
        }
    }
    Ok(())
}

pub(super) fn validate(session:&Session,mode:&ModeState,actions:&ActionState)->Result<(),String> {
    validate_shape(mode,actions)?;
    for policy in &actions.project_carry {
        let actor=binding(mode,policy.subject)?;
        match &policy.capacity {
            ProjectCapacity::Imported=>{
                let imported=actions.imported.as_ref().ok_or("project capacity has no consumed original game owner")?;
                let entries:Vec<_>=imported.products.carry_capacities.iter().filter(|capacity|capacity.subject==policy.subject).collect();
                if entries.len()!=1 || entries[0].entity!=actor.entity {
                    return Err("project capacity differs from its exact original physical body owner".into());
                }
            },
            ProjectCapacity::Native {provider}=>{session.sim.custody_provider(provider)?;},
        }
    }
    Ok(())
}

fn capacity(session:&Session,actions:&ActionState,actor:&PoseBinding)->Result<u64,String> {
    let policy=actions.project_carry.iter().find(|policy|policy.subject==actor.subject)
        .ok_or("Eponym project carrying has no explicit capacity provider")?;
    match &policy.capacity {
        ProjectCapacity::Imported=>{
            let capacity=actions.imported.as_ref().and_then(|imported|imported.products.carry_capacities.iter()
                .find(|capacity|capacity.subject==actor.subject && capacity.entity==actor.entity))
                .ok_or("Eponym project carrying lost its original physical capacity")?;
            Ok(u64::from(capacity.capacity_mg))
        },
        ProjectCapacity::Native {provider}=>session.sim.item_capacity(actor.entity,provider),
    }
}

fn burden(session:&Session,actor:Id)->Result<u128,String> {
    let mut kinds=BTreeSet::from([crate::founding::CARRIES.to_owned()]);
    if let Some(catalogue)=&session.sim.genesis().rules.item_custody {
        kinds.extend(catalogue.providers.values().map(|provider|provider.relation.clone()));
    }
    let mut links=BTreeMap::new();
    for relation in &session.sim.state().relations {
        if !kinds.contains(&relation.kind) {continue;}
        if links.insert(relation.object,relation.subject).is_some() {
            return Err("Eponym item has ambiguous actual native custody".into());
        }
        let item=session.sim.state().population.get(relation.object).ok_or("Eponym carried item is absent")?;
        if item.method!=Method::Inert || session.sim.state().population.get(relation.subject).is_none() {
            return Err("Eponym custody names a non-item body or absent carrier".into());
        }
    }
    let follows=|mut item|->Result<bool,String> {
        let mut seen=BTreeSet::new();
        while let Some(carrier)=links.get(&item) {
            if !seen.insert(item) {return Err("Eponym native custody contains a cycle".into());}
            if *carrier==actor {return Ok(true);}
            item=*carrier;
        }
        Ok(false)
    };
    let mut total=0u128;
    for item in links.keys() {
        if follows(*item)? {total=total.checked_add(session.sim.item_matter(*item)?).ok_or("Eponym item burden overflow")?;}
    }
    for (id,project) in &session.sim.state().projects.records {
        if let Some(carrier)=project.carrier {
            if carrier==actor || follows(carrier)? {
                total=total.checked_add(session.sim.project_matter(*id)?).ok_or("Eponym project burden overflow")?;
            }
        }
    }
    Ok(total)
}

pub(super) fn act(session:&mut Session,mode:&ModeState,actions:&mut ActionState,
    actor:&PoseBinding,participant:Id,id:ProjectId,carried:bool)->Result<String,String> {
    validate(session,mode,actions)?;
    let location=project_location(session,mode,actions,id)?;
    let at=actor_at(session,actor)?;
    if location.site!=actor.site || location.at!=at {
        return Err("Eponym project carry/drop is not at the actor's current explicit position".into());
    }
    if carried {
        if location.carrier.is_some() {return Err("Eponym project already has native custody".into());}
        let attempted=burden(session,actor.entity)?.checked_add(session.sim.project_matter(id)?)
            .ok_or("Eponym project burden overflow")?;
        if attempted>u128::from(capacity(session,actions,actor)?) {
            return Err("actual Eponym items and projects exceed the supplied physical capacity".into());
        }
    } else if location.carrier!=Some(actor.entity) {
        return Err("only the current native carrier drops this Eponym project".into());
    }
    let out=session.command(Command::Project(Change::Carry {participant,actor:actor.entity,project:id,carried}))?;
    let custody:Custody=serde_json::from_str(&out).map_err(|error|error.to_string())?;
    if custody.project!=id || custody.site!=actor.site || custody.carrier!=carried.then_some(actor.entity)
        || custody.matter!=session.sim.project_matter(id)? {
        return Err("Eponym project carrying differs from its exact native acceptance".into());
    }
    if !carried {
        let pose=actions.projects.iter_mut().find(|pose|pose.project==id).ok_or("Eponym project pose vanished")?;
        if pose.origin.is_none() {pose.origin=Some(ProjectOrigin {site:pose.site,at:pose.at});}
        pose.site=actor.site; pose.at=at; pose.window=actor.window.clone();
    }
    Ok(out)
}

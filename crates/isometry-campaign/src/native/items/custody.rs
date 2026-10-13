// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;
use isocosm::custody::{Action,Change,CustodyActed,MountPose,item_bounds};

pub(super) fn command(participant:Id,actor:Id,input:&ItemCustodyInput)->Command {
    Command::Custody(Change {participant,actor,item:input.item,provider:input.provider.clone(),action:input.action,
        cause:input.cause.clone(),selections:input.selections.clone()})
}

fn window_check(position:&PositionBinding,mount:&ItemMount,session:&Session)->Result<(),String> {
    if mount.window!=position.placement.window || mount.places!=position.placement.places
        || mount.terrain_revision!=session.sim.terrain_revision() {
        return Err("item mount window, patch rules or terrain revision differ from the actual carrier geometry".into());
    }
    Ok(())
}

fn collision_check(session:&Session,records:&[PhysicalItemRecord],position:&PositionBinding,pose:&ItemPose)
    ->Result<(),String> {
    let actor=session.sim.state().population.get(position.entity).ok_or("item carrier is absent")?;
    let body=session.sim.state().population.get(pose.entity).ok_or("physical item is absent")?;
    let boxes=item_bounds(body,MountPose {at:pose.at,yaw:pose.yaw})?;
    let carrier=item_bounds(actor,position.current_pose(session)?)?;
    if boxes.iter().any(|ours|carrier.iter().any(|theirs|ours.overlaps(theirs))) {
        return Err("supplied whole-item mount intersects the carrier's actual native tissue".into());
    }
    let mut seen=std::collections::BTreeSet::new();
    for binding in records.iter().rev().filter_map(|record|record.binding.as_ref()) {
        if binding.entity==pose.entity || !seen.insert(binding.entity)
            || session.sim.item_carrier(binding.entity)?!=Some(position.entity) {continue;}
        let other=current::from_records(session,records,binding.entity,&|actor| {
            if actor!=position.entity {return Err("nested physical carrier needs its own admitted position provider".into());}
            Ok(position.clone())
        })?;
        let body=session.sim.state().population.get(other.entity).ok_or("other held item is absent")?;
        let theirs=item_bounds(body,MountPose {at:other.at,yaw:other.yaw})?;
        if boxes.iter().any(|ours|theirs.iter().any(|theirs|ours.overlaps(theirs))) {
            return Err("supplied item mount intersects another actually held native item".into());
        }
    }
    Ok(())
}

pub(super) fn perform(session:&mut Session,records:&[PhysicalItemRecord],position:&PositionBinding,
    input:&ItemInput,participant:Id)->Result<(Option<BoundItem>,CustodyActed),String> {
    let ItemAction::Custody(carry)=&input.action else {return Err("item action has no custody input".into());};
    let item=session.sim.state().population.get(carry.item).ok_or("physical item is absent")?;
    if item.body_revision!=carry.item_body_revision {return Err("item body revision changed".into());}
    let prior=current::from_records(session,records,carry.item,&|actor| {
        if actor!=input.actor {return Err("item is not held by the actual controlled actor".into());}
        Ok(position.clone())
    })?;
    match carry.action {
        Action::Carry if prior.carrier.is_some() || prior.at!=position.current(session)?
            || prior.site!=position.placement.site => return Err("pickup requires the actual resident item at the carrier's admitted root".into()),
        Action::Equip if prior.carrier!=Some(input.actor) || prior.worn => return Err("equip requires this actual carried and unworn item".into()),
        Action::Unequip if prior.carrier!=Some(input.actor) || !prior.worn => return Err("unequip requires this actual worn item".into()),
        Action::Drop if prior.carrier!=Some(input.actor) => return Err("drop requires the actual native carrier".into()),
        _=>{},
    }
    if carry.action==Action::Drop {
        let drop=carry.drop.as_ref().ok_or("drop needs complete supplied native item geometry")?;
        if carry.mount.is_some() || drop.placement.at!=prior.at || drop.yaw!=prior.yaw
            || drop.placement.site!=prior.site || drop.placement.body_revision!=carry.item_body_revision {
            return Err("drop changes the currently admitted item root, orientation, body or site".into());
        }
    } else {
        let mount=carry.mount.as_ref().ok_or("carry/equipment requires a complete explicitly authored mount")?;
        if carry.drop.is_some() || !session.sim.custody_provider(&carry.provider)?.mounts.contains(&mount.attachment) {
            return Err("item mount is not an exact declared custody attachment".into());
        }
        window_check(position,mount,session)?;
    }
    let outcome=session.command(command(participant,input.actor,carry))?;
    let acted:CustodyActed=serde_json::from_str(&outcome).map_err(|error|error.to_string())?;
    if session.sim.state().population.get(input.actor).is_none_or(|body|body.body_revision!=input.body_revision)
        || session.sim.state().population.get(carry.item).is_none_or(|body|body.body_revision!=carry.item_body_revision) {
        return Err("item process changed the supplied body revisions".into());
    }
    if carry.action==Action::Drop {
        let drop=carry.drop.as_ref().ok_or("drop geometry is absent")?;
        placement::check(session,carry.item,&drop.placement,false)?;
        session.command(Command::Patch {entity:carry.item,patch:Some(drop.placement.patch)})?;
        let binding=BoundItem {item:prior.item,entity:carry.item,placement:drop.placement.clone(),
            patch_entry:session.entries.last().ok_or("drop patch entry is absent")?.id.clone(),
            resident_since:session.sim.state().population.get(carry.item).ok_or("dropped item is absent")?.arrived};
        let pose=ItemPose {item:binding.item.clone(),entity:carry.item,site:prior.site,at:prior.at,yaw:prior.yaw,
            window:drop.placement.window.clone(),places:drop.placement.places,carrier:None,worn:false};
        current::clear_pose(session,&pose)?;
        return Ok((Some(binding),acted));
    }
    let record=PhysicalItemRecord {input:input.clone(),participant,actor_position:position.clone(),
        entries:Vec::new(),outcomes:Vec::new(),binding:None,project:None,
        project_custody:None,project_carrier_position:None};
    let pose=current::mounted(session,&record,position,&prior.item)?;
    collision_check(session,records,position,&pose)?;
    Ok((None,acted))
}

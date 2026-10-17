// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;
use isocosm::custody::{Action,MountPose,Yaw};

fn clearance(session:&Session,pose:&ItemPose)->Result<(),String> {
    session.sim.item_clearance(pose.entity,pose.site,MountPose {at:pose.at,yaw:pose.yaw},
        &isocosm::operations::Window {min:pose.window.min,max:pose.window.max,depth:pose.window.depth})?;
    Ok(())
}

pub(super) fn mounted(session:&Session,record:&PhysicalItemRecord,position:&PositionBinding,item_id:&isometry_campaign::ItemId)
    ->Result<ItemPose,String> {
    let ItemAction::Custody(input)=&record.input.action else {return Err("item record has no mount".into());};
    let mount=input.mount.as_ref().ok_or("held item has no supplied mount")?;
    let actor=session.sim.state().population.get(record.input.actor).ok_or("held item carrier is absent")?;
    let item=session.sim.state().population.get(input.item).ok_or("held item is absent")?;
    let provider=session.sim.custody_provider(&input.provider)?;
    if input.action==Action::Drop || actor.body_revision!=record.input.body_revision
        || item.body_revision!=input.item_body_revision || position.entity!=record.input.actor
        || session.sim.item_carrier(input.item)?!=Some(record.input.actor)
        || !provider.mounts.contains(&mount.attachment)
        || !session.sim.state().relations.iter().any(|relation|relation.subject==record.input.actor
            && relation.object==input.item && relation.kind==provider.relation) {
        return Err("item mount differs from its current native bodies, custody or authored provider".into());
    }
    let root=position.current_pose(session)?;
    if actor.resident_site()!=item.resident_site() || item.patch.is_some() {
        return Err("held item does not share its carrier's actual native residence".into());
    }
    let local=isocosm::custody::mount_pose(actor,&mount.attachment)?;
    let offset=super::super::placement::rotate(root.yaw,local.at)?;
    let mut at=[0;3];
    for axis in 0..3 {at[axis]=root.at[axis].checked_add(offset[axis]).ok_or("held item root overflows")?;}
    let pose=ItemPose {item:item_id.clone(),entity:input.item,site:position.placement.site,
        at,yaw:root.yaw.compose(local.yaw),window:position.placement.window.clone(),places:position.placement.places,
        carrier:Some(record.input.actor),worn:input.action==Action::Equip};
    clearance(session,&pose)?;
    Ok(pose)
}

pub(super) fn from_records(session:&Session,records:&[PhysicalItemRecord],entity:Id,
    position:&dyn Fn(Id)->Result<PositionBinding,String>)->Result<ItemPose,String> {
    let original=records.iter().filter_map(|record|record.binding.as_ref())
        .find(|binding|binding.entity==entity).ok_or("item has no accepted physical identity binding")?;
    for record in records.iter().rev() {
        if let ItemAction::Custody(input)=&record.input.action {
            if input.item==entity {
                if input.action!=Action::Drop {
                    let pose=mounted(session,record,&position(record.input.actor)?,&original.item)?;
                    return Ok(pose);
                }
                let drop=input.drop.as_ref().ok_or("dropped item has no exact supplied geometry")?;
                let binding=record.binding.as_ref().ok_or("dropped item has no native patch witness")?;
                let at=binding.current(session)?;
                let pose=ItemPose {item:binding.item.clone(),entity,site:binding.placement.site,at,yaw:drop.yaw,
                    window:binding.placement.window.clone(),places:binding.placement.places,carrier:None,worn:false};
                if session.sim.item_carrier(entity)?.is_some() {return Err("ground item still has native custody".into());}
                clearance(session,&pose)?;
                return Ok(pose);
            }
        }
        if let Some(binding)=record.binding.as_ref().filter(|binding|binding.entity==entity) {
            if session.sim.item_carrier(entity)?.is_some() {return Err("item native custody has no exact physical mount record".into());}
            let pose=ItemPose {item:binding.item.clone(),entity,site:binding.placement.site,at:binding.current(session)?,
                yaw:Yaw::Zero,window:binding.placement.window.clone(),places:binding.placement.places,carrier:None,worn:false};
            clearance(session,&pose)?;
            return Ok(pose);
        }
    }
    Err("item has no current admitted geometry".into())
}

/// Native bodies and current carrier placements supply the projection. Table
/// anchors, declared inventory weight and historical visits supply none.
pub fn current_item(session:&Session,game:&GameBindings,entity:Id)->Result<ItemPose,String> {
    from_records(session,game.item_records(),entity,&|actor|super::actor_position(session,game,actor))
}

pub(super) fn clear_pose(session:&Session,pose:&ItemPose)->Result<(),String> {clearance(session,pose)}

// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;
use isocosm::{anatomy, schema::Method};
use isometer_space::{Atlas,places::{Cap,Places}};

pub(super) fn check(session:&Session,entity:Id,placement:&ItemPlacement,placed:bool)->Result<(),String> {
    let body=session.sim.state().population.get(entity).ok_or("physical item entity is absent")?;
    if body.method!=Method::Inert || body.body.is_none() || body.body_revision!=placement.body_revision
        || body.resident_site()!=Some(placement.site) || placement.patch.site!=placement.site
        || session.sim.terrain_revision()!=placement.terrain_revision
        || placed && body.patch!=Some(placement.patch) || placement.window.depth<0
        || matches!(placement.places.cap,Cap::Own(sides) if sides.contains(&0)) {
        return Err("item placement names stale or unsupported native body, site, patch or terrain".into());
    }
    let window=&placement.window;
    let volume=session.sim.volume(placement.site,window.min,window.max,window.depth)?;
    if !volume.holds(placement.at[0],placement.at[2]) || placement.at[1]<volume.datum
        || placement.at[1]>=volume.ceiling() {
        return Err("item position is outside its explicit native window".into());
    }
    let places=Places::derive(&volume,placement.places)?;
    if !places.places.contains_key(&placement.patch) || places.place_at(placement.at)!=Some(placement.patch)
        || !volume.material(placement.at).is_some_and(|material|
            material==volume.materials.air || material==volume.materials.water) {
        return Err("item position is outside the realized clear native patch".into());
    }
    Ok(())
}

pub(super) fn bind(session:&mut Session,game:&GameBindings,input:&ItemBindingInput,output:Option<Id>)
    ->Result<BoundItem,String> {
    let entity=match input.entity { EntityRef::Existing(id)=>id,
        EntityRef::SelectedProjectOutput=>output.ok_or("item binding has no accepted selected project output")? };
    if output.is_some_and(|output|output!=entity) {
        return Err("item binding does not name this exact accepted native project output".into());
    }
    if input.item.0.trim().is_empty() {
        return Err("item binding needs an explicit identity".into());
    }
    if game.item_records().iter().filter_map(|record|record.binding.as_ref())
        .any(|old|(old.item==input.item && old.entity!=entity)||(old.entity==entity && old.item!=input.item)) {
        return Err("item identity or native entity already has another physical binding".into());
    }
    let body=session.sim.state().population.get(entity).ok_or("item entity is absent")?;
    if session.sim.item_carrier(entity)?.is_some() {
        return Err("a held item uses its actual native custody and supplied mount, not a fresh ground binding".into());
    }
    if body.method!=Method::Inert || body.lineage!=input.lineage || anatomy::books(body)!=input.stocks {
        return Err("item binding differs from its actual native lineage or retained stocks".into());
    }
    if let Some(seed)=input.development_seed {session.command(Command::Develop {entity,seed})?;}
    check(session,entity,&input.placement,false)?;
    session.command(Command::Patch {entity,patch:Some(input.placement.patch)})?;
    let patch_entry=session.entries.last().ok_or("item patch has no native entry")?.id.clone();
    check(session,entity,&input.placement,true)?;
    let resident_since=session.sim.state().population.get(entity).ok_or("bound item is absent")?.arrived;
    Ok(BoundItem {item:input.item.clone(),entity,placement:input.placement.clone(),patch_entry,resident_since})
}

impl BoundItem {
    pub fn current(&self,session:&Session)->Result<[i64;3],String> {
        check(session,self.entity,&self.placement,true)?;
        if session.sim.state().population.get(self.entity).is_none_or(|body|body.arrived!=self.resident_since) {
            return Err("item position belongs to an earlier native visit".into());
        }
        Ok(self.placement.at)
    }
}

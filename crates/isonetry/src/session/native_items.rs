// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;
use crate::native_table::{NativeTable,TableSender,table_digest};
use isocosm::Execution;
use isocosm_overlay::{EntityHandle,OrdinaryAccess,ParticipantHandle,Seat};
use isometry_campaign::native::items::{ItemInput,ItemPreview,prepare_item};
use serde::{Deserialize,Serialize};

#[derive(Clone,Debug,PartialEq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeItemsResolved {
    pub expected_table:Option<String>,pub publication:NativePublication,pub table:NativeTable,
    pub input:ItemInput,pub participant:u64,
}

fn access(campaign:&isometry_campaign::founding::Campaign,table:&NativeTable,participant:u64)
    ->Result<OrdinaryAccess,String> {
    let seat=if participant==table.host_participant {Seat::Dm}
        else if table.peers.iter().any(|peer|peer.participant==participant) {Seat::Player}
        else {return Err("item caller has no actual prior authenticated table binding".into());};
    Ok(OrdinaryAccess {participant:ParticipantHandle(participant),seat,
        subject:campaign.table_session().sim.plays(participant).map(EntityHandle)})
}

pub(crate) fn apply_native_items(state:&mut GameSnapshot,result:&NativeItemsResolved)->Result<(),GameError> {
    let fail=|why:&str|GameError::Native(why.into());
    let old=state.native_table.as_ref().ok_or_else(||fail("item result has no admitted native table"))?;
    if table_digest(Some(old))!=result.expected_table || old.host_participant!=result.table.host_participant
        || old.peers!=result.table.peers || old.boards!=result.table.boards || old.version!=result.table.version {
        return Err(fail("item result changes unrelated endpoint or board bindings"));
    }
    let world=state.native.as_ref().ok_or_else(||fail("item result has no native world"))?;
    let before=world.load(Execution::Grouped).map_err(GameError::Native)?;
    old.validate_history(before.table_session()).map_err(GameError::Native)?;
    let access=access(&before,old,result.participant).map_err(GameError::Native)?;
    let bytes=serde_json::to_vec(&result.input).map_err(|error|fail(&error.to_string()))?;
    let (admitted,game,_)=before.with_table_input(old.game.clone(),access,&bytes).map_err(GameError::Native)?;
    let expected=NativePublication::update(world,&admitted).map_err(GameError::Native)?;
    if game!=result.table.game || expected!=result.publication {
        return Err(fail("item result differs from the whole exact re-admitted native and product pair"));
    }
    let bindings=isometry_campaign::native::GameBindings::decode(&game).map_err(GameError::Native)?;
    if bindings.item_records().last().is_none_or(|record|
        record.participant!=result.participant || record.input!=result.input) {
        return Err(fail("item publication lacks its actual accepted participant/input witness"));
    }
    result.table.validate_history(admitted.table_session()).map_err(GameError::Native)?;
    let mut next=state.clone();
    super::native::apply_publication(&mut next,&result.publication)?;
    next.native_table=Some(result.table.clone());
    *state=next;
    Ok(())
}

impl HostSession {
    pub fn quote_native_items(&self,sender:TableSender,input:&ItemInput)->Result<ItemPreview,String> {
        let world=self.state.native.as_ref().ok_or("item quote needs a native world")?;
        let table=self.state.native_table.as_ref().ok_or("item quote needs an admitted native table")?;
        let campaign=world.load(Execution::Grouped)?;
        table.validate_history(campaign.table_session())?;
        let access=access(&campaign,table,table.participant(sender)?)?;
        prepare_item(campaign.table_session(),&table.game,access,input)
    }
    pub fn native_items(&mut self,sender:TableSender,input:&ItemInput)->Result<Vec<Outbound>,String> {
        let world=self.state.native.as_ref().ok_or("item act needs a native world")?;
        let table=self.state.native_table.as_ref().ok_or("item act needs an admitted native table")?;
        let campaign=world.load(Execution::Grouped)?;
        table.validate_history(campaign.table_session())?;
        let participant=table.participant(sender)?;
        let access=access(&campaign,table,participant)?;
        let bytes=serde_json::to_vec(input).map_err(|error|error.to_string())?;
        let (admitted,game,_)=campaign.with_table_input(table.game.clone(),access,&bytes)?;
        let mut next=table.clone();next.game=game;
        let result=NativeItemsResolved {expected_table:table_digest(Some(table)),
            publication:NativePublication::update(world,&admitted)?,table:next,input:input.clone(),participant};
        self.try_commit(GameEvent::NativeItemsResolved(result))
    }
}

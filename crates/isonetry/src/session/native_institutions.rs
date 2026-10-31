// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Constitutional commands retain their real endpoint, body and polity witnesses.

use super::*;
use crate::native_table::{NativeTable,TableSender,table_digest};
use cosmograph::{Command,Execution,projects::DebitPreview};
use cosmograph_overlay::{EntityHandle,OrdinaryAccess,ParticipantHandle,Seat};
use isocosm_native::native::institutions::{InstitutionInput,prepare_institution};
use serde::{Deserialize,Serialize};

#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeInstitutionInput { pub expected_table:String, pub input:InstitutionInput }

#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeInstitutionQuote {
    pub expected_table:String, pub input:InstitutionInput, pub preview:DebitPreview,
}

/// Filled only from the authority's authenticated request path. Display names
/// and token-owner fields never resolve an institution actor or endpoint.
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize,Deserialize)]
pub enum InstitutionSender { Host, Endpoint([u8;32]) }

impl InstitutionSender {
    fn native(self) -> TableSender {
        match self {Self::Host=>TableSender::Host,Self::Endpoint(key)=>TableSender::Endpoint(key)}
    }
}
impl From<TableSender> for InstitutionSender {
    fn from(sender:TableSender) -> Self {
        match sender {TableSender::Host=>Self::Host,TableSender::Endpoint(key)=>Self::Endpoint(key)}
    }
}

#[derive(Clone,Debug,PartialEq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeInstitutionsResolved {
    pub expected_table:Option<String>, pub publication:NativePublication, pub table:NativeTable,
    pub input:InstitutionInput, pub participant:u64, pub sender:InstitutionSender,
}

fn access(campaign:&isocosm_native::founding::Campaign,table:&NativeTable,sender:TableSender)
    ->Result<OrdinaryAccess,String> {
    let participant=table.participant(sender)?;
    Ok(OrdinaryAccess {participant:ParticipantHandle(participant),
        seat:match sender {TableSender::Host=>Seat::Dm,TableSender::Endpoint(_)=>Seat::Player},
        subject:campaign.table_session().sim.plays(participant).map(EntityHandle)})
}

fn expected(table:&NativeTable,source:&str)->Result<(),String> {
    if table_digest(Some(table)).as_deref()!=Some(source) {
        return Err("institution input names another native/product table cut".into());
    }
    Ok(())
}

fn recorded(before:&cosmograph::Session,after:&cosmograph::Session,input:&InstitutionInput)->Result<(),String> {
    let suffix=after.entries.get(before.entries.len()..).ok_or("institution history lost its parent")?;
    if suffix.len()!=1 || suffix[0].command!=Command::Institution(input.change.clone())
        || after.entries[..before.entries.len()]!=before.entries {
        return Err("institution publication lacks the exact accepted native command and parent".into());
    }
    let _:cosmograph::institutions::Outcome=serde_json::from_str(&suffix[0].outcome).map_err(|error|error.to_string())?;
    Ok(())
}

pub(crate) fn apply_native_institutions(state:&mut GameSnapshot,result:&NativeInstitutionsResolved)
    ->Result<(),GameError> {
    let fail=|why:&str|GameError::Native(why.into());
    let old=state.native_table.as_ref().ok_or_else(||fail("institution result has no admitted native table"))?;
    if table_digest(Some(old))!=result.expected_table || old.host_participant!=result.table.host_participant
        || old.peers!=result.table.peers || old.boards!=result.table.boards || old.version!=result.table.version
        || old.game!=result.table.game {
        return Err(fail("institution result changes unrelated table bindings or opaque product bytes"));
    }
    let world=state.native.as_ref().ok_or_else(||fail("institution result has no native world"))?;
    let before=world.load(Execution::Grouped).map_err(GameError::Native)?;
    old.validate_history(before.table_session()).map_err(GameError::Native)?;
    let access=access(&before,old,result.sender.native()).map_err(GameError::Native)?;
    if access.participant.0!=result.participant {return Err(fail("institution result relabels its authenticated participant"));}
    let bytes=serde_json::to_vec(&result.input).map_err(|error|fail(&error.to_string()))?;
    let (admitted,game,_)=before.with_table_input(old.game.clone(),access,&bytes).map_err(GameError::Native)?;
    recorded(before.table_session(),admitted.table_session(),&result.input).map_err(GameError::Native)?;
    if game!=old.game || NativePublication::update(world,&admitted).map_err(GameError::Native)?!=result.publication {
        return Err(fail("institution result differs from the exact re-admitted native/product pair"));
    }
    result.table.validate_history(admitted.table_session()).map_err(GameError::Native)?;
    let mut next=state.clone();
    super::native::apply_publication(&mut next,&result.publication)?;
    next.native_table=Some(result.table.clone());
    *state=next;
    Ok(())
}

impl HostSession {
    pub fn quote_native_institution(&self,sender:TableSender,input:&NativeInstitutionInput)
        ->Result<NativeInstitutionQuote,String> {
        let world=self.state.native.as_ref().ok_or("institution quote needs a native world")?;
        let table=self.state.native_table.as_ref().ok_or("institution quote needs an admitted native table")?;
        expected(table,&input.expected_table)?;
        let campaign=world.load(Execution::Grouped)?;
        table.validate_history(campaign.table_session())?;
        let preview=prepare_institution(campaign.table_session(),access(&campaign,table,sender)?,&input.input)?;
        Ok(NativeInstitutionQuote {expected_table:input.expected_table.clone(),input:input.input.clone(),preview})
    }

    pub fn native_institution(&mut self,sender:TableSender,input:&NativeInstitutionInput)->Result<Vec<Outbound>,String> {
        let world=self.state.native.as_ref().ok_or("institution act needs a native world")?;
        let table=self.state.native_table.as_ref().ok_or("institution act needs an admitted native table")?;
        expected(table,&input.expected_table)?;
        let campaign=world.load(Execution::Grouped)?;
        table.validate_history(campaign.table_session())?;
        let access=access(&campaign,table,sender)?;
        let bytes=serde_json::to_vec(&input.input).map_err(|error|error.to_string())?;
        let (admitted,game,_)=campaign.with_table_input(table.game.clone(),access,&bytes)?;
        if game!=table.game {return Err("institution act changes unrelated product data".into());}
        recorded(campaign.table_session(),admitted.table_session(),&input.input)?;
        let mut next=table.clone();next.game=game;
        let result=NativeInstitutionsResolved {expected_table:table_digest(Some(table)),
            publication:NativePublication::update(world,&admitted)?,table:next,input:input.input.clone(),
            participant:access.participant.0,sender:sender.into()};
        self.try_commit(GameEvent::NativeInstitutionsResolved(result))
    }
}

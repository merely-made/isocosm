// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

//! An explicit DM decision over a genuine native goal thread.

use super::*;
use crate::native_table::{NativeTable,table_digest};
use isocosm::Execution;
use isocosm_overlay::{EntityHandle,OrdinaryAccess,ParticipantHandle,Seat,vtt::VttIntentEnvelope};
use serde::{Deserialize,Serialize};

#[derive(Clone,Debug,PartialEq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeHookResolved {
    pub expected_table:Option<String>, pub publication:NativePublication, pub table:NativeTable,
}

pub(crate) fn apply_native_hook(state:&mut GameSnapshot,result:&NativeHookResolved)->Result<(),GameError> {
    let fail=|reason:&str|GameError::Native(reason.into());
    let old=state.native_table.as_ref().ok_or_else(||fail("hook has no native table"))?;
    if table_digest(Some(old))!=result.expected_table || old.host_participant!=result.table.host_participant
        || old.peers!=result.table.peers || old.boards!=result.table.boards || old.version!=result.table.version
        || !result.publication.boards.is_empty() || !result.publication.inventories.is_empty()
        || result.publication.activate.is_some() {
        return Err(fail("hook changes unrelated table bindings or content"));
    }
    let before=state.native.as_ref().ok_or_else(||fail("hook has no native world"))?
        .load(Execution::Grouped).map_err(GameError::Native)?;
    let after=result.publication.world.load(Execution::Grouped).map_err(GameError::Native)?;
    if before.content()!=after.content() || before.origin()!=after.origin() {
        return Err(fail("hook changes campaign content or founding provenance"));
    }
    old.validate_history(before.table_session()).map_err(GameError::Native)?;
    result.table.validate_history(after.table_session()).map_err(GameError::Native)?;
    let record=isometry_campaign::native::hooks::validate_transition(before.table_session(),&old.game,
        after.table_session(),&result.table.game).map_err(GameError::Native)?;
    if record.participant!=old.host_participant { return Err(fail("hook has no authenticated native DM")); }
    let mut next=state.clone();
    super::native::apply_publication(&mut next,&result.publication)?;
    next.native_table=Some(result.table.clone());
    *state=next;
    Ok(())
}

impl HostSession {
    /// This authoring door belongs to the actual host, never to a peer name.
    pub fn native_hook(&mut self,envelope:&VttIntentEnvelope)->Result<Vec<Outbound>,String> {
        let world=self.state.native.as_ref().ok_or("hook needs a native campaign")?;
        let table=self.state.native_table.as_ref().ok_or("hook needs an admitted native table")?;
        let campaign=world.load(Execution::Grouped)?;
        let session=campaign.table_session();
        table.validate_history(session)?;
        let participant=table.host_participant;
        if envelope.participant.0!=participant || envelope.tick.0!=session.sim.state().tick {
            return Err("hook names another native DM or tick".into());
        }
        let access=OrdinaryAccess {participant:ParticipantHandle(participant),seat:Seat::Dm,
            subject:session.sim.plays(participant).map(EntityHandle)};
        let bytes=serde_json::to_vec(envelope).map_err(|error|error.to_string())?;
        let (admitted,game,_)=campaign.with_table_input(table.game.clone(),access,&bytes)?;
        let mut next=table.clone(); next.game=game;
        let result=NativeHookResolved {expected_table:table_digest(Some(table)),
            publication:NativePublication::update(world,&admitted)?,table:next};
        self.try_commit(GameEvent::NativeHookResolved(result))
    }
}

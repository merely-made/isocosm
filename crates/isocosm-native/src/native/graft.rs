// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Exact current graft inputs over an existing normative native body process.

use cosmograph::{Command, Session, graft::{Carry, Prepared}, projects::DebitSelection,
    schema::{Id, Key, Tick}};
use cosmograph_overlay::{EntityHandle, OrdinaryAccess};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GraftInput {
    pub version:u32, pub expected_native:Key, pub tick:Tick,
    pub actor:Id, pub donor:Id, pub process:Key, pub cause:Option<Key>,
    pub carry:Carry, pub selections:Vec<DebitSelection>,
}

fn check(session:&Session,access:OrdinaryAccess,input:&GraftInput)->Result<(),String> {
    super::movement::current_access(session,access)?;
    if input.version!=1 || input.expected_native!=super::institutions::source(session)
        || input.tick!=session.sim.state().tick || access.subject!=Some(EntityHandle(input.actor))
        || session.sim.plays(access.participant.0)!=Some(input.actor) {
        return Err("graft input differs from its actual native prefix, tick or played body".into());
    }
    Ok(())
}

pub fn prepare_graft(session:&Session,access:OrdinaryAccess,input:&GraftInput)->Result<Prepared,String> {
    check(session,access,input)?;
    session.prepare_owned_graft(access.participant.0,input.actor,input.donor,&input.process,
        &input.carry,input.cause.clone())
}

pub fn submit_graft(session:&mut Session,game:Vec<u8>,access:OrdinaryAccess,input:&GraftInput)
    ->Result<(String,Vec<u8>),String> {
    check(session,access,input)?;
    let bindings=super::GameBindings::decode(&game)?;
    bindings.validate(session)?;
    let mut candidate=session.clone();
    let outcome=candidate.command(Command::ActGraft {participant:access.participant.0,
        actor:input.actor,target:input.donor,process:input.process.clone(),cause:input.cause.clone(),
        carry:input.carry.clone(),selections:input.selections.clone()})?;
    bindings.validate(&candidate)?;
    *session=candidate;
    Ok((outcome,game))
}

// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Table input over native constitutional operations and their actual body acts.

use isocosm::{Command, Session, institutions::Change, schema::{Id, Key, Tick}};
use isocosm_overlay::{EntityHandle, OrdinaryAccess};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InstitutionInput {
    pub version: u32,
    pub expected_native: Key,
    pub tick: Tick,
    pub change: Change,
}

pub fn source(session: &Session) -> Key { isocosm::digest(&session.save()) }

fn actor(change: &Change) -> (Id, Id) {
    match change {
        Change::Propose { participant, actor, .. }
        | Change::Approve { participant, actor, .. }
        | Change::Apply { participant, actor, .. } => (*participant, *actor),
    }
}

fn check(session: &Session, access: OrdinaryAccess, input: &InstitutionInput) -> Result<(), String> {
    super::movement::current_access(session, access)?;
    let (participant, actor) = actor(&input.change);
    if input.version != 1 || input.expected_native != source(session)
        || input.tick != session.sim.state().tick || access.participant.0 != participant
        || access.subject != Some(EntityHandle(actor)) || session.sim.plays(participant) != Some(actor) {
        return Err("institution input differs from its actual native prefix, tick or controlled body".into());
    }
    Ok(())
}

pub fn prepare_institution(session: &Session, access: OrdinaryAccess, input: &InstitutionInput)
    -> Result<isocosm::projects::DebitPreview, String> {
    check(session, access, input)?;
    session.sim.prepare_institution(&input.change)
}

pub fn submit_institution(session: &mut Session, game: Vec<u8>, access: OrdinaryAccess,
    input: &InstitutionInput) -> Result<(String, Vec<u8>), String> {
    check(session, access, input)?;
    let bindings = super::GameBindings::decode(&game)?;
    bindings.validate(session)?;
    let mut candidate = session.clone();
    let outcome = candidate.command(Command::Institution(input.change.clone()))?;
    bindings.validate(&candidate)?;
    *session = candidate;
    Ok((outcome, game))
}

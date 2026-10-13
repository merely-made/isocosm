// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Explicit supplied Carry geometry over the existing native Session.
//! This drives only an already played normative body. A deliberative
//! critter keeps native process choice and has no forced graft fallback.

use super::Runtime;
use crate::spatial::{Game, graft::Reading};
use isocosm::{Command, Session, graft::{Carry, Prepared}, projects::DebitSelection};
use isocosm_overlay::{ActKey, EntityHandle, OrdinaryAccess, ParticipantHandle, Seat, Tick};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GraftInput {
    pub version: u32,
    pub expected_native: String,
    pub participant: ParticipantHandle,
    pub tick: Tick,
    pub actor: EntityHandle,
    pub donor: EntityHandle,
    pub process: ActKey,
    pub cause: Option<String>,
    pub carry: Carry,
    pub selections: Vec<DebitSelection>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraftQuote { pub native: Prepared, pub spatial: Reading }

fn check(session: &Session, access: OrdinaryAccess, input: &GraftInput) -> Result<(), String> {
    if access.seat == Seat::CreativeObserver || access.subject != Some(input.actor)
        || access.participant != input.participant
        || session.sim.plays(access.participant.0) != Some(input.actor.0)
    { return Err("graft input does not name the participant's actually played body".into()); }
    if input.version != 1 || input.tick.0 != session.sim.state().tick
        || input.expected_native != crate::spatial::graft::source(session)
    { return Err("graft input names a stale or unsupported native source".into()); }
    Ok(())
}

pub fn prepare(session: &Session, access: OrdinaryAccess, game: &Game,
    input: &GraftInput) -> Result<GraftQuote, String> {
    check(session, access, input)?;
    game.validate(session)?;
    let native = session.prepare_owned_graft(input.participant.0, input.actor.0, input.donor.0,
        &input.process.0, &input.carry, input.cause.clone())?;
    Ok(GraftQuote { native, spatial: Reading::of(session, game, input.actor.0, input.donor.0) })
}

fn admit(session: &mut Session, access: OrdinaryAccess, game: &mut Game,
    input: &GraftInput) -> Result<isocosm::flows::FlowResult<String>, String> {
    check(session, access, input)?;
    game.validate(session)?;
    let mut native = session.clone();
    let mut physical = game.clone();
    let accepted = native.command_with_flows(Command::ActGraft {
        participant: input.participant.0, actor: input.actor.0, target: input.donor.0,
        process: input.process.0.clone(), cause: input.cause.clone(), carry: input.carry.clone(),
        selections: input.selections.clone(),
    })?;
    // Rebind only existing exact cells. Obstructed/departed history stays
    // unavailable, without rolling back the native conserved operation.
    physical.refresh(&mut native)?;
    physical.validate(&native)?;
    *session = native;
    *game = physical;
    Ok(accepted)
}

/// The borrowed host publishes the same native and saved-game candidates.
pub fn submit(session: &mut Session, access: OrdinaryAccess, game: &mut Game,
    input: &GraftInput) -> Result<String, String> {
    admit(session, access, game, input).map(|accepted| accepted.result)
}

impl Runtime {
    pub fn graft_source(&self) -> String { crate::spatial::graft::source(self.session()) }

    fn graft_access(&self) -> Result<OrdinaryAccess, String> {
        if self.fault.is_some() || self.checkpoint.is_some() || !self.queued.is_empty() {
            return Err("graft input waits for the existing runtime question, fault or queued input".into());
        }
        Ok(OrdinaryAccess { participant: ParticipantHandle(self.interim.participant), seat: Seat::Player,
            subject: self.critter().map(EntityHandle) })
    }

    pub fn prepare_graft(&self, input: &GraftInput) -> Result<GraftQuote, String> {
        prepare(self.session(), self.graft_access()?, self.spatial(), input)
    }

    pub fn apply_graft(&mut self, input: &GraftInput) -> Result<String, String> {
        let access = self.graft_access()?;
        let accepted = admit(&mut self.interim.session, access, &mut self.spatial, input)?;
        self.windows.absorb(&self.interim.session.sim, &[], &accepted.flows);
        if let Some(glyphs) = &mut self.glyphs { glyphs.absorb(&self.interim.session, &accepted.flows); }
        Ok(accepted.result)
    }
}

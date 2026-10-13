// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Joint departures/individual goals, authenticated to the saved endpoint table.

use super::*;
use crate::native_table::{NativeTable,TableSender,table_digest};
use isocosm::Execution;
use isocosm_overlay::{EntityHandle,OrdinaryAccess,ParticipantHandle,Seat};
use isometry_campaign::native::joint_journey::{JourneyEnvelope,validate_transition};
use serde::{Deserialize,Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeJourneyResolved {
    pub expected_table: Option<String>, pub publication: NativePublication, pub table: NativeTable,
}

pub(crate) fn apply_native_journey(state: &mut GameSnapshot, result: &NativeJourneyResolved) -> Result<(),GameError> {
    let fail = |reason: &str| GameError::Native(reason.into());
    let old = state.native_table.as_ref().ok_or_else(|| fail("joint journey has no admitted table"))?;
    if table_digest(Some(old)) != result.expected_table || old.host_participant != result.table.host_participant
        || old.peers != result.table.peers || old.boards != result.table.boards || old.version != result.table.version
        || !result.publication.boards.is_empty() || !result.publication.inventories.is_empty()
        || result.publication.activate.is_some() {
        return Err(fail("joint journey changes endpoint, board or unrelated content bindings"));
    }
    let before = state.native.as_ref().ok_or_else(|| fail("joint journey has no native campaign"))?
        .load(Execution::Grouped).map_err(GameError::Native)?;
    let after = result.publication.world.load(Execution::Grouped).map_err(GameError::Native)?;
    old.validate_history(before.table_session()).map_err(GameError::Native)?;
    result.table.validate_history(after.table_session()).map_err(GameError::Native)?;
    let change = validate_transition(before.table_session(),&old.game,after.table_session(),&result.table.game,
        &old.time_roster().map_err(GameError::Native)?).map_err(GameError::Native)?;
    match change.seat {
        Seat::Dm if change.participant == old.host_participant => {},
        Seat::Player if change.participant != old.host_participant
            && old.peers.iter().any(|peer| peer.participant == change.participant) => {},
        _ => return Err(fail("journey record has no authenticated current DM or player identity")),
    }
    let mut next = state.clone();
    super::native::apply_publication(&mut next,&result.publication)?;
    next.native_table = Some(result.table.clone());
    *state = next;
    Ok(())
}

impl HostSession {
    pub fn native_journey(&mut self, sender: TableSender, input: &JourneyEnvelope) -> Result<Vec<Outbound>,String> {
        let world = self.state.native.as_ref().ok_or("joint journey needs a native campaign")?;
        let table = self.state.native_table.as_ref().ok_or("joint journey needs its admitted endpoint table")?;
        let campaign = world.load(Execution::Grouped)?;
        let session = campaign.table_session();
        table.validate_history(session)?;
        let participant = table.participant(sender)?;
        if input.participant != participant || input.tick != session.sim.state().tick {
            return Err("journey request names another authenticated endpoint participant or native tick".into());
        }
        let access = OrdinaryAccess { participant:ParticipantHandle(participant),
            seat:if matches!(sender,TableSender::Host) { Seat::Dm } else { Seat::Player },
            subject:session.sim.plays(participant).map(EntityHandle) };
        let (admitted,game,_) = campaign.with_table_journey(table.game.clone(),access,&table.time_roster()?,input)?;
        let mut next = table.clone();
        next.game = game;
        let result = NativeJourneyResolved { expected_table:table_digest(Some(table)),
            publication:NativePublication::update(world,&admitted)?,table:next };
        self.try_commit(GameEvent::NativeJourneyResolved(result))
    }
}

// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Party decisions use the saved native participant and the authenticated endpoint.

use super::*;
use crate::native_table::{TableSender, table_digest};
use isocosm::{Command, Execution};
use isocosm_overlay::{EntityHandle, OrdinaryAccess, ParticipantHandle, Seat, vtt::VttIntentEnvelope};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativePartyResolved { pub expected_table: Option<String>, pub publication: NativePublication }

pub(crate) fn apply_native_party(state: &mut GameSnapshot, result: &NativePartyResolved) -> Result<(), GameError> {
    let fail = |reason: &str| GameError::Native(reason.into());
    let table = state.native_table.as_ref().ok_or_else(|| fail("party result has no admitted native table"))?;
    if table_digest(Some(table)) != result.expected_table || !result.publication.boards.is_empty()
        || !result.publication.inventories.is_empty() || result.publication.activate.is_some() {
        return Err(fail("party result changes authored table bindings or content"));
    }
    let before = state.native.as_ref().ok_or_else(|| fail("party result has no native campaign"))?
        .load(Execution::Grouped).map_err(GameError::Native)?;
    let after = result.publication.world.load(Execution::Grouped).map_err(GameError::Native)?;
    let (old, new) = (before.table_session(), after.table_session());
    table.validate_history(old).map_err(GameError::Native)?;
    table.validate_history(new).map_err(GameError::Native)?;
    if new.sim.state().tick != old.sim.state().tick || new.entries.len() != old.entries.len() + 1
        || !new.entries.starts_with(&old.entries)
        || !matches!(new.entries.last().map(|e| &e.command), Some(Command::Party(_))) {
        return Err(fail("party result is not one recorded membership/disclosure at the current tick"));
    }
    super::native::apply_publication(state, &result.publication)
}

impl HostSession {
    pub fn native_party(&mut self, sender: TableSender, envelope: &VttIntentEnvelope)
        -> Result<Vec<Outbound>, String> {
        let world = self.state.native.as_ref().ok_or("party input needs a native campaign")?;
        let table = self.state.native_table.as_ref().ok_or("party input needs explicit native participant bindings")?;
        let campaign = world.load(Execution::Grouped)?;
        let session = campaign.table_session();
        table.validate_history(session)?;
        let participant = table.participant(sender)?;
        if envelope.participant.0 != participant || envelope.tick.0 != session.sim.state().tick {
            return Err("party input differs from the authenticated participant or native tick".into());
        }
        let access = OrdinaryAccess { participant: ParticipantHandle(participant),
            seat: if matches!(sender, TableSender::Host) { Seat::Dm } else { Seat::Player },
            subject: session.sim.plays(participant).map(EntityHandle) };
        let bytes = serde_json::to_vec(envelope).map_err(|error| error.to_string())?;
        let (accepted, game, _) = campaign.with_table_input(table.game.clone(), access, &bytes)?;
        if game != table.game { return Err("party decision changes opaque game/time bytes".into()); }
        let result = NativePartyResolved { expected_table: table_digest(Some(table)),
            publication: NativePublication::update(world, &accepted)? };
        self.try_commit(GameEvent::NativePartyResolved(result))
    }
}

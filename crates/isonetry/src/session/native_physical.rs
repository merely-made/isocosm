// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The host's supplied physical map input and its exact admitted publication.

use super::*;
use crate::native_table::{NativeTable, table_digest};
use isocosm::Execution;
use isocosm_overlay::{EntityHandle, OrdinaryAccess, ParticipantHandle, Seat};
use isocosm_vtt::native::physical_map::{PhysicalMapInput, PhysicalMapPreview, prepare_physical_map};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativePhysicalResolved {
    pub expected_table: Option<String>,
    pub publication: NativePublication,
    pub table: NativeTable,
    pub input: PhysicalMapInput,
}

fn access(campaign: &isocosm_vtt::founding::Campaign, table: &NativeTable) -> OrdinaryAccess {
    OrdinaryAccess { participant: ParticipantHandle(table.host_participant), seat: Seat::Dm,
        subject: campaign.table_session().sim.plays(table.host_participant).map(EntityHandle) }
}

pub(crate) fn apply_native_physical(state: &mut GameSnapshot, result: &NativePhysicalResolved)
    -> Result<(), GameError> {
    let fail = |why: &str| GameError::Native(why.into());
    let old = state.native_table.as_ref().ok_or_else(|| fail("physical edit has no admitted native table"))?;
    if table_digest(Some(old)) != result.expected_table || old.host_participant != result.table.host_participant
        || old.peers != result.table.peers || old.boards != result.table.boards || old.version != result.table.version {
        return Err(fail("physical edit changes unrelated table bindings"));
    }
    let world = state.native.as_ref().ok_or_else(|| fail("physical edit has no native world"))?;
    let before = world.load(Execution::Grouped).map_err(GameError::Native)?;
    old.validate_history(before.table_session()).map_err(GameError::Native)?;
    let bytes = serde_json::to_vec(&result.input).map_err(|error| fail(&error.to_string()))?;
    let (admitted, game, _) = before.with_table_input(old.game.clone(), access(&before, old), &bytes)
        .map_err(GameError::Native)?;
    let expected = NativePublication::update(world, &admitted).map_err(GameError::Native)?;
    if game != result.table.game || expected != result.publication {
        return Err(fail("physical publication differs from the exact re-admitted input"));
    }
    result.table.validate_history(admitted.table_session()).map_err(GameError::Native)?;
    let mut next = state.clone();
    super::native::apply_publication(&mut next, &result.publication)?;
    next.native_table = Some(result.table.clone());
    *state = next;
    Ok(())
}

impl HostSession {
    /// Read-only measurements against the actual host's current native body.
    pub fn quote_native_physical(&self, input: &PhysicalMapInput) -> Result<PhysicalMapPreview, String> {
        let world = self.state.native.as_ref().ok_or("physical quote needs a native world")?;
        let table = self.state.native_table.as_ref().ok_or("physical quote needs an admitted native table")?;
        let campaign = world.load(Execution::Grouped)?;
        table.validate_history(campaign.table_session())?;
        prepare_physical_map(campaign.table_session(), &table.game, access(&campaign, table), input)
    }

    /// This file-authoring door belongs to the actual host, never a peer packet.
    pub fn native_physical(&mut self, input: &PhysicalMapInput) -> Result<Vec<Outbound>, String> {
        let world = self.state.native.as_ref().ok_or("physical edit needs a native world")?;
        let table = self.state.native_table.as_ref().ok_or("physical edit needs an admitted native table")?;
        let campaign = world.load(Execution::Grouped)?;
        table.validate_history(campaign.table_session())?;
        let bytes = serde_json::to_vec(input).map_err(|error| error.to_string())?;
        let (admitted, game, _) = campaign.with_table_input(table.game.clone(), access(&campaign, table), &bytes)?;
        let mut next = table.clone(); next.game = game;
        let result = NativePhysicalResolved { expected_table: table_digest(Some(table)),
            publication: NativePublication::update(world, &admitted)?, table: next, input: input.clone() };
        self.try_commit(GameEvent::NativePhysicalResolved(result))
    }
}

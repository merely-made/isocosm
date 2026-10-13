// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

//! DM-only saved pack decisions over the table's existing native authority.

use super::*;
use crate::native_table::{NativeTable, TableSender, table_digest};
use isocosm::Execution;
use isocosm_overlay::{EntityHandle, OrdinaryAccess, ParticipantHandle, Seat};
use isometry_campaign::native::pack::{self, PackEnvelope, PackInput, Definition, Choreography};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativePackResolved {
    pub expected_table: Option<String>, pub publication: NativePublication, pub table: NativeTable,
}

pub(crate) fn apply_native_pack(state: &mut GameSnapshot, result: &NativePackResolved) -> Result<(), GameError> {
    let fail = |reason: &str| GameError::Native(reason.into());
    let old = state.native_table.as_ref().ok_or_else(|| fail("pack has no admitted native table"))?;
    if table_digest(Some(old)) != result.expected_table || old.host_participant != result.table.host_participant
        || old.peers != result.table.peers || old.boards != result.table.boards || old.version != result.table.version
        || !result.publication.inventories.is_empty() || result.publication.activate.is_some() {
        return Err(fail("pack result changes endpoint/control or unrelated inventory bindings"));
    }
    let before = state.native.as_ref().ok_or_else(|| fail("pack has no native campaign"))?
        .load(Execution::Grouped).map_err(GameError::Native)?;
    let after = result.publication.world.load(Execution::Grouped).map_err(GameError::Native)?;
    if before.content()!=after.content() || before.origin()!=after.origin() {
        return Err(fail("pack result changes founding provenance or unrelated campaign content"));
    }
    old.validate_history(before.table_session()).map_err(GameError::Native)?;
    result.table.validate_history(after.table_session()).map_err(GameError::Native)?;
    let change = pack::validate_transition(before.table_session(),&old.game,after.table_session(),&result.table.game)
        .map_err(GameError::Native)?;
    if change.participant != old.host_participant { return Err(fail("pack decision has no actual table DM identity")); }
    let expected = boards(&result.table.game,change.activated.as_deref()).map_err(GameError::Native)?;
    if expected != result.publication.boards { return Err(fail("pack map metadata differs from accepted authored content")); }
    for map in &expected {
        if state.maps.get(&map.id).is_some_and(|old| old != map) {
            return Err(fail("pack cannot replace a differing existing board without explicit map edit admission"));
        }
    }
    let mut next = state.clone();
    super::native::apply_publication(&mut next,&result.publication)?;
    next.native_table = Some(result.table.clone());
    *state = next;
    Ok(())
}

fn boards(game: &[u8], activated: Option<&str>) -> Result<Vec<isometry_campaign::CampaignMap>, String> {
    let Some(key) = activated else { return Ok(Vec::new()); };
    let bindings = isometry_campaign::native::GameBindings::decode(game)?;
    let definition = &bindings.catalogue().and_then(|c| c.packs.get(key)).ok_or("activated pack catalogue is absent")?.definition;
    definition.content.as_ref().map(|content| content.lowered_maps().map_err(|e| e.to_string())).transpose()
        .map(|maps| maps.unwrap_or_default())
}

impl HostSession {
    pub fn native_pack(&mut self, sender: TableSender, input: &PackEnvelope) -> Result<Vec<Outbound>, String> {
        if !matches!(sender,TableSender::Host) { return Err("adventure pack decisions require the actual DM host".into()); }
        let world = self.state.native.as_ref().ok_or("pack needs an accepted native campaign")?;
        let table = self.state.native_table.as_ref().ok_or("pack needs an explicit native DM binding")?;
        let campaign = world.load(Execution::Grouped)?;
        let session = campaign.table_session();
        table.validate_history(session)?;
        let participant = table.participant(sender)?;
        if input.participant != participant || input.tick != session.sim.state().tick {
            return Err("pack request differs from actual DM participant or native tick".into());
        }
        let access = OrdinaryAccess { participant:ParticipantHandle(participant),seat:Seat::Dm,
            subject:session.sim.plays(participant).map(EntityHandle) };
        let private_ids: BTreeSet<_> = self.campaign.secret_ids().map(String::from).collect();
        let (admitted,game,_) = campaign.with_table_pack(table.game.clone(),access,&private_ids,input)?;
        let activated = match &input.intent {
            PackInput::Accept { key,.. } | PackInput::Force { key,.. } => Some(key.as_str()), _ => None,
        };
        let mut private = self.campaign.clone();
        if let Some(key) = activated { private.activate_pack_secrets(key)?; }
        let mut publication = NativePublication::update(world,&admitted)?;
        publication.boards = boards(&game,activated)?;
        let mut next = table.clone();
        next.game = game;
        let result = NativePackResolved { expected_table:table_digest(Some(table)),publication,table:next };
        let out = self.try_commit(GameEvent::NativePackResolved(result))?;
        self.campaign = private;
        Ok(out)
    }

    pub fn register_generated_pack(&mut self, record: &isometry_campaign::GenerationRecord,
        requirements: isometry_campaign::StoryletRequirements, roles: Vec<isometry_campaign::RoleSlot>,
        choreography: Vec<Choreography>, max_depth: usize) -> Result<Vec<Outbound>, String> {
        let (definition,secrets) = Definition::from_generation(record,requirements,roles,choreography,max_depth)?;
        let table = self.state.native_table.as_ref().ok_or("generated pack needs an admitted native DM binding")?;
        let campaign = self.state.native.as_ref().ok_or("pack needs native world")?.load(Execution::Grouped)?;
        let input = PackEnvelope { participant:table.host_participant,tick:campaign.table_session().sim.state().tick,
            intent:PackInput::Register { definition:definition.clone() } };
        let mut private = self.campaign.clone();
        private.stage_pack_secrets(&definition.key,&secrets)?;
        let out = self.native_pack(TableSender::Host,&input)?;
        self.campaign = private;
        Ok(out)
    }
}

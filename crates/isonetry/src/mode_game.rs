// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

//! VTT-owned mode bytes. Native history stays exclusively in the host's world save.

use crate::GameSnapshot;
use isocosm::Session;
use isometry_campaign::GenerationRecord;
use isometry_campaign::founding::{Content, GenerationOrigin, NativeWorld};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use isometry_core::{Beat, MapDocument, RollRecord, TokenId, TurnList};
use isometry_campaign::{Inventory, WorldFact};
use isometry_campaign::CampaignMap;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModeGame {
    version: u32,
    pub content: Content,
    pub origin: Option<GenerationOrigin>,
    table: Table,
}

/// Table state has no native Session or saved history. Explicit bindings must
/// still authenticate against that borrowed Session before standalone use.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Table {
    map: MapDocument,
    turns: TurnList,
    roll_log: Vec<RollRecord>,
    journal: Vec<WorldFact>,
    inventories: BTreeMap<TokenId, Inventory>,
    generations: Vec<GenerationRecord>,
    maps: BTreeMap<String, CampaignMap>,
    active_map: Option<String>,
    clocks: BTreeMap<String, u64>,
    party_cap: u32,
    last_beats: Vec<Beat>,
    beat_seq: u64,
    applied_actions: BTreeSet<crate::RequestId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    native_table: Option<crate::NativeTable>,
}

impl ModeGame {
    pub fn capture(state: &GameSnapshot, content: Content, origin: Option<GenerationOrigin>) -> Self {
        Self { version: 1, content, origin, table: Table {
            map: state.map.clone(), turns: state.turns.clone(), roll_log: state.roll_log.clone(),
            journal: state.journal.clone(), inventories: state.inventories.clone(),
            // Campaign assertion streams are native history, not a per-mode save authority.
            generations: state.generations.iter().filter(|record|
                !matches!(record.proposal, isometry_campaign::GenValue::Campaign { .. }))
                .cloned().collect(),
            maps: state.maps.clone(), active_map: state.active_map.clone(), clocks: state.clocks.clone(),
            party_cap: state.party_cap, last_beats: state.last_beats.clone(), beat_seq: state.beat_seq,
            applied_actions: state.applied_actions.clone(),
            native_table: state.native_table.clone(),
        } }
    }

    pub fn encode(&self) -> Result<Vec<u8>, String> {
        serde_json::to_vec(self).map_err(|e| e.to_string())
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, String> {
        let game: Self = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
        if game.version != 1 { return Err("unsupported VTT game section version".into()); }
        game.content.validate().map_err(|e| e.to_string())?;
        if game.table.generations.iter().any(|record|
            matches!(record.proposal, isometry_campaign::GenValue::Campaign { .. }))
        {
            return Err("VTT game section contains a campaign assertion stream".into());
        }
        Ok(game)
    }

    /// Publish from the one borrowed native Session. Mode bytes grant no control.
    pub fn snapshot(&self, session: &Session) -> Result<GameSnapshot, String> {
        let table = &self.table;
        let snapshot = GameSnapshot {
            map: table.map.clone(), turns: table.turns.clone(), roll_log: table.roll_log.clone(),
            journal: table.journal.clone(), inventories: table.inventories.clone(),
            generations: table.generations.clone(), maps: table.maps.clone(),
            active_map: table.active_map.clone(), world: Default::default(),
            native: Some(NativeWorld::from_session(session, self.content.clone(), self.origin.clone())?),
            native_table: table.native_table.clone(),
            clocks: table.clocks.clone(), party_cap: table.party_cap, last_beats: table.last_beats.clone(),
            beat_seq: table.beat_seq, applied_actions: table.applied_actions.clone(),
        };
        if let Some(bindings) = &snapshot.native_table { bindings.validate_history(session)?; }
        Ok(snapshot)
    }
}

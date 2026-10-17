// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;
use isocosm::{Execution, schema::{Id, Ledger}};
use isocosm_vtt::founding::{Campaign, NativeCursor, NativeWorld};
use isometry_campaign::Inventory;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// One public native result, plus inert board results already resolved by the host.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NativePublication {
    pub expected: Option<NativeCursor>,
    pub world: NativeWorld,
    pub boards: Vec<isometry_campaign::CampaignMap>,
    pub inventories: BTreeMap<TokenId, Inventory>,
    pub activate: Option<String>,
}

impl NativePublication {
    pub fn update(previous: &NativeWorld, next: &Campaign) -> Result<Self, String> {
        let world = NativeWorld::from_campaign(next)?;
        world.validate_successor(previous)?;
        Ok(Self {
            expected: Some(previous.cursor()?), world,
            boards: Vec::new(), inventories: BTreeMap::new(), activate: None,
        })
    }
}

pub(crate) fn apply_publication(
    state: &mut GameSnapshot,
    publication: &NativePublication,
) -> Result<(), GameError> {
    let actual = state.native.as_ref().map(NativeWorld::cursor)
        .transpose().map_err(GameError::Native)?;
    if actual != publication.expected {
        return Err(GameError::Native("native publication has a stale parent".into()));
    }
    publication.world.load(Execution::Grouped).map_err(GameError::Native)?;
    if let Some(previous) = &state.native {
        publication.world.validate_successor(previous).map_err(GameError::Native)?;
    }
    let mut next = state.clone();
    for board in &publication.boards {
        apply_game(&mut next, &GameEvent::MapStored(board.clone()))?;
    }
    for (&token, inventory) in &publication.inventories {
        apply_game(&mut next, &GameEvent::InventorySet {
            token, inventory: inventory.clone(),
        })?;
    }
    if let Some(id) = &publication.activate {
        apply_game(&mut next, &GameEvent::MapActivated { id: id.clone() })?;
    }
    next.native = Some(publication.world.clone());
    // Authored fills live in native state; table migration metadata remains in Content.
    next.world = Default::default();
    *state = next;
    Ok(())
}

/// Current native member result. Journey 817 must rebind `resident` before release.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TravelMember {
    pub resident: Option<Id>,
    pub arrived: u64,
    pub alive: bool,
    pub accounts: Ledger,
    pub traits: BTreeSet<String>,
}

/// Every portable consequence is an absolute result, never a receiver inference.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TravelTableResult {
    pub map: isometry_core::MapDocument,
    pub turns: isometry_core::TurnList,
    pub boards: BTreeMap<String, isometry_campaign::CampaignMap>,
    pub clocks: BTreeMap<String, u64>,
    pub active: Option<String>,
    pub inventories: BTreeMap<TokenId, Inventory>,
    pub rolls: Vec<isometry_core::RollRecord>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NativeTravelResolved {
    pub request: RequestId,
    pub publication: NativePublication,
    pub members: BTreeMap<Id, TravelMember>,
    pub table: TravelTableResult,
}

fn members(campaign: &Campaign, ids: &BTreeSet<Id>) -> Result<BTreeMap<Id, TravelMember>, String> {
    let reading = campaign.reading();
    ids.iter().map(|&id| {
        let entity = reading.state().population.get(id).ok_or("unknown travel member")?;
        Ok((id, TravelMember {
            resident: entity.resident_site(), arrived: entity.arrived, alive: entity.alive,
            accounts: entity.accounts.clone(), traits: entity.traits.clone(),
        }))
    }).collect()
}

pub(crate) fn apply_native_travel(
    state: &mut GameSnapshot,
    result: &NativeTravelResolved,
) -> Result<(), GameError> {
    if state.applied_actions.contains(&result.request) { return Ok(()); }
    let campaign = result.publication.world.load(Execution::Grouped).map_err(GameError::Native)?;
    let ids = result.members.keys().copied().collect();
    if members(&campaign, &ids).map_err(GameError::Native)? != result.members {
        return Err(GameError::Native("travel members differ from admitted native result".into()));
    }
    let mut next = state.clone();
    apply_publication(&mut next, &result.publication)?;
    for (id, board) in &result.table.boards {
        if id != &board.id { return Err(GameError::Native("travel board key mismatch".into())); }
    }
    if result.table.active.as_ref().is_some_and(|id| !result.table.boards.contains_key(id)) {
        return Err(GameError::Native("travel active board is absent".into()));
    }
    if result.table.active.as_ref().is_some_and(|id|
        result.table.boards[id].document != result.table.map)
    {
        return Err(GameError::Native("travel active document differs from its board".into()));
    }
    next.maps.clear();
    for board in result.table.boards.values() {
        apply_game(&mut next, &GameEvent::MapStored(board.clone()))?;
    }
    next.map = result.table.map.clone();
    next.turns = result.table.turns.clone();
    for &token in next.turns.entries() { super::apply::require_token(&next, token)?; }
    next.clocks = result.table.clocks.clone();
    next.active_map = result.table.active.clone();
    next.inventories.clear();
    for (&token, inventory) in &result.table.inventories {
        apply_game(&mut next, &GameEvent::InventorySet { token, inventory: inventory.clone() })?;
    }
    for roll in &result.table.rolls { super::apply::push_roll(&mut next, roll); }
    next.applied_actions.insert(result.request);
    *state = next;
    Ok(())
}

impl HostSession {
    /// Inputs stay host-side; peers receive only the one admitted native result.
    pub fn commit_native_travel(
        &mut self,
        request: RequestId,
        commands: &[isocosm::Command],
        ticks: u64,
        travelers: &BTreeSet<Id>,
        table_result: impl FnOnce(&Campaign) -> Result<TravelTableResult, String>,
    ) -> Result<Vec<Outbound>, String> {
        if self.state.applied_actions.contains(&request) { return Ok(Vec::new()); }
        let previous = self.state.native.as_ref().ok_or("no native campaign")?;
        let old = previous.load(Execution::Grouped)?;
        let admitted = old.with_commands(commands, ticks).map_err(|e| e.to_string())?;
        let result = NativeTravelResolved {
            request,
            publication: NativePublication::update(previous, &admitted)?,
            members: members(&admitted, travelers)?,
            table: table_result(&admitted)?,
        };
        self.try_commit(GameEvent::NativeTravelResolved(result))
    }
}

// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Whole physical storylet publication; private eligibility stays on the host.

use super::*;
use crate::native_table::{NativeTable, table_digest};
use cosmograph::Execution;
use cosmograph_overlay::{EntityHandle, OrdinaryAccess, ParticipantHandle, Seat};
use isocosm_campaign::StoryletProposal;
use isocosm_native::{founding::{Campaign, NativeWorld}, native::{GameBindings, storylets::*}};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeComposedStoryletResolved {
    pub expected_table: Option<String>,
    pub publication: NativePublication,
    pub table: NativeTable,
    pub input: ComposedStoryletInput,
    pub public: ComposedStoryletPublic,
    pub item_owner: Option<TokenId>,
}

fn access(campaign: &Campaign, table: &NativeTable) -> OrdinaryAccess {
    OrdinaryAccess { participant: ParticipantHandle(table.host_participant), seat: Seat::Dm,
        subject: campaign.table_session().sim.plays(table.host_participant).map(EntityHandle) }
}

fn offers(campaign: &Campaign, table: &NativeTable) -> Result<BTreeMap<String, StoryletProposal>, String> {
    table.validate_history(campaign.table_session())?;
    let mut offers = campaign.content().storylets.clone();
    for (key, storylet) in isocosm_native::native::pack::active_content(
        campaign.table_session(), &table.game, access(campaign, table))?.storylets {
        if offers.get(&key).is_some_and(|old| old != &storylet) {
            return Err(format!("native campaign and accepted pack disagree about storylet {key}"));
        }
        offers.insert(key, storylet);
    }
    Ok(offers)
}

fn readmit(state: &GameSnapshot, input: &ComposedStoryletInput, public: &ComposedStoryletPublic,
    item_owner: Option<TokenId>) -> Result<NativeComposedStoryletResolved, String> {
    let world = state.native.as_ref().ok_or("physical storylet needs a native world")?;
    let table = state.native_table.as_ref().ok_or("physical storylet needs its admitted host table")?;
    let campaign = world.load(Execution::Grouped)?;
    let offers = offers(&campaign, table)?;
    let source = offers.get(&input.storylet.key).ok_or("physical storylet offer is absent")?;
    // Source/effects are public campaign data. Private eligibility is not a
    // replica inference, and no private cast or requirement list is carried.
    if input.storylet.participant != table.host_participant || input.storylet.expected_content != cosmograph::digest(&offers)
        || public.key != source.key || public.source_digest != cosmograph::digest(source)
        || public.effects != source.effects || !public.cast.is_empty() {
        return Err("physical storylet differs from its host cut or minimal public source projection".into());
    }
    let mut candidate = campaign.table_session().clone();
    let (_, game) = readmit_composed(&mut candidate, table.game.clone(), access(&campaign, table), input, public)?;
    let bindings = GameBindings::decode(&game)?;
    let record = bindings.composed_storylets().last().ok_or("physical storylet record is absent")?;
    if record.input != *input || record.public != *public { return Err("physical storylet record differs from its input".into()); }
    let mut boards = Vec::new();
    let mut inventories = BTreeMap::new();
    for (effect, value) in public.effects.iter().enumerate() {
        match value {
            StoryletEffect::LocalMap { map } => boards.push(map.lower(MapScale::Local)
                .map_err(|error| format!("storylet inert map proposal is invalid: {error}"))?),
            StoryletEffect::Item { item } => {
                let owner = item_owner.ok_or("storylet item proposal needs an explicitly selected table recipient")?;
                let token = table.active(state)?.tokens.iter().find(|held| held.token == owner)
                    .ok_or("storylet item proposal recipient has no admitted character binding")?;
                if !bindings.characters().contains_key(&token.character) { return Err("storylet proposal recipient has no admitted recipe".into()); }
                let named = input.steps.iter().find_map(|step| match step {
                    StoryletStep::Item { effect: at, item, .. } if *at as usize == effect => Some(item), _ => None,
                }).ok_or("storylet item has no explicit bound instance identity")?;
                let inventory = inventories.entry(owner).or_insert_with(|| state.inventories.get(&owner).cloned().unwrap_or_default());
                inventory.insert(ItemInstance { id: named.clone(), template: item.template.clone(), name: item.name.clone(),
                    quantity: 1, tags: item.tags.clone(), modifiers: Vec::new(), appearance_layers: Vec::new() })
                    .map_err(|error| format!("storylet inert item proposal is invalid: {error:?}"))?;
            },
            _ => {},
        }
    }
    let next_world = NativeWorld::from_session(&candidate, campaign.content().clone(), campaign.origin().cloned())?;
    next_world.validate_successor(world)?;
    let publication = NativePublication { expected: Some(world.cursor()?), world: next_world,
        boards, inventories, activate: None };
    let mut next_table = table.clone(); next_table.game = game;
    Ok(NativeComposedStoryletResolved { expected_table: table_digest(Some(table)), publication,
        table: next_table, input: input.clone(), public: public.clone(), item_owner })
}

pub(crate) fn apply_native_composed_storylet(state: &mut GameSnapshot, result: &NativeComposedStoryletResolved)
    -> Result<(), GameError> {
    let expected = readmit(state, &result.input, &result.public, result.item_owner).map_err(GameError::Native)?;
    if expected != *result { return Err(GameError::Native("physical storylet differs from exact native/product effect admission".into())); }
    let mut candidate = state.clone();
    super::native::apply_publication(&mut candidate, &result.publication)?;
    candidate.native_table = Some(result.table.clone());
    for effect in &result.public.effects {
        if let StoryletEffect::Fact { fact } = effect { apply_game(&mut candidate, &GameEvent::Fact(fact.clone()))?; }
    }
    *state = candidate;
    Ok(())
}

impl HostSession {
    pub fn quote_composed_storylet(&self, input: &ComposedStoryletInput, item_owner: Option<TokenId>)
        -> Result<NativeComposedStoryletResolved, String> {
        let campaign = self.state.native.as_ref().ok_or("physical storylet needs a native campaign")?.load(Execution::Grouped)?;
        let table = self.state.native_table.as_ref().ok_or("physical storylet needs an admitted table")?;
        let offers = offers(&campaign, table)?;
        let source = offers.get(&input.storylet.key).ok_or("physical storylet offer is absent")?;
        let resolved = campaign.reading().resolve_storylet(source, self.campaign.secret_ids())
            .map_err(|error| format!("storylet does not match host-private/native conditions: {error:?}"))?;
        let mut public = public_outputs(source, resolved);
        public.cast.clear(); // Omitted private cast is not an assertion that no cast exists.
        readmit(&self.state, input, &public, item_owner)
    }

    pub fn commit_composed_storylet(&mut self, input: &ComposedStoryletInput, item_owner: Option<TokenId>)
        -> Result<Vec<Outbound>, String> {
        let result = self.quote_composed_storylet(input, item_owner)?;
        self.try_commit(GameEvent::NativeComposedStoryletResolved(result))
    }
}

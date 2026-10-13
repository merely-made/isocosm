// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Native storylet admission with table-only proposals in one published batch.

use super::*;
use isocosm::{Execution, asserted::{Applied, Assertion, Fact}};
use isometry_campaign::Inventory;
use std::collections::BTreeMap;

impl HostSession {
    pub(super) fn commit_native_storylet(&mut self, key: &str, item_owner: Option<TokenId>)
        -> Result<Vec<Outbound>, String> {
        let previous = self.state.native.as_ref().ok_or("no native campaign")?;
        let old = previous.load(Execution::Grouped)?;
        let mut offers = old.content().storylets.clone();
        if let Some(table) = &self.state.native_table {
            table.validate_history(old.table_session())?;
            let access = isocosm_overlay::OrdinaryAccess {
                participant:isocosm_overlay::ParticipantHandle(table.host_participant),seat:isocosm_overlay::Seat::Dm,
                subject:old.table_session().sim.plays(table.host_participant).map(isocosm_overlay::EntityHandle) };
            for (key,storylet) in isometry_campaign::native::pack::active_content(old.table_session(),&table.game,access)?.storylets {
                if offers.get(&key).is_some_and(|existing| existing != &storylet) {
                    return Err(format!("native campaign and accepted pack disagree about storylet {key}"));
                }
                offers.insert(key,storylet);
            }
        }
        let storylet = offers.get(key).ok_or_else(|| format!("unknown native storylet: {key}"))?;
        let resolved = old.reading().resolve_storylet(storylet, self.campaign.secret_ids())
            .map_err(|error| format!("storylet does not match native state: {error:?}"))?;
        let occurrence = format!("storylet:{key}:{}", old.reading().entries().len());
        let mut asserts = Vec::new();
        let mut facts = Vec::new();
        let mut boards = Vec::new();
        let mut inventories = BTreeMap::<TokenId, Inventory>::new();
        for (index, effect) in resolved.effects.into_iter().enumerate() {
            match effect {
                StoryletEffect::Fact { fact } => {
                    asserts.push(Assertion::Fact(Fact { about:fact.id.clone(),
                        key:format!("{occurrence}:fact:{index}"), kind:fact.kind.clone(),
                        text:fact.text.clone(), tags:fact.tags.iter().cloned().collect() }));
                    facts.push(fact);
                },
                StoryletEffect::History { event } => asserts.push(Assertion::History(event)),
                StoryletEffect::LocalMap { map } => boards.push(map.lower(MapScale::Local)
                    .map_err(|error| format!("storylet map is invalid: {error}"))?),
                StoryletEffect::Item { item } => {
                    let owner = item_owner.ok_or("select an explicitly bound token for the storylet item")?;
                    let table = self.state.native_table.as_ref().ok_or("storylet item needs explicit native table bindings")?;
                    table.validate(&self.state, old.table_session())?;
                    let binding = table.active(&self.state)?.tokens.iter().find(|held| held.token == owner)
                        .ok_or("selected storylet item recipient has no explicit native character binding")?;
                    let game = isometry_campaign::native::GameBindings::decode(&table.game)?;
                    if !game.characters().contains_key(&binding.character) {
                        return Err("storylet item recipient has no admitted native recipe binding".into());
                    }
                    let inventory = inventories.entry(owner)
                        .or_insert_with(|| self.state.inventories.get(&owner).cloned().unwrap_or_default());
                    let mut id = ItemId::new(format!("storylet.{key}.{index}"));
                    let mut nonce = 1;
                    while inventory.items.contains_key(&id) {
                        id = ItemId::new(format!("storylet.{key}.{index}.{nonce}"));
                        nonce += 1;
                    }
                    // The existing authored table proposal remains metadata;
                    // it neither issues native stocks nor fabricates a body.
                    inventory.insert(ItemInstance { id, template:item.template, name:item.name,
                        quantity:1, tags:item.tags, modifiers:Vec::new(), appearance_layers:Vec::new() })
                        .map_err(|error| format!("storylet item is invalid: {error:?}"))?;
                },
            }
        }
        let admitted = old.with_assertions(&[Assertion::Storylet(Applied {
            key:occurrence, source:storylet.key.clone(), asserts,
        })]).map_err(|error| error.to_string())?;
        let mut publication = super::native::NativePublication::update(previous, &admitted)?;
        publication.boards = boards;
        publication.inventories = inventories;
        let mut candidate = Self::with_history(self.state.clone(), self.campaign.clone(), self.history.clone());
        let mut out = candidate.try_commit(GameEvent::NativePublished(publication))?;
        for fact in facts { out.extend(candidate.try_commit(GameEvent::Fact(fact))?); }
        // Peer attribution, pending requests and private custody stay on this
        // authority. Only a fully admitted public state/history is published.
        self.state = candidate.state;
        self.history = candidate.history;
        self.seq = candidate.seq;
        self.log_hash = candidate.log_hash;
        Ok(out)
    }
}

// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;
use isocosm::Execution;
use isometry_campaign::founding::{Campaign, NativeWorld};
use std::collections::BTreeMap;

impl HostSession {
    /// Found from the selected native profile and an already inspected pack result.
    pub fn commit_native_campaign(
        &mut self,
        profile: &isocosm::Founding,
        record: &GenerationRecord,
        item_owner: Option<TokenId>,
        mode: Execution,
    ) -> Result<Vec<Outbound>, String> {
        if self.state.native.is_some() { return Err("native campaign already founded".into()); }
        let prepared = Campaign::from_profile(profile, record, 16, mode)
            .map_err(|e| e.to_string())?;
        self.publish_native_founding(prepared, item_owner)
    }

    /// Only the inspected native preview may be published by this host.
    pub fn commit_native_setup(
        &mut self, setup: &isometry_campaign::founding::Setup, expected: &NativeWorld,
        record: &GenerationRecord, item_owner: Option<TokenId>,
    ) -> Result<Vec<Outbound>, String> {
        if self.state.native.is_some() { return Err("native campaign already founded".into()); }
        let prepared = Campaign::from_setup(setup, record, 16).map_err(|e| e.to_string())?;
        if &NativeWorld::from_campaign(&prepared.campaign)? != expected {
            return Err("native founding preview changed before acceptance".into());
        }
        self.publish_native_founding(prepared, item_owner)
    }

    fn publish_native_founding(
        &mut self, prepared: isometry_campaign::founding::Prepared, item_owner: Option<TokenId>,
    ) -> Result<Vec<Outbound>, String> {
        let mut private = self.campaign.clone();
        for secret in prepared.private.secrets() {
            if private.secret(&secret.id).is_some_and(|old| old != secret) {
                return Err(format!("conflicting private campaign secret: {}", secret.id));
            }
            private.insert_secret(secret.clone());
        }
        let campaign = &prepared.campaign;
        let boards = campaign.content().lowered_maps().map_err(|e| e.to_string())?;
        let mut inventories = BTreeMap::new();
        if !campaign.content().rewards.is_empty() {
            let owner = item_owner.ok_or("campaign reward needs a character owner")?;
            let mut inventory = self.state.inventories.get(&owner).cloned().unwrap_or_default();
            for (index, item) in campaign.content().rewards.iter().enumerate() {
                inventory.insert(ItemInstance {
                    id: ItemId::new(format!("campaign.{}.reward.{index}", campaign.content().id)),
                    template: item.template.clone(), name: item.name.clone(), quantity: 1,
                    tags: item.tags.clone(), modifiers: Vec::new(), appearance_layers: Vec::new(),
                }).map_err(|e| format!("campaign reward is invalid: {e:?}"))?;
            }
            inventories.insert(owner, inventory);
        }
        let publication = super::native::NativePublication {
            expected: None, world: NativeWorld::from_campaign(campaign)?,
            boards, inventories, activate: Some(campaign.content().starting_map.clone()),
        };
        let out = self.try_commit(GameEvent::NativePublished(publication))?;
        self.campaign = private;
        Ok(out)
    }

    pub fn commit_native_assertions(
        &mut self,
        assertions: &[isocosm_overlay::vtt::Assertion],
    ) -> Result<Vec<Outbound>, String> {
        let previous = self.state.native.as_ref().ok_or("no native campaign")?;
        let old = previous.load(Execution::Grouped)?;
        let admitted = old.with_contract_assertions(assertions).map_err(|e| e.to_string())?;
        let publication = super::native::NativePublication::update(previous, &admitted)?;
        self.try_commit(GameEvent::NativePublished(publication))
    }
}
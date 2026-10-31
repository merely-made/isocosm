// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;
use cosmograph::Execution;
use isocosm_native::founding::{Campaign, NativeWorld};
use cosmograph_host::founding::{Offered, Purpose};
use std::collections::BTreeMap;

impl HostSession {
    /// Found from the selected native profile and an already inspected pack result.
    pub fn commit_native_campaign(
        &mut self,
        profile: &cosmograph::Founding,
        record: &GenerationRecord,
        item_owner: Option<TokenId>,
        mode: Execution,
    ) -> Result<Vec<Outbound>, String> {
        self.commit_native_campaign_for(
            profile,
            record,
            item_owner,
            mode,
            Purpose::Head,
            None,
        )
    }

    pub fn commit_native_campaign_for(
        &mut self,
        profile: &cosmograph::Founding,
        record: &GenerationRecord,
        item_owner: Option<TokenId>,
        mode: Execution,
        purpose: Purpose,
        offered: Option<Offered<'_>>,
    ) -> Result<Vec<Outbound>, String> {
        if self.state.native.is_some() { return Err("native campaign already founded".into()); }
        let prepared = Campaign::from_profile_for(profile, record, 16, mode, purpose, offered)
            .map_err(|e| e.to_string())?;
        self.publish_native_founding(prepared, item_owner)
    }

    /// Found a quick-start pack's world in one step (case 1023; Mark: "Picker + packs from disk"): `pack`
    /// is the datasheet text, carried or read from a content pack's folder.
    pub fn commit_native_quickstart(
        &mut self, pack: &str, seed: u64, mode: Execution,
    ) -> Result<Vec<Outbound>, String> {
        if self.state.native.is_some() { return Err("native campaign already founded".into()); }
        let prepared = Campaign::from_quickstart_for(pack, seed, mode, Purpose::Head, None)
            .map_err(|e| e.to_string())?;
        self.publish_native_founding(prepared, None)
    }

    /// Only the inspected native preview may be published by this host.
    pub fn commit_native_setup(
        &mut self, setup: &isocosm_native::founding::Setup, expected: &NativeWorld,
        record: &GenerationRecord, item_owner: Option<TokenId>,
    ) -> Result<Vec<Outbound>, String> {
        self.commit_native_setup_for(
            setup,
            expected,
            record,
            item_owner,
            Purpose::Head,
            None,
        )
    }

    pub fn commit_native_setup_for(
        &mut self, setup: &isocosm_native::founding::Setup, expected: &NativeWorld,
        record: &GenerationRecord, item_owner: Option<TokenId>,
        purpose: Purpose, offered: Option<Offered<'_>>,
    ) -> Result<Vec<Outbound>, String> {
        if self.state.native.is_some() { return Err("native campaign already founded".into()); }
        let prepared = Campaign::from_setup_for(setup, record, 16, purpose, offered)
            .map_err(|e| e.to_string())?;
        if &NativeWorld::from_campaign(&prepared.campaign)? != expected {
            return Err("native founding preview changed before acceptance".into());
        }
        self.publish_native_founding(prepared, item_owner)
    }

    fn publish_native_founding(
        &mut self, prepared: isocosm_native::founding::Prepared, item_owner: Option<TokenId>,
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
                    tags: item.tags.clone(), modifiers: Vec::new(), appearance_layers: Vec::new(), material: None, make: None, condition: None,
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
        assertions: &[cosmograph_overlay::vtt::Assertion],
    ) -> Result<Vec<Outbound>, String> {
        let previous = self.state.native.as_ref().ok_or("no native campaign")?;
        let old = previous.load(Execution::Grouped)?;
        let admitted = old.with_contract_assertions(assertions).map_err(|e| e.to_string())?;
        let publication = super::native::NativePublication::update(previous, &admitted)?;
        self.try_commit(GameEvent::NativePublished(publication))
    }
}

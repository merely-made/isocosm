// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::FoundingError;
use isometry_campaign::{CampaignMap, LowerDraftMap};
use isometry_campaign::{CampaignDraft, DraftMap, ItemProposal, StoryletProposal};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Existing table edition data retained for its consumer migration.
/// These fields never seed native reach, polity control or body placement.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TableMetadata {
    pub faction_sheets: BTreeMap<String, BTreeMap<String, i64>>,
    pub faction_control: BTreeMap<String, String>,
    pub party_node: BTreeMap<String, String>,
    pub party_pace: BTreeMap<String, i64>,
    pub party_known: BTreeMap<String, BTreeSet<String>>,
    pub party_resources: BTreeMap<String, BTreeMap<String, i64>>,
}

/// The editions a campaign may be founded under, by system key; the ruleset
/// adapter admits each.
pub const EDITIONS: [&str; 2] = ["5e-srd", "pf2e-srd"];

pub fn default_edition() -> String {
    EDITIONS[0].into()
}

pub fn is_default_edition(edition: &String) -> bool {
    edition == EDITIONS[0]
}

/// Authored offers and map proposals; asserted nouns live only in native history.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Content {
    pub id: String,
    pub name: String,
    pub maps: Vec<DraftMap>,
    pub storylets: BTreeMap<String, StoryletProposal>,
    pub rewards: Vec<ItemProposal>,
    pub starting_map: String,
    pub final_storylet: String,
    pub table: TableMetadata,
    /// The edition the campaign is founded under (case 1029): a system key
    /// the ruleset adapter admits. Absent in older saves, which are 5e.
    #[serde(default = "default_edition",
        skip_serializing_if = "is_default_edition")]
    pub edition: String,
}

impl Content {
    pub(crate) fn from_draft(draft: &CampaignDraft) -> Result<Self, FoundingError> {
        let world = &draft.world;
        let content = Self {
            id: draft.id.clone(), name: draft.name.clone(), maps: draft.maps.clone(),
            storylets: world.storylets.clone(), rewards: draft.rewards.clone(),
            starting_map: draft.starting_map.clone(), final_storylet: draft.final_storylet.clone(),
            table: TableMetadata {
                faction_sheets: world.faction_sheets.clone(),
                faction_control: world.faction_control.clone(),
                party_node: world.party_node.clone(), party_pace: world.party_pace.clone(),
                party_known: world.party_known.clone(), party_resources: world.party_resources.clone(),
            },
            edition: default_edition(),
        };
        content.validate()?;
        Ok(content)
    }

    pub fn validate(&self) -> Result<(), FoundingError> {
        let invalid = |reason: String| FoundingError::Content(reason);
        if self.id.trim().is_empty() { return Err(invalid("missing campaign id".into())); }
        if !EDITIONS.contains(&self.edition.as_str()) {
            return Err(invalid(format!("unknown edition {}", self.edition)));
        }
        let mut ids = BTreeSet::new();
        for map in &self.maps {
            if !ids.insert(map.map.id.as_str()) {
                return Err(invalid(format!("duplicate map {}", map.map.id)));
            }
            map.validate().map_err(|e| invalid(format!("map {}: {e}", map.map.id)))?;
        }
        if !ids.contains(self.starting_map.as_str()) {
            return Err(invalid(format!("missing starting map {}", self.starting_map)));
        }
        if !self.storylets.contains_key(&self.final_storylet) {
            return Err(invalid(format!("missing final storylet {}", self.final_storylet)));
        }
        for (key, storylet) in &self.storylets {
            if key != &storylet.key || key.trim().is_empty() {
                return Err(invalid(format!("invalid storylet key {key}")));
            }
        }
        Ok(())
    }

    pub fn lowered_maps(&self) -> Result<Vec<CampaignMap>, FoundingError> {
        self.validate()?;
        self.maps.iter().map(|map| map.lower()
            .map_err(|e| FoundingError::Content(format!("map {}: {e}", map.map.id)))).collect()
    }
}

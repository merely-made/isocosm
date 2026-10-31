// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::{Campaign, CampaignSaved};
use cosmograph::{Execution, history::Saved};
use serde::{Deserialize, Deserializer, Serialize, de::Error};

/// Native identity at an accepted prefix, independent of table aliases.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativeCursor {
    pub genesis: String,
    pub branch: String,
    pub entries: u64,
    pub tick: u64,
    pub state_hash: u64,
}

impl NativeCursor {
    fn of(saved: &Saved) -> Self {
        Self {
            genesis: saved.genesis_digest.clone(),
            branch: saved.branch.clone(),
            entries: saved.entries.len() as u64,
            tick: saved.tick,
            state_hash: saved.state_hash,
        }
    }
}

/// Native replay data in a wire-safe envelope; never a legacy world fold.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct NativeWorld(Vec<u8>);

impl<'de> Deserialize<'de> for NativeWorld {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let bytes = Vec::<u8>::deserialize(deserializer)?;
        let world = Self(bytes);
        world.load(Execution::Grouped).map_err(D::Error::custom)?;
        Ok(world)
    }
}

impl NativeWorld {
    /// Snapshot publication borrows the common host; it keeps no second Session.
    pub fn from_session(
        session: &cosmograph::Session, content: super::Content,
        origin: Option<super::GenerationOrigin>,
    ) -> Result<Self, String> {
        content.validate().map_err(|e| e.to_string())?;
        if let Some(value) = &origin { value.validate().map_err(|e| e.to_string())?; }
        serde_json::to_vec(&CampaignSaved { native: session.save(), content, origin })
            .map(Self).map_err(|e| e.to_string())
    }

    pub fn from_campaign(campaign: &Campaign) -> Result<Self, String> {
        serde_json::to_vec(&campaign.save()).map(Self).map_err(|e| e.to_string())
    }

    fn saved(&self) -> Result<CampaignSaved, String> {
        serde_json::from_slice(&self.0).map_err(|e| e.to_string())
    }

    pub fn load(&self, mode: Execution) -> Result<Campaign, String> {
        Campaign::load(self.saved()?, mode).map_err(|e| e.to_string())
    }

    pub fn cursor(&self) -> Result<NativeCursor, String> {
        Ok(NativeCursor::of(&self.saved()?.native))
    }

    /// Read-only cache identity for the validated saved campaign content.
    pub fn content_digest(&self) -> Result<String, String> {
        Ok(cosmograph::digest(&self.saved()?.content))
    }

    /// A publication may append or advance, never replace an accepted prefix.
    pub fn validate_successor(&self, previous: &Self) -> Result<(), String> {
        let old = previous.saved()?;
        let next = self.saved()?;
        if old.native.genesis != next.native.genesis
            || old.native.branch != next.native.branch
            || next.native.tick < old.native.tick
            || !next.native.entries.starts_with(&old.native.entries)
        {
            return Err("native publication rewrites its accepted prefix".into());
        }
        if old.content != next.content || old.origin != next.origin {
            return Err("native publication changes immutable campaign content".into());
        }
        self.load(Execution::Grouped)?;
        Ok(())
    }
}

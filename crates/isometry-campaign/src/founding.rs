// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Native campaign founding; portable table content stays beside its history.

use crate::{CampaignStore, native::{ContractError, Reading, Replay, ReplayError}};
use isocosm::{Execution, asserted::{Applied, Assertion}, history::Saved, simulation::Genesis};
use crate::CampaignDraft;
use serde::{Deserialize, Serialize};
use std::fmt;

mod setup;
pub use setup::Setup;
mod publication;
pub use publication::{NativeCursor, NativeWorld};
mod content;
mod generation;
mod table;
pub use content::{Content, TableMetadata};
pub use generation::GenerationOrigin;

/// The VTT's declared vocabulary; other kinds remain the caller's founding choice.
pub fn table_genesis(mut genesis: Genesis) -> Genesis {
    for kind in ["reveal", "narration", "history"] {
        genesis.rules.note_kinds.insert(kind.into());
    }
    genesis
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FoundingError {
    Content(String),
    Native(ReplayError),
}

impl fmt::Display for FoundingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Content(reason) => write!(f, "campaign content: {reason}"),
            Self::Native(reason) => reason.fmt(f),
        }
    }
}
impl std::error::Error for FoundingError {}

/// One native authority plus authored map and table content.
#[derive(Clone, Debug)]
pub struct Campaign {
    replay: Replay,
    content: Content,
    origin: Option<GenerationOrigin>,
}

/// Public save carries the native history once, without a CampaignWorld fold.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CampaignSaved {
    pub native: Saved,
    pub content: Content,
    pub origin: Option<GenerationOrigin>,
}

/// Host staging keeps private facts separate from the public campaign save.
#[derive(Clone, Debug)]
pub struct Prepared {
    pub campaign: Campaign,
    pub private: CampaignStore,
}

impl Campaign {
    /// Caller supplies the complete native world; no native rates are selected here.
    pub fn from_draft(
        genesis: Genesis, draft: &CampaignDraft, mode: Execution,
    ) -> Result<Prepared, FoundingError> {
        let content = Content::from_draft(draft)?;
        let mut private = CampaignStore::new();
        for secret in &draft.secrets {
            if private.secret(&secret.id).is_some_and(|was| was != secret) {
                return Err(FoundingError::Content(format!("conflicting secret {}", secret.id)));
            }
            private.insert_secret(secret.clone());
        }
        let replay = Replay::from_assertions(table_genesis(genesis), draft.world.assertions(), mode)
            .map_err(FoundingError::Native)?;
        Ok(Prepared { campaign: Self { replay, content, origin: None }, private })
    }

    pub fn load(saved: CampaignSaved, mode: Execution) -> Result<Self, FoundingError> {
        saved.content.validate()?;
        if let Some(origin) = &saved.origin {
            origin.validate()?;
        }
        let replay = Replay::load(saved.native, mode).map_err(FoundingError::Native)?;
        Ok(Self { replay, content: saved.content, origin: saved.origin })
    }

    pub fn save(&self) -> CampaignSaved {
        CampaignSaved { native: self.replay.save(), content: self.content.clone(),
            origin: self.origin.clone() }
    }

    pub fn reading(&self) -> Reading<'_> { self.replay.reading() }
    pub fn content(&self) -> &Content { &self.content }
    pub fn origin(&self) -> Option<&GenerationOrigin> { self.origin.as_ref() }

    pub fn with_assertions(&self, assertions: &[Assertion]) -> Result<Self, ReplayError> {
        let replay = self.replay.with_assertions(assertions)?;
        Ok(Self { replay, ..self.clone() })
    }

    pub fn with_contract_assertions(
        &self, assertions: &[isocosm_overlay::vtt::Assertion],
    ) -> Result<Self, ContractError> {
        let replay = self.replay.with_contract_assertions(assertions)?;
        Ok(Self { replay, ..self.clone() })
    }

    /// Native admission; table metadata grants no control.
    pub fn with_commands(
        &self, commands: &[isocosm::Command], ticks: isocosm::schema::Tick,
    ) -> Result<Self, ReplayError> {
        let replay = self.replay.with_commands(commands, ticks)?;
        Ok(Self { replay, ..self.clone() })
    }

    /// Explicit forced acceptance; ordinary pack requirements remain host decisions.
    pub fn force_pack(&self, applied: Applied) -> Result<Self, ReplayError> {
        self.with_assertions(&[Assertion::PackForced(applied)])
    }
}

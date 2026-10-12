// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::{Campaign, Content, FoundingError, Prepared, table_genesis};
use crate::{CampaignStore, native::{Replay, ReplayError, Phase}};
use isocosm::{Command, Execution, Founding, Session, asserted::Assertion, site_authoring::Attachment};
use crate::{GenValue, GenerationRecord};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Named operator configuration. Execution belongs to the configured host runtime.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Setup {
    pub name: String,
    pub founding: Founding,
    pub execution: Execution,
    pub attachments: BTreeMap<String, Attachment>,
}

impl Setup {
    pub fn decode(bytes: &[u8]) -> Result<Self, String> {
        serde_json::from_slice(bytes).map_err(|e| format!("world profile: {e}"))
    }
    pub fn replace_attachments(&mut self, bytes: &[u8]) -> Result<(), String> {
        let value = serde_json::from_slice(bytes).map_err(|e| format!("site attachments: {e}"))?;
        self.attachments = value;
        Ok(())
    }
}

impl Campaign {
    /// Inspect the selected profile and every attachment before publishing any state.
    pub fn from_setup(setup: &Setup, record: &GenerationRecord, depth: usize)
        -> Result<Prepared, FoundingError> {
        if setup.name.trim().is_empty() { return Err(FoundingError::Content("world profile has no name".into())); }
        record.validate(depth).map_err(|e| FoundingError::Content(e.to_string()))?;
        let GenValue::Campaign { campaign: draft } = &record.proposal else {
            return Err(FoundingError::Content("generation is not a campaign draft".into()));
        };
        let expected: BTreeSet<_> = draft.world.places().keys().cloned().collect();
        if expected != setup.attachments.keys().cloned().collect() {
            return Err(FoundingError::Content("attach every authored place exactly once".into()));
        }
        let genesis = setup.founding.generate().map_err(|reason|
            FoundingError::Native(ReplayError { phase: Phase::Founding, reason }))?;
        let mut session = Session::new(table_genesis(genesis), setup.execution).map_err(|reason|
            FoundingError::Native(ReplayError { phase: Phase::Founding, reason }))?;
        let content = Content::from_draft(draft)?;
        let mut private = CampaignStore::new();
        for secret in &draft.secrets {
            if private.secret(&secret.id).is_some_and(|was| was != secret) {
                return Err(FoundingError::Content(format!("conflicting secret {}", secret.id)));
            }
            private.insert_secret(secret.clone());
        }
        for (index, assertion) in draft.world.assertions().iter().enumerate() {
            let command = match assertion {
                Assertion::Place(place) => Command::AttachSite {
                    place: place.clone(), attachment: setup.attachments[&place.key].clone(),
                },
                other => Command::Assert(other.clone()),
            };
            session.command(command).map_err(|reason|
                FoundingError::Native(ReplayError { phase: Phase::Assertion { index }, reason }))?;
        }
        let replay = Replay::load(session.save(), setup.execution).map_err(FoundingError::Native)?;
        let origin = Some(super::GenerationOrigin { id: record.id.clone(), request: record.request.clone(),
            entropy: record.entropy, max_value_depth: depth });
        Ok(Prepared { campaign: Self { replay, content, origin }, private })
    }
}
// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::{Campaign, FoundingError, Prepared};
use isocosm::{Execution, simulation::Genesis};
use isocosm_host::founding::{Offered, Purpose};
use isometry_campaign::{GenValue, GenerationRecord, GeneratorRequest};
use serde::{Deserialize, Serialize};

/// Accepted generator provenance without duplicating its campaign assertion stream.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenerationOrigin {
    pub id: String,
    pub request: GeneratorRequest,
    pub entropy: u64,
    pub max_value_depth: usize,
}

impl GenerationOrigin {
    pub(super) fn validate(&self) -> Result<(), FoundingError> {
        let envelope = GenerationRecord { id: self.id.clone(), request: self.request.clone(),
            entropy: self.entropy, proposal: GenValue::Text { value: String::new() } };
        envelope.validate(self.max_value_depth)
            .map_err(|e| FoundingError::Content(e.to_string()))
    }
}

impl Campaign {
    /// Caller chooses the profile; pack entropy remains separate.
    pub fn from_profile(
        profile: &isocosm::Founding,
        record: &GenerationRecord,
        max_value_depth: usize,
        mode: Execution,
    ) -> Result<Prepared, FoundingError> {
        Self::from_profile_for(profile, record, max_value_depth, mode, Purpose::Head, None)
    }

    pub fn from_profile_for(
        profile: &isocosm::Founding, record: &GenerationRecord, max_value_depth: usize,
        mode: Execution, purpose: Purpose, offered: Option<Offered<'_>>,
    ) -> Result<Prepared, FoundingError> {
        let genesis = profile.generate().map_err(|reason| {
            FoundingError::Native(crate::native::ReplayError {
                phase: crate::native::Phase::Founding,
                reason,
            })
        })?;
        Self::from_generation_for(genesis, record, max_value_depth, mode, purpose, offered)
    }

    pub fn from_profile_bench(profile: &isocosm::Founding, record: &GenerationRecord,
        max_value_depth: usize, mode: Execution) -> Result<Prepared, FoundingError> {
        Self::from_profile_for(profile, record, max_value_depth, mode, Purpose::Bench, None)
    }


    /// Consume an inspected pack result; the host still owns execution and entropy.
    pub fn from_generation(
        genesis: Genesis, record: &GenerationRecord, max_value_depth: usize, mode: Execution,
    ) -> Result<Prepared, FoundingError> {
        Self::from_generation_for(genesis, record, max_value_depth, mode, Purpose::Head, None)
    }

    pub fn from_generation_for(
        genesis: Genesis, record: &GenerationRecord, max_value_depth: usize, mode: Execution,
        purpose: Purpose, offered: Option<Offered<'_>>,
    ) -> Result<Prepared, FoundingError> {
        record.validate(max_value_depth).map_err(|e| FoundingError::Content(e.to_string()))?;
        let GenValue::Campaign { campaign: draft } = &record.proposal else {
            return Err(FoundingError::Content("generation is not a campaign draft".into()));
        };
        let mut prepared = Self::from_draft_for(genesis, draft, mode, purpose, offered)?;
        prepared.campaign.origin = Some(GenerationOrigin {
            id: record.id.clone(), request: record.request.clone(), entropy: record.entropy,
            max_value_depth,
        });
        Ok(prepared)
    }

    pub fn from_generation_bench(genesis: Genesis, record: &GenerationRecord,
        max_value_depth: usize, mode: Execution) -> Result<Prepared, FoundingError> {
        Self::from_generation_for(genesis, record, max_value_depth, mode, Purpose::Bench, None)
    }
}

// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;
use crate::{GenerationRecord, GenValue, RoleSlot, SecretFact, StoryletRequirements};
use isocosm::asserted::Assertion;

/// Loaded pack choreography is presentation, never a compatibility condition.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Choreography { pub name: String, pub emote: Option<String>, pub css: String }

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub key: Key, pub name: String, pub source: Key,
    pub requirements: StoryletRequirements, pub roles: Vec<RoleSlot>,
    pub assertions: Vec<Assertion>, pub content: Option<crate::founding::Content>,
    pub generation: Option<GenerationRecord>, pub choreography: Vec<Choreography>,
}

impl Definition {
    /// Existing bounded generator output, inspected by the DM. Secret text
    /// is returned to private custody, never stored in the public catalogue.
    pub fn from_generation(record: &GenerationRecord, requirements: StoryletRequirements,
        roles: Vec<RoleSlot>, choreography: Vec<Choreography>, max_depth: usize)
        -> Result<(Self, Vec<SecretFact>), String> {
        record.validate(max_depth).map_err(|e| e.to_string())?;
        let GenValue::Campaign { campaign } = &record.proposal else {
            return Err("adventure generation needs an actual campaign draft".into());
        };
        campaign.validate().map_err(|e| format!("adventure draft: {e:?}"))?;
        let content = crate::founding::Content::from_draft(campaign).map_err(|e| e.to_string())?;
        let mut public = record.clone();
        if let GenValue::Campaign { campaign } = &mut public.proposal { campaign.secrets.clear(); }
        let definition = Self { key:campaign.id.clone(), name:campaign.name.clone(),
            source:record.request.generator.clone(), requirements, roles,
            assertions:campaign.world.assertions().to_vec(), content:Some(content),
            generation:Some(public), choreography };
        definition.validate()?;
        Ok((definition, campaign.secrets.clone()))
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.key.trim().is_empty() || self.name.trim().is_empty() || self.source.trim().is_empty() {
            return Err("pack key, name and source must be authored".into());
        }
        let mut roles = BTreeSet::new();
        for role in &self.roles {
            if role.key.trim().is_empty() || !roles.insert(&role.key)
                || role.tags.iter().any(|tag| tag.trim().is_empty()) {
                return Err("pack roles need distinct authored keys and nonempty tags".into());
            }
        }
        for keys in [&self.requirements.faction_tags, &self.requirements.hidden_facts,
            &self.requirements.world_laws] {
            if keys.iter().any(|key| key.trim().is_empty()) { return Err("empty pack requirement".into()); }
        }
        if let Some(content) = &self.content { content.validate().map_err(|e| e.to_string())?; }
        if let Some(record) = &self.generation {
            let GenValue::Campaign { campaign } = &record.proposal else { return Err("pack provenance is not a campaign".into()); };
            if !campaign.secrets.is_empty() || campaign.id != self.key || record.request.generator != self.source
                || campaign.world.assertions() != self.assertions
                || self.content.as_ref() != Some(&crate::founding::Content::from_draft(campaign).map_err(|e| e.to_string())?) {
                return Err("public pack provenance differs or contains private secrets".into());
            }
        }
        let mut beats = BTreeSet::new();
        for beat in &self.choreography {
            if beat.name.is_empty() || !beats.insert(&beat.name)
                || !beat.name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
                return Err("pack choreography needs distinct CSS-safe beat names".into());
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum PackInput {
    Content { content: crate::founding::Content },
    Register { definition: Definition },
    Preview { key: Key, force: bool },
    Accept { key: Key, preview: EventHandle },
    Force { key: Key, preview: EventHandle },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PackEnvelope { pub participant: Id, pub tick: Tick, pub intent: PackInput }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Witness { pub entry: Key, pub event_key: Key, pub event: EventHandle, pub tick: Tick }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Probe {
    pub compatible: bool, pub condition_refusal: Option<String>, pub cast: BTreeMap<Key, Key>,
    pub native_refusal: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Prepared {
    pub definition: String, pub game: String, pub native: String,
    /// Only declared private requirements present in the trusted host store.
    pub private_requirements: BTreeSet<Key>, pub forced: bool, pub probe: Probe,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Preview {
    pub prepared: Prepared, pub participant: Id, pub witness: Witness,
    /// Complete accepted native source after the preview record, excluding game bytes.
    pub after_native: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Activation { pub participant: Id, pub preview: EventHandle, pub forced: bool, pub witness: Witness }

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Registered {
    pub definition: Definition, pub participant: Id, pub witness: Witness,
    pub previews: Vec<Preview>, pub activation: Option<Activation>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContentBinding { pub content: crate::founding::Content, pub participant: Id, pub witness: Witness }

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Catalogue { pub content: Option<ContentBinding>, pub packs: BTreeMap<Key, Registered> }

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PackView {
    pub pack: Registered, pub readiness: Probe, pub preview_current: bool,
}

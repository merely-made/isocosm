// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Product-only bindings borrowed beside the host's sole native Session.

use isocosm::{Session, schema::{Id, Key}, operations::Window};
use serde::{Deserialize, Serialize};
use crate::{MotionPose, identity::SubjectId};
use isometer_space::places::PlaceId;
mod binding;
mod reading;
pub mod actions;
pub use reading::{BorrowedReading, Picture, PictureBody};

const MARKER: &str = "eponym:mode-state";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PoseBinding {
    pub subject: SubjectId, pub entity: Id, pub pose: MotionPose,
    pub site: Id, pub patch: PlaceId, pub window: Window,
    pub body_revision: u64, pub terrain_revision: Key, pub resident_since: u64,
}

/// Presentation numbers are supplied, not world position or control authority.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Presentation {
    /// Camera coordinates in the explicitly chosen picture Ground window.
    pub centre: [f32; 3], pub forward: [f32; 3], pub half_height: f32,
    pub depth: f32, pub pan: [f32; 3], pub scale: f32, pub material: u8,
    pub played_tint: [f32; 3], pub other_tint: [f32; 3],
}

impl Presentation {
    pub fn validate(&self) -> Result<(), String> {
        let vectors = [self.centre,self.forward,self.pan,self.played_tint,self.other_tint];
        if vectors.iter().flatten().any(|n| !n.is_finite())
            || !self.half_height.is_finite() || self.half_height <= 0.0
            || !self.depth.is_finite() || self.depth <= 0.0 || !self.scale.is_finite() || self.scale <= 0.0
            || self.forward[0] == 0.0 && self.forward[2] == 0.0
            || self.played_tint.iter().chain(&self.other_tint).any(|n| !(0.0..=1.0).contains(n)) {
            return Err("Eponym presentation needs finite supplied camera, scale and tint values".into());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModeState {
    pub kind: String, pub version: u32, pub bindings: Vec<PoseBinding>,
    pub presentation: Presentation,
    /// The old unmarked mode payload stays byte-exact and uninterpreted.
    pub prior: Vec<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actions: Option<actions::ActionState>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpatialBindingInput {
    pub version: u32, pub expected_native: Key,
    pub bindings: Vec<PoseBinding>, pub presentation: Presentation,
}

pub fn source(session: &Session) -> Key {
    isocosm::digest(&(&session.branch,&session.entries,session.sim.genesis(),session.sim.state_hash()))
}

impl ModeState {
    /// Unmarked bytes remain opaque; only this available product's marker opens them.
    pub fn decode(bytes: &[u8]) -> Result<Option<Self>, String> {
        let Ok(value) = serde_json::from_slice::<serde_json::Value>(bytes) else { return Ok(None); };
        if value.get("kind").and_then(|kind| kind.as_str()) != Some(MARKER) { return Ok(None); }
        let state: Self = serde_json::from_value(value).map_err(|e| e.to_string())?;
        state.validate()?;
        Ok(Some(state))
    }

    pub fn validate(&self) -> Result<(), String> {
        let version = self.actions.as_ref().map_or(1, |a| if a.imported.is_some() {3} else {2});
        if self.kind != MARKER || self.version != version {
            return Err("unsupported Eponym mode state".into());
        }
        if let Some(actions) = &self.actions { actions.validate(self)?; }
        self.presentation.validate()?;
        let mut subjects = std::collections::BTreeSet::new();
        let mut entities = std::collections::BTreeSet::new();
        for binding in &self.bindings {
            if !subjects.insert(binding.subject) || !entities.insert(binding.entity) {
                return Err("Eponym subject/entity mapping is not explicit and unique".into());
            }
            binding.pose.validate().map_err(|e| format!("Eponym pose: {e:?}"))?;
        }
        Ok(())
    }
}

/// Trusted authored binding only. The caller owns author admission and the
/// native/game publication cut. No native command, Session copy or replay occurs.
pub fn bind(session: &Session, previous: &[u8], input: &SpatialBindingInput) -> Result<Vec<u8>, String> {
    if input.version != 1 || input.expected_native != source(session) {
        return Err("Eponym binding input differs from its actual native source".into());
    }
    if input.bindings.len() as u64 > session.sim.genesis().rules.limits.entities {
        return Err("Eponym bindings exceed the native entity limit".into());
    }
    let old = ModeState::decode(previous)?;
    let (prior, actions) = old.map_or_else(|| (previous.to_vec(), None), |old| (old.prior,old.actions));
    let version = actions.as_ref().map_or(1, |a| if a.imported.is_some() {3} else {2});
    let mut state = ModeState { kind:MARKER.into(),version,bindings:input.bindings.clone(),
        presentation:input.presentation.clone(),prior,actions };
    state.bindings.sort_by_key(|binding| binding.subject);
    state.validate()?;
    for binding in &state.bindings { binding.current(session)?; }
    serde_json::to_vec(&state).map_err(|e| e.to_string())
}

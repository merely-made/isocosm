// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Authored game geometry for one recorded native tick, over the borrowed owner.

use super::*;
use std::collections::{BTreeMap, BTreeSet};
use cosmograph::{Command, Simulation, spatial as native};
use isometer_core::Yaw;
use isometer_space::{Edit, Op, Shape, places::{Places, Rules as PlaceRules}};
use actions::{ActionState, SubjectProfile, OrdinaryAccess, game_source};
use cosmograph::directing::readings::View;
use cosmograph_overlay::Seat;

mod profile;
mod geometry;
mod arrival;
mod birth;
mod publication;
mod driver;

pub const KIND: &str = "eponym:clock";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Physical {
    pub subject: SubjectId, pub yaw: Yaw, pub eye: [i64; 3], pub places: PlaceRules,
    /// The real Ground frame in which the paired fixed-point pose was admitted.
    pub frame_origin: [i64; 3],
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TerrainInput {
    pub target: Option<Id>, pub focus_from_root: [i64; 3], pub shape_from_root: Shape,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StepInput {
    pub target: Option<Id>,
    /// An authored path relative to the current root. Its first offset is zero.
    pub path_from_root: Vec<[i64; 3]>, pub yaw: Yaw, pub eye_from_root: [i64; 3],
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArrivalInput {
    pub subject: SubjectId, pub from: Id, pub to: Id,
    pub pose: MotionPose, pub yaw: Yaw, pub eye: [i64; 3], pub window: Window,
    pub places: PlaceRules,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BirthParent { Entity(Id), Lineage(Key) }

/// Every non-position motion field and subcell offset is explicitly authored.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BirthMotion {
    pub fraction: [u16; 3], pub velocity: [i64; 3], pub grounded: bool,
    pub step: u64, pub fall_origin_from_root: Option<i64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BirthSlot {
    pub subject: SubjectId, pub species: cosmograph::lineage::SpeciesId,
    pub revision: crate::identity::BodyRevisionId,
    pub motion: Option<actions::MotionPolicy>, pub combat: Option<actions::CombatPolicy>,
    pub offset_from_parent: [i64; 3], pub yaw_from_parent: Yaw,
    pub eye_from_child: [i64; 3], pub window: Window, pub pose: BirthMotion,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BirthInput { pub profile: Key, pub parent: BirthParent, pub slots: Vec<BirthSlot> }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    pub version: u32,
    pub terrain: BTreeMap<SubjectId, BTreeMap<Key, TerrainInput>>,
    pub steps: BTreeMap<SubjectId, BTreeMap<Key, StepInput>>,
    pub arrivals: Vec<ArrivalInput>, pub births: Vec<BirthInput>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Change {
    pub before: Option<PoseBinding>, pub after: PoseBinding,
    pub physical: Physical,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    pub entry: Key, pub outcome: String, pub before_entries: Vec<Key>,
    pub terrain: Vec<native::TerrainResolution>, pub steps: Vec<native::Resolution>,
    pub arrivals: Vec<native::ArrivalResolution>, pub births: Vec<native::BirthResolution>,
    pub arrival_reports: Vec<native::ArrivalReport>, pub birth_reports: Vec<native::BirthReport>,
    pub order: Vec<native::Publication>, pub changes: Vec<Change>, pub refresh_entries: Vec<Key>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct State {
    pub profile: Profile, pub physical: BTreeMap<SubjectId, Physical>, pub records: Vec<Record>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sight: Option<super::sight::SightConfig>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub version: u32, pub expected_native: Key, pub expected_game: Key,
    pub profile: Profile, pub physical: Vec<Physical>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sight: Option<super::sight::SightConfig>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Input {
    pub kind: String, pub version: u32, pub expected_native: Key, pub expected_game: Key,
    pub subject: SubjectId,
}

pub fn configure(session: &Session, bytes: &[u8], input: &Config) -> Result<Vec<u8>, String> {
    check_source(session, bytes, input.version, &input.expected_native, &input.expected_game)?;
    let mut mode = ModeState::decode(bytes)?.ok_or("Eponym clock needs admitted physical bindings")?;
    mode.validate_native(session)?;
    if mode.clock.is_some() { return Err("Eponym clock already retains its authored profile".into()); }
    if mode.actions.is_none() { return Err("Eponym clock needs explicit action/anatomy profiles".into()); }
    input.profile.validate(session, &mode)?;
    let mut physical = BTreeMap::new();
    for held in &input.physical {
        let binding = binding(&mode, held.subject)?;
        geometry::current(&session.sim, binding, held)?;
        if physical.insert(held.subject, held.clone()).is_some() {
            return Err("duplicate Eponym clock physical subject".into());
        }
    }
    mode.clock = Some(State { profile: input.profile.clone(), physical, records: Vec::new(), sight: input.sight.clone() });
    mode.version = mode.required_version();
    mode.validate()?;
    serde_json::to_vec(&mode).map_err(|e| e.to_string())
}

/// Only the host's unpublished native candidate is mutated. It publishes this
/// candidate and returned product bytes together, or discards both on error.
pub fn submit(session: &mut Session, bytes: Vec<u8>, input: &Input,
    access: OrdinaryAccess, visible: &View) -> Result<(String, Vec<u8>), String> {
    if input.kind != KIND { return Err("not an Eponym clock envelope".into()); }
    check_source(session, &bytes, input.version, &input.expected_native, &input.expected_game)?;
    let mut mode = ModeState::decode(&bytes)?.ok_or("Eponym clock mode is absent")?;
    mode.validate_native(session)?;
    let actor = binding(&mode, input.subject)?;
    if access.seat != Seat::Player || access.subject.map(|id| id.0) != Some(actor.entity)
        || !session.sim.state().population.get(access.participant.0).is_some_and(cosmograph::directing::is_participant)
        || session.sim.plays(access.participant.0) != Some(actor.entity) || !visible.shows(actor.entity) {
        return Err("Eponym clock requires its actual native played subject".into());
    }
    let mut actions = mode.actions.take().ok_or("Eponym clock action configuration is absent")?;
    let outcome = advance(session, &mut mode, &mut actions)?;
    mode.actions = Some(actions); mode.validate()?;
    Ok((outcome, serde_json::to_vec(&mode).map_err(|e| e.to_string())?))
}

pub fn submit_bytes(session: &mut Session, bytes: Vec<u8>, input: &[u8],
    access: OrdinaryAccess, visible: &View) -> Result<(String, Vec<u8>), String> {
    let input: Input = serde_json::from_slice(input).map_err(|e| e.to_string())?;
    submit(session, bytes, &input, access, visible)
}

pub use driver::advance;
pub use publication::{refresh, motion};

impl State {
    pub fn physical(&self, subject: SubjectId) -> Option<&Physical> { self.physical.get(&subject) }

    /// Read-only consumer geometry. The caller's native awareness cut remains
    /// mandatory; this verifies a pose, not knowledge or control of its body.
    pub fn current(&self, session: &Session, binding: &PoseBinding) -> Result<&Physical, String> {
        let physical = self.physical(binding.subject).ok_or("Eponym clock physical reading is unavailable")?;
        geometry::current(&session.sim, binding, physical)?;
        Ok(physical)
    }
}

fn check_source(session: &Session, bytes: &[u8], version: u32, native: &str, game: &str) -> Result<(), String> {
    if version != 1 || native != source(session) || game != game_source(bytes) {
        return Err("Eponym clock input differs from its current native/game source".into());
    }
    Ok(())
}
fn binding(mode: &ModeState, subject: SubjectId) -> Result<&PoseBinding, String> {
    mode.bindings.iter().find(|b| b.subject == subject).ok_or("Eponym clock subject has no explicit native binding".into())
}

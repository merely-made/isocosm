// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Explicit authored walking inputs and destination placements, never spawn defaults.

use super::*;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StepInput {
    pub target: Option<EntityHandle>, pub focus: [i64; 3],
    pub yaw: Yaw, pub eye_offset: [i64; 3],
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArrivalInput {
    pub actor: EntityHandle, pub from: PlaceHandle, pub to: PlaceHandle,
    pub pose: Pose, pub eye: [i64; 3], pub window: Window,
    pub places: isometer_space::places::Rules,
    pub operations: BTreeMap<String, Operation>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    pub version: u32,
    pub steps: BTreeMap<Id, BTreeMap<String, StepInput>>,
    pub arrivals: Vec<ArrivalInput>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Authored { pub placements: Vec<Placement>, pub profile: Profile }

/// Legacy placement arrays retain their original meaning.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Setup { Placements(Vec<Placement>), Driver(Authored) }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Admission { Step { action: String }, Arrival }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Transition { pub actor: Id, pub before: Option<Placement>, pub after: Placement, pub admission: Admission }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Record {
    pub entry: String, pub changes: Vec<Transition>,
    pub steps: BTreeSet<String>, pub arrivals: Vec<isocosm::spatial::ArrivalReport>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Driver {
    pub profile: Profile,
    pub native_source: String,
    pub native_prefix: String,
    pub native_entries: u64,
    pub native_tick: u64,
    pub initial: BTreeMap<Id, Placement>,
    pub records: Vec<Record>,
}

impl Profile {
    pub fn validate(&self, sim: &Simulation) -> Result<(), String> {
        if self.version != 1 { return Err("unsupported authored spatial profile".into()); }
        for (actor, steps) in &self.steps {
            if sim.state().population.get(*actor).is_none() { return Err("step input actor absent".into()); }
            for (key, input) in steps {
                if !sim.genesis().rules.step_profiles.contains_key(key) {
                    return Err("game step input names no native profile".into());
                }
                if let Some(target) = input.target
                    && sim.state().population.get(target.0).is_none()
                { return Err("game step input target absent".into()); }
            }
        }
        let mut destinations = BTreeSet::new();
        for input in &self.arrivals {
            if !destinations.insert((input.actor.0, input.from.0, input.to.0))
                || sim.state().population.get(input.actor.0).is_none()
                || !sim.state().sites.contains_key(&input.to.0)
                || !sim.state().sites.get(&input.from.0).is_some_and(|s| s.routes.iter().any(|r| r.to == input.to.0))
            { return Err("arrival input has duplicate or absent native actor/route/destination".into()); }
        }
        Ok(())
    }
}

impl Game {
    pub fn configure(&mut self, session: &Session, authored: &Authored) -> Result<(), String> {
        if self.driver.is_some() { return Err("spatial driver already has a saved authored source".into()); }
        authored.profile.validate(&session.sim)?;
        let mut next = self.clone();
        next.replace(&session.sim, &authored.placements)?;
        next.version = VERSION;
        next.driver = Some(Driver { profile: authored.profile.clone(),
            native_source: native_source(session), native_entries: session.entries.len() as u64,
            native_prefix: native_prefix(session, session.entries.len(), session.sim.state().tick),
            native_tick: session.sim.state().tick, initial: next.members.clone(), records: Vec::new() });
        *self = next;
        Ok(())
    }

    pub fn setup(&mut self, session: &Session, setup: &Setup) -> Result<(), String> {
        match setup {
            Setup::Placements(placements) => self.replace(&session.sim, placements),
            Setup::Driver(authored) => self.configure(session, authored),
        }
    }
}

pub(super) fn native_source(session: &Session) -> String {
    isocosm::digest(&(&session.branch, &session.entries, session.sim.state_hash(), session.sim.genesis()))
}

pub(super) fn native_prefix(session: &Session, count: usize, tick: u64) -> String {
    isocosm::digest(&(&session.branch, &session.entries[..count], tick, session.sim.genesis()))
}

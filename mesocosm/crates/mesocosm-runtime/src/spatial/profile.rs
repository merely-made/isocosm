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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub births: Vec<BirthInput>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Authored { pub placements: Vec<Placement>, pub profile: Profile }

/// Legacy placement arrays retain their original meaning.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Setup { Placements(Vec<Placement>), Driver(Authored), Sighted(SightedSetup) }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Admission { Step { action: String }, Arrival, Birth { action: String, parent: Id, ordinal: u64 } }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Transition {
    pub actor: Id, pub before: Option<Placement>, pub after: Placement, pub admission: Admission,
    /// The selected birth slot's authored range, bound to this actual child.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sight_range: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Record {
    pub entry: String, pub changes: Vec<Transition>,
    pub steps: BTreeSet<String>, pub arrivals: Vec<isocosm::spatial::ArrivalReport>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub births: Vec<isocosm::spatial::BirthReport>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub order: Vec<isocosm::spatial::Publication>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command_outcome: Option<String>,
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
    pub(super) fn has_birth_sight(&self) -> bool {
        self.births.iter().any(|input| input.slots.iter().any(|slot| slot.sight_range.is_some()))
    }

    pub(super) fn version(&self) -> Result<(), String> {
        if self.version != 3 && !(self.version == 2 && !self.has_birth_sight())
            && !(self.version == 1 && self.births.is_empty()) {
            return Err("unsupported authored spatial profile".into());
        }
        if self.births.iter().flat_map(|input| &input.slots)
            .any(|slot| slot.sight_range.is_some_and(|range| range > i64::MAX as u64)) {
            return Err("authored newborn sight range exceeds its bounded physical source".into());
        }
        Ok(())
    }

    pub fn validate(&self, sim: &Simulation) -> Result<(), String> {
        self.version()?;
        for input in &self.births { input.validate(sim)?; }
        // Reject already-applicable selector overlap instead of choosing
        // entity versus lineage precedence. Future lineage changes are
        // checked again on the actual native birth request.
        for (&id, group) in &sim.state().population.groups {
            let mut processes = BTreeSet::new();
            for input in self.births.iter().filter(|input| match &input.parent {
                BirthParent::Entity(parent) => parent.0 >= id && parent.0 - id < group.count,
                BirthParent::Lineage(lineage) => lineage == &group.entity.lineage,
            }) {
                let process = &sim.genesis().rules.birth_profiles[&input.profile].process;
                // A lineage-wide selector conflicts with any matching
                // entity selector in this group; two distinct entity
                // identities do not select the same native parent.
                let key = (process, match &input.parent { BirthParent::Entity(p) => Some(p.0), _ => None });
                if processes.contains(&(process, None)) || key.1.is_none() && processes.iter().any(|(p, _)| *p == process)
                    || !processes.insert(key)
                { return Err("authored birth profile selectors overlap".into()); }
            }
        }
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
        next.driver = Some(Driver { profile: authored.profile.clone(),
            native_source: native_source(session), native_entries: session.entries.len() as u64,
            native_prefix: native_prefix(session, session.entries.len(), session.sim.state().tick),
            native_tick: session.sim.state().tick, initial: next.members.clone(), records: Vec::new() });
        next.version = next.current_version();
        *self = next;
        Ok(())
    }

    pub fn setup(&mut self, session: &Session, setup: &Setup) -> Result<(), String> {
        match setup {
            Setup::Placements(placements) => self.replace(&session.sim, placements),
            Setup::Driver(authored) => self.configure(session, authored),
            Setup::Sighted(authored) => self.setup_sighted(session, authored),
        }
    }
}

pub(super) fn native_source(session: &Session) -> String {
    isocosm::digest(&(&session.branch, &session.entries, session.sim.state_hash(), session.sim.genesis()))
}

pub(super) fn native_prefix(session: &Session, count: usize, tick: u64) -> String {
    isocosm::digest(&(&session.branch, &session.entries[..count], tick, session.sim.genesis()))
}

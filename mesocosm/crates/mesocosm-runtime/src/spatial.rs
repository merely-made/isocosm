// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Saved game geometry, bound to native bodies, places and terrain (833).

use std::collections::{BTreeMap, BTreeSet};
use isocosm::{Session, schema::Id, simulation::Simulation, operations::{Pending, Window}};
use isocosm_overlay::{ActKey, EntityHandle, PlaceHandle, Tick,
    mesocosm::{TerrainGeometry, TerrainShape}};
use isometer_core::Yaw;
use isometer_space::places::PlaceId;
use serde::{Deserialize, Serialize};

pub const VERSION: u32 = 4;
pub const UNMARKED_VERSION: u32 = 3;

mod input;
pub use input::{GameInput, Input};
mod refresh;
pub use refresh::Refresh;
mod profile;
pub use profile::{Authored, Setup, Profile, StepInput, ArrivalInput, Driver};
mod driver;
mod arrival;
mod birth;
mod birth_command;
pub use birth::{BirthInput, BirthParent, BirthSlot};
mod replay;
mod sight;
pub use sight::{SightConfig, SightedSetup};

/// Exact root-pivot pose in the site's base cells; no presentation scale.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pose { pub at: [i64; 3], pub yaw: Yaw }

/// Explicit geometry used only if its process is chosen.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Operation {
    pub target: Option<EntityHandle>,
    pub focus: [i64; 3],
    pub shape: TerrainShape,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Placement {
    pub actor: EntityHandle,
    pub site: PlaceHandle,
    pub patch: PlaceId,
    pub body_revision: u64,
    pub terrain_revision: String,
    pub pose: Pose,
    pub eye: [i64; 3],
    pub window: Window,
    pub operations: BTreeMap<String, Operation>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub places: Option<isometer_space::places::Rules>,
    /// This exact native visit, not a remembered pose from a prior journey.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resident_since: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Game {
    pub version: u32,
    pub members: BTreeMap<Id, Placement>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub driver: Option<Driver>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sight: Option<SightConfig>,
}

impl Default for Game {
    fn default() -> Self { Self { version: 1, members: BTreeMap::new(), driver: None, sight: None } }
}

impl Game {
    pub fn version(&self) -> Result<(), String> {
        let supported = match self.version {
            1 => self.driver.is_none() && self.sight.is_none(),
            2 | UNMARKED_VERSION => self.sight.is_none(),
            VERSION => self.sight.is_some(),
            _ => false,
        };
        if !supported {
            return Err("unsupported Mesocosm spatial version".into());
        }
        if let Some(sight) = &self.sight { sight.validate()?; }
        if self.members.iter().any(|(id, at)| *id != at.actor.0) {
            return Err("saved spatial identity differs from its native key".into());
        }
        Ok(())
    }

    pub(super) fn current_version(&self) -> u32 {
        if self.sight.is_some() { VERSION } else { UNMARKED_VERSION }
    }

    /// Trusted authored geometry; this never moves or takes a native body.
    pub fn replace(&mut self, sim: &Simulation, placements: &[Placement]) -> Result<(), String> {
        self.version()?;
        if self.driver.is_some() { return Err("saved driver poses change only through admitted native spatial work".into()); }
        let mut ids = BTreeSet::new();
        let mut admitted = Vec::new();
        for at in placements {
            if !ids.insert(at.actor.0) { return Err("duplicate authored spatial member".into()); }
            let mut at = at.clone();
            let member = sim.state().population.get(at.actor.0).ok_or("spatial actor absent")?;
            if at.resident_since.is_none() { at.resident_since = Some(member.arrived); }
            at.check(sim)?;
            admitted.push(at);
        }
        let mut next = self.clone();
        for at in admitted { next.members.insert(at.actor.0, at); }
        if next.members.len() as u64 > sim.genesis().rules.limits.entities {
            return Err("saved spatial member limit".into());
        }
        *self = next;
        Ok(())
    }

    pub fn resolve(&self, pending: &Pending) -> Result<TerrainGeometry, String> {
        self.version()?;
        let request = &pending.request;
        let held = self.members.get(&request.actor).ok_or("chosen actor has no saved physical pose")?;
        // A pending native action can follow an earlier native body/terrain
        // change in this same tick. Revalidate the same actual cells, never
        // substitute a presentation spot or a different native patch.
        let at = held.rebound(&pending.sim)?;
        at.check(&pending.sim)?;
        if at.patch != request.patch || at.body_revision != request.body_revision
            || at.terrain_revision != request.terrain_revision || at.site.0 != request.site
        { return Err("saved spatial inputs are stale for the chosen operation".into()); }
        let operation = at.operations.get(&request.process)
            .ok_or("chosen process has no explicit focus, shape and window inputs")?;
        if operation.target.map(|id| id.0) != request.target {
            return Err("saved focus names another chosen target".into());
        }
        Ok(TerrainGeometry {
            action: request.action.clone(), process: ActKey(request.process.clone()),
            tick: Tick(request.tick), actor: at.actor, site: at.site,
            body_revision: at.body_revision, terrain_revision: at.terrain_revision.clone(),
            eye: at.eye, focus: operation.focus,
            min: at.window.min, max: at.window.max, depth: at.window.depth,
            shape: operation.shape.clone(),
        })
    }

    /// Both candidates stay private until every native operation is admitted.
    pub fn advance(&mut self, session: &mut Session, access: isocosm_overlay::OrdinaryAccess)
        -> Result<isocosm::flows::FlowResult<isocosm::simulation::Work>, crate::Refusal> {
        if access.seat != isocosm_overlay::Seat::Player || access.subject.is_none()
            || session.sim.plays(access.participant.0) != access.subject.map(|id| id.0)
        { return Err(crate::Refusal::NotPlayed); }
        self.advance_clock(session).map_err(crate::Refusal::Sim)
    }
}

impl Placement {
    fn check(&self, sim: &Simulation) -> Result<(), String> {
        let member = sim.state().population.get(self.actor.0).ok_or("spatial actor is absent")?;
        if !member.alive || member.body.is_none() || member.body_revision != self.body_revision
            || member.resident_site() != Some(self.site.0) || member.patch != Some(self.patch)
            || self.resident_since != Some(member.arrived)
            || self.patch.site != self.site.0 || sim.terrain_revision() != self.terrain_revision
        { return Err("saved spatial body, place or terrain binding is stale".into()); }
        let volume = sim.volume(self.site.0, self.window.min, self.window.max, self.window.depth)?;
        let site = crate::perception::InSite::of(sim, volume, self.places.unwrap_or_default())?;
        site.cell(sim, self.actor.0, self.pose.at)?;
        site.cell(sim, self.actor.0, self.eye)
    }
}

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

pub const VERSION: u32 = 1;

mod input;
pub use input::{GameInput, Input};

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
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Game {
    pub version: u32,
    pub members: BTreeMap<Id, Placement>,
}

impl Default for Game {
    fn default() -> Self { Self { version: VERSION, members: BTreeMap::new() } }
}

impl Game {
    pub fn version(&self) -> Result<(), String> {
        if self.version != VERSION { return Err("unsupported Mesocosm spatial version".into()); }
        if self.members.iter().any(|(id, at)| *id != at.actor.0) {
            return Err("saved spatial identity differs from its native key".into());
        }
        Ok(())
    }

    /// Trusted authored geometry; this never moves or takes a native body.
    pub fn replace(&mut self, sim: &Simulation, placements: &[Placement]) -> Result<(), String> {
        self.version()?;
        let mut ids = BTreeSet::new();
        for at in placements {
            if !ids.insert(at.actor.0) { return Err("duplicate authored spatial member".into()); }
            at.check(sim)?;
        }
        let mut next = self.clone();
        for at in placements { next.members.insert(at.actor.0, at.clone()); }
        if next.members.len() as u64 > sim.genesis().rules.limits.entities {
            return Err("saved spatial member limit".into());
        }
        *self = next;
        Ok(())
    }

    pub fn resolve(&self, pending: &Pending) -> Result<TerrainGeometry, String> {
        self.version()?;
        let request = &pending.request;
        let at = self.members.get(&request.actor).ok_or("chosen actor has no saved physical pose")?;
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
    pub fn advance(&self, session: &mut Session, access: isocosm_overlay::OrdinaryAccess)
        -> Result<isocosm::flows::FlowResult<isocosm::simulation::Work>, crate::Refusal> {
        self.version().map_err(crate::Refusal::Sim)?;
        let mut native = session.clone();
        let out = crate::terrain::advance(&mut native, access, &mut |pending: &Pending| {
            self.resolve(pending)
        })?;
        *session = native;
        Ok(out)
    }
}

impl Placement {
    fn check(&self, sim: &Simulation) -> Result<(), String> {
        let member = sim.state().population.get(self.actor.0).ok_or("spatial actor is absent")?;
        if !member.alive || member.body.is_none() || member.body_revision != self.body_revision
            || member.resident_site() != Some(self.site.0) || member.patch != Some(self.patch)
            || self.patch.site != self.site.0 || sim.terrain_revision() != self.terrain_revision
        { return Err("saved spatial body, place or terrain binding is stale".into()); }
        let volume = sim.volume(self.site.0, self.window.min, self.window.max, self.window.depth)?;
        let site = crate::perception::InSite::of(sim, volume, Default::default())?;
        site.cell(sim, self.actor.0, self.pose.at)?;
        site.cell(sim, self.actor.0, self.eye)
    }
}

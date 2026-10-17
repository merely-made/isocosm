// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Rebind the SAME saved actual cells after native geometry changes.
//! Invalid or departed positions remain historical, never relocated. Their
//! stale binding refuses physical operations/rendering, not ordinary clocks.

use super::{Game, Placement};
use isocosm::{Command, Session, schema::Id, Simulation};
use isometer_space::places::Rules;
use std::collections::BTreeMap;

#[path = "geometry_cache.rs"]
mod geometry_cache;
pub(super) use geometry_cache::GeometryCache;
use geometry_cache::GeometryCache as LocalGeometryCache;

#[derive(Clone, Debug, Default)]
pub struct Refresh {
    pub current: Vec<Id>,
    pub inactive: Vec<Id>,
    pub stale: BTreeMap<Id, String>,
}

impl Placement {
    /// Read current native body/terrain at this unchanged pose and eye.
    /// Both cells must remain clear and belong to the same realized patch.
    pub(super) fn rebound(&self, sim: &Simulation) -> Result<Self, String> {
        self.rebound_with(sim, &mut LocalGeometryCache::default())
    }

    pub(super) fn rebound_with(&self, sim: &Simulation, cache: &mut GeometryCache) -> Result<Self, String> {
        let member = sim.state().population.get(self.actor.0).ok_or("spatial actor is absent")?;
        if member.body.is_none() || member.resident_site() != Some(self.site.0) {
            return Err("saved pose has no realized native body at its recorded site".into());
        }
        if self.resident_since != Some(member.arrived) {
            return Err("saved pose is not bound to this native visit".into());
        }
        let window = &self.window;
        let (volume, places) = cache.get(sim, self.site.0, window, self.places.unwrap_or_else(Rules::default))?;
        let mut next = self.clone();
        let patch = places.place_at(self.pose.at).ok_or("saved pose has no current realized patch")?;
        for cell in [self.pose.at, self.eye] {
            if !volume.holds(cell[0], cell[2]) || cell[1] < volume.datum || cell[1] >= volume.ceiling()
                || places.place_at(cell) != Some(patch)
                || !volume.material(cell).is_some_and(|m| m == volume.materials.air || m == volume.materials.water)
            { return Err("saved pose or eye no longer admits in the current realized clear patch".into()); }
        }
        next.patch = patch;
        next.body_revision = member.body_revision;
        next.terrain_revision = sim.terrain_revision();
        Ok(next)
    }

    /// The exact physical binding used for rendering, including a native
    /// corpse if that native seat cut supplies it. This grants no awareness.
    pub fn realized(&self, sim: &Simulation) -> Result<(), String> {
        self.realized_with(sim, &mut LocalGeometryCache::default())
    }

    fn realized_with(&self, sim: &Simulation, cache: &mut GeometryCache) -> Result<(), String> {
        let next = self.rebound_with(sim, cache)?;
        let member = sim.state().population.get(self.actor.0).ok_or("spatial actor is absent")?;
        if next != *self || member.patch != Some(self.patch) {
            return Err("saved physical body, patch or terrain binding is stale".into());
        }
        Ok(())
    }
}

impl Game {
    /// Update valid same-site bindings and log only a changed realized patch.
    /// Dormant or obstructed history is retained for an explicitly selected
    /// movement/arrival solver to resolve later, without invented coordinates.
    pub fn refresh(&mut self, session: &mut Session) -> Result<Refresh, String> {
        self.refresh_with(session, &mut LocalGeometryCache::default())
    }

    pub(super) fn refresh_with(
        &mut self,
        session: &mut Session,
        geometry: &mut GeometryCache,
    ) -> Result<Refresh, String> {
        self.version()?;
        let mut game = self.clone();
        let mut native = session.clone();
        let mut report = Refresh::default();
        for (&id, held) in &self.members {
            let Some(member) = native.sim.state().population.get(id) else {
                report.inactive.push(id); continue;
            };
            if member.body.is_none() || member.resident_site() != Some(held.site.0) {
                report.inactive.push(id); continue;
            }
            let result = (|| {
                let next = held.rebound_with(&native.sim, geometry)?;
                let candidate = if member.patch == Some(next.patch) {
                    next.realized_with(&native.sim, geometry)?;
                    None
                } else {
                    let mut candidate = native.clone();
                    candidate.command(Command::Patch { entity:id, patch:Some(next.patch) })?;
                    next.realized_with(&candidate.sim, geometry)?;
                    Some(candidate)
                };
                Ok::<_, String>((candidate, next))
            })();
            match result {
                Ok((candidate, next)) => {
                    if let Some(candidate) = candidate { native = candidate; }
                    game.members.insert(id, next); report.current.push(id);
                },
                Err(why) => { report.stale.insert(id, why); },
            }
        }
        *session = native;
        *self = game;
        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use isocosm::operations::Window;
    use isocosm_overlay::{EntityHandle, PlaceHandle};

    #[test]
    fn geometry_reuse_does_not_skip_current_visit_or_body_revision_checks() {
        let mut session = geometry_cache::physical_test_session();
        let templates = session.sim.genesis().rules.spatial_templates.clone().unwrap();
        let (site, stand) = templates.stands.iter().find_map(|(&site, stands)|
            stands.first().map(|stand| (site, stand.clone())))
            .expect("generated 3x2 map declares at least one stand");
        let actor = session.sim.state().population.groups.iter().find_map(|(&id, group)| {
            let entity = &group.entity;
            (entity.alive && entity.body.is_some() && entity.resident_site() == Some(site)).then_some(id)
        }).expect("declared stand site has an embodied resident");
        let member = session.sim.state().population.get(actor).unwrap().clone();
        let window = Window {
            min: [stand.at[0] - 1, stand.at[2] - 1],
            max: [stand.at[0] + 2, stand.at[2] + 2],
            depth: templates.window.depth,
        };
        let mut geometry = LocalGeometryCache::default();
        let (volume, places) = geometry.get(&session.sim, site, &window, Rules::default()).unwrap();
        let material = volume.materials;
        let clear = (window.min[0]..window.max[0]).find_map(|x|
            (window.min[1]..window.max[1]).find_map(|z|
                (volume.datum..volume.ceiling()).find_map(|y| {
                    let cell = [x, y, z];
                    let patch = places.place_at(cell)?;
                    (volume.material(cell).is_some_and(|m| m == material.air || m == material.water)
                        && places.places.contains_key(&patch))
                        .then_some((cell, patch))
                })))
            .expect("generated stand window contains a clear native patch cell");
        let (at, patch) = clear;
        session.command(Command::Patch { entity: actor, patch: Some(patch) }).unwrap();
        let held = Placement {
            actor: EntityHandle(actor), site: PlaceHandle(site), patch,
            body_revision: member.body_revision, terrain_revision: session.sim.terrain_revision(),
            pose: super::super::Pose { at, yaw: isometer_core::Yaw::Zero }, eye: at,
            window, operations: BTreeMap::new(), places: Some(Rules::default()),
            resident_since: Some(member.arrived),
        };
        held.realized_with(&session.sim, &mut geometry).unwrap();

        let mut stale_body = held.clone();
        stale_body.body_revision = stale_body.body_revision.wrapping_add(1);
        assert!(stale_body.realized_with(&session.sim, &mut geometry).unwrap_err().contains("stale"));

        let mut stale_visit = held;
        stale_visit.resident_since = Some(member.arrived + 1);
        assert!(stale_visit.rebound_with(&session.sim, &mut geometry).unwrap_err().contains("visit"));
    }
}

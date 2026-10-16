// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Members placed in the lifted site by `Command::Patch` (ruling 783): the
//! played site lifted in one bounded window, its patches derived by
//! isometer, and each unplaced cohort's first member given one in turn.

use isocosm::history::Command;
use isocosm::schema::Id;
use isocosm::simulation::Simulation;
use isometer_space::places::{Kind, PlaceId, Places, Rules};
use isometer_space::volume::Volume;

use isocosm_host::world::World;

use super::Mesocosm;

/// Half the window lifted around a site's centre, in base columns.
pub const WINDOW_HALF: i64 = 32;
/// Rock cells held under the window's lowest soil.
pub const DEPTH: i64 = 8;

/// `site` lifted within the bounded window around its centre.
pub fn lift(sim: &Simulation, site: Id) -> Result<Volume, String> {
    let side = sim.atlas()?.footprint.side as i64;
    let half = WINDOW_HALF.min(side / 2).max(1);
    let mid = side / 2;
    let min = [(mid - half).max(0); 2];
    let max = [(mid + half).min(side); 2];
    sim.volume(site, min, max, DEPTH)
}

/// Source of the terrain lift, including weather/skeleton changes and edits
/// reaching across borders. Body ledgers alone do not require another lift.
pub fn terrain_key(sim: &Simulation) -> String {
    sim.terrain_revision()
}

impl Mesocosm {
    /// Gives each living, unplaced cohort at the played site a patch.
    pub(super) fn place_members(&mut self, world: &World) {
        let assignments = {
            let session = super::session(world);
            let sim = &session.sim;
            let Some(site) = sim.plays(self.lp.participant)
                .and_then(|critter| sim.state().population.get(critter)).and_then(|e| e.resident_site()) else {
                return;
            };
            let unplaced: Vec<Id> = sim
                .state()
                .population
                .groups
                .iter()
                .filter(|(_, g)| g.entity.alive && g.entity.resident_site() == Some(site) && g.entity.patch.is_none())
                .filter(|(_, g)| !isocosm::directing::is_participant(&g.entity))
                .map(|(first, _)| *first)
                .collect();
            if unplaced.is_empty() {
                return;
            }
            let Ok(places) = lift(sim, site).and_then(|v| Places::derive(&v, Rules::default())) else {
                return;
            };
            let patches: Vec<PlaceId> = places
                .places
                .values()
                .filter(|p| p.kind == Kind::Patch)
                .map(|p| p.id)
                .collect();
            if patches.is_empty() {
                return;
            }
            unplaced.into_iter().enumerate().map(|(k, entity)| (entity, patches[k % patches.len()])).collect::<Vec<_>>()
        };
        let _ = world.cut().and_then(|before| world.product(before, |session| {
            for (entity, patch) in assignments {
                // A refusal leaves the member unplaced; the host draws it anyway.
                let _ = session.command(Command::Patch { entity, patch: Some(patch) });
            }
            Ok(())
        }));
    }
}

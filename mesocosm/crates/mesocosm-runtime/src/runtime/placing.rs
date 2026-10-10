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

use super::Runtime;

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

impl Runtime {
    /// Gives each living, unplaced cohort at the played site a patch.
    pub(super) fn place_members(&mut self) {
        let Some(site) = self.played().map(|e| e.place) else {
            return;
        };
        let sim = self.sim();
        let unplaced: Vec<Id> = sim
            .state()
            .population
            .groups
            .iter()
            .filter(|(_, g)| g.entity.alive && g.entity.place == site && g.entity.patch.is_none())
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
        for (k, entity) in unplaced.into_iter().enumerate() {
            let patch = Some(patches[k % patches.len()]);
            // A refusal leaves the member unplaced; the host draws it anyway.
            let _ = self.interim.session.command(Command::Patch { entity, patch });
        }
    }
}

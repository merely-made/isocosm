// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The section's native input at site grain (wing ruling 681): the played
//! critter's site, lifted to a Ground, and the bodies of its living members.

use isocosm::kingdom::{self, Kingdom};
use isocosm::schema::Id;
use isocosm::simulation::Simulation;
use isometer::DeclaredExtentVolumes;
use isometer::render::kingdom_colour;

use crate::section::{PlacedBody, SiteScene};

pub use mesocosm_runtime::runtime::placing::{DEPTH, WINDOW_HALF};

/// The site `played` stands in, or the first site when nobody is played.
pub fn site_of(sim: &Simulation, played: Option<Id>) -> Option<Id> {
    let pop = &sim.state().population;
    played
        .and_then(|id| pop.get(id))
        .map(|e| e.place)
        .or_else(|| sim.state().sites.keys().next().copied())
}

/// Lifts `site` within the runtime's bounded window around its centre.
pub fn lift(sim: &Simulation, site: Id) -> Result<isometer::space::volume::Volume, String> {
    mesocosm_runtime::runtime::placing::lift(sim, site)
}

/// The scene for `played`'s site: its lifted ground and its living bodies,
/// each placed at a presentation spot ([`super::layout`]).
pub fn site_scene(sim: &Simulation, played: Option<Id>) -> Result<SiteScene, String> {
    let site = site_of(sim, played).ok_or("a world with no sites")?;
    let volume = lift(sim, site)?;
    let ground = volume.ground().clone();
    let bodies = placed(sim, site, &volume);
    let volumes = DeclaredExtentVolumes::from_documents(bodies.iter().map(|b| &b.document), 1);
    Ok(SiteScene {
        site,
        ground,
        window: volume,
        bodies,
        played,
        volumes,
    })
}

/// One body per living cohort in `site` that has one, in id order: at its
/// patch where the runtime placed it (783), at a presentation spot otherwise.
pub fn placed(
    sim: &Simulation,
    site: Id,
    volume: &isometer::space::volume::Volume,
) -> Vec<PlacedBody> {
    let ground = volume.ground();
    sim.state()
        .population
        .groups
        .iter()
        .filter(|(_, group)| group.entity.place == site && group.entity.alive)
        .filter_map(|(id, group)| Some((*id, &group.entity, group.entity.body.clone()?)))
        .enumerate()
        .map(|(index, (id, entity, document))| PlacedBody {
            id,
            document,
            at: entity
                .patch
                .filter(|p| volume.holds(p.cell[0], p.cell[2]))
                .map(|p| volume.to_ground(p.cell))
                .unwrap_or_else(|| super::layout::spot(ground, index)),
            tint: tint_of(kingdom::of(entity)),
            alive: entity.alive,
        })
        .collect()
}

/// A body's colour by the kingdom its body reads as.
pub fn tint_of(kingdom: Option<Kingdom>) -> [f32; 3] {
    let index = match kingdom {
        Some(Kingdom::Producer) => 0,
        Some(Kingdom::Consumer) | None => 1,
        Some(Kingdom::Decomposer) => 2,
    };
    kingdom_colour(index)
}

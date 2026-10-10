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

/// Half the bounded window lifted around a site's centre, in base columns.
pub const WINDOW_HALF: i64 = 32;

/// Rock cells held under the window's lowest soil.
pub const DEPTH: i64 = 8;

/// The site `played` stands in, or the first site when nobody is played.
pub fn site_of(sim: &Simulation, played: Option<Id>) -> Option<Id> {
    let pop = &sim.state().population;
    played
        .and_then(|id| pop.get(id))
        .map(|e| e.place)
        .or_else(|| sim.state().sites.keys().next().copied())
}

/// Lifts `site` within a bounded window around its centre.
pub fn lift(sim: &Simulation, site: Id) -> Result<isometer::space::volume::Volume, String> {
    let side = sim.atlas()?.footprint.side as i64;
    let half = WINDOW_HALF.min(side / 2).max(1);
    let mid = side / 2;
    let min = [(mid - half).max(0); 2];
    let max = [(mid + half).min(side); 2];
    sim.volume(site, min, max, DEPTH)
}

/// The scene for `played`'s site: its lifted ground and its living bodies,
/// each placed at a presentation spot ([`super::layout`]).
pub fn site_scene(sim: &Simulation, played: Option<Id>) -> Result<SiteScene, String> {
    let site = site_of(sim, played).ok_or("a world with no sites")?;
    let ground = lift(sim, site)?.ground().clone();
    let bodies = placed(sim, site, &ground);
    let volumes = DeclaredExtentVolumes::from_documents(bodies.iter().map(|b| &b.document), 1);
    Ok(SiteScene {
        site,
        ground,
        bodies,
        played,
        volumes,
    })
}

/// One body per living cohort in `site` that has one, in id order.
pub fn placed(
    sim: &Simulation,
    site: Id,
    ground: &isometer::core::ground::Ground,
) -> Vec<PlacedBody> {
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
            at: super::layout::spot(ground, index),
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

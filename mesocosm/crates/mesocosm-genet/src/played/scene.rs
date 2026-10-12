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
use isocosm::directing::readings::View;
use isometer::DeclaredExtentVolumes;
use isometer::render::kingdom_colour;

use crate::section::{PlacedBody, SiteScene};

pub use mesocosm_runtime::runtime::placing::{DEPTH, WINDOW_HALF};

/// The site `played` stands in, or the first site when nobody is played.
pub fn site_of(sim: &Simulation, played: Option<Id>) -> Option<Id> {
    let pop = &sim.state().population;
    match played {
        Some(id) => pop.get(id).and_then(|e| e.resident_site()),
        None => sim.state().sites.keys().next().copied(),
    }
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

/// The same edited site with only the bodies admitted by the native view.
pub fn site_scene_visible(sim: &Simulation, played: Option<Id>, visible: &View)
    -> Result<SiteScene, String> {
    let mut scene = site_scene(sim, played)?;
    scene.bodies.retain(|b| visible.shows(b.id));
    scene.volumes = DeclaredExtentVolumes::from_documents(scene.bodies.iter().map(|b| &b.document), 1);
    Ok(scene)
}

/// Explicit in-site geometry drives both rays and the corresponding picture.
/// No patch identity anchor or arbitrary layout spot stands in for a sight cell.
pub fn site_scene_sighted(runtime: &mesocosm_runtime::Runtime,
    site: &mesocosm_runtime::perception::InSite, sight: &mesocosm_runtime::perception::Sight)
    -> Result<SiteScene, String> {
    let visible = runtime.sighted_view(site, sight)?;
    let sim = runtime.sim();
    let volume = site.volume().clone();
    let ground = volume.ground().clone();
    let cells = std::iter::once((sight.observer, sight.eye))
        .chain(sight.targets.iter().map(|(&id, &at)| (id, at)));
    let mut bodies = Vec::new();
    for (id, cell) in cells {
        if !visible.shows(id) || bodies.iter().any(|b: &PlacedBody| b.id == id) { continue; }
        let e = sim.state().population.get(id).ok_or("sighted body disappeared")?;
        let Some(document) = e.body.clone() else { continue; };
        bodies.push(PlacedBody { id, document, at:volume.to_ground(cell),
            tint:tint_of(kingdom::of(e)), alive:e.alive });
    }
    let volumes = DeclaredExtentVolumes::from_documents(bodies.iter().map(|b| &b.document), 1);
    Ok(SiteScene { site:site.site(), ground, window:volume, bodies,
        played:runtime.critter(), volumes })
}

pub fn placed_visible(sim: &Simulation, site: Id,
    volume: &isometer::space::volume::Volume, visible: &View) -> Vec<PlacedBody> {
    placed(sim, site, volume).into_iter().filter(|b| visible.shows(b.id)).collect()
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
        .filter(|(_, group)| group.entity.resident_site() == Some(site) && group.entity.alive)
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

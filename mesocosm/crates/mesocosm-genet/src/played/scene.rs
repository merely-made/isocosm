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
use isometer::render::kingdom_colour;

use crate::section::{PlacedBody, SiteScene, body_volumes};

pub use mesocosm_runtime::runtime::placing::{DEPTH, WINDOW_HALF};

/// The site `played` stands in, or the first site when nobody is played.
pub fn site_of(sim: &Simulation, played: Option<Id>) -> Option<Id> {
    let pop = &sim.state().population;
    match played {
        Some(id) => pop.get(id).and_then(|e| e.resident_site().or_else(|| e.journey.as_ref().map(|j| j.from))),
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
    let volumes = body_volumes(&bodies)?;
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
    scene.volumes = body_volumes(&scene.bodies)?;
    Ok(scene)
}

/// Explicit in-site geometry drives both rays and the corresponding picture.
/// No patch identity anchor or arbitrary layout spot stands in for a sight cell.
pub fn site_scene_sighted(runtime: &mesocosm_runtime::Runtime,
    site: &mesocosm_runtime::perception::InSite, sight: &mesocosm_runtime::perception::Sight)
    -> Result<SiteScene, String> {
    let visible = runtime.sighted_view(site, sight)?;
    let sim = runtime.sim();
    let mut realized = mesocosm_runtime::spatial::RealizedChecker::new(&sim);
    let volume = site.volume().clone();
    let ground = volume.ground().clone();
    let cells = std::iter::once((sight.observer, sight.eye))
        .chain(sight.targets.iter().map(|(&id, &at)| (id, at)));
    let mut bodies = Vec::new();
    for (id, cell) in cells {
        if !visible.shows(id) || bodies.iter().any(|b: &PlacedBody| b.id == id) { continue; }
        let e = sim.state().population.get(id).ok_or("sighted body disappeared")?;
        let Some(document) = e.body.clone() else { continue; };
        let physical = if e.alive { None } else {
            let Some(held) = runtime.spatial().members.get(&id) else { continue; };
            realized.check(held)?;
            if held.site.0 != site.site() || held.pose.at != cell {
                return Err("corpse sight cell differs from its current saved physical pose".into());
            }
            Some(held.pose.clone())
        };
        bodies.push(PlacedBody { id, document, at:volume.to_ground(cell), physical,
            tint:tint_of(kingdom::of(e)), alive:e.alive });
    }
    let volumes = body_volumes(&bodies)?;
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
            physical: None,
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

/// Current actual saved poses through the supplied native seat cut. Bodies
/// without physical input are omitted; neither layout nor eye is a root pose.
pub fn placed_physical(sim: &Simulation, site: Id,
    volume: &isometer::space::volume::Volume, visible: &View,
    game: &mesocosm_runtime::spatial::Game) -> Result<Vec<PlacedBody>, String> {
    game.version()?;
    let mut bodies = Vec::new();
    let mut realized = mesocosm_runtime::spatial::RealizedChecker::new(sim);
    for (&id, at) in &game.members {
        if at.site.0 != site || !visible.shows(id) { continue; }
        let entity = sim.state().population.get(id).ok_or("physical picture names an absent native member")?;
        if entity.resident_site() != Some(site) { continue; }
        if realized.check(at).is_err() || game.placement_unoccupied(sim, id).is_err() { continue; }
        let Some(document) = entity.body.clone() else { continue; };
        if !volume.holds(at.pose.at[0], at.pose.at[2]) { continue; }
        let cell = volume.to_ground(at.pose.at);
        let base = [volume.min[0], volume.datum, volume.min[1]];
        let offsets = [i64::from(volume.ground().extent()), 0, i64::from(volume.ground().extent())];
        if (0..3).any(|i| base[i].checked_add(offsets[i]).and_then(|b|
            b.checked_add(i64::from(cell[i]))) != Some(at.pose.at[i])) {
            return Err("physical picture exceeds Ground coordinate bounds".into());
        }
        bodies.push(PlacedBody { id, document, at:cell, physical:Some(at.pose.clone()),
            tint:tint_of(kingdom::of(entity)), alive:entity.alive });
    }
    Ok(bodies)
}

/// The admitted body window, or the generator's declared terrain window while
/// the played body travels or lacks a supported pose. Only checked bodies draw.
pub fn site_scene_physical(sim: &Simulation, played: Option<Id>, visible: &View,
    game: &mesocosm_runtime::spatial::Game) -> Result<SiteScene, String> {
    let id = played.ok_or("physical scene has no played native member")?;
    let admitted = game.members.get(&id).filter(|held|
        held.realized(sim).is_ok() && game.placement_unoccupied(sim, id).is_ok());
    let (site, window) = if let Some(held) = admitted {
        (held.site.0, &held.window)
    } else {
        let templates = sim.genesis().rules.spatial_templates.as_ref()
            .filter(|t| t.version >= 2).ok_or("played member has no current physical window")?;
        (site_of(sim, played).ok_or("played member has no native site or journey")?, &templates.window)
    };
    let volume = sim.volume(site, window.min, window.max, window.depth)?;
    let bodies = placed_physical(sim, site, &volume, visible, game)?;
    let volumes = body_volumes(&bodies)?;
    Ok(SiteScene { site, ground:volume.ground().clone(), window:volume, bodies, played, volumes })
}

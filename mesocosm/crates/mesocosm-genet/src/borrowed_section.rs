// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Mesocosm's actual Section, supplied by one admitted native mode window.
//! The handle carries derived picture inputs; it never owns a Session.

pub mod producer;
#[cfg(test)]
mod tests;

use std::{cell::RefCell, rc::Rc};
use isocosm::{Session, attention::spatial::RegionReading, directing::readings::Mode, schema::Id};
use isometer::{DeclaredExtentVolumes, SlabCamera};
use mesocosm_runtime::perception::{InSite, Sight};
use crate::{played::{layout, scene}, section::{self, CameraMode, Pan, PlacedBody, SiteScene}};

pub struct Snapshot {
    pub(crate) scene: SiteScene,
    pub(crate) fingerprint: [u64; 4],
    pub(crate) quarter_turn: u8,
}

impl Snapshot {
    pub fn site(&self) -> Id { self.scene.site }
    pub fn played(&self) -> Option<Id> { self.scene.played }
    pub fn body_ids(&self) -> Vec<Id> { self.scene.bodies.iter().map(|body| body.id).collect() }
    pub fn camera(&self, aspect: f32) -> SlabCamera {
        let mode = CameraMode::default();
        let at = self.scene.played.and_then(|id| self.scene.at(id))
            .unwrap_or_else(|| layout::centre(&self.scene.ground));
        SlabCamera { centre: section::centre_on(at, Pan::default(), section::SLAB_HALF_HEIGHT, mode),
            forward: forward(self.quarter_turn), half_height: section::SLAB_HALF_HEIGHT,
            aspect, depth: section::SLAB_DEPTH, cutaway: None }
    }
}

/// Rotate the shipped oblique camera around world up, keeping its pitch.
pub fn forward(quarter_turn: u8) -> [f32; 3] {
    let [x, y, z] = CameraMode::default().forward();
    match quarter_turn % 4 { 0 => [x,y,z], 1 => [z,y,-x], 2 => [-x,y,-z], _ => [-z,y,x] }
}

#[derive(Clone, Default)]
pub struct SceneHandle {
    pub(crate) input: Rc<RefCell<Option<Rc<Snapshot>>>>,
    pub(crate) error: Rc<RefCell<Option<String>>>,
}

impl SceneHandle {
    pub fn snapshot(&self) -> Option<Rc<Snapshot>> { self.input.borrow().clone() }
    pub fn last_error(&self) -> Option<String> { self.error.borrow().clone() }
    pub fn clear(&self) -> bool {
        self.error.borrow_mut().take();
        self.input.borrow_mut().take().is_some()
    }
    /// Exact native owner, active token, native awareness and local region
    /// are checked before lowering. A refusal withdraws previous input.
    pub fn update(&self, session: &Session, participant: Id, window: u64, mode: Mode,
        site: Id, quarter_turn: u8) -> Result<bool, String> {
        self.update_geometry(session, participant, window, mode, site, quarter_turn, None)
    }

    /// The same owned-window admission, refined by actual caller geometry.
    /// Native patch membership and Journey absence remain authoritative.
    pub fn update_sighted(&self, session: &Session, participant: Id, window: u64, mode: Mode,
        site: &InSite, sight: &Sight, quarter_turn: u8) -> Result<bool, String> {
        self.update_geometry(session, participant, window, mode, site.site(), quarter_turn, Some((site, sight)))
    }

    fn update_geometry(&self, session: &Session, participant: Id, window: u64, mode: Mode,
        site: Id, quarter_turn: u8, geometry: Option<(&InSite, &Sight)>) -> Result<bool, String> {
        let result: Result<bool, String> = (|| {
            if quarter_turn > 3 { return Err("quarter turn must be zero through three".into()); }
            let mut region = session.sim.region_reading(participant, window, mode, site)?;
            if let Some((space, sight)) = geometry {
                if session.sim.plays(participant) != Some(sight.observer) {
                    return Err("sight observer is not this window's played critter".into());
                }
                let visible = space.refine(&session.sim, mode, sight)?;
                region.bodies.retain(|b| visible.shows(b.id));
            } else if mode == Mode::Survival
                && session.sim.plays(participant).is_some_and(|id|
                    session.sim.state().population.get(id).is_none_or(|e| e.resident_site().is_none())) {
                return Err("played critter is off-site; close-up withdrawn".into());
            }
            let digest = isocosm::digest(&(session.sim.genesis(), session.sim.state(), &region,
                geometry.map(|(_, sight)| sight)));
            let mut fingerprint = [0; 4];
            for (i, word) in fingerprint.iter_mut().enumerate() {
                *word = u64::from_str_radix(&digest[i * 16..(i + 1) * 16], 16).map_err(|e| e.to_string())?;
            }
            if self.input.borrow().as_ref().is_some_and(|old|
                old.fingerprint == fingerprint && old.quarter_turn == quarter_turn) { return Ok(false); }
            let snapshot = lower(session, region, fingerprint, quarter_turn, geometry)?;
            *self.input.borrow_mut() = Some(Rc::new(snapshot));
            self.error.borrow_mut().take();
            Ok(true)
        })();
        if let Err(why) = &result {
            self.clear();
            *self.error.borrow_mut() = Some(why.clone());
        }
        result
    }
}

fn lower(session: &Session, region: RegionReading, fingerprint: [u64; 4], quarter_turn: u8,
    geometry: Option<(&InSite, &Sight)>)
    -> Result<Snapshot, String> {
    // Use the explicitly watched site. The standalone first-site fallback
    // is deliberately not a host admission or an access grant.
    let volume = match geometry { Some((space, _)) => space.volume().clone(),
        None => mesocosm_runtime::runtime::placing::lift(&session.sim, region.site)? };
    let ground = volume.ground().clone();
    let mut bodies = Vec::new();
    for (index, body) in region.bodies.into_iter().enumerate() {
        let entity = session.sim.state().population.get(body.id).ok_or("native body disappeared")?;
        if !entity.alive || entity.resident_site() != Some(region.site) || entity.body.as_ref() != Some(&body.document)
            || entity.body_revision != body.revision || entity.patch != body.patch {
            return Err("native close-up body changed before lowering".into());
        }
        let at = match geometry {
            Some((_, sight)) => {
                let cell = if body.id == sight.observer { Some(sight.eye) }
                    else { sight.targets.get(&body.id).copied() };
                // A truth/debug view can contain bodies without supplied geometry;
                // the geometric scene omits them rather than inventing positions.
                let Some(cell) = cell else { continue; };
                volume.to_ground(cell)
            },
            None => body.patch.filter(|p| volume.holds(p.cell[0], p.cell[2]))
                .map(|p| volume.to_ground(p.cell)).unwrap_or_else(|| layout::spot(&ground, index)),
        };
        bodies.push(PlacedBody { id: body.id, document: body.document, at,
            tint: scene::tint_of(isocosm::kingdom::of(entity)), alive: true });
    }
    let volumes = DeclaredExtentVolumes::from_documents(bodies.iter().map(|body| &body.document), 1);
    Ok(Snapshot { scene: SiteScene { site: region.site, ground, window: volume, bodies,
        played: region.played, volumes }, fingerprint, quarter_turn })
}

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
pub use isometer::core::Yaw as RootYaw;
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

    /// An actual close-up lowers ONLY saved physical poses inside the same
    /// admitted native window and awareness cut. No advisory layout is used.
    pub fn update_spatial(&self, session: &Session, participant: Id, window: u64, mode: Mode,
        site: Id, quarter_turn: u8, game: &mesocosm_runtime::spatial::Game) -> Result<bool, String> {
        let result = (|| {
            if quarter_turn > 3 { return Err("quarter turn must be zero through three".into()); }
            game.version()?;
            let region = session.sim.region_reading(participant, window, mode, site)?;
            let subject = session.sim.plays(participant);
            if mode == Mode::Survival && subject.is_some_and(|id|
                session.sim.state().population.get(id).is_none_or(|e| e.resident_site() != Some(site))) {
                return Err("played critter is off-site; physical close-up withdrawn".into());
            }
            let visible = isocosm::directing::readings::view(&session.sim, subject.unwrap_or(Id::MAX), mode);
            let volume = if let Some(held) = subject.and_then(|id| game.members.get(&id))
                .filter(|held| held.site.0 == site) {
                held.realized(&session.sim)?;
                session.sim.volume(site, held.window.min, held.window.max, held.window.depth)?
            } else if mode == Mode::Survival {
                return Err("played member has no actual saved close-up geometry".into());
            } else {
                // A creative lens's native window frames the picture only;
                // it still supplies no missing body position or control.
                mesocosm_runtime::runtime::placing::lift(&session.sim, site)?
            };
            let bodies = scene::placed_physical(&session.sim, site, &volume, &visible, game)?;
            let digest = isocosm::digest(&(session.sim.genesis(), session.sim.state(), &region, game));
            let mut fingerprint = [0; 4];
            for (i, word) in fingerprint.iter_mut().enumerate() {
                *word = u64::from_str_radix(&digest[i * 16..(i + 1) * 16], 16).map_err(|e| e.to_string())?;
            }
            if self.input.borrow().as_ref().is_some_and(|old|
                old.fingerprint == fingerprint && old.quarter_turn == quarter_turn) { return Ok(false); }
            let volumes = DeclaredExtentVolumes::from_documents(bodies.iter().map(|b| &b.document), 1);
            let snapshot = Snapshot { scene:SiteScene { site, ground:volume.ground().clone(), window:volume,
                bodies, played:region.played, volumes }, fingerprint, quarter_turn };
            *self.input.borrow_mut() = Some(Rc::new(snapshot));
            self.error.borrow_mut().take();
            Ok(true)
        })();
        if let Err(why) = &result { self.clear(); *self.error.borrow_mut() = Some(why.clone()); }
        result
    }

    /// A product validates its own saved root bindings against this same
    /// borrowed Session. The picture then uses the admitted local region's
    /// current native body documents, and gives missing roots no layout pose.
    pub fn update_roots(&self, session: &Session, participant: Id, window: u64, mode: Mode,
        site: Id, quarter_turn: u8, frame: &isocosm::operations::Window,
        roots: &std::collections::BTreeMap<Id, mesocosm_runtime::spatial::Pose>) -> Result<bool, String> {
        let result: Result<bool, String> = (|| {
            if quarter_turn > 3 { return Err("quarter turn must be zero through three".into()); }
            if frame.depth < 0 { return Err("physical picture needs a nonnegative authored depth".into()); }
            let region = session.sim.region_reading(participant, window, mode, site)?;
            if roots.is_empty() || mode == Mode::Survival
                && region.played.is_none_or(|id| !roots.contains_key(&id)) {
                return Err("physical picture has no current root for its played native subject".into());
            }
            let volume = session.sim.volume(site, frame.min, frame.max, frame.depth)?;
            let mut bodies = Vec::new();
            for body in &region.bodies {
                let Some(pose) = roots.get(&body.id) else { continue; };
                let member = session.sim.state().population.get(body.id).ok_or("native picture member is absent")?;
                if member.resident_site() != Some(site) || member.body.as_ref() != Some(&body.document)
                    || member.body_revision != body.revision || member.patch != body.patch
                    || member.alive != body.alive {
                    return Err("native picture body changed before root lowering".into());
                }
                if !volume.holds(pose.at[0], pose.at[2]) { continue; }
                let cell = volume.to_ground(pose.at);
                let base = [volume.min[0], volume.datum, volume.min[1]];
                let offset = [i64::from(volume.extent), 0, i64::from(volume.extent)];
                if (0..3).any(|i| base[i].checked_add(offset[i]).and_then(|b|
                    b.checked_add(i64::from(cell[i]))) != Some(pose.at[i])) {
                    return Err("physical root exceeds Ground coordinate bounds".into());
                }
                bodies.push(PlacedBody { id:body.id, document:body.document.clone(), at:cell,
                    physical:Some(pose.clone()), tint:scene::tint_of(isocosm::kingdom::of(member)),
                    alive:member.alive });
            }
            let digest = isocosm::digest(&(session.sim.genesis(), session.sim.state(), &region, frame, roots));
            let mut fingerprint = [0; 4];
            for (i, word) in fingerprint.iter_mut().enumerate() {
                *word = u64::from_str_radix(&digest[i * 16..(i + 1) * 16], 16).map_err(|e| e.to_string())?;
            }
            if self.input.borrow().as_ref().is_some_and(|old|
                old.fingerprint == fingerprint && old.quarter_turn == quarter_turn) { return Ok(false); }
            let volumes = DeclaredExtentVolumes::from_documents(bodies.iter().map(|b| &b.document), 1);
            let snapshot = Snapshot { scene:SiteScene { site, ground:volume.ground().clone(), window:volume,
                bodies, played:region.played, volumes }, fingerprint, quarter_turn };
            *self.input.borrow_mut() = Some(Rc::new(snapshot)); self.error.borrow_mut().take();
            Ok(true)
        })();
        if let Err(why) = &result { self.clear(); *self.error.borrow_mut() = Some(why.clone()); }
        result
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
        if entity.resident_site() != Some(region.site) || entity.body.as_ref() != Some(&body.document)
            || entity.body_revision != body.revision || entity.patch != body.patch
            || entity.alive != body.alive {
            return Err("native close-up body changed before lowering".into());
        }
        // This advisory path has no authenticated saved root. Resident
        // remains are rendered by update_spatial/update_roots instead.
        if !body.alive { continue; }
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
        bodies.push(PlacedBody { id: body.id, document: body.document, at, physical:None,
            tint: scene::tint_of(isocosm::kingdom::of(entity)), alive: entity.alive });
    }
    let volumes = DeclaredExtentVolumes::from_documents(bodies.iter().map(|body| &body.document), 1);
    Ok(Snapshot { scene: SiteScene { site: region.site, ground, window: volume, bodies,
        played: region.played, volumes }, fingerprint, quarter_turn })
}

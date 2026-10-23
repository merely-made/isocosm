// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Immutable terrain and places shared by providers and native validators.
use crate::{Result, Simulation, operations::Window, schema::Id};
use isometer_space::{volume::Volume, places::{Places, Rules}};
use std::sync::{Arc, Mutex};

const CAPACITY: usize = 64;

#[derive(Clone, Debug, PartialEq, Eq)]
struct Key { site: Id, window: Window, terrain: String }
impl Key {
    fn of(sim: &Simulation, site: Id, window: &Window) -> Self {
        // View::over reads only the skeleton's three conditions. Weather and
        // other biological conditions still change the full admission revision,
        // but cannot change this derived volume or its places.
        let g = sim.genesis();
        let sites: Vec<_> = sim.state().sites.iter().map(|(id, site)|
            (id, site.terrain_seed, g.rules.skeleton.as_ref().map(|s| [
                site.conditions.get(&s.elevation), site.conditions.get(&s.relief),
                site.conditions.get(&s.water)]), &site.routes)).collect();
        let terrain = crate::digest(&(g.seed, &g.world, &g.rules.skeleton, sites, &sim.state().edits));
        Self { site, window: window.clone(), terrain }
    }
}

#[derive(Debug)]
pub struct SiteGeometry { key: Key, rules: Rules, pub volume: Arc<Volume>, pub places: Places }
impl SiteGeometry {
    pub fn matches(&self, sim: &Simulation, site: Id, window: &Window, rules: Rules) -> bool {
        self.rules == rules && self.key == Key::of(sim, site, window)
    }
}

#[derive(Debug)]
struct Entry { key: Key, volume: Arc<Volume>, places: Vec<Arc<SiteGeometry>> }

/// Clones share sixty-four terrain windows, each with at most four place rule sets.
/// Volume-only arrival searches and native validators use the same terrain.
#[derive(Clone, Debug, Default)]
pub(crate) struct Cache(Arc<Mutex<Vec<Entry>>>);
impl Simulation {
    pub fn site_volume(&self, site: Id, window: &Window) -> Result<Arc<Volume>> {
        let key = Key::of(self, site, window);
        {
            let mut entries = self.spatial_geometry.0.lock().map_err(|_| "spatial geometry cache poisoned")?;
            if let Some(index) = entries.iter().position(|entry| entry.key == key) {
                let found = entries.remove(index);
                let volume = found.volume.clone();
                entries.push(found);
                return Ok(volume);
            }
        }
        let span = crate::profile::Span::new("terrain.volume");
        let volume = if super::embodiment::whole_site(self) {
            let view=crate::terrain::View::over(self.genesis(),&self.state().sites)?;
            let edits:Vec<_>=self.state().edits.iter().map(|e|(e.site,e.edit.clone())).collect();
            Volume::paged(&view,site,window.min,window.max,window.depth,&edits)?
        } else {self.volume(site, window.min, window.max, window.depth)?};
        let volume = Arc::new(volume);
        drop(span);
        let mut entries = self.spatial_geometry.0.lock().map_err(|_| "spatial geometry cache poisoned")?;
        if let Some(existing) = entries.iter().find(|entry| entry.key == key) {
            return Ok(existing.volume.clone());
        }
        if entries.len() == CAPACITY { entries.remove(0); }
        entries.push(Entry { key, volume: volume.clone(), places: Vec::new() });
        Ok(volume)
    }

    /// Only the exact terrain/window/place rules are reused. Actor, visit,
    /// body, eye and collision admission remain the caller's current checks.
    pub fn site_geometry(&self, site: Id, window: &Window, rules: Rules) -> Result<Arc<SiteGeometry>> {
        let key = Key::of(self, site, window);
        {
            let mut entries = self.spatial_geometry.0.lock().map_err(|_| "spatial geometry cache poisoned")?;
            if let Some(index) = entries.iter().position(|entry| entry.key == key) {
                let entry = entries.remove(index);
                let found = entry.places.iter().find(|p| p.rules == rules).cloned();
                entries.push(entry);
                if let Some(found) = found { return Ok(found); }
            }
        }
        let volume = self.site_volume(site, window)?;
        let _span = crate::profile::Span::new("terrain.places");
        let places = Places::derive(&volume, rules)?;
        let found = Arc::new(SiteGeometry { key: key.clone(), rules, volume, places });
        let mut entries = self.spatial_geometry.0.lock().map_err(|_| "spatial geometry cache poisoned")?;
        if let Some(entry) = entries.iter_mut().find(|entry| entry.key == key) {
            if let Some(existing) = entry.places.iter().find(|p| p.rules == rules) {
                return Ok(existing.clone());
            }
            if entry.places.len() == 4 { entry.places.remove(0); }
            entry.places.push(found.clone());
        } else {
            if entries.len() == CAPACITY { entries.remove(0); }
            entries.push(Entry { key, volume: found.volume.clone(), places: vec![found.clone()] });
        }
        Ok(found)
    }
}

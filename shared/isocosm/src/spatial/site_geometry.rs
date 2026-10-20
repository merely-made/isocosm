// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Immutable terrain and places shared by providers and native validators.
use crate::{Result, Simulation, operations::Window, schema::Id};
use isometer_space::{volume::Volume, places::{Places, Rules}};
use std::sync::{Arc, Mutex};

const CAPACITY: usize = 8;

#[derive(Clone, Debug, PartialEq, Eq)]
struct Key { site: Id, window: Window, rules: Rules, terrain: String }
impl Key {
    fn of(sim: &Simulation, site: Id, window: &Window, rules: Rules) -> Self {
        Self { site, window: window.clone(), rules, terrain: sim.terrain_revision() }
    }
}

#[derive(Debug)]
pub struct SiteGeometry { key: Key, pub volume: Volume, pub places: Places }
impl SiteGeometry {
    pub fn matches(&self, sim: &Simulation, site: Id, window: &Window, rules: Rules) -> bool {
        self.key == Key::of(sim, site, window, rules)
    }
}

/// Clones share at most eight immutable entries. This is never saved or hashed.
#[derive(Clone, Debug, Default)]
pub(crate) struct Cache(Arc<Mutex<Vec<Arc<SiteGeometry>>>>);
impl Simulation {
    /// Only the exact terrain/window/place rules are reused. Actor, visit,
    /// body, eye and collision admission remain the caller's current checks.
    pub fn site_geometry(&self, site: Id, window: &Window, rules: Rules) -> Result<Arc<SiteGeometry>> {
        let key = Key::of(self, site, window, rules);
        {
            let mut entries = self.spatial_geometry.0.lock().map_err(|_| "spatial geometry cache poisoned")?;
            if let Some(index) = entries.iter().position(|entry| entry.key == key) {
                let found = entries.remove(index);
                entries.push(found.clone());
                return Ok(found);
            }
        }
        let volume = self.volume(site, window.min, window.max, window.depth)?;
        let places = Places::derive(&volume, rules)?;
        let found = Arc::new(SiteGeometry { key, volume, places });
        let mut entries = self.spatial_geometry.0.lock().map_err(|_| "spatial geometry cache poisoned")?;
        if let Some(existing) = entries.iter().find(|entry| entry.key == found.key) {
            return Ok(existing.clone());
        }
        if entries.len() == CAPACITY { entries.remove(0); }
        entries.push(found.clone());
        Ok(found)
    }
}

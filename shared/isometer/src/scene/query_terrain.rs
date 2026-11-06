// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use std::sync::{Arc, Mutex};

use isometer_lens::{BrickMap, bricks::BrickQueryVersion};

use super::Scene;

#[cfg(test)]
mod tests;

/// Keeps the most recently captured terrain available to share. Rendering
/// updates its own map without copying terrain into a snapshot. Old receipts
/// retain their versions even after a different map is captured.
#[derive(Default)]
pub(super) struct TerrainQueryCache(Mutex<Option<CachedTerrain>>);

struct CachedTerrain {
    version: BrickQueryVersion,
    map: Arc<BrickMap>,
}

impl TerrainQueryCache {
    fn capture(&self, map: Option<&BrickMap>) -> (Option<Arc<BrickMap>>, bool) {
        let Some(map) = map else {
            return (None, false);
        };
        let version = map.query_version();
        let mut cache = self.0.lock().expect("terrain query cache poisoned");
        if let Some(cached) = cache.as_mut()
            && (cached.version == *version || map.same_query_payload(&cached.map))
        {
            // A replacement or dirty write can leave the query payload
            // unchanged. Its next capture can compare versions directly.
            cached.version = version.clone();
            return (Some(Arc::clone(&cached.map)), false);
        }
        let map = Arc::new(map.clone());
        *cache = Some(CachedTerrain {
            version: version.clone(),
            map: Arc::clone(&map),
        });
        (Some(map), true)
    }
}

impl Scene {
    /// Freeze one complete encode's CPU queries. A staging producer retains
    /// this separately and publishes it only when its image is accepted.
    pub fn query_snapshot(&self) -> Option<crate::QuerySnapshot> {
        let frame = self.presented?;
        let terrain = self.map.as_ref().filter(|_| frame.terrain);
        let (terrain, copied) = self.query_terrain.capture(terrain);
        Some(crate::QuerySnapshot::new(
            frame,
            [self.width, self.height],
            self.bodies.query_snapshot(),
            terrain,
            copied,
            self.terrain_cell,
            self.bodies.stats.fallback_bodies != 0,
        ))
    }
}

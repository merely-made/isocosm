// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::Reading;
use cosmograph::schema::Id;

impl Reading<'_> {
    /// The native admitted site and edit history supply the lifted space.
    pub fn lift(&self, site: Id, level: u8, at: [u32; 2]) -> Result<isometer_space::Chunk, String> {
        self.session.sim.lift(site, level, at)
    }

    pub fn volume(
        &self, site: Id, min: [i64; 2], max: [i64; 2], depth: i64,
    ) -> Result<isometer_space::volume::Volume, String> {
        self.session.sim.volume(site, min, max, depth)
    }

    pub fn position(&self, site: Id) -> Option<[u32; 2]> {
        cosmograph::map::position(self.genesis(), site)
    }
}
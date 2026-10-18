// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The terrain's light slot.

/// The light the terrain trace reads: the direction toward the sun, the
/// ambient term, and the sun's weight per channel. Terrain brightness is
/// `ambient + sun * max(0, N.L)`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TraceLight {
    pub toward_sun: [f32; 3],
    pub ambient: f32,
    pub sun: [f32; 3],
}

impl Default for TraceLight {
    fn default() -> Self {
        Self { toward_sun: [0.4, 0.8, 0.3], ambient: 0.38, sun: [0.62; 3] }
    }
}

impl TraceLight {
    pub(super) fn packed(self) -> [[f32; 4]; 2] {
        let [x, y, z] = self.toward_sun;
        let [r, g, b] = self.sun;
        [[x, y, z, self.ambient], [r, g, b, 0.0]]
    }
}


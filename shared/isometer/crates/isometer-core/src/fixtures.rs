// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Isometer's own fixtures (wing ruling 720): a seeded rolling ground and a
//! few small bodies, so the family's suites need no simulation to stand on.
//! Behind the `fixtures` feature; nothing a product links.

use crate::body::{Attachment, BodyDocument, PartId, VolumeRef, Yaw};
use crate::ground::{Cavity, Ground, SURFACE_BAND, Terrain};

/// Rolling relief from a seed: two crossed integer waves plus a hashed
/// jitter, a water line, and one sealed room under the middle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rolling {
    pub seed: u64,
}

fn mix(mut z: u64) -> u64 {
    z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// A triangle wave of `period` voxels between 0 and `amp`.
fn wave(at: i32, period: i32, amp: i32) -> i32 {
    let phase = at.rem_euclid(period);
    let half = period / 2;
    let up = if phase < half { phase } else { period - phase };
    up * amp / half.max(1)
}

impl Terrain for Rolling {
    fn sea_level(&self, _extent: i32) -> i32 {
        SURFACE_BAND / 3
    }

    fn surface(&self, _extent: i32, x: i32, z: i32) -> i32 {
        let shift = (mix(self.seed) % 32) as i32;
        let jitter = (mix(self.seed ^ ((x as u64) << 32) ^ z as u32 as u64) % 2) as i32;
        let height = 2 + wave(x + shift, 40, 8) + wave(z - shift, 28, 6) + jitter;
        height.clamp(1, SURFACE_BAND - 1)
    }

    fn cavities(&self, _extent: i32) -> Vec<Cavity> {
        vec![Cavity {
            rooms: vec![([0, 3, 0], 1)],
            route: Vec::new(),
        }]
    }
}

/// The rolling fixture raised over `-extent..extent`.
pub fn ground(seed: u64, extent: i32) -> Ground {
    Ground::grow(&Rolling { seed }, extent)
}

/// A limb hung off `parent` at `offset`, founding stock.
pub fn limb(
    body: &mut BodyDocument,
    parent: PartId,
    tag: u8,
    half_extent: [i32; 3],
    offset: [i32; 3],
) -> PartId {
    let attachment = Attachment {
        parent,
        offset,
        yaw: Yaw::Zero,
    };
    body.attach(VolumeRef::from_tag(tag), half_extent, attachment, None)
        .expect("a fixture limb attaches")
}

/// A small walker: a core, four legs below and a sensor in front, each
/// part's volume tagged by its index plus one.
pub fn walker() -> BodyDocument {
    let mut body = BodyDocument::new(VolumeRef::from_tag(1), [2, 2, 3]);
    let root = body.root;
    for (n, [x, z]) in [[-2, -2], [2, -2], [-2, 2], [2, 2]].into_iter().enumerate() {
        limb(&mut body, root, 2 + n as u8, [0, 2, 0], [x, -4, z]);
    }
    limb(&mut body, root, 6, [1, 1, 1], [0, 0, -4]);
    body
}

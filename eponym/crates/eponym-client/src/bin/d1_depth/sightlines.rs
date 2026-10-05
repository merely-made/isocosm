// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! D1's sightlines: the named world points it judges, whether a frame can
//! judge each, and the projection and box geometry they rest on.

use super::*;

/// One judged sightline: a named world point on a witness surface and what
/// the pixel it projects to must show. Never a hand-tuned pixel.
pub(super) struct ProbeDef {
    pub(super) world: [f32; 3],
    pub(super) expect_cyan: bool,
    pub(super) why: &'static str,
    /// Which pillar owns the probed surface; its box is not an obstacle.
    pub(super) owner: usize,
    /// For a visible-face probe, the face rectangle around the point, so
    /// framing can require the face to project large enough to hold the
    /// sample window. Covered probes need no such guard: the floor around
    /// them is what the window should see.
    pub(super) face: Option<[[f32; 3]; 4]>,
}

/// The four sightlines, off the witness table. West faces the eye's side
/// of the room; the buried probes bracket the floor line on one face.
pub(super) fn probe_defs(pillars: &[Pillar; 3]) -> [ProbeDef; 4] {
    let [front, buried, hidden] = pillars;
    let west = |pillar: &Pillar, y: f32| {
        [
            pillar.min[0] as f32,
            pillar.min[1] as f32 + y,
            pillar.min[2] as f32 + 0.5,
        ]
    };
    let west_face = |pillar: &Pillar, ylo: f32, yhi: f32| {
        let (x, zlo, zhi) = (
            pillar.min[0] as f32,
            pillar.min[2] as f32,
            (pillar.min[2] + pillar.extent[2]) as f32,
        );
        let (ylo, yhi) = (pillar.min[1] as f32 + ylo, pillar.min[1] as f32 + yhi);
        Some([[x, ylo, zlo], [x, ylo, zhi], [x, yhi, zlo], [x, yhi, zhi]])
    };
    [
        ProbeDef {
            world: west(front, 1.5),
            expect_cyan: true,
            why: "the pillar before the wall must cover raymarched rock",
            owner: 0,
            face: west_face(front, 0.0, front.extent[1] as f32),
        },
        ProbeDef {
            world: west(buried, 2.0),
            expect_cyan: true,
            why: "the buried pillar's open span must stay visible (positive control)",
            owner: 1,
            face: west_face(buried, 1.0, buried.extent[1] as f32),
        },
        ProbeDef {
            world: west(buried, 0.8),
            expect_cyan: false,
            why: "the floor must cover the buried pillar base",
            owner: 1,
            face: None,
        },
        ProbeDef {
            world: [
                hidden.min[0] as f32 + 0.5,
                (hidden.min[1] + hidden.extent[1]) as f32,
                hidden.min[2] as f32 + 0.5,
            ],
            expect_cyan: false,
            why: "the wholly sunken pillar must stay invisible under the floor",
            owner: 2,
            face: None,
        },
    ]
}

impl ProbeDef {
    /// Whether this frame's camera can judge the probe: the point projects
    /// well inside the picture, the sightline from the eye crosses neither
    /// other pillar nor the body, and a visible face projects large enough
    /// to hold the sample window. Room rock needs no test here: the carved
    /// chamber is convex, so a segment between in-room points stays in
    /// air, and one to a sub-floor point crosses only the floor meant to
    /// cover it.
    pub(super) fn frames_well(
        &self,
        clip: Mat4,
        eye: [f32; 3],
        pillars: &[Pillar; 3],
        body: ([f32; 3], [f32; 3]),
    ) -> bool {
        let Some(pixel) = project(clip, self.world) else {
            return false;
        };
        if pixel[0] < 6.0
            || pixel[0] > (SIZE[0] - 6) as f32
            || pixel[1] < (CHROME + 6) as f32
            || pixel[1] > (SIZE[1] - 6) as f32
        {
            return false;
        }
        let mut obstacles: Vec<([f32; 3], [f32; 3])> = pillars
            .iter()
            .enumerate()
            .filter(|(index, _)| *index != self.owner)
            .map(|(_, pillar)| pillar_box(pillar))
            .collect();
        obstacles.push(body);
        if obstacles
            .iter()
            .any(|obstacle| segment_hits_box(eye, self.world, *obstacle))
        {
            return false;
        }
        if let Some(face) = self.face {
            let mut low = [f32::MAX, f32::MAX];
            let mut high = [f32::MIN, f32::MIN];
            for corner in face {
                let Some(at) = project(clip, corner) else {
                    return false;
                };
                low = [low[0].min(at[0]), low[1].min(at[1])];
                high = [high[0].max(at[0]), high[1].max(at[1])];
            }
            if (high[0] - low[0]).min(high[1] - low[1]) < 20.0 {
                return false;
            }
        }
        true
    }
}

pub(super) fn project(clip: Mat4, world: [f32; 3]) -> Option<[f32; 2]> {
    let projected = clip * Vec4::new(world[0], world[1], world[2], 1.0);
    if projected.w <= 0.0 {
        return None;
    }
    Some([
        (projected.x / projected.w * 0.5 + 0.5) * SIZE[0] as f32,
        (1.0 - (projected.y / projected.w * 0.5 + 0.5)) * SIZE[1] as f32,
    ])
}

pub(super) fn pillar_box(pillar: &Pillar) -> ([f32; 3], [f32; 3]) {
    (
        [
            pillar.min[0] as f32,
            pillar.min[1] as f32,
            pillar.min[2] as f32,
        ],
        [
            (pillar.min[0] + pillar.extent[0]) as f32,
            (pillar.min[1] + pillar.extent[1]) as f32,
            (pillar.min[2] + pillar.extent[2]) as f32,
        ],
    )
}

pub(super) fn body_box(at: [i32; 3]) -> ([f32; 3], [f32; 3]) {
    (
        [at[0] as f32, at[1] as f32, at[2] as f32],
        [
            (at[0] + 1) as f32,
            at[1] as f32 + mesocosm_core::places::WALKER_HEIGHT as f32,
            (at[2] + 1) as f32,
        ],
    )
}

/// Whether the open segment from `from` toward `to` crosses the box before
/// reaching `to`. The endpoint itself may lie on the box.
pub(super) fn segment_hits_box(
    from: [f32; 3],
    to: [f32; 3],
    (low, high): ([f32; 3], [f32; 3]),
) -> bool {
    let mut enter: f32 = 0.0;
    let mut exit: f32 = 0.999;
    for axis in 0..3 {
        let direction = to[axis] - from[axis];
        if direction.abs() < 1e-6 {
            if from[axis] < low[axis] || from[axis] > high[axis] {
                return false;
            }
            continue;
        }
        let a = (low[axis] - from[axis]) / direction;
        let b = (high[axis] - from[axis]) / direction;
        enter = enter.max(a.min(b));
        exit = exit.min(a.max(b));
    }
    enter <= exit
}

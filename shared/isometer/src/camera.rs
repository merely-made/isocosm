// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! One set of camera numbers for terrain rays, raster depth and body culling.
//!
//! [`SlabCamera`] is an orthographic slab with a free forward vector. A host's
//! named views — Mesocosm's seven `CameraMode` presets, Paredros's
//! `CameraPolicy` — are presets over it: they choose a forward, and everything
//! downstream (the tracer's camera, the raster matrix, the body cut interval,
//! the cull window) is derived here so the four cannot disagree.
//!
//! World up is never a free parameter. A section whose vertical is not the
//! world's vertical stops presenting the axis the section exists to present,
//! and the tracer seeds its rays on a world-vertical wall regardless.

use mesocosm_lens::{SlabWall, TraceCamera};
use mesocosm_render::ClipSlab;

/// World up. Every camera keeps it; it defines the standing wall, not merely
/// the screen basis.
const UP: [f32; 3] = [0.0, 1.0, 0.0];

/// The world-space cutaway the presentation contract names.
///
/// `None` on a camera is the plain slab between the two world-vertical walls.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Cutaway {
    /// Keep only what falls inside this world box, whatever the camera does.
    /// Applied *beside* the camera's own slab, not instead of it: the box is
    /// fixed in the world while the slab turns with the view.
    Bounds { min: [f32; 3], max: [f32; 3] },
    /// Drop everything on the near side of this world plane. The plane
    /// replaces the camera's own front wall, so the kept region is
    /// `dot(normal, point) >= distance` with the camera's depth still
    /// bounding the far side.
    Plane { normal: [f32; 3], distance: f32 },
}

/// The orthographic slab a vessel presents through.
///
/// Every field is public because a host builds one per frame from numbers it
/// already holds; [`SlabCamera::new`] is the checked path for a host that
/// would rather be told no than draw an empty frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SlabCamera {
    pub centre: [f32; 3],
    /// Any direction with a horizontal component; straight down is refused.
    /// Supplied as the host computed it, not renormalized on the way in, so a
    /// preset that is exact stays exact.
    pub forward: [f32; 3],
    pub half_height: f32,
    pub aspect: f32,
    pub depth: f32,
    pub cutaway: Option<Cutaway>,
}

impl SlabCamera {
    /// A camera with no cutaway, or `None` if these numbers cannot frame
    /// anything: a non-finite value, a non-positive extent, or a forward with
    /// no horizontal component to hang a standing wall on.
    pub fn new(
        centre: [f32; 3],
        forward: [f32; 3],
        half_height: f32,
        aspect: f32,
        depth: f32,
    ) -> Option<Self> {
        let camera = Self {
            centre,
            forward,
            half_height,
            aspect,
            depth,
            cutaway: None,
        };
        camera.framable().then_some(camera)
    }

    pub fn with_cutaway(self, cutaway: Option<Cutaway>) -> Self {
        Self { cutaway, ..self }
    }

    fn framable(self) -> bool {
        self.centre.iter().all(|v| v.is_finite())
            && self.forward.iter().all(|v| v.is_finite())
            && [self.half_height, self.aspect, self.depth]
                .iter()
                .all(|v| v.is_finite() && *v > 0.0)
            && normalize(cross(self.forward, UP)).is_some()
    }

    /// Right, up, forward — the orthonormal frame the tracer builds.
    ///
    /// Derived by the **same** construction
    /// [`mesocosm_lens::TraceCamera::orthographic_slab`] uses, because the
    /// cull window and the camera disagreeing about where the slab is would
    /// show bodies that are not drawn and drop bodies that are.
    pub fn basis(self) -> [[f32; 3]; 3] {
        let forward = normalize(self.forward).unwrap_or([0.0, 0.0, -1.0]);
        let right = normalize(cross(forward, UP)).unwrap_or([1.0, 0.0, 0.0]);
        let up = normalize(cross(right, forward)).unwrap_or(UP);
        [right, up, forward]
    }

    /// How far along forward the slab reaches from the centre.
    ///
    /// Read straight off [`SlabWall`] rather than restated here, because the
    /// tracer seeds its rays on that wall and a cull box that disagreed with
    /// it would drop bodies the frame draws. A level camera takes the wall's
    /// own level branch, which returns exactly half the depth. A tilted one
    /// reaches further: its rays begin on an upright wall rather than on the
    /// tilted near plane, so where they enter the slab depends on how high up
    /// the frame they sit.
    ///
    /// The wall is built from [`Self::forward`] exactly as supplied, not from
    /// the renormalized [`Self::basis`], so a host that hands an exact preset
    /// vector gets the same float it got before this crate existed.
    pub fn reach(self) -> f32 {
        SlabWall::new(self.forward, UP, self.half_height, self.aspect, self.depth)
            .map_or(self.depth * 0.5, |wall| wall.reach)
    }

    /// The tracer's camera, or `None` when the numbers cannot frame anything.
    pub fn trace(self) -> Option<TraceCamera> {
        if !self.framable() {
            return None;
        }
        // The constructor's up vector defines the standing wall, not merely
        // the screen basis. Supplying screen-up tilts the ray interval away
        // from the body shader's world-vertical cut slab.
        TraceCamera::orthographic_slab(
            self.centre,
            self.basis()[2],
            UP,
            self.half_height,
            self.aspect,
            self.depth,
        )
    }

    /// The world box this camera shows: what falls outside cannot reach a
    /// pixel, so it is what a roster culls against.
    pub fn window(self) -> SlabWindow {
        SlabWindow {
            centre: self.centre,
            axes: self.basis(),
            half: [
                self.half_height * self.aspect,
                self.half_height,
                self.reach(),
            ],
        }
    }

    /// The one `clip_from_world` terrain and bodies both consume. Standard-z:
    /// depth runs 0 at the front wall to 1 at the back.
    pub fn clip_from_world(self) -> [[f32; 4]; 4] {
        let [right, up, forward] = self.basis();
        let x = right.map(|v| v / (self.half_height * self.aspect));
        let y = up.map(|v| v / self.half_height);
        let z = forward.map(|v| v / (2.0 * (self.reach() + 1.0)));
        [
            [x[0], y[0], z[0], 0.0],
            [x[1], y[1], z[1], 0.0],
            [x[2], y[2], z[2], 0.0],
            [
                -dot(x, self.centre),
                -dot(y, self.centre),
                0.5 - dot(z, self.centre),
                1.0,
            ],
        ]
    }

    /// The world cut the body renderer applies: two world-vertical walls
    /// about the centre, plus whatever the cutaway adds.
    pub fn clip(self) -> ClipSlab {
        let normal = standing_normal(self.forward);
        let middle = dot(normal, self.centre);
        let slab = ClipSlab {
            normal,
            min: middle - self.depth * 0.5,
            max: middle + self.depth * 0.5,
            bounds: None,
        };
        match self.cutaway {
            None => slab,
            Some(Cutaway::Bounds { min, max }) => ClipSlab {
                bounds: Some((min, max)),
                ..slab
            },
            Some(Cutaway::Plane { normal, distance }) => ClipSlab {
                normal: normalize(normal).unwrap_or(slab.normal),
                min: distance,
                max: distance + self.depth,
                bounds: None,
            },
        }
    }

    /// Where a world point lands on screen, in normalized clip coordinates:
    /// x right, y up, each in [-1, 1] inside the frame and outside it beyond.
    ///
    /// The projection is [`Self::clip_from_world`]'s, so a test that names a
    /// pixel names the one the draw wrote. `None` when the numbers cannot
    /// frame anything, or the point is not finite.
    pub fn ndc_of(self, point: [f32; 3]) -> Option<[f32; 2]> {
        if !self.framable() || !point.iter().all(|v| v.is_finite()) {
            return None;
        }
        let matrix = self.clip_from_world();
        // Columns are indexed by world axis, rows by clip axis; orthographic,
        // so w is 1 and there is nothing to divide by.
        let clip = [0, 1].map(|row| {
            (0..3)
                .map(|axis| matrix[axis][row] * point[axis])
                .sum::<f32>()
                + matrix[3][row]
        });
        clip.iter().all(|v| v.is_finite()).then_some(clip)
    }

    /// The texture pixel a world point falls in, for a target of `size`.
    ///
    /// The inverse of the pixel-centre convention a pick uses: coordinates
    /// start at the top left, and a point outside the frame has no pixel.
    pub fn pixel_of(self, point: [f32; 3], size: [u32; 2]) -> Option<[u32; 2]> {
        if size[0] == 0 || size[1] == 0 {
            return None;
        }
        let ndc = self.ndc_of(point)?;
        let x = ((ndc[0] + 1.0) * size[0] as f32 * 0.5 - 0.5).round();
        let y = ((1.0 - ndc[1]) * size[1] as f32 * 0.5 - 0.5).round();
        (x >= 0.0 && y >= 0.0 && x < size[0] as f32 && y < size[1] as f32)
            .then_some([x as u32, y as u32])
    }

    /// How far above and below its centre this camera actually frames, in
    /// world voxels.
    ///
    /// `half_height` for a level camera, and more than that for a tilted one,
    /// whose slab depth leans into the vertical — further than the depth
    /// alone, because its rays begin on the world-vertical front wall rather
    /// than on the tilted near plane. A host's bedrock clamp reads this
    /// rather than the half-height so a tilted frame does not dip below the
    /// world's floor.
    pub fn vertical_half(self) -> f32 {
        let [right, up, forward] = self.basis();
        right[1].abs() * self.half_height * self.aspect
            + up[1].abs() * self.half_height
            + forward[1].abs() * self.reach()
    }
}

/// The oriented box the camera shows, in voxels around its centre.
///
/// **Oriented, not axis-aligned.** The half extents are along the camera's
/// right, up and forward — not along world x, y and z — so the same numbers
/// describe the same slab whichever way the view is turned.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SlabWindow {
    pub centre: [f32; 3],
    /// Right, up and forward, orthonormal — [`SlabCamera::basis`]'s output.
    pub axes: [[f32; 3]; 3],
    /// Half extents along those three axes, in the same order.
    pub half: [f32; 3],
}

impl SlabWindow {
    /// Whether a world position falls inside the window. Position alone, not
    /// the body's extent: a body straddling the cut plane is drawn whole and
    /// the tracer's own ray interval does the trimming.
    pub fn holds(&self, at: [f32; 3]) -> bool {
        let offset = [0, 1, 2].map(|i| at[i] - self.centre[i]);
        (0..3).all(|axis| dot(offset, self.axes[axis]).abs() <= self.half[axis])
    }
}

/// The world-vertical plane normal a forward vector stands on. Level for
/// every camera, so the cut does not depend on how high the frame sits.
fn standing_normal(forward: [f32; 3]) -> [f32; 3] {
    let length = forward[0].hypot(forward[2]);
    [forward[0] / length, 0.0, forward[2] / length]
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a.into_iter().zip(b).map(|(a, b)| a * b).sum()
}

fn normalize(value: [f32; 3]) -> Option<[f32; 3]> {
    let length = dot(value, value).sqrt();
    (length > 1e-6).then(|| [value[0] / length, value[1] / length, value[2] / length])
}

#[cfg(test)]
mod tests;

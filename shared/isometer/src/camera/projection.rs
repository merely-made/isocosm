// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The scene camera's second projection (cases 1131, 1140): a perspective
//! from an eye, answering for every quantity the slab answers for, so the
//! tracer, the cull, the raster and the pick cannot disagree under it.
//!
//! Under [`Projection::Perspective`] a [`SlabCamera`]'s `centre` is the eye,
//! `forward` the view direction, `aspect` the frame's, and `depth` the far
//! distance; `half_height` is not read (a constructor sets it to the half
//! height one voxel ahead of the eye, for a reader's sake). World up stays the
//! screen's up, so a view straight down is refused as the slab refuses it.

use super::{SlabCamera, SlabWindow, UP, cross, dot, normalize};
use isometer_lens::TraceCamera;
use isometer_render::ClipSlab;

/// How a [`SlabCamera`] projects.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Projection {
    /// The orthographic slab between two world-vertical walls.
    #[default]
    Slab,
    /// A pinhole at `centre`: `vertical_fov` radians top to bottom, nothing
    /// nearer than `near` voxels drawn.
    Perspective { vertical_fov: f32, near: f32 },
}

/// The view a perspective camera sees: what lies ahead of its eye between
/// the near and far planes and within its field, in the window's axes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Frustum {
    pub eye: [f32; 3],
    /// The frame's half width and half height one voxel ahead of the eye.
    pub tan: [f32; 2],
    pub near: f32,
    pub far: f32,
}

impl Frustum {
    /// Whether a point is in view.
    pub fn holds(&self, axes: &[[f32; 3]; 3], at: [f32; 3]) -> bool {
        let offset = [0, 1, 2].map(|i| at[i] - self.eye[i]);
        let [x, y, z] = [0, 1, 2].map(|axis| dot(offset, axes[axis]));
        z >= self.near && z <= self.far && x.abs() <= self.tan[0] * z && y.abs() <= self.tan[1] * z
    }

    /// Whether a world box can reach a pixel: no plane of the six has the
    /// whole box behind it. Conservative at the corners, as plane tests are.
    pub fn touches(&self, axes: &[[f32; 3]; 3], (min, max): ([f32; 3], [f32; 3])) -> bool {
        let [right, up, forward] = *axes;
        let side = |n: [f32; 3]| normalize(n).unwrap_or(n);
        // Inward normals through the eye; near and far along forward.
        let planes = [
            side(sub(scale(forward, self.tan[0]), right)),
            side(add(scale(forward, self.tan[0]), right)),
            side(sub(scale(forward, self.tan[1]), up)),
            side(add(scale(forward, self.tan[1]), up)),
        ];
        let outside = |normal: [f32; 3], offset: f32| {
            // The box corner furthest along the normal.
            let corner = [0, 1, 2].map(|i| if normal[i] >= 0.0 { max[i] } else { min[i] });
            dot(normal, corner) < offset
        };
        let eye = dot(forward, self.eye);
        if outside(forward, eye + self.near) || outside(scale(forward, -1.0), -(eye + self.far)) {
            return false;
        }
        planes.into_iter().all(|normal| !outside(normal, dot(normal, self.eye)))
    }
}

impl SlabCamera {
    /// A perspective camera at `eye` looking along `forward`, or `None` if it
    /// cannot frame anything: as [`SlabCamera::new`], and a field of view
    /// outside (0, pi) or a near plane not ahead of the far one.
    pub fn perspective(
        eye: [f32; 3],
        forward: [f32; 3],
        vertical_fov: f32,
        aspect: f32,
        near: f32,
        far: f32,
    ) -> Option<Self> {
        let camera = Self {
            centre: eye,
            forward,
            half_height: (vertical_fov * 0.5).tan(),
            aspect,
            depth: far,
            cutaway: None,
            projection: Projection::Perspective { vertical_fov, near },
        };
        camera.framable().then_some(camera)
    }

    /// First person (case 1143): from the eye a body sees with, along its
    /// heading, tipped by `pitch` radians (positive looks down).
    pub fn first_person(eye: [f32; 3], heading: [f32; 3], pitch: f32, vertical_fov: f32,
        aspect: f32, near: f32, far: f32) -> Option<Self> {
        Self::perspective(eye, pitched(heading, pitch)?, vertical_fov, aspect, near, far)
    }

    /// Over the shoulder: `distance` voxels behind `target` along its
    /// heading and `rise` above it, looking at it.
    #[allow(clippy::too_many_arguments)]
    pub fn over_the_shoulder(target: [f32; 3], heading: [f32; 3], distance: f32, rise: f32,
        vertical_fov: f32, aspect: f32, near: f32, far: f32) -> Option<Self> {
        let flat = normalize([heading[0], 0.0, heading[2]])?;
        let eye = [0, 1, 2].map(|i| target[i] - flat[i] * distance + UP[i] * rise);
        Self::perspective(eye, sub(target, eye), vertical_fov, aspect, near, far)
    }

    /// The perspective numbers, when the camera has them.
    pub fn frustum(self) -> Option<Frustum> {
        match self.projection {
            Projection::Slab => None,
            Projection::Perspective { vertical_fov, near } => {
                let tan = (vertical_fov * 0.5).tan();
                Some(Frustum { eye: self.centre, tan: [tan * self.aspect, tan], near, far: self.depth })
            },
        }
    }

    pub(super) fn perspective_framable(self) -> bool {
        match self.projection {
            Projection::Slab => true,
            Projection::Perspective { vertical_fov, near } => vertical_fov.is_finite()
                && vertical_fov > 0.0 && vertical_fov < std::f32::consts::PI
                && near.is_finite() && near > 0.0 && near < self.depth,
        }
    }

    pub(super) fn perspective_trace(self, frustum: Frustum) -> Option<TraceCamera> {
        let Projection::Perspective { vertical_fov, .. } = self.projection else { return None };
        let forward = self.basis()[2];
        let target = add(self.centre, forward);
        TraceCamera::perspective(self.centre, target, UP, vertical_fov, self.aspect, frustum.far)
    }

    /// The frustum's enclosing box, with the frustum beside it for an exact
    /// cull: residency frames the box, bodies are culled by the frustum.
    pub(super) fn perspective_window(self, frustum: Frustum) -> SlabWindow {
        let axes = self.basis();
        let half_far = frustum.far * 0.5;
        SlabWindow {
            centre: add(self.centre, scale(axes[2], half_far)),
            axes,
            half: [frustum.tan[0] * frustum.far, frustum.tan[1] * frustum.far, half_far],
            frustum: Some(frustum),
        }
    }

    /// Right-handed perspective with standard z: 0 at the near plane, 1 at
    /// the far, columns by world axis as the slab's matrix is laid out.
    pub(super) fn perspective_clip_from_world(self, frustum: Frustum) -> [[f32; 4]; 4] {
        let [right, up, forward] = self.basis();
        let (n, f) = (frustum.near, frustum.far);
        let x = right.map(|v| v / frustum.tan[0]);
        let y = up.map(|v| v / frustum.tan[1]);
        // z_clip = a * view_z + b with w = view_z: depth n -> 0, f -> 1.
        let a = f / (f - n);
        let b = -f * n / (f - n);
        let z = forward.map(|v| v * a);
        let w = forward;
        [
            [x[0], y[0], z[0], w[0]],
            [x[1], y[1], z[1], w[1]],
            [x[2], y[2], z[2], w[2]],
            [-dot(x, self.centre), -dot(y, self.centre), b - dot(z, self.centre), -dot(w, self.centre)],
        ]
    }

    /// No section walls under perspective: the far plane bounds the view, so
    /// the body cut is a slab wide enough to keep the whole frustum; a
    /// cutaway still applies.
    pub(super) fn perspective_clip(self, frustum: Frustum) -> ClipSlab {
        let normal = super::standing_normal(self.forward);
        let middle = dot(normal, self.centre);
        let slab = ClipSlab { normal, min: middle - frustum.far, max: middle + frustum.far, bounds: None };
        super::cut(slab, self.cutaway, frustum.far)
    }

    /// How far above and below the eye the frustum reaches.
    pub(super) fn perspective_vertical_half(self, frustum: Frustum) -> f32 {
        let [right, up, forward] = self.basis();
        frustum.far * (right[1].abs() * frustum.tan[0] + up[1].abs() * frustum.tan[1] + forward[1].abs())
    }
}

/// `heading` tipped down by `pitch` radians about its own right.
fn pitched(heading: [f32; 3], pitch: f32) -> Option<[f32; 3]> {
    let flat = normalize([heading[0], 0.0, heading[2]])?;
    let (sin, cos) = pitch.sin_cos();
    normalize([flat[0] * cos, -sin, flat[2] * cos])
        .filter(|f| normalize(cross(*f, UP)).is_some())
}

fn add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn scale(a: [f32; 3], k: f32) -> [f32; 3] {
    a.map(|v| v * k)
}

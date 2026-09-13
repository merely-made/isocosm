// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! One set of camera numbers for terrain rays, raster depth and body culling.

use super::{CameraMode, SlabWindow};
use mesocosm_lens::{SlabWall, TraceCamera};
use mesocosm_render::ClipSlab;

#[derive(Clone, Copy)]
pub(super) struct View {
    pub mode: CameraMode,
    pub centre: [f32; 3],
    pub half: f32,
    pub aspect: f32,
    pub depth: f32,
    pub pitch: Option<f32>,
    pub bounds: Option<([f32; 3], [f32; 3])>,
}

impl super::Section {
    pub(super) fn view(&self, centre: [f32; 3]) -> View {
        View {
            mode: self.mode,
            centre,
            half: self.half_height,
            aspect: self.aspect(),
            depth: if self.bodies.isolated {
                self.bodies.preview_depth
            } else {
                self.terrarium
                    .as_ref()
                    .map_or(super::SLAB_DEPTH, |view| view.depth())
            },
            pitch: self.terrarium.as_ref().map(|view| view.pitch()),
            bounds: if self.bodies.isolated {
                None
            } else {
                self.terrarium.as_ref().map(|view| view.bounds())
            },
        }
    }
}

impl View {
    pub fn basis(self) -> [[f32; 3]; 3] {
        camera_basis(self.mode, self.pitch)
    }

    fn reach(self) -> f32 {
        if self.pitch.is_none() && self.depth == super::SLAB_DEPTH {
            return self.mode.slab_reach(self.half, self.aspect);
        }
        let forward = self.basis()[2];
        SlabWall::new(forward, [0.0, 1.0, 0.0], self.half, self.aspect, self.depth)
            .map_or(self.depth * 0.5, |wall| wall.reach)
    }

    pub fn trace(self) -> Option<TraceCamera> {
        if self.centre.iter().any(|v| !v.is_finite())
            || [self.half, self.aspect, self.depth]
                .iter()
                .any(|v| !v.is_finite() || *v <= 0.0)
            || self.pitch.is_some_and(|pitch| !pitch.is_finite())
        {
            return None;
        }
        let forward = self.basis()[2];
        // The constructor's up vector defines the standing wall, not merely
        // the screen basis. Supplying screen-up tilts the ray interval away
        // from the body shader's world-vertical cut slab.
        TraceCamera::orthographic_slab(
            self.centre,
            forward,
            [0.0, 1.0, 0.0],
            self.half,
            self.aspect,
            self.depth,
        )
    }

    pub fn window(self) -> SlabWindow {
        SlabWindow {
            centre: self.centre,
            axes: self.basis(),
            half: [self.half * self.aspect, self.half, self.reach()],
        }
    }

    pub fn matrix(self) -> [[f32; 4]; 4] {
        let [right, up, forward] = self.basis();
        let x = right.map(|v| v / (self.half * self.aspect));
        let y = up.map(|v| v / self.half);
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

    pub fn clip(self) -> ClipSlab {
        let forward = if self.pitch.is_none() {
            self.mode.forward()
        } else {
            self.basis()[2]
        };
        let length = (forward[0] * forward[0] + forward[2] * forward[2]).sqrt();
        let normal = [forward[0] / length, 0.0, forward[2] / length];
        let middle = dot(normal, self.centre);
        ClipSlab {
            normal,
            min: middle - self.depth * 0.5,
            max: middle + self.depth * 0.5,
            bounds: self.bounds,
        }
    }
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a.into_iter().zip(b).map(|(a, b)| a * b).sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pitched_rays_start_and_end_on_the_same_standing_walls_as_body_clipping() {
        for pitch in [12.0, 45.0] {
            let mut mode = CameraMode::TerrariumEast;
            for _ in 0..4 {
                let view = View {
                    mode,
                    centre: [3.0, 200.0, -7.0],
                    half: 5.0,
                    aspect: 1.0,
                    depth: 16.0,
                    pitch: Some(pitch),
                    bounds: None,
                };
                let camera = view.trace().unwrap();
                let clip = view.clip();
                for ndc in [[0.0, 0.5], [-0.75, -0.75], [0.75, 0.75]] {
                    let (origin, direction) = camera.ray_at(ndc).unwrap();
                    let end = [0, 1, 2].map(|i| origin[i] + direction[i] * camera.far());
                    assert!(
                        (dot(clip.normal, origin) - clip.min).abs() < 1e-4,
                        "{mode:?}/{pitch}: front wall"
                    );
                    assert!(
                        (dot(clip.normal, end) - clip.max).abs() < 1e-4,
                        "{mode:?}/{pitch}: far wall"
                    );
                    for point in [origin, end] {
                        let matrix = view.matrix();
                        let depth =
                            matrix[3][2] + (0..3).map(|i| matrix[i][2] * point[i]).sum::<f32>();
                        assert!(
                            (0.0..=1.0).contains(&depth),
                            "standing interval must fit raster depth"
                        );
                    }
                }
                mode = mode.quarter_turn(false);
            }
        }
    }
    #[test]
    fn variable_pitch_and_depth_match_traced_rays_after_every_turn() {
        for pitch in [0.0, 12.0, 45.0] {
            let mut mode = CameraMode::TerrariumEast;
            for _ in 0..4 {
                let view = View {
                    mode,
                    centre: [-8.0, 23.0, 16.0],
                    half: 38.0,
                    aspect: 16.0 / 9.0,
                    depth: 180.0,
                    pitch: Some(pitch),
                    bounds: None,
                };
                let camera = serde_json::to_value(view.trace().unwrap()).unwrap();
                let vector =
                    |name: &str| [0, 1, 2].map(|i| camera[name][i].as_f64().unwrap() as f32);
                let origin = vector("origin");
                let right = vector("right");
                let up = vector("up");
                let direction = vector("forward");
                let wall = vector("wall");
                for uv in [[-0.8, 0.5], [0.0, 0.0], [0.6, -0.7]] {
                    let advance = wall[0] * uv[0] + wall[1] * uv[1] + wall[2];
                    let point = [0, 1, 2].map(|i| {
                        origin[i]
                            + right[i] * uv[0]
                            + up[i] * uv[1]
                            + direction[i] * (advance + 40.0)
                    });
                    let matrix = view.matrix();
                    let projected = [0, 1, 2].map(|row| {
                        matrix[3][row]
                            + (0..3).map(|col| matrix[col][row] * point[col]).sum::<f32>()
                    });
                    assert!((projected[0] - uv[0]).abs() < 1e-4);
                    assert!((projected[1] - uv[1]).abs() < 1e-4);
                }
                mode = mode.quarter_turn(false);
            }
        }
    }
}

pub fn camera_basis(mode: CameraMode, pitch: Option<f32>) -> [[f32; 3]; 3] {
    let [right, _, forward] = mode.basis();
    let Some(pitch) = pitch.filter(|_| mode.is_terrarium()) else {
        return mode.basis();
    };
    let pitch = pitch.to_radians();
    let length = forward[0].hypot(forward[2]);
    let forward = [
        forward[0] / length * pitch.cos(),
        -pitch.sin(),
        forward[2] / length * pitch.cos(),
    ];
    let up = [
        right[1] * forward[2] - right[2] * forward[1],
        right[2] * forward[0] - right[0] * forward[2],
        right[0] * forward[1] - right[1] * forward[0],
    ];
    [right, up, forward]
}

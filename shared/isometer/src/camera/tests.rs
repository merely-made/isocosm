// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The camera parity suite, carried over from Mesocosm's `section/camera.rs`
//! and `section/view.rs` with the product's `CameraMode` replaced by the bare
//! forward vectors those presets produce.

use super::*;

/// Mesocosm's shipped slice thickness, used here as a representative depth.
const SLAB_DEPTH: f32 = 16.0;
const OBLIQUE_DEGREES: f32 = 20.0;

/// The forward `CameraMode::Side` produces.
const SIDE: [f32; 3] = [0.0, 0.0, -1.0];
/// The forward `CameraMode::Across` produces.
const ACROSS: [f32; 3] = [-1.0, 0.0, 0.0];

/// The forward `CameraMode::Oblique` produces: yawed off `-z`, then pitched
/// down, both by the same angle.
fn oblique() -> [f32; 3] {
    let (yaw, pitch) = (OBLIQUE_DEGREES.to_radians(), OBLIQUE_DEGREES.to_radians());
    [
        -yaw.sin() * pitch.cos(),
        -pitch.sin(),
        -yaw.cos() * pitch.cos(),
    ]
}

/// The four terrarium horizontals, in the order a quarter turn visits them:
/// east looks across `-x`, then south, west, north.
const HORIZONTALS: [[f32; 3]; 4] = [
    [-1.0, 0.0, 0.0],
    [0.0, 0.0, -1.0],
    [1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0],
];

/// A terrarium forward at an arbitrary pitch: the same vector
/// `camera_basis(mode, Some(pitch))` hands this crate.
fn pitched(horizontal: [f32; 3], degrees: f32) -> [f32; 3] {
    let pitch = degrees.to_radians();
    [
        horizontal[0] * pitch.cos(),
        -pitch.sin(),
        horizontal[2] * pitch.cos(),
    ]
}

fn camera(forward: [f32; 3], centre: [f32; 3], half: f32, aspect: f32) -> SlabCamera {
    SlabCamera {
        centre,
        forward,
        half_height: half,
        aspect,
        depth: SLAB_DEPTH,
        cutaway: None,
    }
}

fn all_presets() -> Vec<[f32; 3]> {
    let mut presets = vec![SIDE, ACROSS, oblique()];
    presets.extend(HORIZONTALS.map(|h| pitched(h, 12.0)));
    presets
}

/// The window is the camera's box, so `across` keeps what it can see and
/// drops what it cannot — the exact inverse of `side` on the same world.
#[test]
fn the_cull_window_turns_with_the_camera() {
    let centre = [0.0, 30.0, 0.0];
    let side = camera(SIDE, centre, 28.0, 1.78).window();
    let across = camera(ACROSS, centre, 28.0, 1.78).window();

    // Far along x, on the cut plane in z: in shot side-on, behind the
    // camera's own slab across.
    let along_x = [40.0, 30.0, 0.0];
    assert!(side.holds(along_x));
    assert!(!across.holds(along_x));

    // And the other way about.
    let along_z = [0.0, 30.0, 40.0];
    assert!(!side.holds(along_z));
    assert!(across.holds(along_z));

    // Whoever is being followed is in shot under every preset, which is what
    // makes the captures comparable at all.
    for forward in all_presets() {
        let window = camera(forward, centre, 28.0, 1.78).window();
        assert!(window.holds(centre), "{forward:?} lost the centre");
    }
}

/// `side` is the control arm and has to stay the shipped framing: the
/// generalized window must agree with the axis-aligned box the tree culled
/// against before this crate existed.
#[test]
fn the_side_window_is_the_axis_aligned_box_it_replaces() {
    let (centre, half_height, aspect) = ([3.0, 30.0, -5.0], 28.0, 1.78);
    let window = camera(SIDE, centre, half_height, aspect).window();
    let old = [half_height * aspect, half_height, SLAB_DEPTH * 0.5];
    for at in [
        [3.0, 30.0, -5.0],
        [52.0, 30.0, -5.0],
        [53.0, 30.0, -5.0],
        [3.0, 57.0, -5.0],
        [3.0, 59.0, -5.0],
        [3.0, 30.0, 2.0],
        [3.0, 30.0, 4.0],
    ] {
        let axis_aligned = (0..3).all(|axis| (at[axis] - centre[axis]).abs() <= old[axis]);
        assert_eq!(
            window.holds(at),
            axis_aligned,
            "the side window moved at {at:?}"
        );
    }
}

/// Every basis the crate hands out is orthonormal. The frame the cull window
/// tests against has to be the frame the tracer marches.
#[test]
fn every_basis_is_orthonormal() {
    for forward in all_presets() {
        let axes = camera(forward, [0.0; 3], 28.0, 1.78).basis();
        for axis in axes {
            assert!(
                (dot(axis, axis) - 1.0).abs() < 1e-5,
                "{forward:?} has a non-unit axis {axis:?}"
            );
        }
        for (a, b) in [(0, 1), (1, 2), (0, 2)] {
            assert!(
                dot(axes[a], axes[b]).abs() < 1e-5,
                "{forward:?}: axes {a} and {b} are not perpendicular"
            );
        }
    }
}

/// A level camera reaches exactly half the depth, and a tilted one reaches
/// past it. The same exactness `CameraMode::slab_reach` asserts on the
/// product side, restated on the vector the product hands over.
#[test]
fn a_level_camera_reaches_exactly_the_half_depth_it_always_did() {
    for forward in [SIDE, ACROSS] {
        for half in [20.0, 28.0, 48.0] {
            for aspect in [1.0, 1.78] {
                assert_eq!(
                    camera(forward, [0.0; 3], half, aspect).reach(),
                    SLAB_DEPTH * 0.5,
                    "{forward:?} moved its slab reach"
                );
            }
        }
    }
    assert!(
        camera(oblique(), [0.0; 3], 28.0, 1.78).reach() > SLAB_DEPTH * 0.5,
        "a tilted section reaches past its half depth"
    );
}

/// The bedrock clamp's floor: a level camera frames exactly its half-height,
/// and a tilted one frames more because its depth leans into the vertical.
/// Read at the widest aspect a host clamps with, which is where Mesocosm's
/// aspect-free `CameraMode::vertical_half` meets this one.
#[test]
fn a_tilted_camera_declares_the_extra_height_it_frames() {
    assert_eq!(camera(SIDE, [0.0; 3], 28.0, 1.0).vertical_half(), 28.0);
    assert_eq!(camera(ACROSS, [0.0; 3], 28.0, 1.0).vertical_half(), 28.0);
    let tilted = camera(oblique(), [0.0; 3], 28.0, 1.0).vertical_half();
    assert!(
        (32.5..33.5).contains(&tilted),
        "28 cos20 plus the seeded slab's reach leaning into y: {tilted}"
    );
}

/// A camera that cannot frame anything says so rather than drawing a void.
#[test]
fn a_camera_with_no_horizontal_forward_is_refused() {
    assert!(SlabCamera::new([0.0; 3], [0.0, -1.0, 0.0], 28.0, 1.78, 16.0).is_none());
    assert!(SlabCamera::new([0.0; 3], SIDE, 0.0, 1.78, 16.0).is_none());
    assert!(SlabCamera::new([0.0; 3], SIDE, 28.0, f32::NAN, 16.0).is_none());
    let good = SlabCamera::new([0.0; 3], SIDE, 28.0, 1.78, 16.0).unwrap();
    assert!(good.trace().is_some());
    assert!(
        camera(SIDE, [f32::INFINITY, 0.0, 0.0], 28.0, 1.78)
            .trace()
            .is_none()
    );
}

/// A `Bounds` cutaway rides beside the camera's own slab; a `Plane` replaces
/// its front wall. Neither touches the world-vertical normal's level-ness.
#[test]
fn a_cutaway_is_carried_into_the_body_cut() {
    let plain = camera(oblique(), [4.0, 20.0, 7.0], 28.0, 1.0);
    let bounded = plain.with_cutaway(Some(Cutaway::Bounds {
        min: [-1.0; 3],
        max: [2.0; 3],
    }));
    assert_eq!(plain.clip().bounds, None);
    assert_eq!(bounded.clip().bounds, Some(([-1.0; 3], [2.0; 3])));
    assert_eq!(bounded.clip().min, plain.clip().min);

    let cut = plain.with_cutaway(Some(Cutaway::Plane {
        normal: [0.0, 0.0, -2.0],
        distance: 3.0,
    }));
    let clip = cut.clip();
    assert_eq!(clip.normal, [0.0, 0.0, -1.0]);
    assert_eq!(clip.min, 3.0);
    assert_eq!(clip.max, 3.0 + SLAB_DEPTH);
    assert_eq!(clip.bounds, None);
}

/// The cut plane is world-vertical under every preset, and its interval is
/// exactly the declared depth.
#[test]
fn the_cut_plane_is_vertical_and_exactly_one_depth_thick() {
    for forward in all_presets() {
        let slab = camera(forward, [4.0, 20.0, 7.0], 28.0, 1.0).clip();
        assert_eq!(slab.normal[1], 0.0);
        assert!((slab.max - slab.min - SLAB_DEPTH).abs() < 1e-5);
    }
}

/// Rays start on the front wall and end on the far wall, for every turn and
/// every pitch, and both ends fit the raster depth interval. Carried from
/// `section/view.rs`.
#[test]
fn pitched_rays_start_and_end_on_the_same_standing_walls_as_body_clipping() {
    for pitch in [12.0, 45.0] {
        for horizontal in HORIZONTALS {
            let view = SlabCamera {
                centre: [3.0, 200.0, -7.0],
                forward: pitched(horizontal, pitch),
                half_height: 5.0,
                aspect: 1.0,
                depth: 16.0,
                cutaway: None,
            };
            let camera = view.trace().unwrap();
            let clip = view.clip();
            for ndc in [[0.0, 0.5], [-0.75, -0.75], [0.75, 0.75]] {
                let (origin, direction) = camera.ray_at(ndc).unwrap();
                let end = [0, 1, 2].map(|i| origin[i] + direction[i] * camera.far());
                assert!(
                    (dot(clip.normal, origin) - clip.min).abs() < 1e-4,
                    "{horizontal:?}/{pitch}: front wall"
                );
                assert!(
                    (dot(clip.normal, end) - clip.max).abs() < 1e-4,
                    "{horizontal:?}/{pitch}: far wall"
                );
                for point in [origin, end] {
                    let matrix = view.clip_from_world();
                    let depth = matrix[3][2] + (0..3).map(|i| matrix[i][2] * point[i]).sum::<f32>();
                    assert!(
                        (0.0..=1.0).contains(&depth),
                        "standing interval must fit raster depth"
                    );
                }
            }
        }
    }
}

/// The raster matrix and the traced rays agree about where a world point
/// lands, at every pitch, depth and turn. Carried from `section/view.rs`.
#[test]
fn variable_pitch_and_depth_match_traced_rays_after_every_turn() {
    for pitch in [0.0, 12.0, 45.0] {
        for horizontal in HORIZONTALS {
            let view = SlabCamera {
                centre: [-8.0, 23.0, 16.0],
                forward: pitched(horizontal, pitch),
                half_height: 38.0,
                aspect: 16.0 / 9.0,
                depth: 180.0,
                cutaway: None,
            };
            let camera = serde_json::to_value(view.trace().unwrap()).unwrap();
            let vector = |name: &str| [0, 1, 2].map(|i| camera[name][i].as_f64().unwrap() as f32);
            let origin = vector("origin");
            let right = vector("right");
            let up = vector("up");
            let direction = vector("forward");
            let wall = vector("wall");
            for uv in [[-0.8, 0.5], [0.0, 0.0], [0.6, -0.7]] {
                let advance = wall[0] * uv[0] + wall[1] * uv[1] + wall[2];
                let point = [0, 1, 2].map(|i| {
                    origin[i] + right[i] * uv[0] + up[i] * uv[1] + direction[i] * (advance + 40.0)
                });
                let matrix = view.clip_from_world();
                let projected = [0, 1, 2].map(|row| {
                    matrix[3][row] + (0..3).map(|col| matrix[col][row] * point[col]).sum::<f32>()
                });
                assert!((projected[0] - uv[0]).abs() < 1e-4);
                assert!((projected[1] - uv[1]).abs() < 1e-4);
            }
        }
    }
}

/// The plane cutaway reaches the body cut, not just the `ClipSlab` struct.
///
/// The section 7.2 case the extraction plan owed: `Bounds` was the only
/// cutaway anything exercised. A world plane six voxels behind the body drops
/// it from the pick even though the camera's own slab holds it, and the same
/// plane at the camera's own front wall changes nothing — so the cut is the
/// plane's, in world coordinates, and not a restatement of the slab.
#[test]
fn a_plane_cutaway_drops_the_near_side_of_a_body_cut() {
    use isometer_core::VolumeRef;
    use isometer_mesh::{BodyMesh, Volume};
    use isometer_render::live_body::{LiveBody, pick_bodies};

    let mesh = BodyMesh::single(VolumeRef::from_tag(1), &Volume::solid([2, 2, 2], 1));
    let bodies = [LiveBody::new(&mesh, [0.0, 0.0, 0.0])];
    let hit = |cutaway| {
        pick_bodies(
            &bodies,
            [0.0, 0.0, 20.0],
            SIDE,
            100.0,
            Some(
                camera(SIDE, [0.0, 0.0, 0.0], 28.0, 1.0)
                    .with_cutaway(cutaway)
                    .clip(),
            ),
        )
        .expect("pick")
    };

    assert!(hit(None).is_some(), "the plain slab holds the body");
    assert!(
        hit(Some(Cutaway::Plane {
            normal: [0.0, 0.0, -1.0],
            distance: 6.0,
        }))
        .is_none(),
        "a plane six voxels along the view drops everything on its near side"
    );
    assert!(
        hit(Some(Cutaway::Plane {
            normal: [0.0, 0.0, -1.0],
            distance: -SLAB_DEPTH * 0.5,
        }))
        .is_some(),
        "the same plane laid on the camera's own front wall cuts nothing"
    );
}

/// The screen pair is the raster matrix read forwards: the centre lands on
/// the centre, the frame edges land on ±1, and the pixel convention is the
/// one a pick inverts.
#[test]
fn a_world_point_lands_where_the_raster_matrix_puts_it() {
    for look in [SIDE, ACROSS, oblique()] {
        let camera = SlabCamera {
            centre: [3.0, 5.0, -2.0],
            forward: look,
            half_height: 4.0,
            aspect: 2.0,
            depth: SLAB_DEPTH,
            cutaway: None,
        };
        let [right, up, _] = camera.basis();
        let ndc = camera.ndc_of(camera.centre).unwrap();
        assert!(ndc[0].abs() < 1e-5 && ndc[1].abs() < 1e-5, "{ndc:?}");

        // One half-height up and one half-width right are the frame corners.
        let corner = [0, 1, 2].map(|axis| {
            camera.centre[axis]
                + right[axis] * camera.half_height * camera.aspect
                + up[axis] * camera.half_height
        });
        let ndc = camera.ndc_of(corner).unwrap();
        assert!(
            (ndc[0] - 1.0).abs() < 1e-5 && (ndc[1] - 1.0).abs() < 1e-5,
            "{ndc:?}"
        );

        // Top-left pixel origin, pixel centres, and nothing outside the frame.
        assert_eq!(camera.pixel_of(camera.centre, [64, 32]), Some([32, 16]));
        assert_eq!(camera.pixel_of(corner, [64, 32]), None);
        let inside = [0, 1, 2]
            .map(|axis| camera.centre[axis] + up[axis] * camera.half_height * (31.0 / 32.0));
        assert_eq!(camera.pixel_of(inside, [64, 32]), Some([32, 0]));
        assert_eq!(camera.pixel_of(camera.centre, [0, 32]), None);
        assert_eq!(camera.ndc_of([f32::NAN; 3]), None);
    }
}

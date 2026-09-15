// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::live_body::PartMaterial;
use mesocosm_core::{VolumeRef, Yaw};
use mesocosm_mesh::{BodyMesh, Volume};

fn cube() -> BodyMesh {
    BodyMesh::single(VolumeRef::from_tag(1), &Volume::solid([1; 3], 1))
}

fn close(actual: [f32; 3], expected: [f32; 3]) {
    for axis in 0..3 {
        assert!(
            (actual[axis] - expected[axis]).abs() < 1e-4,
            "actual {actual:?}, expected {expected:?}"
        );
    }
}

fn attached_box() -> BodyMesh {
    let mut mesh = BodyMesh::single(VolumeRef::from_tag(1), &Volume::solid([2, 2, 4], 1));
    let part = &mut mesh.placements[0];
    part.part = PartId(47);
    part.pivot = [1, 1, 2];
    part.pivot_at = [4, 5, 6];
    part.yaw = Yaw::Quarter;
    mesh
}

#[test]
fn bounds_include_parent_yaw_nonzero_pivot_attachment_scale_and_origin() {
    let mesh = attached_box();
    // Quarter-turned local box has body-space bounds [2,4,5]..[6,6,7].
    // These expectations rotate that box analytically, without draw helpers.
    let mut body = LiveBody::new(&mesh, [10.0, 20.0, 30.0]);
    body.scale = 2.0;
    let root_two = core::f32::consts::SQRT_2;
    for (yaw, expected_min, expected_max) in [
        (
            core::f32::consts::FRAC_PI_2,
            [20.0, 28.0, 18.0],
            [24.0, 32.0, 26.0],
        ),
        (
            core::f32::consts::FRAC_PI_4,
            [10.0 + 7.0 * root_two, 28.0, 30.0 - root_two],
            [10.0 + 13.0 * root_two, 32.0, 30.0 + 5.0 * root_two],
        ),
    ] {
        body.yaw_radians = yaw;
        let (min, max) = body_bounds(body).unwrap().unwrap();
        close(min, expected_min);
        close(max, expected_max);
    }
}

#[test]
fn rotated_bounds_reach_a_region_outside_the_unrotated_extent() {
    let mesh = BodyMesh::single(VolumeRef::from_tag(1), &Volume::solid([8, 1, 1], 1));
    let mut body = LiveBody::new(&mesh, [0.0; 3]);
    let (old_min, _) = body_bounds(body).unwrap().unwrap();
    assert_eq!(old_min[2], 0.0);
    body.yaw_radians = core::f32::consts::FRAC_PI_2;
    let (min, max) = body_bounds(body).unwrap().unwrap();
    close(min, [0.0, 0.0, -8.0]);
    close(max, [1.0, 1.0, 0.0]);
    let hit = pick_bodies(&[body], [-1.0, 0.5, -7.0], [1.0, 0.0, 0.0], 3.0, None)
        .unwrap()
        .unwrap();
    close(hit.point, [0.0, 0.5, -7.0]);
}

#[test]
fn attached_faces_pick_at_independently_calculated_45_and_90_degree_poses() {
    let mesh = attached_box();
    let mut body = LiveBody::new(&mesh, [10.0, 20.0, 30.0]);
    body.scale = 2.0;
    let root_two = core::f32::consts::SQRT_2;
    for (yaw, origin, direction, point, distance) in [
        (
            core::f32::consts::FRAC_PI_2,
            [0.0, 30.0, 22.0],
            [2.0, 0.0, 0.0],
            [20.0, 30.0, 22.0],
            20.0,
        ),
        (
            core::f32::consts::FRAC_PI_4,
            [10.0 + 3.0 * root_two, 30.0, 30.0 + 9.0 * root_two],
            [1.0, 0.0, -1.0],
            [10.0 + 8.0 * root_two, 30.0, 30.0 + 4.0 * root_two],
            10.0,
        ),
    ] {
        body.yaw_radians = yaw;
        let hit = pick_bodies(&[body], origin, direction, 30.0, None)
            .unwrap()
            .unwrap();
        assert_eq!((hit.body_index, hit.part, hit.tied), (0, PartId(47), false));
        close(hit.point, point);
        assert!((hit.distance - distance).abs() < 1e-4);
    }
}

#[test]
fn a_gap_inside_body_bounds_is_not_a_surface() {
    let volume = Volume::new([3, 1, 1], vec![1, 0, 1]).unwrap();
    let mesh = BodyMesh::single(VolumeRef::from_tag(1), &volume);
    let body = LiveBody::new(&mesh, [0.0; 3]);
    assert_eq!(
        body_bounds(body).unwrap(),
        Some(([0.0; 3], [3.0, 1.0, 1.0]))
    );
    assert_eq!(
        pick_bodies(&[body], [1.5, 0.5, -1.0], [0.0, 0.0, 1.0], 4.0, None),
        Ok(None)
    );
    let hit = pick_bodies(&[body], [2.5, 0.5, -1.0], [0.0, 0.0, 1.0], 4.0, None)
        .unwrap()
        .unwrap();
    assert_eq!(hit.point, [2.5, 0.5, 0.0]);
    assert_eq!(hit.distance, 1.0);
}

#[test]
fn nearest_body_and_part_do_not_depend_on_submission_order_or_ray_magnitude() {
    let mut mesh = cube();
    mesh.placements[0].part = PartId(71);
    let near = LiveBody::new(&mesh, [0.0, 0.0, 3.0]);
    let far = LiveBody::new(&mesh, [0.0, 0.0, 8.0]);
    for (bodies, index) in [([far, near], 1), ([near, far], 0)] {
        let hit = pick_bodies(&bodies, [0.5, 0.5, 0.0], [0.0, 0.0, 20.0], 10.0, None)
            .unwrap()
            .unwrap();
        assert_eq!(
            (hit.body_index, hit.part, hit.distance),
            (index, PartId(71), 3.0)
        );
        assert!(!hit.tied);
    }

    mesh.placements[0].pivot_at[2] = 5;
    let mut near_part = mesh.placements[0].clone();
    near_part.part = PartId(29);
    near_part.pivot_at[2] = 2;
    mesh.placements.push(near_part);
    let hit = pick_bodies(
        &[LiveBody::new(&mesh, [0.0; 3])],
        [0.5, 0.5, 0.0],
        [0.0, 0.0, 1.0],
        10.0,
        None,
    )
    .unwrap()
    .unwrap();
    assert_eq!((hit.part, hit.distance), (PartId(29), 2.0));
}

#[test]
fn faces_are_double_sided_and_zero_far_and_edges_are_inclusive() {
    let mesh = cube();
    let bodies = [LiveBody::new(&mesh, [0.0; 3])];
    let inside = pick_bodies(&bodies, [0.5; 3], [1.0, 0.0, 0.0], 0.5, None)
        .unwrap()
        .unwrap();
    assert_eq!((inside.point, inside.distance), ([1.0, 0.5, 0.5], 0.5));
    let on_face = pick_bodies(&bodies, [0.0, 0.5, 0.5], [-1.0, 0.0, 0.0], 0.0, None)
        .unwrap()
        .unwrap();
    assert_eq!(on_face.distance, 0.0);
    let at_far = pick_bodies(&bodies, [-1.0, 0.0, 0.5], [1.0, 0.0, 0.0], 1.0, None)
        .unwrap()
        .unwrap();
    assert_eq!(at_far.point, [0.0, 0.0, 0.5]);
    assert_eq!(
        pick_bodies(
            &bodies,
            [-1.0, 0.0, 0.5],
            [1.0, 0.0, 0.0],
            f32::from_bits(1.0_f32.to_bits() - 1),
            None,
        ),
        Ok(None)
    );
    assert_eq!(
        pick_bodies(&bodies, [-1.0, 0.5, 0.5], [0.0, 1.0, 0.0], 10.0, None),
        Ok(None)
    );
    let edge = pick_bodies(&bodies, [-1.0, -1.0, 0.5], [1.0, 1.0, 0.0], 3.0, None)
        .unwrap()
        .unwrap();
    close(edge.point, [0.0, 0.0, 0.5]);
    assert!(!edge.tied, "two faces of one part are one identity");
}

#[test]
fn slab_rejects_front_faces_without_inventing_cut_caps() {
    let mesh = cube();
    let body = LiveBody::new(&mesh, [0.0; 3]);
    let ray = ([0.5, 0.5, -1.0], [0.0, 0.0, 1.0]);
    let back = pick_bodies(
        &[body],
        ray.0,
        ray.1,
        3.0,
        Some(ClipSlab::new([0.0, 0.0, 2.0], 1.0, 2.0)),
    )
    .unwrap()
    .unwrap();
    assert_eq!((back.point, back.distance), ([0.5, 0.5, 1.0], 2.0));
    assert_eq!(
        pick_bodies(
            &[body],
            ray.0,
            ray.1,
            3.0,
            Some(ClipSlab::new([0.0, 0.0, 1.0], 0.25, 0.75)),
        ),
        Ok(None)
    );
    let behind = LiveBody::new(&mesh, [0.0, 0.0, 3.0]);
    let hit = pick_bodies(
        &[body, behind],
        ray.0,
        ray.1,
        5.0,
        Some(ClipSlab::new([0.0, 0.0, 1.0], 2.0, 4.0)),
    )
    .unwrap()
    .unwrap();
    assert_eq!((hit.body_index, hit.distance), (1, 4.0));
}

#[test]
fn optional_clip_bounds_include_the_boundary_and_reject_points_outside_it() {
    let mesh = cube();
    let bodies = [LiveBody::new(&mesh, [0.0; 3])];
    let mut clip = ClipSlab::new([0.0, 0.0, 1.0], 0.0, 1.0);
    clip.bounds = Some(([0.25, 0.25, 0.0], [0.75, 0.75, 0.0]));
    let hit = pick_bodies(
        &bodies,
        [0.25, 0.75, -1.0],
        [0.0, 0.0, 1.0],
        2.0,
        Some(clip),
    )
    .unwrap()
    .unwrap();
    assert_eq!(hit.point, [0.25, 0.75, 0.0]);
    assert_eq!(
        pick_bodies(&bodies, [0.0, 0.5, -1.0], [0.0, 0.0, 1.0], 2.0, Some(clip)),
        Ok(None)
    );
}

#[test]
fn coincident_parts_use_stable_identity_ties_and_a_nearer_face_clears_them() {
    let mut mesh = cube();
    mesh.placements[0].part = PartId(8);
    let mut duplicate = mesh.placements[0].clone();
    duplicate.part = PartId(2);
    mesh.placements.push(duplicate);
    for _ in 0..2 {
        let body = LiveBody::new(&mesh, [0.0; 3]);
        let hit = pick_bodies(&[body, body], [0.5, 0.5, -1.0], [0.0, 0.0, 1.0], 3.0, None)
            .unwrap()
            .unwrap();
        assert_eq!((hit.body_index, hit.part, hit.tied), (0, PartId(2), true));
        mesh.placements.reverse();
    }
    let single = cube();
    let bodies = [
        LiveBody::new(&mesh, [0.0, 0.0, 3.0]),
        LiveBody::new(&single, [0.0; 3]),
    ];
    let hit = pick_bodies(&bodies, [0.5, 0.5, -1.0], [0.0, 0.0, 1.0], 6.0, None)
        .unwrap()
        .unwrap();
    assert_eq!((hit.body_index, hit.part, hit.tied), (1, PartId(0), false));
}

#[test]
fn empty_geometry_has_no_bounds_or_hits() {
    let mesh = BodyMesh::single(VolumeRef::from_tag(1), &Volume::empty([1; 3]));
    for mesh in [&mesh, &BodyMesh::default()] {
        let body = LiveBody::new(mesh, [0.0; 3]);
        assert_eq!(body_bounds(body), Ok(None));
        assert_eq!(
            pick_bodies(&[body], [0.0; 3], [1.0, 0.0, 0.0], 2.0, None),
            Ok(None)
        );
    }
}

#[test]
fn tiny_positive_scale_and_subnormal_direction_remain_valid() {
    let mesh = cube();
    let mut body = LiveBody::new(&mesh, [0.0; 3]);
    body.scale = f32::MIN_POSITIVE;
    let scale = body.scale;
    let hit = pick_bodies(
        &[body],
        [-scale, scale * 0.5, scale * 0.5],
        [f32::from_bits(1), 0.0, 0.0],
        scale,
        None,
    )
    .unwrap()
    .unwrap();
    assert_eq!(hit.distance, scale);
    assert_eq!(hit.point, [0.0, scale * 0.5, scale * 0.5]);
}

#[test]
fn transformed_geometry_overflow_is_not_a_finite_world_hit() {
    let mesh = BodyMesh::single(VolumeRef::from_tag(1), &Volume::solid([2, 1, 1], 1));
    let mut body = LiveBody::new(&mesh, [0.0; 3]);
    body.scale = 2e38;
    assert_eq!(body_bounds(body), Err(LiveBodyError::InvalidBody));
    assert_eq!(
        pick_bodies(&[body], [2e38, 1e38, 1e38], [1.0, 0.0, 0.0], 3e38, None),
        Err(BodyQueryError::Scene(LiveBodyError::InvalidBody))
    );
}

#[test]
fn invalid_rays_are_distinct_from_invalid_scene_input() {
    for (origin, direction, far) in [
        ([f32::NAN, 0.0, 0.0], [1.0, 0.0, 0.0], 1.0),
        ([0.0; 3], [f32::INFINITY, 0.0, 0.0], 1.0),
        ([0.0; 3], [0.0; 3], 1.0),
        ([0.0; 3], [1.0, 0.0, 0.0], -1.0),
        ([0.0; 3], [1.0, 0.0, 0.0], f32::INFINITY),
        ([0.0; 3], [1.0, 0.0, 0.0], f32::NAN),
    ] {
        assert_eq!(
            pick_bodies(&[], origin, direction, far, None),
            Err(BodyQueryError::InvalidRay)
        );
    }
    let mesh = cube();
    let valid = LiveBody::new(&mesh, [0.0; 3]);
    for invalid in [
        LiveBody {
            origin: [f32::NAN, 0.0, 0.0],
            ..valid
        },
        LiveBody {
            scale: 0.0,
            ..valid
        },
        LiveBody {
            scale: -1.0,
            ..valid
        },
        LiveBody {
            yaw_radians: f32::INFINITY,
            ..valid
        },
        LiveBody {
            tint: [0.0, f32::NAN, 0.0],
            ..valid
        },
    ] {
        assert_eq!(body_bounds(invalid), Err(LiveBodyError::InvalidBody));
        assert_eq!(
            pick_bodies(
                &[valid, invalid],
                [0.5, 0.5, -1.0],
                [0.0, 0.0, 1.0],
                2.0,
                None
            ),
            Err(BodyQueryError::Scene(LiveBodyError::InvalidBody))
        );
    }
}

#[test]
fn missing_mesh_and_invalid_materials_cannot_publish_partial_queries() {
    let mut mesh = cube();
    let missing = VolumeRef::from_tag(99);
    mesh.placements[0].volume = missing;
    let body = LiveBody::new(&mesh, [0.0; 3]);
    let error = LiveBodyError::MissingMesh { volume: missing };
    assert_eq!(body_bounds(body), Err(error));
    assert_eq!(
        pick_bodies(&[body], [0.0; 3], [1.0, 0.0, 0.0], 1.0, None),
        Err(BodyQueryError::Scene(error))
    );
    let mesh = cube();
    let materials = [PartMaterial {
        part: PartId(0),
        material: 0,
        fraction: 1.1,
    }];
    let mut body = LiveBody::new(&mesh, [0.0; 3]);
    body.materials = &materials;
    assert_eq!(body_bounds(body), Err(LiveBodyError::InvalidMaterials));
    assert_eq!(
        pick_bodies(&[body], [0.0; 3], [1.0, 0.0, 0.0], 1.0, None),
        Err(BodyQueryError::Scene(LiveBodyError::InvalidMaterials))
    );
}

#[test]
fn draw_and_query_clip_validation_rejects_nonfinite_or_reversed_bounds() {
    let base = ClipSlab::new([0.0, 1.0, 0.0], -1.0, 1.0);
    for clip in [
        ClipSlab {
            normal: [0.0; 3],
            ..base
        },
        ClipSlab {
            normal: [0.0, f32::NAN, 0.0],
            ..base
        },
        ClipSlab { min: 2.0, ..base },
        ClipSlab {
            max: f32::INFINITY,
            ..base
        },
        ClipSlab {
            bounds: Some(([0.0; 3], [0.0, -1.0, 0.0])),
            ..base
        },
        ClipSlab {
            bounds: Some(([f32::NAN, 0.0, 0.0], [1.0; 3])),
            ..base
        },
        ClipSlab {
            bounds: Some(([0.0; 3], [1.0, f32::INFINITY, 1.0])),
            ..base
        },
    ] {
        assert_eq!(validate_clip(Some(clip)), Err(LiveBodyError::InvalidClip));
        assert_eq!(
            pick_bodies(&[], [0.0; 3], [1.0, 0.0, 0.0], 1.0, Some(clip)),
            Err(BodyQueryError::Scene(LiveBodyError::InvalidClip))
        );
    }
}

// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use isometer::lens::TraceCamera;
use isometer::render::PartMaterial;

#[test]
fn grounded_terrarium_anatomy_fits_the_fixed_volume() {
    let founding = mesocosm_core::Founding::SpacedRoster;
    let world = World::terrarium(7, founding, founding.palette()).unwrap();
    let bounds = super::super::framed_habitat(&world).bounds;
    let scale = super::super::TERRARIUM_BODY_SCALE;
    let host = HostBodies {
        scale,
        ground_anatomy: true,
        ..HostBodies::new()
    };
    const NONE: &[PartMaterial] = &[];
    for organism in &world.organisms {
        let origin = host.scene_body(organism, NONE, None).origin();
        let body = organism.body().aabb();
        assert_eq!(
            origin[1] + body.min[1] as f32 * scale,
            organism.position[1] as f32
        );
        for axis in 0..3 {
            assert!(origin[axis] + body.min[axis] as f32 * scale >= bounds.min[axis] as f32);
            assert!(origin[axis] + body.max[axis] as f32 * scale <= bounds.max[axis] as f32 + 1.0);
        }
    }
}

fn transform(matrix: [[f32; 4]; 4], point: [f32; 3]) -> [f32; 3] {
    [0, 1, 2]
        .map(|row| matrix[3][row] + (0..3).map(|col| matrix[col][row] * point[col]).sum::<f32>())
}

#[test]
fn mesh_projection_matches_traced_rays_in_all_camera_modes() {
    for mode in CameraMode::ALL {
        let centre = [7.0, 31.0, -4.0];
        let [_, up, forward] = mode.basis();
        let camera =
            TraceCamera::orthographic_slab(centre, forward, up, 28.0, 16.0 / 9.0, SLAB_DEPTH)
                .unwrap();
        let camera = serde_json::to_value(camera).unwrap();
        let vector = |name: &str| [0, 1, 2].map(|i| camera[name][i].as_f64().unwrap() as f32);
        let ray_origin = vector("origin");
        let right = vector("right");
        let up = vector("up");
        let direction = vector("forward");
        let wall = vector("wall");
        let matrix = clip_from_world(mode, centre, 28.0, 16.0 / 9.0);
        for uv in [[0.0, 0.0], [-0.75, 0.5], [0.75, -0.5]] {
            // Read the actual uploaded ray parameters; this is the WGSL
            // origin construction, compared against the raster matrix.
            let advance = wall[0] * uv[0] + wall[1] * uv[1] + wall[2];
            let origin = [0, 1, 2]
                .map(|i| ray_origin[i] + right[i] * uv[0] + up[i] * uv[1] + direction[i] * advance);
            let a = transform(matrix, origin);
            let b = transform(matrix, [0, 1, 2].map(|i| origin[i] + direction[i] * 2.0));
            assert!((a[0] - uv[0]).abs() < 1e-5);
            assert!((a[1] - uv[1]).abs() < 1e-5);
            assert!((a[0] - b[0]).abs() < 1e-5 && (a[1] - b[1]).abs() < 1e-5);
            assert!(b[2] > a[2], "standard-z must order the same ray forward");
        }
    }
}

#[test]
fn vertical_cut_plane_is_independent_of_height() {
    for mode in CameraMode::ALL {
        let slab = super::super::view::slab_camera(mode, [4.0, 20.0, 7.0], 28.0, 1.0).clip();
        assert_eq!(slab.normal[1], 0.0);
        assert!((slab.max - slab.min - SLAB_DEPTH).abs() < 1e-5);
    }
}

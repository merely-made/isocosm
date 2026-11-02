// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The perspective projection answers as the slab does (cases 1131, 1140):
//! the traced ray and the rastered pixel agree, the cull keeps what is in
//! view and drops what is not, and the presets frame what they say.

use super::*;
use crate::bodies::placement::intersects;

const SIZE: [u32; 2] = [320, 240];

fn eye_level() -> SlabCamera {
    SlabCamera::perspective([2.0, 5.0, -3.0], [0.6, -0.1, 0.8], 1.0, 4.0 / 3.0, 0.1, 64.0).unwrap()
}

fn transform(m: [[f32; 4]; 4], p: [f32; 3]) -> [f32; 4] {
    [0, 1, 2, 3].map(|row| m[3][row] + (0..3).map(|axis| m[axis][row] * p[axis]).sum::<f32>())
}

fn ahead(camera: SlabCamera, x: f32, y: f32, z: f32) -> [f32; 3] {
    let [right, up, forward] = camera.basis();
    [0, 1, 2].map(|i| camera.centre[i] + right[i] * x + up[i] * y + forward[i] * z)
}

#[test]
fn the_matrix_and_the_ndc_query_agree_and_depth_spans_near_to_far() {
    let camera = eye_level();
    let matrix = camera.clip_from_world();
    for point in [ahead(camera, 0.0, 0.0, 10.0), ahead(camera, -3.0, 2.0, 7.5), ahead(camera, 9.0, -6.0, 40.0)] {
        let clip = transform(matrix, point);
        let ndc = camera.ndc_of(point).unwrap();
        assert!((clip[0] / clip[3] - ndc[0]).abs() < 1e-5 && (clip[1] / clip[3] - ndc[1]).abs() < 1e-5);
    }
    let depth = |z: f32| {
        let clip = transform(matrix, ahead(camera, 0.0, 0.0, z));
        clip[2] / clip[3]
    };
    assert!(depth(0.1).abs() < 1e-4, "near plane at depth 0");
    assert!((depth(64.0) - 1.0).abs() < 1e-4, "far plane at depth 1");
    assert!(depth(5.0) < depth(20.0), "depth grows away from the eye");
    assert_eq!(camera.ndc_of(ahead(camera, 0.0, 0.0, -1.0)), None, "behind the eye is nowhere on screen");
}

/// A voxel's traced hit and its rastered pixel are the same pixel: the ray
/// through the pixel a point rasters to passes within that pixel's own
/// footprint of the point.
#[test]
fn the_traced_ray_and_the_rastered_pixel_coincide() {
    for camera in [
        eye_level(),
        SlabCamera::first_person([0.0, 3.0, 0.0], [1.0, 0.0, 0.0], 0.2, 1.2, 1.5, 0.05, 48.0).unwrap(),
        SlabCamera::over_the_shoulder([4.0, 2.0, 4.0], [0.0, 0.0, 1.0], 6.0, 2.5, 1.0, 1.5, 0.05, 48.0).unwrap(),
    ] {
        let trace = camera.trace().expect("a perspective tracer camera");
        let frustum = camera.frustum().unwrap();
        for point in [ahead(camera, 0.5, -0.25, 6.0), ahead(camera, -2.0, 1.0, 12.0), ahead(camera, 5.0, 3.0, 30.0)] {
            let [px, py] = camera.pixel_of(point, SIZE).expect("in frame");
            let ndc = [(px as f32 + 0.5) / SIZE[0] as f32 * 2.0 - 1.0, 1.0 - (py as f32 + 0.5) / SIZE[1] as f32 * 2.0];
            let (origin, direction) = trace.ray_at(ndc).unwrap();
            let to = [0, 1, 2].map(|i| point[i] - origin[i]);
            let along = dot(to, direction);
            let miss = [0, 1, 2].map(|i| to[i] - direction[i] * along);
            // One pixel's width at the point's depth.
            let z = dot([0, 1, 2].map(|i| point[i] - camera.centre[i]), camera.basis()[2]);
            let pixel = 2.0 * frustum.tan[1] * z / SIZE[1] as f32;
            assert!(dot(miss, miss).sqrt() <= pixel, "{point:?}: ray misses by more than a pixel");
            assert!(along > 0.0 && along <= trace.far());
        }
    }
}

/// A body just inside the frustum is kept and one just outside is culled,
/// on every side, in the same frame.
#[test]
fn the_cull_keeps_what_is_in_view_and_drops_what_is_not() {
    let camera = eye_level();
    let window = camera.window();
    let frustum = camera.frustum().unwrap();
    let z = 20.0;
    let edge = [frustum.tan[0] * z, frustum.tan[1] * z];
    let cube = |at: [f32; 3]| (at.map(|v| v - 0.5), at.map(|v| v + 0.5));
    for (inside, outside) in [
        (ahead(camera, edge[0] - 0.1, 0.0, z), ahead(camera, edge[0] + 1.5, 0.0, z)),
        (ahead(camera, -edge[0] + 0.1, 0.0, z), ahead(camera, -edge[0] - 1.5, 0.0, z)),
        (ahead(camera, 0.0, edge[1] - 0.1, z), ahead(camera, 0.0, edge[1] + 1.5, z)),
        (ahead(camera, 0.0, -edge[1] + 0.1, z), ahead(camera, 0.0, -edge[1] - 1.5, z)),
        (ahead(camera, 0.0, 0.0, 63.0), ahead(camera, 0.0, 0.0, 66.0)),
        (ahead(camera, 0.0, 0.0, 1.0), ahead(camera, 0.0, 0.0, -2.0)),
    ] {
        assert!(window.holds(inside) && intersects(cube(inside), window), "{inside:?} is in view");
        assert!(!window.holds(outside) && !intersects(cube(outside), window), "{outside:?} is out of view");
    }
    // The enclosing box still holds the whole view, so residency frames it.
    let corner = ahead(camera, edge[0] * 3.0, edge[1] * 3.0, 60.0);
    let boxed = SlabWindow { frustum: None, ..window };
    assert!(boxed.holds(ahead(camera, frustum.tan[0] * 63.0, frustum.tan[1] * 63.0, 63.0)));
    assert!(!window.holds(corner));
}

#[test]
fn the_presets_frame_what_they_name() {
    let eye = [10.0, 6.0, 10.0];
    let fp = SlabCamera::first_person(eye, [0.0, 0.0, -1.0], 0.0, 1.0, 1.0, 0.05, 32.0).unwrap();
    assert_eq!(fp.centre, eye);
    assert!(fp.ndc_of(ahead(fp, 0.0, 0.0, 5.0)).unwrap().iter().all(|v| v.abs() < 1e-5), "looks along the heading");
    let target = [4.0, 2.0, 4.0];
    let shoulder = SlabCamera::over_the_shoulder(target, [1.0, 0.0, 0.0], 5.0, 2.0, 1.0, 1.0, 0.05, 32.0).unwrap();
    assert!((shoulder.centre[0] - (target[0] - 5.0)).abs() < 1e-5 && (shoulder.centre[1] - 4.0).abs() < 1e-5);
    assert!(shoulder.ndc_of(target).unwrap().iter().all(|v| v.abs() < 1e-4), "the target is centred");
    // What the slab refuses, the perspective refuses; and a near plane must
    // be ahead of the far one.
    assert!(SlabCamera::perspective(eye, [0.0, -1.0, 0.0], 1.0, 1.0, 0.1, 10.0).is_none());
    assert!(SlabCamera::perspective(eye, [1.0, 0.0, 0.0], 1.0, 1.0, 12.0, 10.0).is_none());
    assert!(SlabCamera::perspective(eye, [1.0, 0.0, 0.0], 3.2, 1.0, 0.1, 10.0).is_none());
    // The slab is untouched: a default camera is still a slab.
    assert_eq!(SlabCamera::new(eye, [1.0, -0.5, 0.0], 4.0, 1.0, 8.0).unwrap().projection, Projection::Slab);
}

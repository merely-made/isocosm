// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Independent voxel ray oracle. Does not read meshes, placement extraction,
//! renderer matrices, or renderer depth queries.
use super::{
    model::{Config, Kind, Workload},
    renderer::Renderer,
};

const SIDE: u32 = 128;
const EXTENT: f32 = 18.0;

fn read(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    depth: bool,
) -> Vec<u8> {
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("population independent oracle"),
        size: u64::from(SIDE * SIDE * 4),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&Default::default());
    let mut source = texture.as_image_copy();
    if depth {
        source.aspect = wgpu::TextureAspect::DepthOnly;
    }
    encoder.copy_texture_to_buffer(
        source,
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(SIDE * 4),
                rows_per_image: Some(SIDE),
            },
        },
        wgpu::Extent3d {
            width: SIDE,
            height: SIDE,
            depth_or_array_layers: 1,
        },
    );
    queue.submit([encoder.finish()]);
    let slice = buffer.slice(..);
    slice.map_async(wgpu::MapMode::Read, |_| {});
    device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    let bytes = slice.get_mapped_range().unwrap().to_vec();
    buffer.unmap();
    bytes
}

// Explicit fixture recipe, independent of model::design and greedy meshing.
fn cells(kind: Kind, design: usize, seed: u64) -> Vec<[f32; 3]> {
    let mut result = Vec::new();
    let mask = if kind == Kind::Geometry {
        design as u16 ^ seed as u16
    } else {
        0
    };
    for x in 0..6 {
        for y in 0..6 {
            for z in 0..6 {
                let notch = x == 5
                    && (1..=4).contains(&y)
                    && (1..=4).contains(&z)
                    && mask & (1 << ((y - 1) * 4 + z - 1)) != 0;
                if !notch {
                    result.push([x as f32 - 3., y as f32, z as f32 - 3.]);
                }
            }
        }
    }
    let n = (design + (seed % 1000) as usize) % 1000;
    let head = if kind == Kind::Assembly {
        [
            (n % 10) as f32 - 5.,
            7. + ((n / 10) % 10) as f32,
            ((n / 100) % 10) as f32 - 5.,
        ]
    } else {
        [0., 7., 0.]
    };
    for x in 0..3 {
        for y in 0..3 {
            for z in 0..3 {
                result.push([
                    head[0] + x as f32 - 1.,
                    head[1] + y as f32,
                    head[2] + z as f32 - 1.,
                ]);
            }
        }
    }
    for x in 0..2 {
        for y in 0..2 {
            for z in 0..5 {
                result.push([x as f32 - 1., y as f32 + 1., z as f32 - 7.]);
            }
        }
    }
    result
}

fn box_hit(origin: [f32; 3], direction: [f32; 3], min: [f32; 3]) -> Option<f32> {
    let mut near = 0_f32;
    let mut far = f32::INFINITY;
    for axis in 0..3 {
        if direction[axis].abs() < 1e-7 {
            if origin[axis] < min[axis] || origin[axis] > min[axis] + 1. {
                return None;
            }
        } else {
            let a = (min[axis] - origin[axis]) / direction[axis];
            let b = (min[axis] + 1. - origin[axis]) / direction[axis];
            near = near.max(a.min(b));
            far = far.min(a.max(b));
        }
    }
    (far >= near).then_some(near)
}

fn ray(x: f32, y: f32) -> ([f32; 3], [f32; 3]) {
    let (s, c) = std::f32::consts::FRAC_PI_4.sin_cos();
    let (sp, cp) = 0.6154797_f32.sin_cos();
    let toward_eye = [c * cp, sp, s * cp];
    let right = [s, 0., -c];
    let up = [-c * sp, cp, -s * sp];
    let distance = EXTENT * 4. + 32.;
    let sx = (2. * x / SIDE as f32 - 1.) * EXTENT;
    let sy = (1. - 2. * y / SIDE as f32) * EXTENT;
    (
        [0, 1, 2].map(|i| [0., 7., 0.][i] + distance * toward_eye[i] + sx * right[i] + sy * up[i]),
        toward_eye.map(|v| -v),
    )
}

fn hits(x: f32, y: f32, yaw: f32, bodies: &[(Vec<[f32; 3]>, [f32; 3])]) -> Vec<f32> {
    let (origin, direction) = ray(x, y);
    let (s, c) = yaw.sin_cos();
    let inverse = |p: [f32; 3]| [c * p[0] - s * p[2], p[1], s * p[0] + c * p[2]];
    let mut hits = Vec::new();
    for (cells, at) in bodies {
        let local = inverse([0, 1, 2].map(|i| origin[i] - at[i]));
        let dir = inverse(direction);
        if let Some(hit) = cells
            .iter()
            .filter_map(|min| box_hit(local, dir, *min))
            .min_by(f32::total_cmp)
        {
            hits.push(hit);
        }
    }
    hits.sort_by(f32::total_cmp);
    hits
}

#[test]
fn population_pixels_match_independent_voxel_occupancy_and_nearest_depth() {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let adapter = pollster::block_on(instance.request_adapter(&Default::default()))
        .expect("oracle GPU adapter");
    let (device, queue) = pollster::block_on(adapter.request_device(&Default::default())).unwrap();
    for kind in Kind::ALL {
        let mut workload = Workload::new(Config {
            bodies: 2,
            designs: 2,
            kind,
            seed: 7,
            ..Config::default()
        })
        .unwrap();
        // Deliberately overlapping projections, nearer body submitted first:
        // painter overwrite would select the wrong surface on the overlap.
        workload.instances[0].origin = [1.5, 1.5, 1.5];
        workload.instances[1].origin = [0.; 3];
        let bodies: Vec<_> = (0..2)
            .map(|i| {
                (
                    cells(kind, if kind == Kind::Appearance { 0 } else { i }, 7),
                    workload.instances[i].origin,
                )
            })
            .collect();
        let mut renderer = Renderer::new(&device, &queue, (SIDE, SIDE), &workload).unwrap();
        renderer.oracle_extent(EXTENT);
        for yaw in [0., 0.41] {
            renderer
                .render(&device, &queue, &workload, yaw, [1.; 3])
                .unwrap();
            let rgba = read(&device, &queue, renderer.oracle_texture(), false);
            let depth = read(&device, &queue, renderer.oracle_depth(), true);
            let background = &rgba[..4];
            let mut foreground_count = 0;
            let mut background_count = 0;
            let mut overlap_count = 0;
            for y in 0..SIDE {
                for x in 0..SIDE {
                    let center = hits(x as f32 + 0.5, y as f32 + 0.5, yaw, &bodies);
                    // Discard only boundary pixels whose nearby rays change
                    // coverage or nearest surface depth discontinuously.
                    let stable = [(0.3, 0.5), (0.7, 0.5), (0.5, 0.3), (0.5, 0.7)]
                        .into_iter()
                        .all(|(dx, dy)| {
                            let nearby = hits(x as f32 + dx, y as f32 + dy, yaw, &bodies);
                            match (center.first(), nearby.first()) {
                                (None, None) => true,
                                (Some(a), Some(b)) => (a - b).abs() < 0.3,
                                _ => false,
                            }
                        });
                    if !stable {
                        continue;
                    }
                    let offset = ((y * SIDE + x) * 4) as usize;
                    let observed =
                        f32::from_le_bytes(depth[offset..offset + 4].try_into().unwrap());
                    if let Some(distance) = center.first() {
                        let far = 2. * (EXTENT * 4. + 32.) + EXTENT * 4.;
                        let expected = (distance - 0.1) / (far - 0.1);
                        assert!(
                            (observed - expected).abs() < 2e-5,
                            "{kind:?} yaw{yaw} ({x},{y}): depth {observed} expected {expected}"
                        );
                        assert_ne!(&rgba[offset..offset + 4], background);
                        foreground_count += 1;
                        if center.len() > 1 && center[1] - center[0] > 0.5 {
                            overlap_count += 1;
                        }
                    } else {
                        assert_eq!(observed, 1.0, "{kind:?} unexpected geometry at {x},{y}");
                        assert_eq!(&rgba[offset..offset + 4], background);
                        background_count += 1;
                    }
                }
            }
            assert!(
                foreground_count > 100 && background_count > 1000 && overlap_count > 20,
                "non-vacuous {kind:?}/{yaw}: foreground {foreground_count}, background {background_count}, overlap {overlap_count}"
            );
        }
    }
}

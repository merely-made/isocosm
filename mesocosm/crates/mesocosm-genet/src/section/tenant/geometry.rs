// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Palette faces from Isometer's posed geometry, cut by its query slab.

use isometer::SlabCamera;
use isometer::render::live_body::posed_quad;
use isometer::render::{ClipSlab, LiveBody, face_shade, material_colour};
use tenant::{Palette, PaletteMesh};

pub(super) fn camera(slab: SlabCamera) -> tenant::Camera {
    let [right, up, forward] = slab.basis();
    let reach = slab.reach() + 1.0;
    let eye = [0, 1, 2].map(|i| slab.centre[i] - forward[i] * reach);
    tenant::Camera {
        view: [
            [right[0], up[0], -forward[0], 0.0],
            [right[1], up[1], -forward[1], 0.0],
            [right[2], up[2], -forward[2], 0.0],
            [-dot(right, eye), -dot(up, eye), dot(forward, eye), 1.0],
        ],
        projection: [
            [1.0 / (slab.half_height * slab.aspect), 0.0, 0.0, 0.0],
            [0.0, 1.0 / slab.half_height, 0.0, 0.0],
            [0.0, 0.0, -1.0 / (2.0 * reach), 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ],
        near: 0.0,
        far: 2.0 * reach,
    }
}

pub(super) fn mesh(
    body: LiveBody<'_>,
    clip: ClipSlab,
) -> Result<(PaletteMesh, Palette, usize), String> {
    let mut positions = Vec::new();
    let mut colours = Vec::new();
    let mut parts = 0;
    for placement in &body.mesh.placements {
        let part_mesh = body
            .mesh
            .mesh_for(placement.volume)
            .ok_or("unresolved body mesh")?;
        let before = positions.len();
        let mut fractions = [0.0; 5];
        for material in body.materials.iter().filter(|m| m.part == placement.part) {
            let slot = fractions
                .get_mut(material.material as usize)
                .ok_or("unknown tissue channel")?;
            *slot += material.fraction;
        }
        for (index, quad) in part_mesh.quads.iter().enumerate() {
            let world = posed_quad(body, placement.part, index).map_err(|e| format!("{e:?}"))?;
            let base = body
                .palette
                .and_then(|p| p.linear(quad.material))
                .unwrap_or_else(|| material_colour(quad.material));
            let shade = face_shade(quad.axis, quad.positive);
            let colour = [0, 1, 2]
                .map(|i| base[i] * shade * body.tint[i] * if body.focused { 1.08 } else { 1.0 });
            let selected = body.selected_part == Some(placement.part);
            if fractions.iter().all(|v| *v == 0.0) {
                append(
                    &mut positions,
                    &mut colours,
                    &cut(world.to_vec(), clip),
                    accent(colour, selected),
                );
                continue;
            }
            // Density marks are constant per local voxel. Split only faces
            // carrying expression, rather than flattening a greedy rectangle.
            let local = quad.corners();
            let lengths = [length(local[0], local[1]), length(local[0], local[3])];
            for u in 0..lengths[0] {
                for v in 0..lengths[1] {
                    let point = interpolate(
                        local.map(|p| p.map(|x| x as f32)),
                        (u as f32 + 0.5) / lengths[0] as f32,
                        (v as f32 + 0.5) / lengths[1] as f32,
                    );
                    let phase =
                        ((point[0].floor() + 3.0 * point[1].floor() + 5.0 * point[2].floor())
                            / 16.0)
                            .rem_euclid(1.0);
                    let marked = tissue_colour(colour, phase, fractions);
                    let cell = [[u, v], [u + 1, v], [u + 1, v + 1], [u, v + 1]].map(|[x, y]| {
                        interpolate(
                            world,
                            x as f32 / lengths[0] as f32,
                            y as f32 / lengths[1] as f32,
                        )
                    });
                    append(
                        &mut positions,
                        &mut colours,
                        &cut(cell.to_vec(), clip),
                        accent(marked, selected),
                    );
                }
            }
        }
        parts += usize::from(positions.len() > before);
    }
    let (mesh, palette) = PaletteMesh::from_colored(positions, &colours);
    Ok((mesh, palette, parts))
}

fn length(a: [i32; 3], b: [i32; 3]) -> u32 {
    (0..3).map(|i| a[i].abs_diff(b[i])).sum()
}

fn interpolate(points: [[f32; 3]; 4], u: f32, v: f32) -> [f32; 3] {
    [0, 1, 2].map(|i| {
        points[0][i] + (points[1][i] - points[0][i]) * u + (points[3][i] - points[0][i]) * v
    })
}

fn accent(colour: [f32; 3], selected: bool) -> [f32; 3] {
    let amber = [1.0, 0.55, 0.10];
    [0, 1, 2].map(|i| {
        if selected {
            colour[i] * 0.72 + amber[i] * 0.28
        } else {
            colour[i]
        }
    })
}

fn tissue_colour(base: [f32; 3], phase: f32, fractions: [f32; 5]) -> [f32; 3] {
    let marks = [
        [0.82, 0.30, 0.25],
        [0.82, 0.55, 0.20],
        [0.32, 0.72, 0.92],
        [0.34, 0.76, 0.30],
        [0.78, 0.25, 0.72],
    ];
    let mut end = 0.0;
    for (fraction, mark) in fractions.into_iter().zip(marks) {
        end += fraction;
        if phase < end {
            let shade = base.into_iter().fold(0.0, f32::max);
            return [0, 1, 2].map(|i| base[i] * 0.28 + mark[i] * shade * 0.72);
        }
    }
    base
}

fn append(
    positions: &mut Vec<[f32; 3]>,
    colours: &mut Vec<[f32; 4]>,
    polygon: &[[f32; 3]],
    colour: [f32; 3],
) {
    for i in 1..polygon.len().saturating_sub(1) {
        // LiveBody was double-sided. Preserve that cutaway presentation.
        for index in [0, i, i + 1, 0, i + 1, i] {
            positions.push(polygon[index]);
            colours.push([colour[0], colour[1], colour[2], 1.0]);
        }
    }
}

fn cut(mut polygon: Vec<[f32; 3]>, clip: ClipSlab) -> Vec<[f32; 3]> {
    polygon = plane(polygon, clip.normal, clip.min);
    polygon = plane(polygon, clip.normal.map(|v| -v), -clip.max);
    if let Some((min, max)) = clip.bounds {
        for axis in 0..3 {
            let mut normal = [0.0; 3];
            normal[axis] = 1.0;
            polygon = plane(polygon, normal, min[axis]);
            normal[axis] = -1.0;
            polygon = plane(polygon, normal, -max[axis]);
        }
    }
    polygon
}

fn plane(polygon: Vec<[f32; 3]>, normal: [f32; 3], distance: f32) -> Vec<[f32; 3]> {
    let mut out = Vec::new();
    let Some(mut previous) = polygon.last().copied() else {
        return out;
    };
    let mut previous_distance = dot(normal, previous) - distance;
    for point in polygon {
        let current_distance = dot(normal, point) - distance;
        if (current_distance >= 0.0) != (previous_distance >= 0.0) {
            let t = previous_distance / (previous_distance - current_distance);
            out.push([0, 1, 2].map(|i| previous[i] + (point[i] - previous[i]) * t));
        }
        if current_distance >= 0.0 {
            out.push(point)
        }
        previous = point;
        previous_distance = current_distance;
    }
    out
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    (0..3).map(|i| a[i] * b[i]).sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use isometer::core::{BodyDocument, VolumeRef};
    use isometer::mesh::{Volume, VolumeMap, mesh_body};

    #[test]
    fn transformed_face_crossing_the_slab_is_clipped_and_outside_faces_vanish() {
        let volume = VolumeRef::from_tag(19);
        let mut volumes = VolumeMap::new();
        volumes.insert(volume, Volume::solid([2, 2, 2], 1));
        let document = BodyDocument::new(volume, [1, 1, 1]);
        let projected = mesh_body(&document, &volumes).unwrap();
        let mut body = LiveBody::new(&projected, [3.0, 7.0, -1.0]);
        body.scale = 2.0;
        body.yaw_radians = std::f32::consts::FRAC_PI_4;
        let part = projected.placements[0].part;
        let part_mesh = projected.mesh_for(projected.placements[0].volume).unwrap();
        let face = part_mesh
            .quads
            .iter()
            .position(|q| q.axis == 1 && q.positive)
            .unwrap();
        let points = posed_quad(body, part, face).unwrap();
        let min = points.iter().map(|p| p[0]).fold(f32::INFINITY, f32::min);
        let max = points
            .iter()
            .map(|p| p[0])
            .fold(f32::NEG_INFINITY, f32::max);
        let wall = (min + max) * 0.5;
        let clipped = cut(
            points.to_vec(),
            ClipSlab::new([1.0, 0.0, 0.0], wall, max + 1.0),
        );
        assert!(clipped.len() >= 3);
        assert!(clipped.iter().all(|p| p[0] >= wall - 1e-5));
        assert!(clipped.iter().any(|p| (p[0] - wall).abs() < 1e-5));
        let area = |polygon: &[[f32; 3]]| {
            let mut sum = 0.0;
            for i in 0..polygon.len() {
                let a = polygon[i];
                let b = polygon[(i + 1) % polygon.len()];
                sum += a[0] * b[2] - b[0] * a[2];
            }
            sum.abs() * 0.5
        };
        assert!((area(&clipped) / area(&points) - 0.5).abs() < 1e-5);
        assert!(
            cut(
                points.to_vec(),
                ClipSlab::new([1.0, 0.0, 0.0], max + 1.0, max + 2.0)
            )
            .is_empty()
        );
    }

    #[test]
    fn tenant_and_tracer_project_the_same_depth_under_mesocosm_presets() {
        for mode in [
            crate::section::CameraMode::Side,
            crate::section::CameraMode::Across,
            crate::section::CameraMode::Oblique,
            crate::section::CameraMode::TerrariumEast,
            crate::section::CameraMode::TerrariumSouth,
            crate::section::CameraMode::TerrariumWest,
            crate::section::CameraMode::TerrariumNorth,
        ] {
            let slab =
                SlabCamera::new([3.0, 19.0, 7.0], mode.forward(), 28.0, 16.0 / 9.0, 16.0).unwrap();
            let tenant = camera(slab);
            assert!(tenant.is_valid());
            let expected = slab.clip_from_world();
            let actual = tenant.clip_from_world();
            assert!(
                actual
                    .iter()
                    .flatten()
                    .zip(expected.iter().flatten())
                    .all(|(a, b)| (a - b).abs() < 1e-5)
            );
        }
    }
}

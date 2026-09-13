// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Versioned presentation fixtures. These are renderer workloads, not organisms
//! or developmental claims. Geometry variation is bounded exterior notching.

use mesocosm_core::{PartId, VolumeRef};
use mesocosm_mesh::{BodyMesh, Volume};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const MAX_BODIES: usize = 1000;
pub const GRID_STRIDE: f32 = 24.0;
pub const CAMERA_EXTENT: f32 = 500.0;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    Appearance,
    Assembly,
    Geometry,
}
impl Kind {
    #[cfg(test)]
    pub const ALL: [Self; 3] = [Self::Appearance, Self::Assembly, Self::Geometry];
    pub fn label(self) -> &'static str {
        match self {
            Self::Appearance => "appearance",
            Self::Assembly => "assembly",
            Self::Geometry => "geometry",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub bodies: usize,
    pub designs: usize,
    pub seed: u64,
    pub kind: Kind,
    pub mesh_capacity: usize,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            bodies: 16,
            designs: 1,
            seed: 7,
            kind: Kind::Appearance,
            mesh_capacity: 1024,
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct WorkloadStats {
    pub body_count: usize,
    pub design_count: usize,
    pub unique_meshes: usize,
    pub assembly_count: usize,
    pub occupancy_count: usize,
    pub part_instances: usize,
    pub unique_quads: usize,
    pub instance_quads: usize,
    /// CPU meshing calls during preparation, including repeated volume copies.
    pub cpu_mesh_builds: usize,
    pub cpu_retained_meshes: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct Instance {
    pub design: usize,
    pub origin: [f32; 3],
    pub tint: [f32; 3],
}

#[derive(Clone, Debug)]
pub struct Workload {
    pub config: Config,
    pub stats: WorkloadStats,
    /// FNV-1a-64 receipt, not a cryptographic content address.
    pub digest: String,
    pub preparation_us: u64,
    pub(super) designs: Vec<Vec<BodyMesh>>,
    pub(super) instances: Vec<Instance>,
    pub(super) bounds: Vec<([f32; 3], [f32; 3])>,
}

impl Workload {
    pub fn new(config: Config) -> Result<Self, String> {
        let started = std::time::Instant::now();
        if !(1..=MAX_BODIES).contains(&config.bodies) {
            return Err("population must be 1..1000".into());
        }
        if config.designs == 0 || config.designs > config.bodies {
            return Err("design count must be 1..body count".into());
        }
        if !(1..=4096).contains(&config.mesh_capacity) {
            return Err("mesh capacity must be 1..4096".into());
        }
        let unique = if config.kind == Kind::Geometry {
            config.designs + 2
        } else {
            3
        };
        if unique > config.mesh_capacity {
            return Err(format!(
                "population requires {unique} live mesh keys; capacity is {}",
                config.mesh_capacity
            ));
        }
        let mesh_designs = if config.kind == Kind::Appearance {
            1
        } else {
            config.designs
        };
        let designs: Vec<_> = (0..mesh_designs)
            .map(|i| design(config.kind, i, config.seed))
            .collect();
        let bounds = designs
            .iter()
            .map(|parts| {
                let mut min = [f32::INFINITY; 3];
                let mut max = [f32::NEG_INFINITY; 3];
                for mesh in parts {
                    let (lo, hi) = mesh.bounds().expect("nonempty fixture");
                    for axis in 0..3 {
                        min[axis] = min[axis].min(lo[axis] as f32);
                        max[axis] = max[axis].max(hi[axis] as f32);
                    }
                }
                (min, max)
            })
            .collect();
        let instances: Vec<_> = (0..config.bodies)
            .map(|i| {
                let ordinal = i % config.designs;
                let [x, z] = grid(i);
                Instance {
                    design: if config.kind == Kind::Appearance {
                        0
                    } else {
                        ordinal
                    },
                    origin: [x as f32 * GRID_STRIDE, 0.0, z as f32 * GRID_STRIDE],
                    tint: if config.kind == Kind::Appearance {
                        colour(ordinal, config.seed)
                    } else {
                        [1.0; 3]
                    },
                }
            })
            .collect();
        let mut refs = BTreeSet::new();
        let mut unique_quads = 0;
        for parts in &designs {
            for mesh in parts {
                let reference = mesh.placements[0].volume;
                if refs.insert(reference) {
                    unique_quads += mesh.mesh_for(reference).unwrap().len();
                }
            }
        }
        let instance_quads = instances
            .iter()
            .map(|instance| {
                designs[instance.design]
                    .iter()
                    .map(BodyMesh::drawn_quads)
                    .sum::<usize>()
            })
            .sum();
        let stats = WorkloadStats {
            body_count: config.bodies,
            design_count: config.designs,
            unique_meshes: refs.len(),
            assembly_count: if config.kind == Kind::Assembly {
                config.designs
            } else {
                1
            },
            occupancy_count: refs.len(),
            part_instances: config.bodies * 3,
            unique_quads,
            instance_quads,
            cpu_mesh_builds: mesh_designs * 3,
            cpu_retained_meshes: mesh_designs * 3,
        };
        let mut hash = 0xcbf2_9ce4_8422_2325;
        feed(&mut hash, b"bench-population-v1");
        feed(&mut hash, config.kind.label().as_bytes());
        for instance in &instances {
            for value in instance.origin.into_iter().chain(instance.tint) {
                feed(&mut hash, &value.to_bits().to_le_bytes());
            }
            for mesh in &designs[instance.design] {
                let p = &mesh.placements[0];
                feed(&mut hash, &p.volume.0);
                for value in p.pivot.into_iter().chain(p.pivot_at) {
                    feed(&mut hash, &value.to_le_bytes());
                }
            }
        }
        Ok(Self {
            config,
            stats,
            digest: format!("fnv1a64:{hash:016x}"),
            preparation_us: started.elapsed().as_micros().min(u128::from(u64::MAX)) as u64,
            designs,
            instances,
            bounds,
        })
    }
}

fn feed(hash: &mut u64, bytes: &[u8]) {
    for byte in bytes {
        *hash = (*hash ^ u64::from(*byte)).wrapping_mul(0x100_0000_01b3);
    }
}

fn design(kind: Kind, index: usize, seed: u64) -> Vec<BodyMesh> {
    // XOR is a bijection over this 16-bit family. Every requested geometry
    // design therefore has distinct occupancy, independent of draw order.
    let mask = if kind == Kind::Geometry {
        (index as u16) ^ (seed as u16)
    } else {
        0
    };
    let mut torso = Volume::solid([6, 6, 6], 3);
    for bit in 0..16 {
        if mask & (1 << bit) != 0 {
            torso.set(5, 1 + bit / 4, 1 + bit % 4, 0);
        }
    }
    let head_at = if kind == Kind::Assembly {
        let n = (index + (seed % 1000) as usize) % 1000;
        [
            (n % 10) as i32 - 5,
            7 + ((n / 10) % 10) as i32,
            ((n / 100) % 10) as i32 - 5,
        ]
    } else {
        [0, 7, 0]
    };
    vec![
        part(0, torso, mask, [3, 0, 3], [0, 0, 0]),
        part(1, Volume::solid([3, 3, 3], 5), 0, [1, 0, 1], head_at),
        part(2, Volume::solid([2, 2, 5], 7), 0, [1, 0, 2], [0, 1, -5]),
    ]
}

fn part(id: u32, volume: Volume, mask: u16, pivot: [i32; 3], at: [i32; 3]) -> BodyMesh {
    // A lossless canonical encoding of this restricted fixture's actual
    // content recipe. Same bytes always mean same volume; no random IDs.
    let mut key = [0; 32];
    key[..8].copy_from_slice(b"benchD01");
    key[8..12].copy_from_slice(&id.to_le_bytes());
    key[12..14].copy_from_slice(&mask.to_le_bytes());
    for (i, size) in volume.size.iter().enumerate() {
        key[16 + i * 4..20 + i * 4].copy_from_slice(&size.to_le_bytes());
    }
    key[28] = match id {
        0 => 3,
        1 => 5,
        _ => 7,
    };
    let mut mesh = BodyMesh::single(VolumeRef(key), &volume);
    let p = &mut mesh.placements[0];
    p.part = PartId(id);
    p.pivot = pivot;
    p.pivot_at = at;
    mesh
}

fn colour(index: usize, seed: u64) -> [f32; 3] {
    // Distinct base-10 colour triples for the admitted 1000-design range.
    let n = (index + (seed % 1000) as usize) % 1000;
    [n % 10, (n / 10) % 10, (n / 100) % 10].map(|v| 0.35 + v as f32 * 0.065)
}

fn grid(index: usize) -> [i32; 2] {
    if index == 0 {
        return [0, 0];
    }
    let mut x = 0;
    let mut z = 0;
    let mut length = 1;
    let mut n = 0;
    loop {
        for direction in [[1, 0], [0, 1], [-1, 0], [0, -1]] {
            for _ in 0..length {
                x += direction[0];
                z += direction[1];
                n += 1;
                if n == index {
                    return [x, z];
                }
            }
            if direction == [0, 1] || direction == [0, -1] {
                length += 1;
            }
        }
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;

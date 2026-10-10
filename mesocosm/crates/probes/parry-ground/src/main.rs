// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;

use conatus::{
    BodyDesc, BodyError, BodyKind, BodyWorld, ColliderDesc, ColliderId, ColliderShape,
    SpatialFilter, Transform, VoxelEdit,
};
use isometer_core::ground::{AIR, BRICK, Ground};
use terrain::Places;

mod terrain;

const SEED: u64 = 0xC011_1DE3;
const SIDE: u16 = 3;
const EXTENT: i32 = 24;
const CARVE_RADIUS: i32 = 1;

#[derive(Clone, Debug, PartialEq, Eq)]
struct QueryReceipt {
    occupancy: bool,
    ray_toi_bits: u32,
    point_inside: bool,
    contacts: usize,
    minimum_contact_bits: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ExactReceipt {
    compared_voxels: usize,
    occupied_voxels: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct DeltaReceipt {
    source_revision: u64,
    committed_revision: u64,
    regions_scanned: Vec<[i16; 3]>,
    voxels_compared: usize,
    voxels_changed: usize,
}

#[derive(Debug, PartialEq, Eq)]
enum ProjectionError {
    SourceRevision { projection: u64, provided: u64 },
    TargetRevision { expected: u64, ground: u64 },
    MissingBrick([i16; 3]),
    Conatus(String),
}

impl fmt::Display for ProjectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SourceRevision {
                projection,
                provided,
            } => write!(
                f,
                "collision projection is at revision {projection}, not supplied source {provided}"
            ),
            Self::TargetRevision { expected, ground } => write!(
                f,
                "collision delta expected Ground revision {expected}, found {ground}"
            ),
            Self::MissingBrick(key) => write!(f, "dirty Ground brick {key:?} is missing"),
            Self::Conatus(error) => write!(f, "conatus refused a query: {error}"),
        }
    }
}

impl Error for ProjectionError {}

fn refused(error: BodyError) -> ProjectionError {
    ProjectionError::Conatus(error.to_string())
}

/// Ground's committed occupancy as one fixed conatus voxel collider.
struct GroundCollision {
    revision: u64,
    occupied: usize,
    world: BodyWorld,
    collider: ColliderId,
}

impl GroundCollision {
    fn from_ground(ground: &Ground) -> Self {
        let mut occupied = Vec::new();
        for key in ground.keys() {
            let (brick, base) = ground
                .brick_materials(key)
                .expect("a public Ground key resolves to its brick");
            for y in 0..BRICK {
                for z in 0..BRICK {
                    for x in 0..BRICK {
                        if brick.get([x, y, z]) != AIR {
                            occupied.push([base[0] + x, base[1] + y, base[2] + z]);
                        }
                    }
                }
            }
        }

        let count = occupied.len();
        let mut world = BodyWorld::try_new([0.0, 0.0, 0.0]).expect("a zero-gravity world");
        let body = world
            .spawn(
                BodyDesc::new(BodyKind::Fixed).with_collider(ColliderDesc::new(
                    ColliderShape::VoxelGrid {
                        cell_size: [1.0, 1.0, 1.0],
                        occupied,
                    },
                )),
            )
            .expect("Ground's cells make a valid voxel grid");
        let collider = world
            .collider_ids(body)
            .expect("the terrain body exists")
            .into_iter()
            .next()
            .expect("the terrain body was spawned with one collider");
        world.refresh_queries();

        Self {
            revision: ground.revision(),
            occupied: count,
            world,
            collider,
        }
    }

    fn filled(&self, at: [i32; 3]) -> bool {
        self.world
            .voxel_filled(self.collider, at)
            .expect("the terrain collider is a voxel grid")
    }

    fn apply_delta(
        &mut self,
        ground: &Ground,
        source_revision: u64,
        dirty: &[[i16; 3]],
    ) -> Result<DeltaReceipt, ProjectionError> {
        if source_revision != self.revision {
            return Err(ProjectionError::SourceRevision {
                projection: self.revision,
                provided: source_revision,
            });
        }

        let expected_target = source_revision + 1;
        if ground.revision() != expected_target {
            return Err(ProjectionError::TargetRevision {
                expected: expected_target,
                ground: ground.revision(),
            });
        }

        let regions_scanned = dirty.iter().copied().collect::<BTreeSet<_>>();
        let mut updates = Vec::new();
        for key in &regions_scanned {
            let (brick, base) = ground
                .brick_materials(*key)
                .ok_or(ProjectionError::MissingBrick(*key))?;
            for y in 0..BRICK {
                for z in 0..BRICK {
                    for x in 0..BRICK {
                        let at = [base[0] + x, base[1] + y, base[2] + z];
                        let filled = brick.get([x, y, z]) != AIR;
                        if self.filled(at) != filled {
                            updates.push(VoxelEdit { cell: at, filled });
                        }
                    }
                }
            }
        }

        let summary = self
            .world
            .edit_voxels(self.collider, updates.iter().copied())
            .map_err(refused)?;
        debug_assert_eq!(summary.changed, updates.len());
        for edit in &updates {
            if edit.filled {
                self.occupied += 1;
            } else {
                self.occupied -= 1;
            }
        }
        self.world.refresh_queries();
        self.revision = ground.revision();

        Ok(DeltaReceipt {
            source_revision,
            committed_revision: self.revision,
            regions_scanned: regions_scanned.into_iter().collect(),
            voxels_compared: dirty.len() * (BRICK * BRICK * BRICK) as usize,
            voxels_changed: updates.len(),
        })
    }

    fn query(&self, ground: &Ground, target: [i32; 3]) -> Result<QueryReceipt, ProjectionError> {
        let point = voxel_center(target);
        let ray_origin = [point[0], point[1] + 3.0, point[2]];
        let hit = self
            .world
            .raycast(
                ray_origin,
                [0.0, -1.0, 0.0],
                64.0,
                true,
                SpatialFilter::default(),
            )
            .map_err(refused)?
            .expect("stored ground lies below the query ray");
        let expected_toi = ground_ray_toi(ground, ray_origin, target[0], target[2]);
        assert_eq!(
            hit.distance.to_bits(),
            expected_toi.to_bits(),
            "the conatus ray must hit the same stored Ground voxel"
        );

        let ball_center = [point[0], target[1] as f32 + 0.9, point[2]];
        let (contacts, minimum_contact_bits) = self.contact_receipt(ball_center)?;
        let inside = self
            .world
            .colliders_at_point(point, SpatialFilter::default())
            .map_err(refused)?;

        Ok(QueryReceipt {
            occupancy: self.filled(target),
            ray_toi_bits: hit.distance.to_bits(),
            point_inside: inside.contains(&self.collider),
            contacts,
            minimum_contact_bits,
        })
    }

    /// A ball of radius 0.2 against the ground, prediction 0: how many
    /// contacts, and the deepest one's distance bits.
    fn contact_receipt(&self, ball_center: [f32; 3]) -> Result<(usize, Option<u32>), ProjectionError> {
        let contacts = self
            .world
            .contacts(
                Transform::from_translation(ball_center),
                &ColliderShape::sphere(0.2),
                0.0,
                SpatialFilter::default(),
            )
            .map_err(refused)?;
        let minimum = contacts
            .iter()
            .map(|contact| contact.distance)
            .min_by(f32::total_cmp)
            .map(f32::to_bits);
        Ok((contacts.len(), minimum))
    }

    fn assert_exact(&self, ground: &Ground) -> ExactReceipt {
        let mut compared = 0;
        let mut occupied = 0;
        for key in ground.keys() {
            let (brick, base) = ground
                .brick_materials(key)
                .expect("a public Ground key resolves to its brick");
            for y in 0..BRICK {
                for z in 0..BRICK {
                    for x in 0..BRICK {
                        let at = [base[0] + x, base[1] + y, base[2] + z];
                        let expected = brick.get([x, y, z]) != AIR;
                        assert_eq!(self.filled(at), expected, "occupancy differs at {at:?}");
                        compared += 1;
                        occupied += expected as usize;
                    }
                }
            }
        }

        assert_eq!(occupied, self.occupied);
        let collider_occupied = self
            .world
            .voxel_cells(self.collider, None)
            .expect("the terrain collider is a voxel grid")
            .inspect(|at| {
                assert!(at[1] >= 0, "implicit bedrock is not materialized");
                assert!(ground.solid(*at), "conatus holds an extra voxel at {at:?}");
            })
            .count();
        assert_eq!(collider_occupied, occupied);

        ExactReceipt {
            compared_voxels: compared,
            occupied_voxels: occupied,
        }
    }
}

fn make_ground() -> Ground {
    let grown = Places::grown(SEED, SIDE, EXTENT);
    Ground::grow(&grown, EXTENT)
}

fn choose_boundary_surface(ground: &Ground) -> [i32; 3] {
    for z in (-EXTENT + 1)..EXTENT {
        if z.rem_euclid(BRICK) != BRICK - 1 {
            continue;
        }
        for x in (-EXTENT + 1)..EXTENT {
            if x.rem_euclid(BRICK) != BRICK - 1 {
                continue;
            }
            let Some(y) = ground.surface(x, z) else {
                continue;
            };
            if y < 3 || ground.solid([x, y + 1, z]) {
                continue;
            }
            let supported = (-1..=1).all(|dz| {
                (-1..=1).all(|dx| {
                    ground.solid([x + dx, y, z + dz]) && ground.solid([x + dx, y - 1, z + dz])
                })
            });
            if supported {
                return [x, y, z];
            }
        }
    }
    panic!("fixture needs an exposed brick-boundary surface with two solid layers");
}

fn ground_ray_toi(ground: &Ground, origin: [f32; 3], x: i32, z: i32) -> f32 {
    let start_y = origin[1].floor() as i32;
    for y in (0..=start_y).rev() {
        if ground.solid([x, y, z]) {
            return origin[1] - (y + 1) as f32;
        }
    }
    panic!("stored Ground has no ray target at x={x}, z={z}");
}

fn region_signatures(collision: &GroundCollision, ground: &Ground) -> BTreeMap<[i16; 3], u64> {
    ground
        .keys()
        .map(|key| {
            let (_, base) = ground
                .brick_materials(key)
                .expect("a public Ground key resolves to its brick");
            let mut hash = 0xcbf2_9ce4_8422_2325u64;
            for y in 0..BRICK {
                for z in 0..BRICK {
                    for x in 0..BRICK {
                        hash ^= collision.filled([base[0] + x, base[1] + y, base[2] + z]) as u64;
                        hash = hash.wrapping_mul(0x100_0000_01b3);
                    }
                }
            }
            (key, hash)
        })
        .collect()
}

fn voxel_center(at: [i32; 3]) -> [f32; 3] {
    [at[0] as f32 + 0.5, at[1] as f32 + 0.5, at[2] as f32 + 0.5]
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut ground = make_ground();
    assert_eq!(ground.revision(), 0);
    assert!(ground.drain_dirty().is_empty());
    let target = choose_boundary_surface(&ground);

    let mut collision = GroundCollision::from_ground(&ground);
    let initial_exact = collision.assert_exact(&ground);
    let initial_queries = collision.query(&ground, target)?;
    assert!(initial_queries.occupancy);
    assert!(initial_queries.point_inside);
    assert!(initial_queries.contacts > 0);
    let initial_regions = region_signatures(&collision, &ground);
    let initial_ground = ground.clone();

    let removed = ground.carve(target, CARVE_RADIUS);
    assert!(removed > 0);
    let dirty = ground.drain_dirty();
    assert!(
        dirty.len() >= 4,
        "the boundary carve must cross Ground bricks"
    );

    let refusal = collision
        .apply_delta(&ground, collision.revision + 1, &dirty)
        .expect_err("a stale source revision must be refused");
    assert!(matches!(refusal, ProjectionError::SourceRevision { .. }));
    assert_eq!(collision.revision, 0);
    assert_eq!(
        initial_queries,
        collision.query(&initial_ground, target)?,
        "revision refusal must leave collision state unchanged"
    );

    let mut skipped_ground = initial_ground.clone();
    assert!(skipped_ground.carve(target, CARVE_RADIUS) > 0);
    assert!(
        skipped_ground.carve(target, CARVE_RADIUS + 1) > 0,
        "the larger second carve must create revision two"
    );
    let skipped_dirty = skipped_ground.drain_dirty();
    let skipped = collision
        .apply_delta(&skipped_ground, 0, &skipped_dirty)
        .expect_err("a skipped target revision must be refused");
    assert!(matches!(skipped, ProjectionError::TargetRevision { .. }));
    assert_eq!(
        initial_queries,
        collision.query(&initial_ground, target)?,
        "skipped revision refusal must leave collision state unchanged"
    );

    let delta = collision.apply_delta(&ground, 0, &dirty)?;
    assert_eq!(delta.voxels_changed, removed as usize);
    assert_eq!(delta.regions_scanned.len(), dirty.len());
    let committed_exact = collision.assert_exact(&ground);
    let committed_queries = collision.query(&ground, target)?;
    assert!(!committed_queries.occupancy);
    assert!(!committed_queries.point_inside);
    assert_eq!(committed_queries.contacts, 0);
    assert_ne!(initial_queries.ray_toi_bits, committed_queries.ray_toi_bits);

    let dirty_set = dirty.iter().copied().collect::<BTreeSet<_>>();
    let committed_regions = region_signatures(&collision, &ground);
    for (key, signature) in &initial_regions {
        if !dirty_set.contains(key) {
            assert_eq!(
                committed_regions.get(key),
                Some(signature),
                "unchanged Ground region {key:?} changed occupancy"
            );
        }
    }

    let mut replay_ground = make_ground();
    let replay_target = choose_boundary_surface(&replay_ground);
    assert_eq!(replay_target, target);
    let mut replay_collision = GroundCollision::from_ground(&replay_ground);
    assert_eq!(
        replay_collision.query(&replay_ground, target)?,
        initial_queries
    );
    let replay_removed = replay_ground.carve(target, CARVE_RADIUS);
    let replay_dirty = replay_ground.drain_dirty();
    assert_eq!(replay_removed, removed);
    assert_eq!(replay_dirty, dirty);
    let replay_delta = replay_collision.apply_delta(&replay_ground, 0, &replay_dirty)?;
    assert_eq!(replay_delta, delta);
    assert_eq!(
        replay_collision.query(&replay_ground, target)?,
        committed_queries
    );

    println!(
        "projection receipt: Ground revision 0, {} stored bricks, {} compared cells, {} occupied conatus voxels",
        ground.brick_count(),
        initial_exact.compared_voxels,
        initial_exact.occupied_voxels
    );
    println!(
        "delta receipt: revision {} -> {}, target {target:?}, {removed} removed voxels, {} dirty 8^3 regions, {} cells rescanned",
        delta.source_revision,
        delta.committed_revision,
        delta.regions_scanned.len(),
        delta.voxels_compared
    );
    println!(
        "query receipt: ray {:.1} -> {:.1}, point inside {} -> {}, contacts {} -> {}, deepest {:?}",
        f32::from_bits(initial_queries.ray_toi_bits),
        f32::from_bits(committed_queries.ray_toi_bits),
        initial_queries.point_inside,
        committed_queries.point_inside,
        initial_queries.contacts,
        committed_queries.contacts,
        initial_queries.minimum_contact_bits.map(f32::from_bits)
    );
    println!(
        "authority receipt: stale source and skipped target revisions refused before mutation; {} unchanged regions retained occupancy; replay query bits identical",
        initial_regions.len() - dirty_set.len()
    );
    println!(
        "scope receipt: {} committed occupied voxels; implicit y<0 bedrock remains an analytic half-space",
        committed_exact.occupied_voxels
    );

    Ok(())
}

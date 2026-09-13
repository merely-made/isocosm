// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
use super::*;

#[test]
fn classes_separate_appearance_assembly_and_geometry_reuse() {
    for kind in Kind::ALL {
        let workload = Workload::new(Config {
            bodies: 128,
            designs: 16,
            kind,
            ..Default::default()
        })
        .unwrap();
        assert_eq!(workload.stats.body_count, 128);
        assert_eq!(workload.stats.part_instances, 384);
        assert_eq!(
            workload.stats.unique_meshes,
            if kind == Kind::Geometry { 18 } else { 3 }
        );
        assert_eq!(
            workload.stats.assembly_count,
            if kind == Kind::Assembly { 16 } else { 1 }
        );
        let appearances: BTreeSet<_> = workload
            .instances
            .iter()
            .map(|i| i.tint.map(f32::to_bits))
            .collect();
        assert_eq!(
            appearances.len(),
            if kind == Kind::Appearance { 16 } else { 1 }
        );
    }
}

#[test]
fn deterministic_prefixes_hold_when_population_grows() {
    for kind in Kind::ALL {
        let config = Config {
            bodies: 16,
            designs: 16,
            kind,
            ..Default::default()
        };
        let small = Workload::new(config.clone()).unwrap();
        let large = Workload::new(Config {
            bodies: 1000,
            ..config.clone()
        })
        .unwrap();
        assert_eq!(small.instances, large.instances[..16]);
        assert_eq!(small.digest, Workload::new(config).unwrap().digest);
        assert_ne!(small.digest, large.digest);
    }
}

#[test]
fn grid_is_unique_bounded_and_first_instance_is_centered() {
    let positions: BTreeSet<_> = (0..1000).map(grid).collect();
    assert_eq!(positions.len(), 1000);
    assert_eq!(grid(0), [0, 0]);
    assert!(positions.iter().flatten().all(|v| v.abs() <= 16));
}

#[test]
fn invalid_counts_and_insufficient_capacity_are_refused_before_gpu_creation() {
    for config in [
        Config {
            bodies: 0,
            ..Default::default()
        },
        Config {
            bodies: 1001,
            ..Default::default()
        },
        Config {
            designs: 17,
            ..Default::default()
        },
        Config {
            mesh_capacity: 2,
            ..Default::default()
        },
        Config {
            kind: Kind::Geometry,
            bodies: 1000,
            designs: 1000,
            mesh_capacity: 1001,
            ..Default::default()
        },
    ] {
        assert!(Workload::new(config).is_err());
    }
}

#[test]
fn thousand_generated_designs_have_distinct_actual_occupancy() {
    let workload = Workload::new(Config {
        kind: Kind::Geometry,
        bodies: 1000,
        designs: 1000,
        ..Default::default()
    })
    .unwrap();
    let mut geometries = BTreeSet::new();
    for design in &workload.designs {
        let mesh = &design[0];
        let quads = &mesh.mesh_for(mesh.placements[0].volume).unwrap().quads;
        let key: Vec<_> = quads
            .iter()
            .map(|q| (q.origin, q.axis, q.positive, q.size))
            .collect();
        geometries.insert(key);
    }
    assert_eq!(geometries.len(), 1000);
    assert_eq!(workload.stats.unique_meshes, 1002);
}

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

#[test]
fn default_layout_is_unchanged_and_density_preserves_geometry() {
    let config = Config {
        bodies: 16,
        designs: 16,
        kind: Kind::Geometry,
        ..Default::default()
    };
    let baseline = Workload::new(config.clone()).unwrap();
    for (i, instance) in baseline.instances.iter().enumerate() {
        let [x, z] = grid(i);
        assert_eq!(instance.origin, [x as f32 * 24.0, 0.0, z as f32 * 24.0]);
    }
    let dense = Workload::new(Config {
        camera_extent: 64,
        grid_spacing: 8,
        depth_layers: 4,
        ..config
    })
    .unwrap();
    let keys = |workload: &Workload| {
        workload
            .designs
            .iter()
            .flatten()
            .map(|m| m.placements[0].volume)
            .collect::<BTreeSet<_>>()
    };
    assert_eq!(keys(&baseline), keys(&dense));
    assert_eq!(baseline.stats.body_count, dense.stats.body_count);
    assert_eq!(baseline.stats.instance_quads, dense.stats.instance_quads);
    assert_eq!(dense.stats.occupied_groups, 4);
    assert_eq!(dense.stats.occupied_layers, 4);
    assert_ne!(baseline.digest, dense.digest);
}

#[test]
fn depth_layers_have_distinct_origins_on_the_camera_axis_and_stable_prefixes() {
    let config = Config {
        bodies: 16,
        designs: 1,
        depth_layers: 8,
        ..Default::default()
    };
    let small = Workload::new(config.clone()).unwrap();
    let large = Workload::new(Config {
        bodies: 128,
        ..config
    })
    .unwrap();
    assert_eq!(small.instances, large.instances[..16]);
    let origins: BTreeSet<_> = large
        .instances
        .iter()
        .map(|i| i.origin.map(f32::to_bits))
        .collect();
    assert_eq!(origins.len(), 128);
    let camera = isometer_render::Camera::default();
    let view_axis = [
        camera.yaw.cos() * camera.pitch.cos(),
        camera.pitch.sin(),
        camera.yaw.sin() * camera.pitch.cos(),
    ];
    for pair in small.instances[..8].windows(2) {
        let delta = [0, 1, 2].map(|i| pair[1].origin[i] - pair[0].origin[i]);
        assert!((0..3).all(|i| (delta[i] - 40.0 * view_axis[i]).abs() < 0.0001));
    }
}

#[test]
fn density_controls_are_validated_and_independently_recorded() {
    for config in [
        Config {
            camera_extent: 7,
            ..Default::default()
        },
        Config {
            camera_extent: 2001,
            ..Default::default()
        },
        Config {
            grid_spacing: 0,
            ..Default::default()
        },
        Config {
            grid_spacing: 129,
            ..Default::default()
        },
        Config {
            depth_layers: 0,
            ..Default::default()
        },
        Config {
            depth_layers: 33,
            ..Default::default()
        },
    ] {
        assert!(Workload::new(config).is_err());
    }
    let base = Workload::new(Config::default()).unwrap();
    for config in [
        Config {
            camera_extent: 499,
            ..Default::default()
        },
        Config {
            grid_spacing: 23,
            ..Default::default()
        },
        Config {
            depth_layers: 2,
            ..Default::default()
        },
        Config {
            reverse_instances: true,
            ..Default::default()
        },
    ] {
        assert_ne!(base.digest, Workload::new(config).unwrap().digest);
    }
}

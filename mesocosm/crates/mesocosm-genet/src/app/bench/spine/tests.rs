// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;

#[test]
fn spine_draws_join_and_preserve_the_founded_world() {
    for seed in 0..8 {
        for overview in [false, true] {
            let land = draw(seed, overview, false).unwrap();
            assert!(land.receipt.border_equal, "seed {seed}");
            assert_eq!(
                land.receipt.border_digests[0],
                land.receipt.border_digests[1]
            );
            assert_eq!(land.receipt.world_before, land.receipt.world_after);
            assert!(land.ground.brick_count() > 0);
            assert_eq!(
                land.receipt.material_sample_digests[0],
                land.receipt.material_sample_digests[1]
            );
            assert_eq!(
                land.receipt.material_samples,
                u64::from(land.receipt.columns[0]) * u64::from(land.receipt.columns[1]) * 6
            );
            assert!(land.receipt.filled_voxels <= 8 * 1024 * 1024);
            assert!(land.receipt.columns[0] <= 128);
            assert_eq!(land.receipt.level == 0, !overview);
            assert!(land.ground.surface(0, 0).is_some());
        }
    }
}

#[test]
fn spine_perturbed_side_breaks_the_same_border_check_and_geometry() {
    for seed in 0..4 {
        let good = draw(seed, false, false).unwrap();
        let broken = draw(seed, false, true).unwrap();
        assert!(
            !broken.receipt.border_equal,
            "negative control was not detected for {seed}"
        );
        assert_ne!(
            broken.receipt.border_digests[0],
            broken.receipt.border_digests[1]
        );
        assert_ne!(good.ground, broken.ground);
        assert_eq!(broken.receipt.world_before, broken.receipt.world_after);
    }
}

#[test]
fn spine_local_datum_and_materials_preserve_water_soil_and_rock() {
    let window = Window {
        columns: [2, 2],
        cells: vec![
            Column {
                top: 100_000,
                water: 100_002,
                soil: 2
            };
            4
        ],
        datum: 99_998,
        extent: 1,
        height: 4,
    };
    assert_eq!(window.surface(1, 0, 0), 4);
    assert_eq!(window.material(0, 0, 0), 1);
    assert_eq!(window.material(0, 0, 1), 1);
    assert_eq!(window.material(0, 0, 2), 2);
    assert_eq!(window.material(0, 0, 3), 2);
    assert_eq!(window.material(0, 0, 4), 3);
    let ground = Ground::grow_with(&window, 1, |x, z, d| window.material(x, z, d));
    assert_eq!(ground.surface(0, 0), Some(4));
    assert_eq!(ground.brick_count(), 4);
    assert_eq!(ground.surface(1, 1), None);
}

#[test]
fn spine_palette_resolves_world_keys_instead_of_assuming_ids() {
    let grid = Grid::drawn(8);
    let mut world = Founding {
        seed: 8,
        sites: grid.width * grid.height,
        map: Some(Layout::Grid(grid.clone())),
        ..Default::default()
    }
    .generate()
    .unwrap();
    let original = build(&world, grid.clone(), true, false).unwrap();
    world.world.materials.reverse();
    let reordered = build(&world, grid, true, false).unwrap();
    assert_eq!(original.ground, reordered.ground);
    assert!(reordered.receipt.border_equal);
}

#[test]
fn spine_rebuilding_a_window_is_identical() {
    let a = draw(9, false, false).unwrap();
    let b = draw(9, false, false).unwrap();
    assert_eq!(a.ground, b.ground);
    assert_eq!(
        serde_json::to_string(&a.receipt).unwrap(),
        serde_json::to_string(&b.receipt).unwrap()
    );
}

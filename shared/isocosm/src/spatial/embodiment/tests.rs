// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;

fn terrain() -> Simulation {
    let g = crate::Founding { seed: 7, sites: 4, population: 8,
        map: Some(crate::map::Layout::Grid(crate::map::Grid {
            shape: "shape:plane".into(), width: 2, height: 2, side: 768,
            elevation: [16, 16], relief: [0, 0], sea_per_mille: 0,
        })), ..Default::default() }.generate().unwrap();
    Simulation::new(g, crate::Execution::Individuals).unwrap()
}

#[test]
fn geometry_key_ignores_weather_but_tracks_each_terrain_condition() {
    let mut sim = terrain();
    let window = Window { min: [380; 2], max: [384; 2], depth: 8 };
    let held = sim.site_geometry(0, &window, Default::default()).unwrap();
    let revision = sim.terrain_revision();
    sim.state.sites.get_mut(&0).unwrap().conditions.insert("world:weather".into(), 12);
    assert_ne!(sim.terrain_revision(), revision, "admission keeps the full revision");
    let weather = sim.site_geometry(0, &window, Default::default()).unwrap();
    assert!(std::sync::Arc::ptr_eq(&held, &weather));
    assert!(weather.volume.same_cells(&sim.volume(0, window.min, window.max, window.depth).unwrap()));
    let skeleton = sim.genesis().rules.skeleton.as_ref().unwrap();
    for name in [&skeleton.elevation, &skeleton.relief, &skeleton.water] {
        let mut changed = sim.clone();
        *changed.state.sites.get_mut(&0).unwrap().conditions.get_mut(name).unwrap() += 1;
        let actual = changed.site_geometry(0, &window, Default::default()).unwrap();
        assert!(!std::sync::Arc::ptr_eq(&held, &actual));
        assert!(actual.volume.same_cells(&changed.volume(0, window.min, window.max, window.depth).unwrap()));
    }
}

#[test]
fn terrain_search_reuses_only_exact_shapes_and_keeps_current_blockers() {
    let sim = terrain();
    let volume = sim.volume(0, [350; 2], [418; 2], 16).unwrap();
    let mut cached = Grounding::new(&volume);
    let mut fresh = Grounding::new(&volume);
    for shape in [vec![[[0; 3], [1, 2, 1]]], vec![[[0; 3], [3, 1, 2]]]] {
        for x in 375..380 {
            for yaw in [Yaw::Zero, Yaw::Quarter] {
                let expected = fresh.at_boxes(&shape, x, 377, yaw).ok();
                assert_eq!(cached.stand(&shape, x, 377, yaw), expected);
                assert_eq!(cached.stand(&shape, x, 377, yaw), expected);
            }
        }
    }
    let document = crate::geometry::document(&crate::geometry::Frame {
        half_extent: [1; 3], ..Default::default()
    });
    let mut blocked = Vec::new();
    for _ in 0..3 {
        let expected = Grounding::new(&volume).find(&document, [384, 0, 384], Yaw::Zero,
            SEARCH_REACH, SEARCH_STRIDE, SEARCH_CANDIDATES, &blocked);
        let actual = cached.find(&document, [384, 0, 384], Yaw::Zero,
            SEARCH_REACH, SEARCH_STRIDE, SEARCH_CANDIDATES, &blocked);
        assert_eq!(actual, expected);
        if let Ok(pose) = actual { blocked.extend(boxes(&document, &pose).unwrap()); }
    }
    assert!(!blocked.is_empty(), "fixture exercises successful terrain placements");
    let local = vec![[[0; 3], [1; 3]]];
    for x in 0..5000 { cached.stand(&local, x, 0, Yaw::Zero); }
    assert_eq!(cached.stand_count, 4096);
}

// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::{Host, HostConfig};

fn bench_host() -> Host {
    Host::new(HostConfig {
        bench_world: true,
        seed: 9,
        population: 256,
        ..HostConfig::default()
    })
}

#[test]
fn drawn_and_authored_native_anatomies_project_without_capsule_fallback() {
    for roster in [false, true] {
        let mut founded = mesocosm_runtime::Founded::generated(16_003_851_315_348_448_882, 60).unwrap();
        let mut domain = founded.genesis.founding.clone().unwrap();
        domain.bodies.as_mut().unwrap().roster = roster;
        domain.ecology_rates = isocosm::ecology::Rates::A1;
        founded.genesis = domain.generate().unwrap();
        let runtime = mesocosm_runtime::Runtime::bench_placed(founded, 2).unwrap();
        let sim = runtime.sim();
        let documents: Vec<_> = sim.state().population.groups.values()
            .filter_map(|group| group.entity.body.as_ref()).collect();
        assert!(!documents.is_empty());
        let volumes = isometer::DeclaredExtentVolumes::generated_from_documents(documents.iter().copied()).unwrap();
        let mut projector = isometer::mesh::LiveBodyProjector::new();
        for document in documents { projector.project_body(document, &volumes).unwrap(); }
    }
}

#[test]
fn a_marked_physical_world_does_not_replace_a_missing_pose_with_a_grid_spot() {
    let mut host = bench_host();
    host.refresh_scene();
    assert!(host.scene.is_some());
    let mut water = mesocosm_runtime::Founded::generated(2_814_468_213_915_169_548, 60).unwrap();
    for site in water.genesis.sites.values_mut() {
        for (key, value) in [("terrain:elevation", 0), ("terrain:relief", 0), ("terrain:water", 8)] {
            site.conditions.insert(key.into(), value);
        }
    }
    host.runtime = mesocosm_runtime::Runtime::bench_placed(water, 2).unwrap();
    host.scene_source_key = None;
    assert!(host.runtime.spatial().driver.is_some());
    assert!(!host.runtime.spatial().members.contains_key(&host.runtime.critter().unwrap()));
    host.refresh_scene();
    let scene = host.scene.as_ref().expect("declared native terrain remains visible");
    assert!(scene.bodies.is_empty(), "an unavailable pose must never acquire a grid body");
    assert_eq!(Some(scene.site), host.runtime.played().unwrap().resident_site());
}

#[test]
fn successful_scene_is_reused_only_for_the_same_world_cut() {
    let mut host = bench_host();
    assert!(host.refresh_scene());
    let scene = host
        .scene
        .as_ref()
        .expect("generated bench has a native site");
    assert!(
        !scene.bodies.is_empty(),
        "cache control needs actual body documents"
    );
    let body_storage = scene.bodies.as_ptr();
    let initial_output = format!("{:?}", scene.bodies);
    let initial_revision = host.runtime.revision().unwrap();

    assert!(!host.refresh_scene());
    assert_eq!(host.scene.as_ref().unwrap().bodies.as_ptr(), body_storage);

    host.runtime
        .world()
        .put_section("scene-cache-control", vec![1, 2, 3])
        .unwrap();
    let section_revision = host.runtime.revision().unwrap();
    assert!(section_revision > initial_revision);
    // The first section publication invalidates the source. It does not alter
    // the scene's ground or the rendered body data.
    assert!(!host.refresh_scene());
    let after_section = host.scene.as_ref().unwrap();
    assert_ne!(after_section.bodies.as_ptr(), body_storage);
    assert_eq!(format!("{:?}", after_section.bodies), initial_output);
    let section_body_storage = after_section.bodies.as_ptr();

    host.runtime
        .world()
        .put_section("scene-cache-control", vec![1, 2, 3])
        .unwrap();
    assert_eq!(host.runtime.revision().unwrap(), section_revision);
    assert!(!host.refresh_scene());
    assert_eq!(
        host.scene.as_ref().unwrap().bodies.as_ptr(),
        section_body_storage
    );

    host.runtime
        .world()
        .put_section("scene-cache-control", vec![4])
        .unwrap();
    let changed_section_revision = host.runtime.revision().unwrap();
    assert!(changed_section_revision > section_revision);
    assert!(!host.refresh_scene());
    assert_ne!(
        host.scene.as_ref().unwrap().bodies.as_ptr(),
        section_body_storage
    );
    assert_eq!(
        format!("{:?}", host.scene.as_ref().unwrap().bodies),
        initial_output
    );

    let before_native_step = host.scene.as_ref().unwrap().bodies.as_ptr();
    let before_native_revision = host.runtime.revision().unwrap();
    assert_eq!(host.runtime.step(1), 1, "bench advances one native round");
    assert!(host.runtime.revision().unwrap() > before_native_revision);
    host.refresh_scene();
    assert_ne!(
        host.scene.as_ref().unwrap().bodies.as_ptr(),
        before_native_step
    );
}

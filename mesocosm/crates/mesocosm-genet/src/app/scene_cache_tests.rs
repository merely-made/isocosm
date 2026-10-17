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

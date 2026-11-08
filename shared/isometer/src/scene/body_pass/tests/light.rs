// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::BodyLight;

const NOON: BodyLight = BodyLight {
    toward_sun: [0.4, 0.8, 0.3],
    ambient: 0.38,
    sun: [0.62; 3],
};

fn moved(by: f32) -> BodyLight {
    BodyLight {
        toward_sun: [0.4 + by, 0.8, 0.3],
        ..NOON
    }
}

/// A far sprite under `first`, then under `second`: (refreshes, hits, stamp changed).
fn relight(step: f32, first: BodyLight, second: BodyLight) -> (u32, u32, bool) {
    let (device, queue) =
        crate::test_gpu::device().expect("GPU is required for the sprite receipt");
    let doc = BodyDocument::new(VolumeRef::from_tag(81), [2; 3]);
    let volumes = DeclaredExtentVolumes::from_documents([&doc], 1);
    let bodies = [body(1, 0.0, &doc)];
    let mut scene = Scene::new(device, queue, VIEW, VIEW).unwrap();
    let mut host = host();
    host.settings = host.settings.with_light_step(step);
    host.light = Some(first);
    let before = render(&mut scene, camera(6.0), &bodies, &volumes, &mut host);
    assert_eq!((before.refreshes, before.sprite_bodies), (1, 1));
    host.light = Some(second);
    let after = render(&mut scene, camera(6.0), &bodies, &volumes, &mut host);
    assert_eq!(after.sprite_bodies, 1);
    (
        after.refreshes,
        after.hits,
        after.light_stamp != before.light_stamp,
    )
}

#[test]
fn the_sun_rebakes_an_image_only_past_its_quantum() {
    assert_eq!(relight(1.0 / 16.0, NOON, moved(-0.01)), (0, 1, false));
    assert_eq!(relight(1.0 / 16.0, NOON, moved(0.2)), (1, 0, true));
    // Control: an exact light rebakes on any movement.
    assert_eq!(relight(0.0, NOON, moved(-0.01)), (1, 0, true));
}

#[test]
fn a_held_sprite_bound_keeps_an_examined_subject_a_sprite() {
    let (device, queue) =
        crate::test_gpu::device().expect("GPU is required for the sprite receipt");
    let doc = BodyDocument::new(VolumeRef::from_tag(81), [2; 3]);
    let volumes = DeclaredExtentVolumes::from_documents([&doc], 1);
    let near = body(1, 0.0, &doc);
    let mut scene = Scene::new(device, queue, VIEW, VIEW).unwrap();
    let mut host = host();
    host.examined = Some(near.subject);
    host.held_sprite = Some(near.subject);
    let held = render(&mut scene, camera(1.5), &[near], &volumes, &mut host);
    assert_eq!((held.sprite_bodies, held.refreshes), (1, 1));
    // Control: once live detail is held, examination promotes at once.
    host.held_sprite = None;
    let live = render(&mut scene, camera(1.5), &[near], &volumes, &mut host);
    assert_eq!(live.sprite_bodies, 0);
}

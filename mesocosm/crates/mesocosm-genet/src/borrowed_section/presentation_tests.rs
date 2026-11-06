// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;

fn presented() -> (SceneHandle, Rc<SiteScene>, Presentation) {
    let (session, participant, _, window, site) = tests::fixture();
    let handle = SceneHandle::default();
    handle
        .update(&session, participant, window, Mode::Survival, site, 0)
        .unwrap();
    let native = handle.snapshot().unwrap();
    let camera = native.camera(1.0);
    let view = Presentation {
        mode: CameraMode::default(),
        depth: None,
        forward: camera.forward,
        centre: camera.centre,
        half_height: camera.half_height,
        bodies: section::BodyMode::Voxels,
        budget: section::DEFAULT_BODY_BUDGET,
        light: 1.0,
        sky: section::SkyBlock::fixed(),
        terrain: section::TerrainStyle::default(),
        terrain_y: camera.centre[1],
        focus: None,
        selection: None,
    };
    (handle, native.scene.clone(), view)
}

#[test]
fn action_revision_advances_without_copying_or_republishing_the_picture() {
    let (handle, scene, view) = presented();
    handle.present(&scene, 7, view.clone()).unwrap();
    let first = handle.snapshot().unwrap();
    handle.present(&scene, 7, view.clone()).unwrap();
    assert!(Rc::ptr_eq(&first, &handle.snapshot().unwrap()));
    handle.present(&scene, 8, view).unwrap();
    let next = handle.snapshot().unwrap();
    assert_eq!(next.revision, Some(8));
    assert_eq!(next.fingerprint, first.fingerprint);
    assert!(Rc::ptr_eq(&first.scene, &next.scene));
    assert_eq!(first.scene.bodies.as_ptr(), next.scene.bodies.as_ptr());
}

#[test]
fn presentation_changes_new_scene_lifetimes_and_remounts_get_new_images() {
    let (handle, scene, mut view) = presented();
    handle.present(&scene, 0, view.clone()).unwrap();
    let first = handle.snapshot().unwrap();
    view.half_height *= 1.25;
    handle.present(&scene, 0, view.clone()).unwrap();
    let camera = handle.snapshot().unwrap();
    assert_ne!(camera.fingerprint, first.fingerprint);
    assert!(Rc::ptr_eq(&camera.scene, &first.scene));
    let replacement = Rc::new((*scene).clone());
    handle.present(&replacement, 0, view.clone()).unwrap();
    let next = handle.snapshot().unwrap();
    assert_ne!(
        next.fingerprint, camera.fingerprint,
        "a new scene lifetime at the same numeric revision needs a fresh image"
    );
    handle.clear();
    handle.present(&replacement, 0, view).unwrap();
    assert_ne!(next.fingerprint, handle.snapshot().unwrap().fingerprint);
}

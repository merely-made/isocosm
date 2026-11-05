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
        looking: section::perspective::Looking::overview(),
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

#[test]
fn perspective_revisions_retain_the_projected_picture_and_return_to_native_overview() {
    let (handle, scene, mut view) = presented();
    let played = scene.played.expect("fixture played body");
    let original = scene.body(played).unwrap().document.clone();
    for mode in [isometer::View::OverTheShoulder, isometer::View::FirstPerson] {
        view.looking.view = mode;
        handle.present(&scene, 20, view.clone()).unwrap();
        let first = handle.snapshot().unwrap();
        assert!(
            first.eye.is_some(),
            "native body provides a perspective eye"
        );
        assert!(
            first.camera(1.5).frustum().is_some(),
            "{mode:?} keeps main's projection"
        );
        assert!(
            !Rc::ptr_eq(&first.scene, &scene),
            "perspective owns its filtered picture"
        );
        assert!(Rc::ptr_eq(first.source_scene.as_ref().unwrap(), &scene));
        handle.present(&scene, 21, view.clone()).unwrap();
        let revision = handle.snapshot().unwrap();
        assert_eq!(revision.revision, Some(21));
        assert!(
            !Rc::ptr_eq(&first, &revision),
            "new action admission has a new input receipt"
        );
        assert!(Rc::ptr_eq(&first.scene, &revision.scene));
        assert_eq!(first.scene.bodies.as_ptr(), revision.scene.bodies.as_ptr());
        assert_eq!(first.eye, revision.eye);
        assert_eq!(first.fingerprint, revision.fingerprint);
        assert_eq!(
            scene.body(played).unwrap().document,
            original,
            "first person cannot crop the native source"
        );
    }
    view.looking = section::perspective::Looking::overview();
    handle.present(&scene, 22, view).unwrap();
    let overview = handle.snapshot().unwrap();
    assert!(Rc::ptr_eq(&overview.scene, &scene));
    assert_eq!(overview.eye, None);
    assert!(overview.camera(1.5).frustum().is_none());
    assert_eq!(overview.scene.body(played).unwrap().document, original);
}

#[test]
fn perspective_changes_and_replaced_native_scene_refresh_the_picture_even_at_the_same_revision() {
    let (handle, scene, mut view) = presented();
    view.looking.view = isometer::View::FirstPerson;
    handle.present(&scene, 30, view.clone()).unwrap();
    let first = handle.snapshot().unwrap();
    view.looking.heading = [1.0, 0.0, 0.0];
    handle.present(&scene, 30, view.clone()).unwrap();
    let turned = handle.snapshot().unwrap();
    assert_ne!(first.fingerprint, turned.fingerprint);
    assert!(!Rc::ptr_eq(&first.scene, &turned.scene));
    assert_ne!(first.camera(1.5).forward, turned.camera(1.5).forward);
    assert_eq!(
        first.eye, turned.eye,
        "turning does not move the body's eye"
    );
    let played = scene.played.unwrap();
    let mut replacement = (*scene).clone();
    let body = replacement
        .bodies
        .iter_mut()
        .find(|b| b.id == played)
        .unwrap();
    body.at[0] += 12;
    body.physical = None;
    let replacement = Rc::new(replacement);
    handle.present(&replacement, 30, view).unwrap();
    let moved = handle.snapshot().unwrap();
    assert_ne!(turned.fingerprint, moved.fingerprint);
    assert!(!Rc::ptr_eq(&turned.scene, &moved.scene));
    assert_ne!(
        turned.eye, moved.eye,
        "a fresh source cannot keep an old body's eye"
    );
    assert!(Rc::ptr_eq(
        moved.source_scene.as_ref().unwrap(),
        &replacement
    ));
    assert!(moved.camera(1.5).frustum().is_some());
}

#[test]
fn a_missing_native_eye_cannot_retain_a_previous_perspective_camera() {
    let (handle, scene, mut view) = presented();
    view.looking.view = isometer::View::FirstPerson;
    handle.present(&scene, 40, view.clone()).unwrap();
    assert!(handle.snapshot().unwrap().camera(1.5).frustum().is_some());
    let mut absent = (*scene).clone();
    let played = absent.played.unwrap();
    absent.bodies.retain(|body| body.id != played);
    handle.present(&Rc::new(absent), 41, view).unwrap();
    let next = handle.snapshot().unwrap();
    assert_eq!(next.eye, None);
    assert!(next.camera(1.5).frustum().is_none());
    assert!(!next.body_ids().contains(&played));
}

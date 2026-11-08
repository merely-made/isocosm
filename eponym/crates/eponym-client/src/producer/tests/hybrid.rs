// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::producer::BodyQuality;

#[test]
fn hybrid_zoom_and_examination_preserve_the_native_contact_result() {
    let Some(gpu) = harness::gpu("hybrid Eponym viewport") else {
        return;
    };
    let mut fixture = timed_action_world();
    let mut control = timed_action_world();
    let handle = fixture.scene();
    let live_handle = control.scene();
    let mut producer = SceneProducer::new(SceneModelSource::new(handle.clone()));
    let mut live = SceneProducer::new(SceneModelSource::new(live_handle.clone()));

    let (pan, forward) = {
        let model = handle.borrow();
        let follow = model.follow_centre().unwrap();
        let target = model.game().pose(fixture.target).unwrap().position;
        let delta = [0, 1, 2]
            .map(|axis| target[axis] as f32 / eponym_play::MOTION_SCALE as f32 - follow[axis]);
        (delta, [delta[2], 0.0, -delta[0]])
    };
    for handle in [&handle, &live_handle] {
        harness::aim(handle, forward, 30.0, false);
        handle.borrow_mut().camera.pan = pan;
    }
    let before = handle.borrow().session().state_hash().unwrap();
    let first = harness::frame(&gpu, &mut producer, true);
    assert!(first.count(Ink::Other) > 0);
    harness::frame(&gpu, &mut live, true);

    handle.borrow_mut().set_body_quality(BodyQuality::Hybrid);
    let far = harness::frame(&gpu, &mut producer, false);
    let cached = producer.hybrid_stats();
    eprintln!("Eponym far viewport {cached:?}");
    assert_eq!(
        (cached.sprite_bodies, cached.refreshes),
        (1, 1),
        "{cached:?}"
    );
    assert!(
        far.count(Ink::Other) > 0,
        "the distant actor has visible sprite pixels"
    );
    let picked = (0..SIZE[1]).any(|y| {
        (0..SIZE[0]).any(|x| {
            far.ink(x, y) == Ink::Other
                && producer
                    .pick_body(ndc_of_pixel(x, y))
                    .is_some_and(|(subject, _)| subject == fixture.target)
        })
    });
    assert!(
        picked,
        "a distant actor remains addressable through its current body"
    );

    handle
        .borrow_mut()
        .set_examined_subject(Some(fixture.target));
    harness::frame(&gpu, &mut producer, false);
    assert_eq!(
        producer.hybrid_stats().refreshes,
        0,
        "examination needs a live body"
    );
    assert_eq!(producer.hybrid_stats().sprite_bodies, 0);
    handle.borrow_mut().set_examined_subject(None);
    handle.borrow_mut().camera.half_height = 2.0;
    let near = harness::frame(&gpu, &mut producer, false);
    assert!(
        near.count(Ink::Other) > 0,
        "the zoom keeps the actor in view"
    );
    assert_eq!(producer.hybrid_stats().sprite_bodies, 0);
    handle.borrow_mut().camera.half_height = 30.0;
    harness::frame(&gpu, &mut producer, false);
    assert_eq!(producer.hybrid_stats().sprite_bodies, 1);
    assert_eq!(producer.hybrid_stats().transition_bodies, 1);
    for _ in 0..8 {
        if !handle.borrow().hybrid_needs_frame() {
            break;
        }
        harness::frame(&gpu, &mut producer, false);
    }
    assert_eq!(producer.hybrid_stats().transition_bodies, 0);
    assert!(
        producer
            .render_scene(&harness::request(&gpu, false))
            .unwrap()
            .is_none()
    );
    assert_eq!(handle.borrow().session().state_hash().unwrap(), before);

    let injured = fixture.sever_target_limb();
    assert!(!injured.is_empty());
    assert_eq!(injured, control.sever_target_limb());
    for (handle, fixture) in [(&handle, &fixture), (&live_handle, &control)] {
        handle
            .borrow_mut()
            .set_session(fixture.action.session().clone());
    }
    harness::frame(&gpu, &mut producer, false);
    harness::frame(&gpu, &mut live, false);
    eprintln!("Eponym wounded viewport {:?}", producer.hybrid_stats());
    assert_eq!(
        producer.hybrid_stats().refreshes,
        1,
        "the severance replaces the image"
    );
    assert_eq!(
        handle.borrow().session().state_hash().unwrap(),
        live_handle.borrow().session().state_hash().unwrap()
    );
    assert!(
        injured
            .into_iter()
            .all(|part| producer.part_bounds(fixture.target, part).is_none())
    );
}

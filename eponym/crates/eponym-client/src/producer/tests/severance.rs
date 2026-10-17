// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;

#[test]
fn a_severed_part_leaves_the_next_frame_without_reuploading_the_rest() {
    let Some(gpu) = harness::gpu("severance receipt") else {
        return;
    };
    let mut fixture = timed_action_world();
    let handle = fixture.scene();
    let mut producer = SceneProducer::new(SceneModelSource::new(handle.clone()));
    let pan = {
        let model = handle.borrow();
        let target = model.game().pose(fixture.target).unwrap().position;
        let follow = model.follow_centre().unwrap();
        [0, 1, 2].map(|axis| target[axis] as f32 / eponym_play::MOTION_SCALE as f32 - follow[axis])
    };
    let limb = PartId(2);
    let mut found = None;
    let mut initial_stats = None;
    let mut best_silhouette = (0usize, 0usize, [0.0f32; 3]);
    for forward in harness::candidates() {
        harness::aim(&handle, forward, 6.0, false);
        handle.borrow_mut().camera.pan = pan;
        let frame = harness::frame(&gpu, &mut producer, true);
        initial_stats.get_or_insert(producer.body_stats());
        let view = producer.presented_camera().expect("presented camera");
        // The limb's own visible pixels: the target's nearest surface here is
        // the limb, and the frame agrees the target is drawn.
        let region: Vec<[u32; 2]> = (0..SIZE[1])
            .flat_map(|y| (0..SIZE[0]).map(move |x| [x, y]))
            .filter(|[x, y]| {
                part_under(&producer, fixture.target, view, *x, *y) == Some(limb)
                    && frame.ink(*x, *y) == Ink::Other
            })
            .collect();
        // After removing this limb, a target pixel is truly vacated only if
        // none of the target's surviving parts has a surface on the same ray.
        // Reject views where the next part merely replaces the limb in front
        // of the other body's pixels.
        let trace = view.trace().expect("presented camera trace");
        let remaining_bounds: Vec<_> = producer
            .drawn_parts(fixture.target)
            .into_iter()
            .filter(|part| *part != limb)
            .filter_map(|part| producer.part_bounds(fixture.target, part))
            .collect();
        let vacated_if_severed = region
            .iter()
            .filter(|pixel| {
                let [x, y] = **pixel;
                let Some((origin, direction)) = trace.ray_at(ndc_of_pixel(x, y)) else {
                    return false;
                };
                !remaining_bounds
                    .iter()
                    .any(|bounds| ray_box(origin, direction, trace.far(), *bounds).is_some())
            })
            .count();
        if vacated_if_severed > best_silhouette.1 {
            best_silhouette = (region.len(), vacated_if_severed, forward);
        }
        if region.len() >= 40 && vacated_if_severed >= 20 {
            found = Some((frame, region, view, forward));
            break;
        }
    }
    let (before, region, view, forward) = found.unwrap_or_else(|| {
        panic!(
            "no camera showed the target's native limb with at least 20 pixels uncovered by its remaining parts; best (limb_pixels, expected_vacated, forward)={best_silhouette:?}"
        )
    });
    before.write_png("sever-before");
    assert!(
        initial_stats.expect("at least one camera rendered").mesh_builds > 0,
        "the first frame must build its volume geometry"
    );

    let severed = fixture.sever_target_limb();
    assert!(
        severed.contains(&limb),
        "the volley severs the limb: {severed:?}"
    );
    handle
        .borrow_mut()
        .set_session(fixture.action.session().clone());

    let after = harness::frame(&gpu, &mut producer, false);
    after.write_png("sever-after");
    let stats = producer.body_stats();
    // Only the pixels the shortened body no longer reaches at all: what stands
    // behind a lost limb is its own body, and it is allowed to show.
    let vacated: Vec<_> = region
        .iter()
        .filter(|[x, y]| !covers(&producer, fixture.target, view, *x, *y))
        .collect();
    assert!(
        vacated.len() >= 20,
        "{forward:?}: severance vacated only {} of {} limb pixels",
        vacated.len(),
        region.len()
    );
    let still_drawn = vacated
        .iter()
        .filter(|[x, y]| after.ink(*x, *y) == Ink::Other)
        .count();
    assert_eq!(
        still_drawn, 0,
        "{forward:?}: the severed limb still has pixels"
    );
    assert!(
        after.count(Ink::Other) > 0,
        "the rest of the target must still be drawn"
    );
    assert_eq!(
        (stats.mesh_builds, stats.mesh_upload_bytes),
        (0, 0),
        "unchanged bodies must not re-upload static geometry"
    );
}

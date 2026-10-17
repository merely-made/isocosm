// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;

#[test]
fn terrain_hides_a_body_standing_behind_it() {
    let Some(gpu) = harness::gpu("terrain-over-body occlusion") else {
        return;
    };
    let fixture = timed_action_world();
    let handle = fixture.scene();
    let mut producer = SceneProducer::new(SceneModelSource::new(handle.clone()));
    // Either body may be the buried one; whichever stands in the open is the
    // control, in the same frame. Seed 7 puts the keeper on open ground and no
    // searched camera buries it whole, so in practice this receipt hides the
    // target — the claim is still about terrain, not about a chosen subject.
    // Native combat spacing is larger than the old authored bodies. Frame
    // their midpoint so both the occluded subject and its control fit.
    let (pan, half_height) = {
        let model = handle.borrow();
        let target = model.game().pose(fixture.target).unwrap().position;
        let follow = model.follow_centre().unwrap();
        let delta = [0, 1, 2].map(|axis| target[axis] as f32 / eponym_play::MOTION_SCALE as f32 - follow[axis]);
        (delta.map(|value| value * 0.5), 10.0_f32.max(delta[0].abs().max(delta[2].abs()) * 0.5 + 4.0))
    };
    let mut hidden_keeper = None;
    let mut hidden_target = None;
    let mut max_keeper_burial = (0usize, [0.0; 3], 0.0f32);
    let mut max_target_burial = (0usize, [0.0; 3], 0.0f32);
    // A camera below a hillside looks slightly upward toward the bodies. The
    // shared candidates cover level and downward views; mirror their mild
    // pitch upward as well so the ray walk tests actual foreground ground.
    let aims = harness::candidates()
        .into_iter()
        .chain((0..24).map(|step| {
            let yaw = step as f32 / 24.0 * std::f32::consts::TAU;
            let pitch = 0.35f32;
            let level = pitch.cos();
            [yaw.cos() * level, pitch.sin(), yaw.sin() * level]
        }))
        // How deep the slab cuts decides how much rock a ray crosses before it
        // reaches a body, so it is part of the search, not a constant.
        .flat_map(|forward| [48.0f32, 16.0].map(|depth| (forward, depth)));
    for (forward, depth) in aims {
        if hidden_keeper.is_some() && hidden_target.is_some() {
            break;
        }
        harness::aim(&handle, forward, half_height, false);
        {
            let mut model = handle.borrow_mut();
            model.camera.depth = depth;
            model.camera.pan = pan;
        }
        let bare = harness::frame(&gpu, &mut producer, true);
        let view = producer.presented_camera().expect("presented camera");
        let (Some(keeper_bounds), Some(target_bounds)) = (
            producer.body_bounds(fixture.keeper),
            producer.body_bounds(fixture.target),
        ) else {
            continue;
        };
        // Every corner of the hidden body must be behind rock; a body the
        // ground only half covers proves nothing. The control body's standing
        // is not asserted from the CPU walk but read out of the frame below.
        let keeper_corners = corners_behind_rock(&handle, view, keeper_bounds);
        let target_corners = corners_behind_rock(&handle, view, target_bounds);
        let keeper_buried = keeper_corners == 8;
        let target_buried = target_corners == 8;
        if keeper_corners > max_keeper_burial.0 {
            max_keeper_burial = (keeper_corners, forward, depth);
        }
        if target_corners > max_target_burial.0 {
            max_target_burial = (target_corners, forward, depth);
        }
        let (hidden_ink, open_ink) = match (keeper_buried, target_buried) {
            (true, false) if hidden_keeper.is_none() => (Ink::Played, Ink::Other),
            (false, true) if hidden_target.is_none() => (Ink::Other, Ink::Played),
            _ => continue,
        };
        if bare.count(hidden_ink) < 60 || bare.count(open_ink) < 60 {
            continue;
        }
        handle.borrow_mut().terrain = true;
        let grounded = harness::frame(&gpu, &mut producer, true);
        // The control has to actually survive the trace, or this camera says
        // nothing about terrain and everything about framing.
        if grounded.count(open_ink) < 60 {
            continue;
        }
        let receipt = (bare, grounded, hidden_ink, open_ink, forward);
        if hidden_ink == Ink::Played {
            hidden_keeper = Some(receipt);
        } else {
            hidden_target = Some(receipt);
        }
    }
    let receipts: Vec<_> = [("keeper", hidden_keeper), ("target", hidden_target)]
        .into_iter()
        .filter_map(|(which, receipt)| receipt.map(|receipt| (which, receipt)))
        .collect();
    assert!(
        !receipts.is_empty(),
        "no searched camera put a whole fixture body behind rock; max corners: keeper={max_keeper_burial:?}, target={max_target_burial:?}"
    );
    for (which, receipt) in receipts {
        let (bare, grounded, hidden_ink, open_ink, forward) = receipt;
        bare.write_png(&format!("terrain-control-bodies-only-{which}"));
        grounded.write_png(&format!("terrain-occludes-{which}"));
        let hidden = grounded.count(hidden_ink);
        let visible = grounded.count(open_ink);
        // Bounded above as well as below: a terrain grade that happened to
        // share the control's colour would inflate this, and that is exactly
        // the fault this receipt was caught by once already.
        assert!(
            (60..=bare.count(open_ink) * 2).contains(&visible),
            "{forward:?}: the unblocked body must survive the terrain trace, saw {visible} against {}",
            bare.count(open_ink)
        );
        assert!(
            hidden <= bare.count(hidden_ink) / 50,
            "{forward:?}: terrain left {hidden} of {} blocked-{which} pixels",
            bare.count(hidden_ink)
        );
    }
}

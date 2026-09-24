// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! P1's receipts, on a headless device, over the timed-action fixture world.
//!
//! Every claim is a coverage or region assertion over read-back pixels. No
//! frame is ever compared byte for byte, and each occlusion receipt carries its
//! own positive control in the same run: the body that must end up hidden is
//! shown to be drawable under the same camera before terrain or a nearer body
//! is allowed to hide it.
//!
//! World state only ever changes through recorded intents. `AdvanceMotion` is
//! the session's controlled-subject path, so the moving body in these receipts
//! is always the keeper.

use isometer::SlabCamera;
use isometer::core::PartId;
use eponym_identity::SubjectId;
use eponym_world::MotionInput;

use super::fixture::{FixtureScene, advance_motion, timed_action_world};
use super::harness::{self, Ink, SIZE};
use super::{SceneHandle, SceneModelSource, SceneProducer};

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a.into_iter().zip(b).map(|(a, b)| a * b).sum()
}

fn ndc_of_pixel(x: u32, y: u32) -> [f32; 2] {
    [
        (x as f32 + 0.5) / SIZE[0] as f32 * 2.0 - 1.0,
        1.0 - (y as f32 + 0.5) / SIZE[1] as f32 * 2.0,
    ]
}

/// Does this one body's own surface lie under this pixel, ignoring every other
/// body? The same camera the frame drew with, over the same posed part bounds
/// the scene reports, so the answer is comparable to the frame's own coverage.
///
/// Exact rather than approximate: a Eponym part is one solid declared box, so
/// `Scene::part_bounds` — the world bounds of that part's meshed faces under
/// the draw's own part matrix — *is* the surface, and a ray against it is the
/// same intersection the renderer rasterized.
fn covers(
    producer: &SceneProducer,
    subject: SubjectId,
    camera: SlabCamera,
    x: u32,
    y: u32,
) -> bool {
    part_under(producer, subject, camera, x, y).is_some()
}

/// Which of that body's parts is nearest under this pixel.
fn part_under(
    producer: &SceneProducer,
    subject: SubjectId,
    camera: SlabCamera,
    x: u32,
    y: u32,
) -> Option<PartId> {
    let trace = camera.trace()?;
    let (origin, direction) = trace.ray_at(ndc_of_pixel(x, y))?;
    let far = trace.far();
    let mut nearest: Option<(f32, PartId)> = None;
    for part in producer.drawn_parts(subject) {
        let Some(bounds) = producer.part_bounds(subject, part) else {
            continue;
        };
        let Some(distance) = ray_box(origin, direction, far, bounds) else {
            continue;
        };
        if nearest.is_none_or(|(held, _)| distance < held) {
            nearest = Some((distance, part));
        }
    }
    nearest.map(|(_, part)| part)
}

/// Where a ray first meets a world box, within the slab's own reach. `None`
/// when it misses or the box lies wholly behind the far wall.
fn ray_box(
    origin: [f32; 3],
    direction: [f32; 3],
    far: f32,
    (min, max): ([f32; 3], [f32; 3]),
) -> Option<f32> {
    let mut enter = 0.0f32;
    let mut leave = far;
    for axis in 0..3 {
        if direction[axis].abs() < 1e-9 {
            if origin[axis] < min[axis] || origin[axis] > max[axis] {
                return None;
            }
            continue;
        }
        let first = (min[axis] - origin[axis]) / direction[axis];
        let second = (max[axis] - origin[axis]) / direction[axis];
        enter = enter.max(first.min(second));
        leave = leave.min(first.max(second));
    }
    (enter <= leave).then_some(enter)
}

/// How many corners of this world box have solid ground standing between them
/// and the slab's near wall. The producer's own `Ground`, walked on the CPU, so
/// a camera can be chosen without assuming what seed 7 grew.
fn corners_behind_rock(
    handle: &SceneHandle,
    camera: SlabCamera,
    bounds: ([f32; 3], [f32; 3]),
) -> usize {
    let model = handle.borrow();
    let ground = model.game().world().ground();
    let reach = camera.depth * 0.5 + 4.0;
    let (min, max) = bounds;
    (0..8)
        .filter(|mask| {
            let corner = [0, 1, 2].map(|axis| {
                if mask & (1 << axis) == 0 {
                    min[axis]
                } else {
                    max[axis]
                }
            });
            let mut step = 0.25;
            while step <= reach {
                let cell = [0, 1, 2].map(|i| (corner[i] - camera.forward[i] * step).floor() as i32);
                if ground.solid(cell) {
                    return true;
                }
                step += 0.25;
            }
            false
        })
        .count()
}

fn centre_of((min, max): ([f32; 3], [f32; 3])) -> [f32; 3] {
    [0, 1, 2].map(|i| (min[i] + max[i]) * 0.5)
}

#[test]
fn the_nearer_body_owns_every_pixel_the_two_bodies_share() {
    let Some(gpu) = harness::gpu("body-over-body occlusion") else {
        return;
    };
    let fixture = timed_action_world();
    let handle = fixture.scene();
    let mut producer = SceneProducer::new(SceneModelSource::new(handle.clone()));
    // Two bodies only overlap on screen when the section looks along the line
    // between them. Derive that yaw from where they actually stand, then lean a
    // few degrees off it so the farther one is partly, not wholly, covered.
    harness::aim(&handle, [1.0, 0.0, 0.0], 6.0, false);
    harness::frame(&gpu, &mut producer, true);
    let separation = {
        let keeper = centre_of(producer.body_bounds(fixture.keeper).unwrap());
        let target = centre_of(producer.body_bounds(fixture.target).unwrap());
        [0, 1, 2].map(|i| target[i] - keeper[i])
    };
    let base = separation[2].atan2(separation[0]);
    let mut aims = Vec::new();
    for half_turn in [0.0, std::f32::consts::PI] {
        for lean in [3.0f32, -3.0, 5.0, -5.0, 7.0, -7.0] {
            for pitch in [0.0f32, -0.15] {
                let yaw = base + half_turn + lean.to_radians();
                let level = pitch.cos();
                aims.push([yaw.cos() * level, pitch.sin(), yaw.sin() * level]);
            }
        }
    }
    let mut found = None;
    for forward in aims {
        harness::aim(&handle, forward, 6.0, false);
        let frame = harness::frame(&gpu, &mut producer, true);
        let view = producer.presented_camera().expect("presented camera");
        let (Some(keeper_bounds), Some(target_bounds)) = (
            producer.body_bounds(fixture.keeper),
            producer.body_bounds(fixture.target),
        ) else {
            continue;
        };
        let keeper_depth = dot(view.forward, centre_of(keeper_bounds));
        let target_depth = dot(view.forward, centre_of(target_bounds));
        if (keeper_depth - target_depth).abs() < 0.5 {
            continue;
        }
        let (near, far, near_ink, far_ink) = if keeper_depth < target_depth {
            (fixture.keeper, fixture.target, Ink::Played, Ink::Other)
        } else {
            (fixture.target, fixture.keeper, Ink::Other, Ink::Played)
        };
        let (mut shared, mut owned, mut stolen, mut far_alone) = (0usize, 0usize, 0usize, 0usize);
        for y in 0..SIZE[1] {
            for x in 0..SIZE[0] {
                match (
                    covers(&producer, near, view, x, y),
                    covers(&producer, far, view, x, y),
                ) {
                    (true, true) => {
                        shared += 1;
                        owned += usize::from(frame.ink(x, y) == near_ink);
                        stolen += usize::from(frame.ink(x, y) == far_ink);
                    },
                    (false, true) => far_alone += usize::from(frame.ink(x, y) == far_ink),
                    _ => {},
                }
            }
        }
        if shared >= 40 && far_alone >= 40 {
            found = Some((frame, shared, owned, stolen, far_alone, forward));
            break;
        }
    }
    let (frame, shared, owned, stolen, far_alone, forward) =
        found.expect("a camera where the two fixture bodies overlap on screen");
    frame.write_png("bodies-overlap");
    // The far body is genuinely drawn under this camera: its unshared pixels
    // are its own colour. That is the positive control for the claim below.
    assert!(far_alone >= 40, "{forward:?}: far body must be drawn");
    assert!(
        owned as f32 >= shared as f32 * 0.95,
        "{forward:?}: nearer body owns {owned} of {shared} shared pixels"
    );
    assert!(
        stolen as f32 <= shared as f32 * 0.02,
        "{forward:?}: farther body reached {stolen} of {shared} shared pixels"
    );
}

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
    let mut hidden_keeper = None;
    let mut hidden_target = None;
    let aims = harness::candidates()
        .into_iter()
        // How deep the slab cuts decides how much rock a ray crosses before it
        // reaches a body, so it is part of the search, not a constant.
        .flat_map(|forward| [96.0f32, 48.0, 16.0].map(|depth| (forward, depth)));
    for (forward, depth) in aims {
        if hidden_keeper.is_some() && hidden_target.is_some() {
            break;
        }
        harness::aim(&handle, forward, 10.0, false);
        handle.borrow_mut().camera.depth = depth;
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
        let keeper_buried = corners_behind_rock(&handle, view, keeper_bounds) == 8;
        let target_buried = corners_behind_rock(&handle, view, target_bounds) == 8;
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
        "no searched camera put a whole fixture body behind rock"
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

#[test]
fn a_severed_part_leaves_the_next_frame_without_reuploading_the_rest() {
    let Some(gpu) = harness::gpu("severance receipt") else {
        return;
    };
    let mut fixture = timed_action_world();
    let handle = fixture.scene();
    let mut producer = SceneProducer::new(SceneModelSource::new(handle.clone()));
    let limb = PartId(2);
    let mut found = None;
    for forward in harness::candidates() {
        harness::aim(&handle, forward, 6.0, false);
        let frame = harness::frame(&gpu, &mut producer, true);
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
        if region.len() >= 40 {
            found = Some((frame, region, view, producer.body_stats(), forward));
            break;
        }
    }
    let (before, region, view, first_stats, forward) =
        found.expect("a camera showing the target's authored limb");
    before.write_png("sever-before");
    assert!(
        first_stats.mesh_builds > 0,
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

#[test]
fn a_fractional_motion_step_moves_the_drawn_body_by_the_projected_fraction() {
    let Some(gpu) = harness::gpu("fractional pose receipt") else {
        return;
    };
    let fixture = timed_action_world();
    let handle = fixture.scene();
    let mut producer = SceneProducer::new(SceneModelSource::new(handle.clone()));
    // The camera follows the played keeper, so a keeper step moves every other
    // drawn body across the frame by the same projected vector, negated.
    let mut framing = None;
    for forward in harness::candidates() {
        harness::aim(&handle, forward, 14.0, false);
        let frame = harness::frame(&gpu, &mut producer, true);
        let view = producer.presented_camera().expect("presented camera");
        let (Some(keeper_bounds), Some(target_bounds)) = (
            producer.body_bounds(fixture.keeper),
            producer.body_bounds(fixture.target),
        ) else {
            continue;
        };
        let (Some(keeper_box), Some(target_box)) = (
            harness::screen_box(view, keeper_bounds, SIZE),
            harness::screen_box(view, target_bounds, SIZE),
        ) else {
            continue;
        };
        let clear_of_the_edge = target_box[0] >= 8
            && target_box[1] >= 8
            && target_box[2] + 8 <= SIZE[0]
            && target_box[3] + 8 <= SIZE[1];
        if !clear_of_the_edge || harness::overlap(keeper_box, target_box).is_some() {
            continue;
        }
        if frame.count(Ink::Other) < 60 {
            continue;
        }
        framing = Some((frame, view, forward));
        break;
    }
    let (before, view, forward) =
        framing.expect("a camera showing the target clear of the keeper and the frame edge");
    let start = handle
        .borrow()
        .game()
        .pose(fixture.keeper)
        .expect("keeper pose")
        .position;
    let centroid_before = before.centroid(Ink::Other).expect("target pixels");

    // Step across the frame, not into it: motion along the view axis would move
    // no pixels at all and prove nothing about projection.
    let [right, _, _] = view.basis();
    let across = if right[0].abs() >= right[2].abs() {
        MotionInput {
            move_x: 32_767 * right[0].signum() as i16,
            move_z: 0,
        }
    } else {
        MotionInput {
            move_x: 0,
            move_z: 32_767 * right[2].signum() as i16,
        }
    };
    let mut session = handle.borrow().session().clone();
    for _ in 0..10 {
        advance_motion(&mut session, fixture.keeper, across);
    }
    handle.borrow_mut().set_session(session);

    let after = harness::frame(&gpu, &mut producer, false);
    after.write_png("fractional-motion");
    let moved = handle
        .borrow()
        .game()
        .pose(fixture.keeper)
        .expect("keeper pose")
        .position;
    let scale = eponym_world::MOTION_SCALE;
    assert!(
        moved != start && moved.iter().any(|value| value.rem_euclid(scale) != 0),
        "{forward:?}: the step must leave a fractional pose: {start:?} -> {moved:?}"
    );
    // The keeper moved, so the frame's centre moved: a stationary body slides
    // the other way by exactly the projected fraction.
    let delta = [0, 1, 2].map(|i| -((moved[i] - start[i]) as f32) / scale as f32);
    let [right, up, _] = view.basis();
    let expected = [
        dot(right, delta) / (view.half_height * view.aspect) * 0.5 * SIZE[0] as f32,
        -dot(up, delta) / view.half_height * 0.5 * SIZE[1] as f32,
    ];
    assert!(
        expected[0].hypot(expected[1]) >= 1.0,
        "{forward:?}: the step must be large enough to read: {expected:?}"
    );
    let centroid_after = after.centroid(Ink::Other).expect("target pixels");
    let measured = [
        (centroid_after[0] - centroid_before[0]) as f32,
        (centroid_after[1] - centroid_before[1]) as f32,
    ];
    for axis in 0..2 {
        assert!(
            (measured[axis] - expected[axis]).abs() <= 1.0,
            "{forward:?} axis {axis}: drawn {measured:?} against projected {expected:?}"
        );
    }
}

#[test]
fn an_unchanged_frame_produces_nothing_and_a_suspension_keeps_the_geometry() {
    let Some(gpu) = harness::gpu("skip and suspend receipt") else {
        return;
    };
    let fixture = timed_action_world();
    let handle = fixture.scene();
    let mut producer = SceneProducer::new(SceneModelSource::new(handle.clone()));
    harness::aim(&handle, [1.0, 0.0, 0.0], 12.0, true);
    let render = |producer: &mut SceneProducer, needs: bool| {
        producer
            .render_scene(&harness::request(&gpu, needs))
            .expect("scene renders")
    };
    assert!(render(&mut producer, false).is_some(), "the first frame");
    let first = producer.renders();
    assert!(
        render(&mut producer, false).is_none(),
        "an unchanged frame must produce nothing"
    );
    assert_eq!(producer.renders(), first, "a skip advances no generation");
    assert!(
        render(&mut producer, true).is_some(),
        "needs_frame must force a frame"
    );
    assert_eq!(producer.renders(), first + 1, "a real render advances one");

    // A moved pose is a changed picture even with needs_frame false.
    let mut session = handle.borrow().session().clone();
    advance_motion(
        &mut session,
        fixture.keeper,
        MotionInput {
            move_x: 32_767,
            move_z: 0,
        },
    );
    handle.borrow_mut().set_session(session);
    assert!(
        render(&mut producer, false).is_some(),
        "a moved pose must produce a frame"
    );

    let cached = producer.cached_bodies();
    assert!(cached > 0, "geometry was cached");
    use cambium_rootstock::TextureProducer;
    producer.suspend();
    assert_eq!(
        producer.cached_bodies(),
        cached,
        "suspension keeps the renderer's cached geometry"
    );
    assert!(
        render(&mut producer, false).is_some(),
        "a resumed producer must rebuild its targets and draw"
    );
}

#[test]
fn the_cpu_pick_answers_the_body_the_frame_drew() {
    let Some(gpu) = harness::gpu("pick receipt") else {
        return;
    };
    let fixture = timed_action_world();
    let handle = fixture.scene();
    let mut producer = SceneProducer::new(SceneModelSource::new(handle.clone()));
    harness::aim(&handle, [1.0, 0.0, 0.0], 5.0, false);
    let frame = harness::frame(&gpu, &mut producer, true);
    frame.write_png("pick");
    let mut checked = 0;
    for y in 0..SIZE[1] {
        for x in 0..SIZE[0] {
            let ink = frame.ink(x, y);
            if ink == Ink::Scene {
                continue;
            }
            let Some((subject, _)) = producer.pick_body(ndc_of_pixel(x, y)) else {
                continue;
            };
            let expected = if subject == fixture.keeper {
                Ink::Played
            } else {
                Ink::Other
            };
            assert_eq!(expected, ink, "pick disagrees with the frame at {x},{y}");
            checked += 1;
        }
    }
    assert!(checked >= 200, "the pick receipt tested {checked} pixels");
}

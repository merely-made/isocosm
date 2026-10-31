// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
use super::*;

// The unprepared admission path independently re-reads the document and soles.
fn fresh(
    volume: &Volume,
    doc: &BodyDocument,
    f: &StrideFrame,
    yaw: Yaw,
    blocked: &[Box3],
) -> Result<()> {
    let feet = local_feet(f, yaw)?;
    let rotation = Rotation::at(yaw, f.turn);
    let occupied = boxes_at(doc, f.root, rotation, &feet)?;
    narrow::clear_rotated(volume, doc, f.root, rotation, &feet, &occupied, blocked)?;
    support::check_rotated(volume, doc, &feet, f.root, rotation, &f.bearing)
}

#[test]
fn reused_postures_keep_exact_support_clearance_and_paid_reach_answers() {
    let volume = volume();
    let doc = quadruped();
    let mut g = Grounding::new(&volume).with_articulation(true);
    let from = g.at(&doc, 384, 384, Yaw::Zero).unwrap();
    let to = g.at(&doc, 385, 384, Yaw::Zero).unwrap();
    let stride = plan_stride(&volume, &doc, &from, &to, Walking::default(), &[]).unwrap();
    let mut checks = Checks::new(&volume, &doc, &[]);
    let mut frames = Vec::new();
    for phase in 0..=stride.samples() {
        frames.push(stride_sample(&from, &to, &stride, phase * UNIT / stride.samples()).unwrap());
    }
    let mut no_support = frames[0].clone();
    no_support.bearing.clear();
    assert!(fresh(&volume, &doc, &no_support, from.yaw, &[]).is_err());
    frames.push(no_support);
    let mut far = frames[0].clone();
    far.feet.values_mut().next().unwrap()[0] += 100 * UNIT;
    frames.push(far);
    let mut turned = frames[0].clone();
    turned.turn = UNIT / 4;
    frames.push(turned);
    let mut low = frames[0].clone();
    low.root[1] -= 2 * UNIT;
    frames.push(low);
    for frame in &frames {
        assert_eq!(
            check_frame(&mut checks, frame, from.yaw),
            fresh(&volume, &doc, frame, from.yaw, &[])
        );
    }
    let misses = checks.misses;
    for frame in &frames {
        check_frame(&mut checks, frame, from.yaw).ok();
    }
    assert_eq!(
        checks.misses, misses,
        "failed and successful postures both reuse"
    );
    assert!(checks.reads >= misses * 2);

    // A changed physical input gets a new scope, including newly occupied space.
    let blocked = boxes(&doc, &from).unwrap();
    let mut occupied = Checks::new(&volume, &doc, &blocked);
    assert!(
        check_frame(&mut occupied, &frames[0], from.yaw)
            .unwrap_err()
            .contains("another native body")
    );
    let mut other = doc.clone();
    other.parts[other.root.0 as usize].half_extent[0] += 1;
    let mut changed = Checks::new(&volume, &other, &[]);
    assert_eq!(
        check_frame(&mut changed, &frames[0], from.yaw),
        fresh(&volume, &other, &frames[0], from.yaw, &[])
    );
    let mut edited = volume.clone();
    let sim = super::super::super::tests::terrain();
    let atlas = crate::terrain::View::of(sim.genesis()).unwrap();
    edited
        .apply(
            &atlas,
            0,
            &isometer_space::Edit {
                op: isometer_space::Op::Fill(edited.materials.soil),
                shape: isometer_space::Shape::Box {
                    min: blocked[0][0],
                    max: blocked[0][1],
                },
            },
        )
        .unwrap();
    let mut changed = Checks::new(&edited, &doc, &[]);
    assert!(check_frame(&mut changed, &frames[0], from.yaw).is_err());
}

#[test]
fn eviction_and_cache_hits_do_not_change_strides_or_the_whole_swing_budget() {
    let volume = volume();
    let doc = quadruped();
    let mut g = Grounding::new(&volume).with_articulation(true);
    let from = g.at(&doc, 384, 384, Yaw::Zero).unwrap();
    let to = g.at(&doc, 385, 384, Yaw::Zero).unwrap();
    let mut reused = Checks::new(&volume, &doc, &[]);
    reused.set_capacity(4);
    let mut fresh = Checks::new(&volume, &doc, &[]);
    fresh.set_capacity(0);
    assert_eq!(
        reused.step(&from, &to, Some(Walking::default())),
        fresh.step(&from, &to, Some(Walking::default()))
    );
    assert!(reused.misses < fresh.misses);
    assert!(reused.retained() <= 4);
    let f = frame(&from).unwrap();
    let stopped = Stride::default();
    // Repeating one valid posture eventually hits the same cumulative bound.
    let mut answers = Vec::new();
    for checks in [&mut reused, &mut fresh] {
        answers.push(samples(
            checks, from.yaw, &stopped, f.root, f.root, &f.feet, &f.feet, 10000, 0,
        ));
    }
    assert_eq!(answers[0], answers[1]);
    assert_eq!(answers[0], Err("whole swing work budget exceeded".into()));
}

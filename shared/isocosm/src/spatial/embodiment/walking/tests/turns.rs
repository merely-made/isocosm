// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
use super::*;

#[test]
fn the_first_unchosen_support_refusal_can_shift_its_body_during_a_paid_turn() {
    let f: Fixture = serde_json::from_str(include_str!("../fixtures/turn-balance.json")).unwrap();
    let atlas = crate::terrain::View {
        seed: f.seed,
        sites: &f.sites,
        footprint: f.footprint,
        skeleton: &f.skeleton,
        materials: &f.materials,
    };
    let volume = Volume::paged(
        &atlas,
        f.site,
        [f.before.at[0] - 40, f.before.at[2] - 40],
        [f.before.at[0] + 40, f.before.at[2] + 40],
        64,
        &[],
    )
    .unwrap();
    check(&volume, &f.document, &f.before).unwrap();
    let mut admitted = 0;
    for to in f.after {
        check(&volume, &f.document, &to).unwrap();
        match plan_stride(
            &volume,
            &f.document,
            &f.before,
            &to,
            Walking::default(),
            &[],
        ) {
            Ok(stride) => {
                check_stride(
                    &volume,
                    &f.document,
                    &f.before,
                    &to,
                    Walking::default(),
                    &stride,
                    &[],
                )
                .unwrap();
                let mut root = f.before.at.map(|v| v * UNIT);
                let mut angle = 0;
                let mut balanced = false;
                for s in stride.stages {
                    balanced |= s.turn != angle && s.root != root;
                    root = s.root;
                    angle = s.turn;
                }
                assert!(
                    balanced,
                    "the torso must move over the real supporting feet during its pivot"
                );
                admitted += 1;
            },
            Err(why) => eprintln!("retained turn {:?}: {why}", to.at),
        }
    }
    assert!(
        admitted > 0,
        "recover an actual first-unchosen terrain/support refusal"
    );
}

#[test]
fn a_supported_quarter_turn_keeps_every_pivot_sole_planted() {
    let volume = volume();
    let doc = quadruped();
    let rules = Walking::default();
    let mut g = Grounding::new(&volume)
        .with_articulation(true)
        .with_walking(Some(rules));
    let from = g.at(&doc, 384, 384, Yaw::Zero).unwrap();
    let to = g.at(&doc, 384, 384, Yaw::Quarter).unwrap();
    let stride = plan_stride(&volume, &doc, &from, &to, rules, &[]).unwrap();
    check_stride(&volume, &doc, &from, &to, rules, &stride, &[]).unwrap();
    let mut last = stride_sample(&from, &to, &stride, 0).unwrap();
    let mut rotated = false;
    for phase in 1..=stride.samples() {
        let frame = stride_sample(&from, &to, &stride, phase * UNIT / stride.samples()).unwrap();
        if frame.turn != last.turn {
            rotated = true;
            if last.bearing.len() == last.feet.len() {
                assert_eq!(frame.feet, last.feet);
            }
            assert_eq!(frame.bearing, frame.feet.keys().copied().collect());
            let rotation = Rotation::at(from.yaw, frame.turn);
            for (id, local) in local_feet(&frame, from.yaw).unwrap() {
                let p = rotation.rotate(local).unwrap();
                assert!((0..3).all(|k| (p[k] + frame.root[k]).abs_diff(frame.feet[&id][k]) <= 1));
            }
        }
        last = frame;
    }
    assert!(rotated);
    let mut forged = stride.clone();
    forged.turn = -1;
    assert!(check_stride(&volume, &doc, &from, &to, rules, &forged, &[]).is_err());
    assert!(
        check_stride(
            &volume,
            &doc,
            &from,
            &to,
            Walking {
                turns: false,
                ..rules
            },
            &stride,
            &[]
        )
        .is_err()
    );
    let blocked = stride_boxes(&doc, &from, &to, &stride, UNIT / 2).unwrap();
    assert!(check_stride(&volume, &doc, &from, &to, rules, &stride, &blocked).is_err());
}

#[test]
fn paid_quarter_turn_uses_actual_support_on_two_ground_heights() {
    let mut volume = volume();
    let doc = quadruped();
    let rules = Walking::default();
    let from = Grounding::new(&volume)
        .with_articulation(true)
        .at(&doc, 384, 384, Yaw::Zero)
        .unwrap();
    let base = foot_points(&from).unwrap().values().next().unwrap()[1] / UNIT;
    let sim = crate::spatial::embodiment::tests::terrain();
    volume
        .apply(
            &sim.atlas().unwrap(),
            0,
            &isometer_space::Edit {
                op: isometer_space::Op::Fill(volume.materials.soil),
                shape: isometer_space::Shape::Box {
                    min: [385, base, 374],
                    max: [400, base + 1, 405],
                },
            },
        )
        .unwrap();
    let mut g = Grounding::new(&volume)
        .with_articulation(true)
        .with_walking(Some(rules));
    let from = g.at(&doc, 384, 384, Yaw::Zero).unwrap();
    let to = g.at(&doc, 384, 384, Yaw::Quarter).unwrap();
    let stride = plan_stride(&volume, &doc, &from, &to, rules, &[]).unwrap();
    let heights: BTreeSet<_> = (0..=stride.samples())
        .flat_map(|i| {
            let frame = stride_sample(&from, &to, &stride, i * UNIT / stride.samples()).unwrap();
            if frame.bearing.len() != frame.feet.len() {
                return Vec::new();
            }
            frame.feet.into_values().map(|p| p[1]).collect::<Vec<_>>()
        })
        .collect();
    assert!(heights.len() >= 2);
    check_stride(&volume, &doc, &from, &to, rules, &stride, &[]).unwrap();
}

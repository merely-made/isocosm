// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
use super::super::articulation_tests::{quadruped, volume};
use super::*;
mod steep;
mod turns;

#[derive(Deserialize)]
struct Fixture {
    seed: u64,
    sites: BTreeMap<u64, crate::schema::Site>,
    footprint: crate::schema::Footprint,
    skeleton: crate::rules::Skeleton,
    materials: Vec<crate::schema::Material>,
    document: BodyDocument,
    site: u64,
    before: Pose,
    after: Vec<Pose>,
}
#[test]
fn omitted_clearance_keeps_the_first_saved_straight_transfer_policy() {
    let old = r#"{"shift":2,"foot_search":2,"lift":3,"postures":256}"#;
    let policy: Walking = serde_json::from_str(old).unwrap();
    assert!(!policy.clearance);
    assert_eq!(serde_json::to_string(&policy).unwrap(), old);
    assert!(
        serde_json::to_value(Stride::default())
            .unwrap()
            .get("clearance")
            .is_none()
    );
    assert!(Walking::default().clearance);
    assert_eq!(policy.stances, 0);
    assert!(!policy.turns);
    assert!(Walking::default().turns);
    let earlier = r#"{"clearance":true,"shift":4,"foot_search":2,"lift":3,"postures":256}"#;
    let earlier: Walking = serde_json::from_str(earlier).unwrap();
    assert_eq!(earlier.stances, 0);
    assert!(
        serde_json::to_value(earlier)
            .unwrap()
            .get("stances")
            .is_none()
    );
}
use isometer_core::{Attachment, VolumeRef};

#[test]
fn weight_transfers_support_a_loaded_corner_that_cannot_swing_in_fixed_order() {
    let volume = volume();
    let mut doc = quadruped();
    doc.attach(
        VolumeRef::from_tag(3),
        [5, 2, 5],
        Attachment {
            parent: doc.root,
            offset: [-8, 0, -8],
            yaw: Yaw::Zero,
        },
        None,
    )
    .unwrap();
    let mut g = Grounding::new(&volume).with_articulation(true);
    let a = g.at(&doc, 384, 384, Yaw::Zero).unwrap();
    let b = g.at(&doc, 385, 384, Yaw::Zero).unwrap();
    assert!(
        sweep_clear(&volume, &doc, &a, &b, &[]).is_err(),
        "loaded corner must require a support transfer"
    );
    let rules = Walking::default();
    let stride = plan_stride(&volume, &doc, &a, &b, rules, &[]).unwrap();
    check_stride(&volume, &doc, &a, &b, rules, &stride, &[]).unwrap();
    assert!(stride.stages.iter().any(|s| s.root[2] != a.at[2] * UNIT));
    let mut wrong = stride.clone();
    wrong.stages[0].root[0] += 100 * UNIT;
    assert!(check_stride(&volume, &doc, &a, &b, rules, &wrong, &[]).is_err());
    let blocked = stride_boxes(&doc, &a, &b, &stride, UNIT / 2).unwrap();
    assert!(check_stride(&volume, &doc, &a, &b, rules, &stride, &blocked).is_err());
    let start = stride_sample(&a, &b, &stride, 0).unwrap();
    let end = stride_sample(&a, &b, &stride, UNIT).unwrap();
    assert_eq!(start.feet, foot_points(&a).unwrap());
    assert_eq!(end.feet, foot_points(&b).unwrap());
    for phase in 0..=stride.samples() {
        let f = stride_sample(&a, &b, &stride, phase * UNIT / stride.samples()).unwrap();
        for id in &f.bearing {
            assert!(f.feet[id] == start.feet[id] || f.feet[id] == end.feet[id]);
        }
    }
}

#[test]
fn nearby_dry_pad_targets_recover_a_missing_nominal_foothold() {
    let mut volume = volume();
    let doc = quadruped();
    let pose = Grounding::new(&volume)
        .with_articulation(true)
        .at(&doc, 384, 384, Yaw::Zero)
        .unwrap();
    let leg = legs::chains(&doc).remove(0);
    let p = foot_points(&pose).unwrap()[&leg.foot].map(|v| v / UNIT);
    let sim = super::super::tests::terrain();
    let atlas = crate::terrain::View::of(sim.genesis()).unwrap();
    volume
        .apply(
            &atlas,
            0,
            &isometer_space::Edit {
                op: isometer_space::Op::Carve,
                shape: isometer_space::Shape::Box {
                    min: [p[0] - 1, p[1] - 12, p[2] - 3],
                    max: [p[0] + 1, p[1], p[2] + 3],
                },
            },
        )
        .unwrap();
    assert!(
        Grounding::new(&volume)
            .with_articulation(true)
            .at(&doc, 384, 384, Yaw::Zero)
            .is_err()
    );
    let found = Grounding::new(&volume)
        .with_articulation(true)
        .with_walking(Some(Walking::default()))
        .at(&doc, 384, 384, Yaw::Zero)
        .unwrap();
    check(&volume, &doc, &found).unwrap();
    assert_ne!(
        [found.feet[&leg.foot][0], found.feet[&leg.foot][2]],
        [leg.sole[0], leg.sole[2]]
    );
    assert_eq!(contacts(&volume, &doc, &found).unwrap().len(), 4);
    let mut wrong = found;
    wrong.feet.get_mut(&leg.foot).unwrap()[1] += UNIT;
    assert!(check(&volume, &doc, &wrong).is_err());
}

#[test]
fn a_moving_foot_cannot_be_omitted_or_lifted_beyond_paid_policy() {
    let volume = volume();
    let doc = quadruped();
    let mut g = Grounding::new(&volume).with_articulation(true);
    let a = g.at(&doc, 384, 384, Yaw::Zero).unwrap();
    let b = g.at(&doc, 385, 384, Yaw::Zero).unwrap();
    let rules = Walking::default();
    let stride = plan_stride(&volume, &doc, &a, &b, rules, &[]).unwrap();
    let mut missing = stride.clone();
    missing.stages.pop();
    assert!(check_stride(&volume, &doc, &a, &b, rules, &missing, &[]).is_err());
    let mut wrong_transfer = stride.clone();
    wrong_transfer.clearance = !rules.clearance;
    assert!(check_stride(&volume, &doc, &a, &b, rules, &wrong_transfer, &[]).is_err());
    let mut high = stride;
    high.stages[0].lift = (i64::from(rules.lift) + 1) * UNIT;
    assert!(check_stride(&volume, &doc, &a, &b, rules, &high, &[]).is_err());
}

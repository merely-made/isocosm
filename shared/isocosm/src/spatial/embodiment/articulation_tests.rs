// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
use super::*;
use isometer_core::{
    Attachment, VolumeRef,
    legs::{self, UNIT},
};

fn quadruped() -> BodyDocument {
    let mut doc = BodyDocument::new(VolumeRef::from_tag(1), [2, 2, 4]);
    for x in [-1, 1] {
        for z in [-3, 3] {
            let mut parent = doc.root;
            for (half, offset) in [
                ([3, 1, 1], [x * 5, 0, z]),
                ([1, 3, 1], [x * 3, -4, 0]),
                ([1, 1, 3], [0, -3, 0]),
            ] {
                parent = doc
                    .attach(
                        VolumeRef::from_tag(2),
                        half,
                        Attachment {
                            parent,
                            offset,
                            yaw: Yaw::Zero,
                        },
                        None,
                    )
                    .unwrap();
            }
        }
    }
    doc
}

fn volume() -> Volume {
    super::tests::terrain()
        .volume(0, [350; 2], [418; 2], 16)
        .unwrap()
}

#[test]
fn paid_feet_support_a_body_on_different_terrain_heights() {
    let mut volume = volume();
    let doc = quadruped();
    let flat = Grounding::new(&volume)
        .with_articulation(true)
        .at(&doc, 384, 384, Yaw::Zero)
        .unwrap();
    let base = foot_points(&flat).unwrap().values().next().unwrap()[1] / UNIT;
    let sim = super::tests::terrain();
    let atlas = crate::terrain::View::of(sim.genesis()).unwrap();
    volume
        .apply(
            &atlas,
            0,
            &isometer_space::Edit {
                op: isometer_space::Op::Fill(volume.materials.soil),
                shape: isometer_space::Shape::Box {
                    min: [385, base, 374],
                    max: [400, base + 2, 395],
                },
            },
        )
        .unwrap();
    let slope = Grounding::new(&volume)
        .with_articulation(true)
        .at(&doc, 384, 384, Yaw::Zero)
        .unwrap();
    check(&volume, &doc, &slope).unwrap();
    let heights: BTreeSet<_> = foot_points(&slope)
        .unwrap()
        .values()
        .map(|p| p[1])
        .collect();
    assert_eq!(
        heights.len(),
        2,
        "feet must conform to the actual two-level terrain"
    );
    let contacts = contacts(&volume, &doc, &slope).unwrap();
    for foot in slope.feet.keys() {
        assert!(contacts.iter().any(|c| c.part == *foot));
    }
    let mut forged = slope.clone();
    for sole in forged.feet.values_mut() {
        sole[1] += UNIT;
    }
    assert!(
        check(&volume, &doc, &forged).is_err(),
        "floating feet cannot borrow trunk support"
    );
    let mut belly = doc.clone();
    belly
        .attach(
            VolumeRef::from_tag(3),
            [1; 3],
            Attachment {
                parent: belly.root,
                offset: [0, (base - slope.at[1] + 1) as i32, 0],
                yaw: Yaw::Zero,
            },
            None,
        )
        .unwrap();
    check_occupied(&volume, boxes(&belly, &forged).unwrap())
        .expect("the trunk alone supplies rigid-box support");
    assert!(
        check(&volume, &belly, &forged)
            .unwrap_err()
            .contains("foot"),
        "a real trunk contact must not stand in for paid feet"
    );
    let mut unreachable = slope.clone();
    unreachable.feet.values_mut().next().unwrap()[0] += 100 * UNIT;
    assert!(boxes(&doc, &unreachable).is_err());
}

#[test]
fn one_foot_swings_while_other_soles_remain_world_anchored() {
    let volume = volume();
    let doc = quadruped();
    let mut g = Grounding::new(&volume).with_articulation(true);
    let a = g.at(&doc, 384, 384, Yaw::Zero).unwrap();
    let b = g.at(&doc, 385, 384, Yaw::Zero).unwrap();
    sweep_clear(&volume, &doc, &a, &b, &[]).unwrap();
    let phase = UNIT / 8;
    let feet = swing_feet(&doc, &a, &b, phase).unwrap();
    let first = *feet.keys().next().unwrap();
    assert!(feet[&first][1] > a.feet[&first][1]);
    for (&id, p) in &feet {
        if id != first {
            assert_eq!(p[0] + phase, a.feet[&id][0]);
            assert_eq!(p[1], a.feet[&id][1]);
        }
    }
    let moving = swing_boxes(&doc, &a, &b, phase).unwrap();
    assert!(
        sweep_clear(&volume, &doc, &a, &b, &moving).is_err(),
        "another body in the swept path must refuse"
    );
    assert_eq!(swing_feet(&doc, &a, &b, 0).unwrap(), a.feet);
    assert_eq!(swing_feet(&doc, &a, &b, UNIT).unwrap(), b.feet);
}

#[test]
fn severance_withdraws_paid_support_and_serialization_keeps_old_pose_bytes() {
    let volume = volume();
    let mut doc = quadruped();
    let p = Grounding::new(&volume)
        .with_articulation(true)
        .at(&doc, 384, 384, Yaw::Zero)
        .unwrap();
    let old = Pose {
        at: p.at,
        yaw: p.yaw,
        ..Default::default()
    };
    assert!(!serde_json::to_string(&old).unwrap().contains("feet"));
    let encoded = serde_json::to_string(&p).unwrap();
    assert_eq!(serde_json::from_str::<Pose>(&encoded).unwrap(), p);
    let leg = legs::chains(&doc).remove(0);
    doc.sever(leg.upper);
    assert!(
        check(&volume, &doc, &p).is_err(),
        "a severed chain cannot keep an old support"
    );
    let current = Grounding::new(&volume)
        .with_articulation(true)
        .at(&doc, 384, 384, Yaw::Zero);
    if let Ok(current) = current {
        assert!(!current.feet.contains_key(&leg.foot));
    }
}

#[test]
fn supported_feet_on_one_side_do_not_balance_the_whole_body() {
    let volume = volume();
    let doc = quadruped();
    let mut pose = Grounding::new(&volume)
        .with_articulation(true)
        .at(&doc, 384, 384, Yaw::Zero)
        .unwrap();
    for p in pose.feet.values_mut() {
        if p[0] > 0 {
            p[0] = -2 * UNIT;
        }
    }
    for leg in legs::chains(&doc) {
        assert!(leg.solve(pose.feet[&leg.foot]).is_some());
    }
    let why = check(&volume, &doc, &pose).unwrap_err();
    assert!(
        why.contains("centre leaves"),
        "must refuse imbalance, not reach: {why}"
    );
}

#[test]
fn a_paid_foot_clears_a_riser_without_a_rigid_box_snap() {
    let mut volume = volume();
    let doc = quadruped();
    let flat = Grounding::new(&volume)
        .with_articulation(true)
        .at(&doc, 384, 384, Yaw::Zero)
        .unwrap();
    let base = foot_points(&flat).unwrap().values().next().unwrap()[1] / UNIT;
    let sim = super::tests::terrain();
    let atlas = crate::terrain::View::of(sim.genesis()).unwrap();
    volume
        .apply(
            &atlas,
            0,
            &isometer_space::Edit {
                op: isometer_space::Op::Fill(volume.materials.soil),
                shape: isometer_space::Shape::Box {
                    min: [377, base, 370],
                    max: [405, base + 2, 405],
                },
            },
        )
        .unwrap();
    let a = Grounding::new(&volume)
        .with_articulation(true)
        .at(&doc, 384, 384, Yaw::Zero)
        .unwrap();
    let places = Places::derive(&volume, Default::default()).unwrap();
    let profile = StepProfile {
        process: "walk".into(),
        cells: 1.into(),
        places: Default::default(),
        body: isometer_space::places::Body {
            width: 1,
            height: 1,
            climb: 1,
            wades: false,
        },
        stance_offset: [0; 3],
    };
    let b = neighbours(&volume, &places, &profile, &doc, &a)
        .into_iter()
        .find(|p| p.at[0] == 385 && p.at[2] == 384 && p.yaw == Yaw::Zero)
        .expect("a nearby supported root height");
    assert!(
        a.feet
            .iter()
            .any(|(id, p)| p[1] + a.at[1] * UNIT != b.feet[id][1] + b.at[1] * UNIT)
    );
    sweep_clear(&volume, &doc, &a, &b, &[]).expect("one lifted sole clears the real riser");
}

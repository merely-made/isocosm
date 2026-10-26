// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
use super::super::articulation_tests::volume;
use super::*;

#[test]
fn equal_bounds_with_different_paid_capsules_have_different_clearance() {
    let mut volume = volume();
    let y = volume.ceiling() - 3;
    let sim = super::super::tests::terrain();
    let atlas = crate::terrain::View::of(sim.genesis()).unwrap();
    volume
        .apply(
            &atlas,
            0,
            &isometer_space::Edit {
                op: isometer_space::Op::Fill(volume.materials.soil),
                shape: isometer_space::Shape::Box {
                    min: [383, y, 383],
                    max: [384, y + 1, 384],
                },
            },
        )
        .unwrap();
    let point = |x, z| [x * UNIT, y * UNIT + UNIT / 2, z * UNIT];
    let a = Capsule {
        a: point(383, 383),
        b: point(386, 386),
        radius: UNIT / 8,
    };
    let b = Capsule {
        a: point(383, 386),
        b: point(386, 383),
        radius: UNIT / 8,
    };
    assert_eq!(a.bounds(), b.bounds());
    let bounds = [
        a.bounds()[0].map(|v| v.div_euclid(UNIT)),
        a.bounds()[1].map(|v| -(-v).div_euclid(UNIT)),
    ];
    let id = PartId(99);
    let parts = BTreeMap::from([(id, vec![Solid::Link(a)])]);
    let other = BTreeMap::from([(id, vec![Solid::Link(b)])]);
    let mut cache = Clearance::new(&volume, 4);
    assert!(
        cache
            .check_parts(std::iter::once((id, bounds)), &parts)
            .is_err()
    );
    cache
        .check_parts(std::iter::once((id, bounds)), &other)
        .unwrap();
    assert_eq!(
        cache.misses, 2,
        "matching AABBs cannot share a paid-link answer"
    );
    cache
        .check_parts(std::iter::once((id, bounds)), &other)
        .unwrap();
    assert_eq!(cache.misses, 2);
}

#[test]
fn reused_parts_still_charge_each_visit_to_the_pose_clearance_budget() {
    let volume = volume();
    let y = volume.ceiling() - 2;
    let part = (PartId(99), [[370, y, 370], [386, y + 1, 386]]);
    let shapes = BTreeMap::new();
    let mut cached = Clearance::new(&volume, 4);
    let mut fresh = Clearance::new(&volume, 0);
    let a = cached.check_parts(std::iter::repeat_n(part, 8192), &shapes);
    let b = fresh.check_parts(std::iter::repeat_n(part, 8192), &shapes);
    assert_eq!(a, b);
    assert_eq!(a, Err("posed clearance budget exceeded".into()));
    assert_eq!(cached.misses, 1);
}

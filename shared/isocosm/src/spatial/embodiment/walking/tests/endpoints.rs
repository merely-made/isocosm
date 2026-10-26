// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
use super::*;

#[test]
fn arrival_reuse_preserves_stance_order_and_keeps_residents_out_of_terrain_admission() {
    let volume = volume();
    let doc = quadruped();
    let mut ground = Grounding::new(&volume).with_articulation(true);
    let from = ground.at(&doc, 384, 384, Yaw::Zero).unwrap();
    let to = ground.at(&doc, 385, 384, Yaw::Zero).unwrap();
    let blocked = boxes(&doc, &to).unwrap();
    let mut checks = Checks::new(&volume, &doc, &blocked);
    checks
        .admit(&to)
        .expect("terrain admission precedes the original resident filter");
    let step = checks.step(&from, &to, Some(Walking::default()));
    assert!(
        step.unwrap_err().contains("another native body"),
        "terrain admission cannot grant a walk through another body"
    );
    for offset in [-UNIT, -UNIT / 4, 0, UNIT / 4, UNIT] {
        let mut source = from.clone();
        source.feet.values_mut().next().unwrap()[0] += offset;
        let a = footholds::arrivals(&volume, &doc, &source, &to, Walking::default());
        let b = footholds::arrivals_checked(&source, &to, Walking::default(), |p| {
            checks.admit(p).is_ok()
        });
        assert_eq!(
            a, b,
            "resident filtering cannot change stance truncation or order"
        );
    }
    let mut bad = to.clone();
    bad.feet.values_mut().next().unwrap()[0] += 100 * UNIT;
    assert_eq!(checks.admit(&bad), check(&volume, &doc, &bad));
    assert!(checks.admit(&bad).is_err());
    let misses = checks.reuse_counts().3;
    checks.admit(&to).unwrap();
    assert!(checks.admit(&bad).is_err());
    assert_eq!(
        checks.reuse_counts().3,
        misses,
        "admitted and refused stances both reuse"
    );

    let mut changed = doc.clone();
    changed.parts[changed.root.0 as usize].half_extent[0] += 1;
    let mut fresh = Checks::new(&volume, &changed, &[]);
    assert_eq!(fresh.admit(&to), check(&volume, &changed, &to));
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
    assert!(Checks::new(&edited, &doc, &[]).admit(&to).is_err());
}

#[test]
fn resident_reuse_matches_half_open_collision_and_a_new_scope_reads_moved_bodies() {
    let volume = volume();
    let doc = quadruped();
    let blocked = [[[0, 0, 0], [4, 4, 4]]];
    let queries = [
        [[4, 0, 0], [5, 4, 4]],
        [[-1, 0, 0], [0, 4, 4]],
        [[1, 4, 1], [2, 5, 2]],
        [[1, 1, 1], [2, 2, 2]],
        [[-2, -2, -2], [-1, -1, -1]],
    ];
    let mut checks = Checks::new(&volume, &doc, &blocked);
    for b in &queries {
        assert_eq!(
            checks.intersects_residents(&[*b]),
            intersects(&[*b], &blocked)
        );
        assert_eq!(
            checks.intersects_residents(&[*b]),
            intersects(&[*b], &blocked)
        );
    }
    assert_eq!(checks.reuse_counts().0, queries.len() * 2);
    assert_eq!(checks.reuse_counts().1, queries.len());
    checks.set_capacity(2);
    for b in &queries {
        assert_eq!(
            checks.intersects_residents(&[*b]),
            intersects(&[*b], &blocked)
        );
    }
    assert_eq!(
        checks.intersects_residents(&queries),
        intersects(&queries, &blocked)
    );
    let moved = [[[4, 0, 0], [8, 4, 4]]];
    let mut fresh = Checks::new(&volume, &doc, &moved);
    assert!(fresh.intersects_residents(&[queries[0]]));
    assert!(!checks.intersects_residents(&[queries[0]]));
}

#[test]
fn prepared_arrival_components_keep_native_admission_errors_for_all_facings() {
    let volume = volume();
    let doc = quadruped();
    let mut checks = Checks::new(&volume, &doc, &[]);
    let mut g = Grounding::new(&volume).with_articulation(true);
    for yaw in [Yaw::Zero, Yaw::Quarter, Yaw::Half, Yaw::ThreeQuarter] {
        let pose = g.at(&doc, 384, 384, yaw).unwrap();
        let mut variants = vec![pose.clone()];
        let mut raised = pose.clone();
        raised.feet.values_mut().next().unwrap()[1] += UNIT;
        variants.push(raised);
        let mut extra = pose.clone();
        extra.feet.insert(PartId(999), [0; 3]);
        variants.push(extra);
        let mut low = pose.clone();
        low.at[1] -= 2;
        variants.push(low);
        let mut no_feet = pose;
        no_feet.feet.clear();
        variants.push(no_feet);
        for p in variants {
            assert_eq!(checks.admit(&p), check(&volume, &doc, &p));
        }
    }
}

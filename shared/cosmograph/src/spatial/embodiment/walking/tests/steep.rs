// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
use super::*;

#[test]
fn retained_steep_pose_uses_paid_intermediate_stances() {
    let f: Fixture = serde_json::from_str(include_str!("../fixtures/steep.json")).unwrap();
    let atlas = crate::terrain::View {
        seed: f.seed,
        sites: &f.sites,
        footprint: f.footprint,
        skeleton: &f.skeleton,
        materials: &f.materials,
    };
    let volume = Volume::paged(&atlas, f.site, [555, 165], [630, 240], 64, &[]).unwrap();
    check(&volume, &f.document, &f.before).unwrap();
    let rules = Walking::default();
    for canonical in &f.after {
        check(&volume, &f.document, canonical).unwrap();
        assert!(
            plan_stride(
                &volume,
                &f.document,
                &f.before,
                canonical,
                Walking {
                    stances: 0,
                    ..rules
                },
                &[]
            )
            .is_err()
        );
    }
    let mut accepted = 0;
    for canonical in &f.after {
        for to in footholds::arrivals(&volume, &f.document, &f.before, canonical, rules) {
            let result = plan_stride(&volume, &f.document, &f.before, &to, rules, &[]);
            eprintln!(
                "arrival {:?} {:?}: {}",
                to.at,
                to.feet,
                serde_json::to_string(&result).unwrap()
            );
            if let Ok(stride) = result {
                check_stride(&volume, &f.document, &f.before, &to, rules, &stride, &[]).unwrap();
                assert_ne!(to.feet, canonical.feet);
                accepted += 1;
                break;
            }
        }
    }
    assert!(
        accepted == 4,
        "retained terrain must admit a connected paid arrival stance"
    );
}

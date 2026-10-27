// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
use super::super::articulation_tests::{quadruped, volume};
use super::*;

#[test]
fn resolved_limb_mass_follows_the_existing_skin_in_fixed_point() {
    let doc = quadruped();
    let v = volume();
    let pose = Grounding::new(&v)
        .with_articulation(true)
        .at(&doc, 384, 384, Yaw::Zero)
        .unwrap();
    let leg = &isometer_core::legs::chains(&doc)[0];
    let mass = Balance::from_parts(&doc, [(leg.foot, 100)]).unwrap();
    let centre = mass
        .at(&doc, &pose.feet, Rotation::at(Yaw::Zero, 0))
        .unwrap();
    let skin = leg.skin(pose.feet[&leg.foot]).unwrap();
    assert_eq!(
        centre,
        [0, 1, 2].map(|k| skin.pad[0][k] + (skin.pad[1][k] - skin.pad[0][k]) / 2)
    );
    let mut feet = pose.feet.clone();
    feet.get_mut(&leg.foot).unwrap()[0] += UNIT / 4;
    let moved = mass.at(&doc, &feet, Rotation::at(Yaw::Zero, 0)).unwrap();
    assert_eq!(moved[0] - centre[0], UNIT / 4);
    for yaw in [Yaw::Quarter, Yaw::Half, Yaw::ThreeQuarter] {
        let rotation = Rotation::at(yaw, 0);
        assert_eq!(
            mass.at(&doc, &feet, rotation).unwrap(),
            rotation.rotate(moved).unwrap()
        );
    }
}

#[test]
fn matter_distribution_changes_native_support_and_scoped_admission() {
    let mut doc = quadruped();
    let ballast = doc
        .attach(
            isometer_core::VolumeRef::from_tag(3),
            [1; 3],
            isometer_core::Attachment {
                parent: doc.root,
                offset: [25, 3, 0],
                yaw: Yaw::Zero,
            },
            None,
        )
        .unwrap();
    let v = volume();
    let pose = Grounding::new(&v)
        .with_articulation(true)
        .at(&doc, 384, 384, Yaw::Zero)
        .unwrap();
    let centred = Balance::from_parts(&doc, [(doc.root, 1000), (ballast, 1)]).unwrap();
    let loaded = Balance::from_parts(&doc, [(doc.root, 1), (ballast, 1000)]).unwrap();
    let mut checks = StrideChecks::new(&v, &doc, &[]).with_balance(Some(centred.clone()));
    assert!(checks.admit(&pose).is_ok());
    let mut checks = checks.with_balance(Some(loaded.clone()));
    assert!(
        checks
            .admit(&pose)
            .unwrap_err()
            .contains("posed matter centre")
    );
    assert!(check_balanced(&v, &doc, &pose, Some(&loaded)).is_err());
    assert!(check_balanced(&v, &doc, &pose, Some(&centred)).is_ok());
    assert!(
        check(&v, &doc, &pose).is_ok(),
        "saved volume policy stays unchanged"
    );
}

#[test]
fn native_part_ledgers_supply_mass_and_zero_or_overflow_is_refused() {
    let sim = super::super::tests::terrain();
    let rules = &sim.genesis().rules;
    let key = rules
        .accounts
        .iter()
        .find(|(_, kind)| matches!(kind, crate::rules::AccountKind::Matter { .. }))
        .unwrap()
        .0;
    let mut entity = sim
        .state()
        .population
        .groups
        .values()
        .next()
        .unwrap()
        .entity
        .clone();
    let entity = std::sync::Arc::make_mut(&mut entity);
    let doc = quadruped();
    entity.parts = doc
        .living()
        .map(|p| (p.id, std::sync::Arc::new(crate::schema::Part::default())))
        .collect();
    std::sync::Arc::make_mut(entity.parts.get_mut(&doc.root).unwrap())
        .matter
        .insert(key.clone(), 321);
    entity.body = Some(doc.clone());
    assert_eq!(Balance::of(&entity, rules).unwrap().mass_mg(), 321);
    assert!(Balance::from_parts(&doc, []).is_err());
    assert!(Balance::from_parts(&doc, [(doc.root, u128::MAX)]).is_err());
}

// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The attachment hypothesis, end to end.
//!
//! The body pipeline plan names this as the wing's unproven assumption: a part
//! attaches to a living body **during play**, acquires collision and mass,
//! moves the centre of balance, and stays legible.
//!
//! Isometer's half, on its own fixtures (wing ruling 720): a part attached to
//! a body document changes all four at once, deterministically. Whether a
//! meal lands a part is the sim's, and certified there.

use isometer_core::fixtures::walker;
use isometer_core::{
    Attachment, BodyDocument, Origin, PartId, Provenance, SpeciesId, VolumeRef, Yaw,
};
use isometer_mesh::{Volume, VolumeMap, mesh_body};

/// A volume for every tag a fixture may cite.
fn source() -> VolumeMap {
    let mut map = VolumeMap::new();
    map.insert(VolumeRef::from_tag(1), Volume::solid([3, 3, 3], 1));
    for tag in 2..=u8::MAX {
        map.insert(VolumeRef::from_tag(tag), Volume::solid([2, 2, 2], tag));
    }
    map
}

/// A meal's part, landed on the root at `offset`, taken from species 42.
fn eat(body: &mut BodyDocument, tag: u8, offset: [i32; 3], yaw: Yaw) -> PartId {
    let provenance = Provenance {
        origin: Origin::Incorporated {
            from_species: SpeciesId(42),
            from_part: PartId(0),
        },
        epoch: 1,
    };
    let attachment = Attachment {
        parent: body.root,
        offset,
        yaw,
    };
    body.attach(
        VolumeRef::from_tag(tag),
        400,
        [1, 1, 1],
        attachment,
        provenance,
    )
    .expect("the meal's part lands")
}

#[test]
fn eating_changes_mass_balance_collision_and_geometry() {
    let source = source();
    let mut body = walker();
    let mass_before = body.total_mass_mg();
    let centre_before = body.centre_of_mass();
    let collision_before = body.aabb();
    let drawn_before = mesh_body(&body, &source).unwrap();

    eat(&mut body, 40, [9, 0, 0], Yaw::Zero);
    let drawn_after = mesh_body(&body, &source).unwrap();

    assert!(body.total_mass_mg() > mass_before, "the body got heavier");
    let centre_after = body.centre_of_mass();
    assert!(
        centre_after[0] > centre_before[0],
        "centre of mass moved toward the new part: {centre_before:?} -> {centre_after:?}"
    );
    assert!(
        body.aabb().extent()[0] > collision_before.extent()[0],
        "the collision box grew"
    );
    assert_eq!(
        drawn_after.placement_count(),
        drawn_before.placement_count() + 1
    );
    assert!(drawn_after.drawn_quads() > drawn_before.drawn_quads());
    let (_, max_before) = drawn_before.bounds().unwrap();
    let (_, max_after) = drawn_after.bounds().unwrap();
    assert!(
        max_after[0] > max_before[0],
        "the drawn body reaches further"
    );
}

#[test]
fn an_eaten_part_still_says_whose_it_was() {
    let mut body = walker();
    let part = eat(&mut body, 40, [9, 0, 0], Yaw::Quarter);

    let mesh = mesh_body(&body, &source()).unwrap();
    let placement = mesh
        .placements
        .iter()
        .find(|p| p.part == part)
        .expect("the new part is placed");
    assert_eq!(placement.yaw, Yaw::Quarter);
    match body.part(part).unwrap().provenance.origin {
        Origin::Incorporated { from_species, .. } => assert_eq!(from_species, SpeciesId(42)),
        Origin::Founding => panic!("an eaten part is not founding stock"),
    }
}

#[test]
fn attaching_remeshes_only_what_is_new() {
    let source = source();
    let mut body = walker();
    let before = mesh_body(&body, &source).unwrap();
    let root_mesh_before = before.mesh_for(VolumeRef::from_tag(1)).cloned();

    eat(&mut body, 40, [6, 0, 0], Yaw::Zero);
    let after = mesh_body(&body, &source).unwrap();

    assert_eq!(
        after.mesh_for(VolumeRef::from_tag(1)).cloned(),
        root_mesh_before,
        "the existing body's geometry is untouched by an attachment"
    );
    assert!(after.mesh_count() >= before.mesh_count());
}

#[test]
fn a_body_grown_over_many_meals_stays_deterministic() {
    let source = source();
    let grow = || {
        let mut body = walker();
        for meal in 0..5 {
            eat(&mut body, 40 + meal as u8, [5 + 3 * meal, 0, 0], Yaw::Zero);
        }
        body
    };

    let (a, b) = (grow(), grow());
    assert_eq!(a, b);
    let (mesh_a, mesh_b) = (
        mesh_body(&a, &source).unwrap(),
        mesh_body(&b, &source).unwrap(),
    );
    assert_eq!(mesh_a.placements, mesh_b.placements);
    assert_eq!(mesh_a.drawn_quads(), mesh_b.drawn_quads());
    assert!(mesh_a.placement_count() > 1, "the fixture actually ate");
}

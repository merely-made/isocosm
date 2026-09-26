// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The body events, each a test. One claim per event: what founding anatomy
//! bears, what growth and grafting add, what severing takes, and what a body
//! whose living parts express nothing bears.

use super::*;
use crate::body::{Attachment, BodyDocument, Origin, Provenance, SpeciesId, VolumeRef, Yaw};
use crate::phenotype::graft::Lowering;
use crate::plan::{Role, classify};
use wing_glyphs::{ExpressionSpec, ExpressionTable, GlyphExpression, SCHEMA_VERSION};

const EARTH: &str = "mesocosm:earth";
const SUN: &str = "mesocosm:sun";
const REACH: &str = "mesocosm:reach";
const STING: &str = "mesocosm:sting";

/// Half-extents whose `classify` role is the fixture's whole premise, asserted
/// rather than assumed: a shape that stopped being a plate would make every
/// expectation below vacuous.
const BULK: [i32; 3] = [2, 2, 2];
const FROND: [i32; 3] = [6, 4, 1];
const LIMB: [i32; 3] = [4, 1, 1];

/// The authored table these tests read against: two glyphs on one trait each,
/// one glyph expressed by two traits (disjunction), and one glyph whose trait
/// nothing here grows.
fn table() -> ExpressionTable {
    table_from(vec![
        (EARTH, vec!["mesocosm:intake"]),
        (SUN, vec!["mesocosm:fix"]),
        (REACH, vec!["mesocosm:contract", "mesocosm:sense"]),
        (STING, vec!["mesocosm:secrete"]),
    ])
}

fn table_from(entries: Vec<(&str, Vec<&str>)>) -> ExpressionTable {
    ExpressionTable::new(ExpressionSpec {
        version: SCHEMA_VERSION,
        id: "mesocosm:test-expression".into(),
        canon_revision: 1,
        entries: entries
            .into_iter()
            .map(|(glyph, traits)| GlyphExpression {
                glyph: glyph.into(),
                traits: traits.into_iter().map(str::to_owned).collect(),
            })
            .collect(),
        limits: Default::default(),
    })
    .unwrap()
}

fn grown(parent: PartId, offset: [i32; 3]) -> (Attachment, Provenance) {
    (
        Attachment {
            parent,
            offset,
            yaw: Yaw::Zero,
        },
        Provenance {
            origin: Origin::Founding,
            epoch: 0,
        },
    )
}

/// A producer's founding anatomy: a bulk root that takes in, and a frond that
/// fixes. Exactly the shape the trial's bound organism has.
fn producer() -> (BodyPhenotype, PartId) {
    assert_eq!(classify(BULK), Role::Mass);
    assert_eq!(classify(FROND), Role::Plate);
    let body = BodyDocument::new(SpeciesId(1), VolumeRef::from_tag(1), 800, BULK);
    let mut phenotype = BodyPhenotype::seed(body);
    let (at, from) = grown(PartId(0), [0, 3, 0]);
    let frond = phenotype
        .attach(VolumeRef::from_tag(2), 400, FROND, at, from)
        .unwrap();
    (phenotype, frond)
}

fn glyphs(set: &BTreeSet<GlyphId>) -> Vec<&str> {
    set.iter().map(String::as_str).collect()
}

#[test]
fn founding_anatomy_embodies_exactly_the_glyphs_its_seeded_tracts_express() {
    let (phenotype, frond) = producer();
    let registry = Registry::native();
    let table = table();
    let found = embodied(&phenotype, registry, &table);

    // Intake on the bulk root and Fix on the frond, from the first frame,
    // having done nothing. Nothing seeds Secrete, so its glyph is absent.
    assert_eq!(glyphs(&found), [EARTH, SUN]);
    assert_eq!(
        expressed_traits(&phenotype, registry)
            .into_iter()
            .collect::<Vec<_>>(),
        ["mesocosm:fix", "mesocosm:intake"],
        "the resolution half on its own"
    );
    assert!(!found.contains(STING), "nothing grows a gland");
    assert!(
        !found.contains(REACH),
        "this body has no limb and no sensor"
    );

    // What bears each glyph, which is what a mark anchors to.
    assert_eq!(bearing_parts(&phenotype, registry, &table, SUN), [frond]);
    assert_eq!(
        bearing_parts(&phenotype, registry, &table, EARTH),
        [PartId(0)]
    );
    assert!(bearing_parts(&phenotype, registry, &table, STING).is_empty());

    // The reading is `&`-only and a pure function of the phenotype: run twice
    // over one unchanged body it is identical, and the body is unchanged by
    // being read.
    let before = phenotype.digest();
    for _ in 0..8 {
        assert_eq!(embodied(&phenotype, registry, &table), found);
    }
    assert_eq!(phenotype.digest(), before);

    // A registry that does not hold a tract's definition contributes nothing,
    // rather than the nearest local definition.
    let empty = Registry::admit(Vec::new()).unwrap();
    assert!(embodied(&phenotype, &empty, &table).is_empty());
    // And a table carrying no entry for a borne trait is simply silent.
    assert!(
        embodied(
            &phenotype,
            registry,
            &table_from(vec![(STING, vec!["mesocosm:secrete"])])
        )
        .is_empty()
    );
}

#[test]
fn growth_is_monotone_on_the_body_that_grew() {
    assert_eq!(classify(LIMB), Role::Limb);
    let (mut phenotype, _) = producer();
    let registry = Registry::native();
    let table = table();
    let before = embodied(&phenotype, registry, &table);

    let (at, from) = grown(PartId(0), [0, -3, 0]);
    let limb = phenotype
        .attach(VolumeRef::from_tag(3), 200, LIMB, at, from)
        .unwrap();
    let after = embodied(&phenotype, registry, &table);

    assert!(before.is_subset(&after), "a new part only ever adds");
    assert_eq!(glyphs(&after), [EARTH, REACH, SUN]);
    assert_eq!(bearing_parts(&phenotype, registry, &table, REACH), [limb]);

    // Disjunction: a second trait expressing the same glyph changes nothing
    // about whether it is embodied.
    let sensor_only = table_from(vec![(REACH, vec!["mesocosm:sense"])]);
    assert!(
        !embodied(&phenotype, registry, &sensor_only).contains(REACH),
        "the limb contracts; it does not sense"
    );
}

#[test]
fn a_received_branch_is_monotone_on_the_recipient() {
    let registry = Registry::native();
    let table = table();

    // A donor whose branch is a limb, so what arrives is a trait the
    // recipient did not have.
    let (mut donor, _) = producer();
    let (at, from) = grown(PartId(0), [0, -3, 0]);
    let source = donor
        .attach(VolumeRef::from_tag(3), 200, LIMB, at, from)
        .unwrap();
    let branch = donor.harvest(source).expect("a living non-root subtree");

    let (mut recipient, _) = producer();
    let before = embodied(&recipient, registry, &table);
    assert_eq!(glyphs(&before), [EARTH, SUN]);
    let landing = Attachment {
        parent: PartId(0),
        offset: [3, 0, 0],
        yaw: Yaw::Zero,
    };
    let graftage = recipient
        .receive(registry, &branch, landing, 0, Lowering::Regrown)
        .expect("the recipient can hold a limb");
    let after = embodied(&recipient, registry, &table);

    assert!(before.is_subset(&after), "a graft only ever adds here");
    assert_eq!(glyphs(&after), [EARTH, REACH, SUN]);
    assert_eq!(
        bearing_parts(&recipient, registry, &table, REACH),
        [graftage.root],
        "the arrived part bears it under its new id"
    );
    // The donor still has the branch attached: `harvest` reads, it does not
    // take, so this test says nothing about the donor losing anything.
    assert!(embodied(&donor, registry, &table).contains(REACH));
}

#[test]
fn severing_drops_the_tombstoned_parts_contribution_while_its_mosaic_still_answers() {
    let (mut phenotype, frond) = producer();
    let registry = Registry::native();
    let table = table();
    assert!(embodied(&phenotype, registry, &table).contains(SUN));

    let lost = phenotype.sever(frond);
    assert_eq!(lost, [frond]);
    let after = embodied(&phenotype, registry, &table);
    assert_eq!(glyphs(&after), [EARTH], "the frond stopped fixing");
    assert!(bearing_parts(&phenotype, registry, &table, SUN).is_empty());

    // The injury is still explainable: the mosaic answers, and its tract is
    // still readable. It is history, and it expresses nothing.
    let mosaic = phenotype.mosaic(frond).expect("a severed mosaic stays");
    assert!(
        mosaic
            .tracts()
            .iter()
            .any(|tract| registry.resolve(tract.process).unwrap().id.qualified() == "mesocosm:fix"),
        "what this branch used to do is still on the record"
    );
    assert!(!phenotype.body().is_living(frond));
    assert!(
        !expressed_traits(&phenotype, registry).contains("mesocosm:fix"),
        "history is not expression"
    );
}

#[test]
fn a_body_whose_living_parts_express_nothing_in_the_table_bears_nothing() {
    // `BodyDocument::sever` refuses the root by construction, so "every part
    // severed" is a body cut down to its root. Under a table that expresses
    // nothing the root bears, that body embodies the empty set — the same
    // answer a body has after death, which the runtime reading tests where
    // death is actually known.
    let (mut phenotype, frond) = producer();
    let registry = Registry::native();
    let fixing_only = table_from(vec![(SUN, vec!["mesocosm:fix"])]);
    assert_eq!(glyphs(&embodied(&phenotype, registry, &fixing_only)), [SUN]);

    phenotype.sever(frond);
    assert!(
        embodied(&phenotype, registry, &fixing_only).is_empty(),
        "no living part expresses anything this table carries"
    );
    // The body is still there and still takes in; it just bears no glyph.
    assert!(phenotype.body().is_living(PartId(0)));
    assert!(expressed_traits(&phenotype, registry).contains("mesocosm:intake"));
}

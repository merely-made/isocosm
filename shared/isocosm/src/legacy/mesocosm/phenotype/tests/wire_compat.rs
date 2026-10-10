// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Ruling 224 (2026-09-25): the tract rename writes the new word and still
//! reads the old. New data serializes under the new vocabulary; a value
//! captured before the rename lane landed still says `site`/`sites`, and
//! must still deserialize to exactly what a fresh value of the same shape
//! produces today.

use super::*;

/// A mosaic recorded under the old field names (`sites`, `next_site`) reads
/// back identical to the same mosaic under the new ones.
///
/// The "old form" here is not invented: it is a real mosaic's own JSON,
/// mechanically renamed on exactly the two keys the rename touched, so
/// everything else (dims, lost cells, port, scruple, cell ids, cause) is
/// asserted to still line up rather than merely assumed to.
#[test]
fn an_old_form_mosaic_still_deserializes_to_the_same_value() {
    let (phenotype, [root, ..]) = critter();
    let mosaic = phenotype
        .mosaic(root)
        .expect("a living part has a mosaic")
        .clone();

    let mut old_form = serde_json::to_value(&mosaic).expect("a mosaic serializes to JSON");
    let object = old_form.as_object_mut().expect("a mosaic is a JSON object");
    let tracts = object
        .remove("tracts")
        .expect("the new field name is present");
    object.insert("sites".to_owned(), tracts);
    let next_tract = object
        .remove("next_tract")
        .expect("the new field name is present");
    object.insert("next_site".to_owned(), next_tract);

    let restored: Mosaic =
        serde_json::from_value(old_form).expect("the old field names still deserialize");
    assert_eq!(
        restored, mosaic,
        "a mosaic recorded under the old vocabulary reads back identical to the new one"
    );
}

/// A `Refusal` recorded under its old variant names (`SiteMismatch`,
/// `EmptySite`, `TooManySites`) still deserializes to the renamed variant.
#[test]
fn old_form_refusal_variants_still_deserialize_to_the_same_value() {
    let cases = [
        (
            serde_json::json!({"SiteMismatch": {"part": 3, "process": {"definition": 9}}}),
            Refusal::TractMismatch {
                part: PartId(3),
                process: ProcessRef {
                    definition: crate::process::DefinitionDigest(9),
                },
            },
        ),
        (
            serde_json::json!({"EmptySite": 2}),
            Refusal::EmptyTract(PartId(2)),
        ),
        (
            serde_json::json!({"TooManySites": 1}),
            Refusal::TooManyTracts(PartId(1)),
        ),
    ];
    for (old_form, expected) in cases {
        let restored: Refusal =
            serde_json::from_value(old_form).expect("the old variant name still deserializes");
        assert_eq!(restored, expected, "{expected:?}");
    }
}

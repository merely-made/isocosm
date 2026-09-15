// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
use super::*;
use crate::{CanonSpec, GlyphDefinition, VariantDefinition};

/// Five bases, one trait each, plus one trait shared by two bases so the
/// many-to-many inverse has something to answer with.
fn spec() -> ExpressionSpec {
    ExpressionSpec {
        version: SCHEMA_VERSION,
        id: "world:demo-expression".into(),
        canon_revision: 0,
        entries: (0..5)
            .map(|n| GlyphExpression {
                glyph: format!("glyph:{n}"),
                traits: vec![format!("trait:t-{n}")],
            })
            .collect(),
        limits: Default::default(),
    }
}

fn canon(revision: u64) -> Canon {
    Canon::new(CanonSpec {
        version: SCHEMA_VERSION,
        id: "world:demo".into(),
        revision,
        glyphs: (0..5)
            .map(|n| GlyphDefinition {
                id: format!("glyph:{n}"),
                display: format!("{n}"),
                effect: format!("effect:reference-{n}"),
            })
            .collect(),
        variants: vec![VariantDefinition {
            id: "glyph:variant".into(),
            display: "\u{e9}".into(),
            base: "glyph:0".into(),
            modifiers: vec!["modifier:acute".into()],
        }],
        limits: Default::default(),
    })
    .unwrap()
}

#[test]
fn a_table_indexes_both_directions_and_round_trips_json_within_its_limit() {
    let mut spec = spec();
    // One trait expressing two bases is expected, not an error (R2).
    spec.entries[1].traits.push("trait:t-0".into());
    let table = ExpressionTable::new(spec).unwrap();
    assert_eq!(table.traits_of("glyph:1"), ["trait:t-1", "trait:t-0"]);
    assert_eq!(table.glyphs_of("trait:t-0"), ["glyph:0", "glyph:1"]);
    assert_eq!(table.glyphs_of("trait:t-1"), ["glyph:1"]);
    // An unknown key is an empty answer, never a panic and never a guess.
    assert!(table.traits_of("glyph:absent").is_empty());
    assert!(table.glyphs_of("trait:absent").is_empty());

    let json = table.to_json().unwrap();
    assert_eq!(ExpressionTable::from_json(&json).unwrap(), table);
    assert!(table.to_json_with_limit(1).is_err());
    assert!(ExpressionTable::from_json_with_limit(&json, json.len() - 1).is_err());
    let clone = table.clone();
    assert!(Arc::ptr_eq(&table.spec, &clone.spec));
    assert!(Arc::ptr_eq(&table.by_trait, &clone.by_trait));
}

#[test]
fn covers_requires_every_base_to_have_a_trait_and_the_revisions_to_agree() {
    let table = ExpressionTable::new(spec()).unwrap();
    assert!(
        table.covers(&canon(0)).is_ok(),
        "every base is expressed at the authored revision"
    );
    // The table is authored against revision 0; a later canon is a different
    // question and is refused rather than silently answered.
    let why = table.covers(&canon(1)).unwrap_err();
    assert!(why.contains("canon revision 0, not 1"), "{why}");

    let mut short = spec();
    short.entries.pop();
    let why = ExpressionTable::new(short)
        .unwrap()
        .covers(&canon(0))
        .unwrap_err();
    assert!(why.contains("no trait expresses base glyph:4"), "{why}");
    // Coverage is a claim about bases. A variant carries no expression of its
    // own and is not one of them.
    assert!(table.traits_of("glyph:variant").is_empty());
}

#[test]
fn an_empty_or_duplicated_trait_list_and_a_bad_identifier_refuse_at_new() {
    let mut empty = spec();
    empty.entries[2].traits.clear();
    let why = ExpressionTable::new(empty).unwrap_err();
    assert!(why.contains("glyph:2 requires 1.."), "{why}");

    let mut twice = spec();
    twice.entries[0].traits.push("trait:t-0".into());
    assert!(
        ExpressionTable::new(twice)
            .unwrap_err()
            .contains("duplicate expressing trait")
    );
    let mut same = spec();
    same.entries[1].glyph = "glyph:0".into();
    assert!(
        ExpressionTable::new(same)
            .unwrap_err()
            .contains("duplicate glyph")
    );
    // The identifier rule is `canon.rs`'s, reused rather than restated.
    let mut unqualified = spec();
    unqualified.entries[0].traits[0] = "bare".into();
    assert!(ExpressionTable::new(unqualified).is_err());
    let mut bad_glyph = spec();
    bad_glyph.entries[0].glyph = "glyph:".into();
    assert!(ExpressionTable::new(bad_glyph).is_err());
    let mut version = spec();
    version.version = SCHEMA_VERSION + 1;
    assert!(ExpressionTable::new(version).is_err());

    let mut limited = spec();
    limited.limits.entries = 4;
    assert!(ExpressionTable::new(limited.clone()).is_err());
    limited.limits.entries = 5;
    limited.limits.traits_per_glyph = 0;
    assert!(ExpressionTable::new(limited).is_err());
    // Deserialization goes through the same door, so a hand-written file
    // cannot admit what `new` refuses.
    let mut stray = serde_json::to_value(spec()).unwrap();
    stray["extra"] = true.into();
    assert!(serde_json::from_value::<ExpressionTable>(stray).is_err());
}

#[test]
fn a_seeded_shuffle_has_a_literal_receipt_and_is_pinned_rather_than_rerun() {
    let authored = ExpressionTable::new(spec()).unwrap();
    let shuffled = authored.shuffled(7, 1).unwrap();
    assert_eq!(
        shuffled
            .spec
            .entries
            .iter()
            .map(|e| e.traits[0].as_str())
            .collect::<Vec<_>>(),
        [
            "trait:t-4",
            "trait:t-1",
            "trait:t-3",
            "trait:t-0",
            "trait:t-2"
        ],
        "the same permutation Canon::shuffled draws at this seed and length"
    );
    // Identity and the glyph set are untouched: only which traits express
    // which glyph moved, and the whole trait set is still spent exactly once.
    assert_eq!(shuffled.spec.id, authored.spec.id);
    assert_eq!(shuffled.spec.canon_revision, 1);
    for (a, b) in authored.spec.entries.iter().zip(&shuffled.spec.entries) {
        assert_eq!(a.glyph, b.glyph);
    }
    let mut moved: Vec<_> = shuffled
        .spec
        .entries
        .iter()
        .flat_map(|e| e.traits.clone())
        .collect();
    moved.sort();
    assert_eq!(
        moved,
        [
            "trait:t-0",
            "trait:t-1",
            "trait:t-2",
            "trait:t-3",
            "trait:t-4"
        ]
    );
    // The inverse index followed the permutation.
    assert_eq!(shuffled.glyphs_of("trait:t-4"), ["glyph:0"]);
    assert_eq!(authored.glyphs_of("trait:t-4"), ["glyph:4"]);

    // Deterministic, and the authored table is unchanged by being shuffled:
    // a world saves this result and never reruns it.
    assert_eq!(shuffled, authored.shuffled(7, 1).unwrap());
    assert_eq!(authored.spec(), &spec());
    assert_ne!(authored.shuffled(8, 1).unwrap(), shuffled);
    // A shuffle is a new revision, so it cannot quietly restate the old one.
    assert!(authored.shuffled(7, 0).is_err());
    assert!(shuffled.shuffled(7, 1).is_err());
    // The shuffled table describes the canon revision it names, not the one
    // it came from.
    assert!(shuffled.covers(&canon(1)).is_ok());
    assert!(shuffled.covers(&canon(0)).is_err());
}

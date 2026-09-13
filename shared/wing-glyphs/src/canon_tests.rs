// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
use super::*;

fn spec() -> CanonSpec {
    CanonSpec {
        version: 1,
        id: "world:demo".into(),
        revision: 0,
        glyphs: (0..5)
            .map(|n| GlyphDefinition {
                id: format!("glyph:{n}"),
                display: format!("{n}"),
                effect: format!("effect:reference-{n}"),
            })
            .collect(),
        variants: vec![VariantDefinition {
            id: "glyph:variant".into(),
            display: "é".into(),
            base: "glyph:0".into(),
            modifiers: vec!["modifier:acute".into()],
        }],
        limits: Default::default(),
    }
}

#[test]
fn seeded_correspondence_has_literal_receipt_and_preserves_identity_display_and_bijection() {
    let original = Canon::new(spec()).unwrap();
    let shuffled = original.shuffled(7, 1).unwrap();
    assert_eq!(
        shuffled
            .spec
            .glyphs
            .iter()
            .map(|g| g.effect.as_str())
            .collect::<Vec<_>>(),
        [
            "effect:reference-4",
            "effect:reference-1",
            "effect:reference-3",
            "effect:reference-0",
            "effect:reference-2"
        ]
    );
    assert_eq!(original.spec(), &spec());
    assert_eq!(shuffled.spec.id, original.spec.id);
    assert_eq!(shuffled.spec.variants, original.spec.variants);
    for (a, b) in original.spec.glyphs.iter().zip(&shuffled.spec.glyphs) {
        assert_eq!((&a.id, &a.display), (&b.id, &b.display));
    }
    assert_eq!(shuffled, original.shuffled(7, 1).unwrap());
    assert!(original.shuffled(7, 0).is_err());
}

#[test]
fn unicode_display_is_opaque_and_explicit_variant_is_not_inferred() {
    let mut spec = spec();
    spec.glyphs[0].display = "é".into();
    spec.glyphs[1].display = "e\u{301}".into();
    spec.glyphs[2].display = "👩‍🎤{}".into();
    spec.glyphs[3].display = "é".into();
    let canon = Canon::new(spec).unwrap();
    assert_eq!(canon.base_id("glyph:variant"), Some("glyph:0"));
    assert!(canon.is_base("glyph:1"));
    assert_eq!(canon.spec.glyphs[1].display, "e\u{301}");
    assert_eq!(Canon::from_json(&canon.to_json().unwrap()).unwrap(), canon);
}

#[test]
fn deserialization_cannot_admit_duplicate_correspondence_or_bad_variants() {
    let mut duplicate = spec();
    duplicate.glyphs[1].effect = duplicate.glyphs[0].effect.clone();
    assert!(serde_json::from_value::<Canon>(serde_json::to_value(duplicate).unwrap()).is_err());
    let mut duplicate = spec();
    duplicate.glyphs[1].id = duplicate.glyphs[0].id.clone();
    assert!(Canon::new(duplicate).is_err());
    let mut missing = spec();
    missing.variants[0].base = "glyph:missing".into();
    assert!(Canon::new(missing).is_err());
    let mut unknown = serde_json::to_value(spec()).unwrap();
    unknown["extra"] = true.into();
    assert!(serde_json::from_value::<Canon>(unknown).is_err());
}

#[test]
fn admission_and_json_bounds_are_explicit_and_configurable() {
    let mut limited = spec();
    limited.limits.glyphs = 4;
    assert!(Canon::new(limited.clone()).is_err());
    limited.limits.glyphs = 5;
    let canon = Canon::new(limited).unwrap();
    assert!(canon.to_json_with_limit(1).is_err());
    let json = canon.to_json().unwrap();
    assert!(Canon::from_json_with_limit(&json, json.len() - 1).is_err());
    assert_eq!(
        Canon::from_json_with_limit(&json, json.len()).unwrap(),
        canon
    );
}

#[test]
fn larger_canon_is_caller_admitted_and_clones_share_immutable_storage() {
    let mut spec = spec();
    spec.glyphs = (0..5000)
        .map(|n| GlyphDefinition {
            id: format!("glyph:{n}"),
            display: "a".into(),
            effect: format!("effect:reference-{n}"),
        })
        .collect();
    spec.limits.glyphs = 5000;
    let canon = Canon::new(spec).unwrap();
    let clone = canon.clone();
    assert!(Arc::ptr_eq(&canon.spec, &clone.spec));
    assert!(Arc::ptr_eq(&canon.base_indices, &clone.base_indices));
    assert_eq!(clone.effect("glyph:4999"), Some("effect:reference-4999"));
    assert_eq!(clone.effect("glyph:variant"), Some("effect:reference-0"));
}

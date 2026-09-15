// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
use super::*;
use wing_glyphs::{CanonSpec, EffectPack, GlyphDefinition};

const AT: [i32; 3] = [4, 12, -3];
const TO: [i32; 3] = [5, 12, -3];

fn canon(effect: &str) -> Canon {
    Canon::new(CanonSpec {
        version: SCHEMA_VERSION,
        id: "mesocosm:test-canon".into(),
        revision: 1,
        glyphs: vec![
            GlyphDefinition {
                id: "mesocosm:earth".into(),
                display: "#".into(),
                effect: effect.into(),
            },
            GlyphDefinition {
                id: "mesocosm:other".into(),
                display: "/".into(),
                effect: "mesocosm:unclaimed-reference".into(),
            },
        ],
        variants: vec![wing_glyphs::VariantDefinition {
            id: "mesocosm:earth-acute".into(),
            display: "#\u{301}".into(),
            base: "mesocosm:earth".into(),
            modifiers: vec!["mesocosm:acute".into()],
        }],
        limits: Default::default(),
    })
    .unwrap()
}

#[test]
fn an_unacquired_effect_paints_nothing_at_any_pole_and_an_undeclared_one_has_no_rule() {
    let table = EffectPackTable::default_pack();
    for pole in [Acquiring::Carved, Acquiring::Fed, Acquiring::Moved] {
        assert_eq!(
            table.resolve(DEFAULT_EFFECT, false, pole, AT, Some(TO), Amount::None),
            Err(Refusal::NotAcquired {
                effect: DEFAULT_EFFECT.into()
            })
        );
        // Ownership is checked before the table, so an unowned unknown effect
        // still refuses as unacquired rather than leaking the table's shape.
        assert_eq!(
            table.resolve("mesocosm:absent", true, pole, AT, None, Amount::None),
            Err(Refusal::NoRule {
                effect: "mesocosm:absent".into()
            })
        );
    }
}

#[test]
fn each_pole_returns_its_authored_form_stroke_colour_cost_and_citation() {
    let table = EffectPackTable::default_pack();
    let expected = [
        (
            Acquiring::Carved,
            MarkForm::SurfaceInscription,
            Glyph::Slashes,
            [255u8, 115, 51, 255],
            Amount::Voxels(0),
            BASE_LIFETIME_TICKS,
        ),
        (
            Acquiring::Fed,
            MarkForm::SustainedEmission,
            Glyph::Quotes,
            [255, 191, 77, 255],
            Amount::MealMass(0),
            BASE_LIFETIME_TICKS,
        ),
        (
            Acquiring::Moved,
            MarkForm::PathTrail,
            Glyph::Slashes,
            [115, 242, 204, 255],
            Amount::None,
            BASE_LIFETIME_TICKS,
        ),
    ];
    for (pole, form, stroke, color, amount, lifetime) in expected {
        let mark = table
            .resolve(DEFAULT_EFFECT, true, pole, AT, Some(TO), amount)
            .unwrap();
        assert_eq!((mark.form, mark.stroke, mark.color), (form, stroke, color));
        assert_eq!(mark.at, AT);
        assert_eq!(mark.scale_permille, BASE_SCALE_PERMILLE);
        assert_eq!(mark.lifetime_ticks, lifetime);
        // Only a trail has an arrival; the others drop the caller's `to`.
        assert_eq!(
            mark.to,
            (form == MarkForm::PathTrail).then_some(TO),
            "{form:?}"
        );
        let rule = table.rule(DEFAULT_EFFECT, pole).unwrap();
        assert!(
            mark.explanation.contains(&rule.citation),
            "the explanation template reads the rule's citation: {}",
            mark.explanation
        );
    }
    // The three poles are visibly different, which is the point of the axis.
    let forms: Vec<_> = table.rules().iter().map(|r| r.form).collect();
    assert_eq!(
        forms,
        [
            MarkForm::SurfaceInscription,
            MarkForm::SustainedEmission,
            MarkForm::PathTrail
        ]
    );
    assert!(matches!(
        table.rule(DEFAULT_EFFECT, Acquiring::Carved).unwrap().cost,
        CostShape::PerUse { .. }
    ));
    assert!(matches!(
        table.rule(DEFAULT_EFFECT, Acquiring::Fed).unwrap().cost,
        CostShape::WhileSustained { .. }
    ));
    assert_eq!(
        table.rule(DEFAULT_EFFECT, Acquiring::Moved).unwrap().cost,
        CostShape::Free
    );
}

#[test]
fn the_amount_curve_is_monotone_saturating_and_exact_at_its_endpoints() {
    let top = BASE_SCALE_PERMILLE + SCALE_SPAN_PERMILLE;
    assert_eq!(Amount::None.scale_permille(), BASE_SCALE_PERMILLE);
    for (zero, cap, over, half) in [
        (
            Amount::Voxels(0),
            Amount::Voxels(VOXEL_CAP as u32),
            Amount::Voxels(u32::MAX),
            Amount::Voxels((VOXEL_CAP / 2) as u32),
        ),
        (
            Amount::MealMass(0),
            Amount::MealMass(MASS_CAP_MG),
            Amount::MealMass(u64::MAX),
            Amount::MealMass(MASS_CAP_MG / 2),
        ),
        (
            Amount::UptakeMass(0),
            Amount::UptakeMass(MASS_CAP_MG),
            Amount::UptakeMass(u64::MAX),
            Amount::UptakeMass(MASS_CAP_MG / 2),
        ),
    ] {
        assert_eq!(zero.scale_permille(), BASE_SCALE_PERMILLE);
        assert_eq!(cap.scale_permille(), top);
        assert_eq!(over.scale_permille(), top, "saturating, never wrapping");
        assert_eq!(half.scale_permille(), BASE_SCALE_PERMILLE + 500);
    }
    let mut previous = 0;
    for mg in 0..=(MASS_CAP_MG + 64) {
        let scale = Amount::UptakeMass(mg).scale_permille();
        assert!(scale >= previous, "monotone at {mg}");
        assert!((BASE_SCALE_PERMILLE..=top).contains(&scale));
        previous = scale;
    }
    // Uptake binds to the feeding glyph, so its milligrams drive that pole's
    // size and its lifetime; a one-shot pole keeps the fixed base life.
    let table = EffectPackTable::default_pack();
    let small = table
        .resolve(
            DEFAULT_EFFECT,
            true,
            Acquiring::Fed,
            AT,
            None,
            Amount::UptakeMass(0),
        )
        .unwrap();
    let large = table
        .resolve(
            DEFAULT_EFFECT,
            true,
            Acquiring::Fed,
            AT,
            None,
            Amount::UptakeMass(MASS_CAP_MG),
        )
        .unwrap();
    assert!(large.scale_permille > small.scale_permille);
    assert_eq!(small.lifetime_ticks, BASE_LIFETIME_TICKS);
    assert_eq!(
        large.lifetime_ticks,
        BASE_LIFETIME_TICKS + SUSTAINED_LIFETIME_SPAN
    );
    let carved = table
        .resolve(
            DEFAULT_EFFECT,
            true,
            Acquiring::Carved,
            AT,
            None,
            Amount::Voxels(u32::MAX),
        )
        .unwrap();
    assert_eq!(carved.lifetime_ticks, BASE_LIFETIME_TICKS);
    assert_eq!(carved.scale_permille, top);
}

#[test]
fn validation_refuses_two_rules_on_one_effect_and_pole_pair() {
    let table = EffectPackTable::default_pack();
    assert!(table.validate().is_ok());
    let mut rules = table.rules().to_vec();
    rules.push(rules[0].clone());
    let why = EffectPackTable::new(rules.clone()).unwrap_err();
    assert!(
        why.contains("duplicate pack rule") && why.contains("Carved"),
        "{why}"
    );
    // A different effect on the same pole is a different pair and is admitted.
    rules.last_mut().unwrap().effect = "mesocosm:other-reference".into();
    assert!(EffectPackTable::new(rules).is_ok());
    assert!(EffectPackTable::new(Vec::new()).is_err());
    let mut blank = table.rules().to_vec();
    blank[0].citation.clear();
    assert!(EffectPackTable::new(blank).is_err());
}

#[test]
fn the_default_declarations_cover_the_table_and_a_canon_built_on_it() {
    let pack = EffectPack::new(EffectPackTable::default_declarations()).unwrap();
    let table = EffectPackTable::default_pack();
    assert!(table.validate_against(&pack).is_ok());
    assert_eq!(
        pack.declaration(DEFAULT_EFFECT)
            .unwrap()
            .cost
            .unit()
            .unwrap()
            .id,
        VOXEL_UNIT
    );
    // The demo canon carries a second, undeclared base, so `covers` refuses
    // it: coverage is a claim about the canon, not about the table.
    assert!(pack.covers(&canon(DEFAULT_EFFECT)).is_err());
    let mut single = canon(DEFAULT_EFFECT).spec().clone();
    single.glyphs.truncate(1);
    assert!(
        pack.covers(&Canon::new(single).unwrap()).is_ok(),
        "every base effect declared"
    );
    let mut stray = table.rules().to_vec();
    stray[0].effect = "mesocosm:undeclared".into();
    assert!(
        EffectPackTable::new(stray)
            .unwrap()
            .validate_against(&pack)
            .is_err()
    );
    assert_eq!(EffectPackTable::default(), table);
}

#[test]
fn the_lookup_keys_on_the_current_canons_effect_not_the_one_at_acquisition() {
    let table = EffectPackTable::default_pack();
    let founding = canon(DEFAULT_EFFECT);
    let owned = "mesocosm:earth";
    let before = table
        .resolve_for_glyph(
            &founding,
            owned,
            true,
            Acquiring::Carved,
            AT,
            None,
            Amount::Voxels(8),
        )
        .unwrap();
    assert_eq!(before.form, MarkForm::SurfaceInscription);
    // A variant resolves through its base's current effect, same as the base.
    assert_eq!(
        table
            .resolve_for_glyph(
                &founding,
                "mesocosm:earth-acute",
                true,
                Acquiring::Carved,
                AT,
                None,
                Amount::Voxels(8),
            )
            .unwrap(),
        before
    );
    // Publish a later revision in which this base's effect moved away. The
    // glyph is still owned; the live effect is not the table's, so the mark
    // is withheld rather than drawn from the acquisition-time effect.
    let moved = (0..64)
        .find_map(|seed| {
            let candidate = founding.shuffled(seed, 2).ok()?;
            (candidate.effect(owned) != Some(DEFAULT_EFFECT)).then_some(candidate)
        })
        .expect("a seeded reshuffle moves this base's effect");
    assert_eq!(moved.effect(owned), Some("mesocosm:unclaimed-reference"));
    assert_eq!(moved.spec().revision, 2);
    assert_eq!(
        table.resolve_for_glyph(
            &moved,
            owned,
            true,
            Acquiring::Carved,
            AT,
            None,
            Amount::Voxels(8)
        ),
        Err(Refusal::NoRule {
            effect: "mesocosm:unclaimed-reference".into()
        })
    );
    // The other base picked up the table's effect and now draws it.
    assert_eq!(moved.effect("mesocosm:other"), Some(DEFAULT_EFFECT));
    assert_eq!(
        table
            .resolve_for_glyph(
                &moved,
                "mesocosm:other",
                true,
                Acquiring::Carved,
                AT,
                None,
                Amount::Voxels(8),
            )
            .unwrap(),
        before
    );
    assert_eq!(
        table.resolve_for_glyph(
            &founding,
            "mesocosm:absent",
            true,
            Acquiring::Carved,
            AT,
            None,
            Amount::None
        ),
        Err(Refusal::NoRule {
            effect: "mesocosm:absent".into()
        })
    );
}

#[test]
fn a_mark_request_round_trips_json_and_repeated_resolution_is_a_pure_function() {
    let table = EffectPackTable::default_pack();
    let mark = table
        .resolve(
            DEFAULT_EFFECT,
            true,
            Acquiring::Fed,
            AT,
            Some(TO),
            Amount::UptakeMass(1234),
        )
        .unwrap();
    for _ in 0..8 {
        assert_eq!(
            table
                .resolve(
                    DEFAULT_EFFECT,
                    true,
                    Acquiring::Fed,
                    AT,
                    Some(TO),
                    Amount::UptakeMass(1234)
                )
                .unwrap(),
            mark
        );
    }
    let json = serde_json::to_string(&mark).unwrap();
    assert_eq!(serde_json::from_str::<MarkRequest>(&json).unwrap(), mark);
    let table_json = serde_json::to_string(&table).unwrap();
    assert_eq!(
        serde_json::from_str::<EffectPackTable>(&table_json).unwrap(),
        table
    );
}

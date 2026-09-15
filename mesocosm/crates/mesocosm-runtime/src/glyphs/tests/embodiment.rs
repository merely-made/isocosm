// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Embodiment read through the trial's reading: what the bound body bears,
//! which is a fact about the body and not about the journey.

use super::*;

/// The five native definitions, each expressing one glyph, so whatever the
/// seeded founder is made of has something to bear.
fn expression_table() -> wing_glyphs::ExpressionTable {
    wing_glyphs::ExpressionTable::new(wing_glyphs::ExpressionSpec {
        version: 1,
        id: "test:expression".into(),
        canon_revision: 1,
        entries: ["contract", "fix", "intake", "secrete", "sense"]
            .into_iter()
            .map(|name| wing_glyphs::GlyphExpression {
                glyph: format!("test:{name}-glyph"),
                traits: vec![format!("mesocosm:{name}")],
            })
            .collect(),
        limits: Default::default(),
    })
    .unwrap()
}

/// Embodiment is a fact about the body, experience a fact about the journey.
/// The bound body bears its seeded glyphs from the first frame, having done
/// nothing, and reading that changes neither the world nor the journey.
#[test]
fn the_bound_body_embodies_from_tick_zero_while_its_journey_is_still_empty() {
    let (world, at) = carve_fixture();
    let table = expression_table();
    let mut trial = Trial::new(&world).unwrap();
    trial
        .enable_glyphs(rules(world.controlled_id().unwrap(), AcceptedKind::Carved))
        .unwrap();

    let reading = trial.glyphs().unwrap();
    let before = reading.embodied_glyphs(trial.world(), &table);
    assert!(
        !before.is_empty(),
        "founding anatomy bears what its shapes seeded"
    );
    assert!(
        reading.journey().acquisitions().is_empty() && !reading.owns_effect("test:reshape"),
        "and has experienced nothing at all"
    );

    // Reading is not acting: the world hash and the journey are untouched.
    let hash = trial.state_hash();
    let snapshot = serde_json::to_string(&reading.journey().snapshot()).unwrap();
    for _ in 0..8 {
        assert_eq!(reading.embodied_glyphs(trial.world(), &table), before);
    }
    assert_eq!(trial.state_hash(), hash);
    assert_eq!(
        serde_json::to_string(&reading.journey().snapshot()).unwrap(),
        snapshot
    );

    // A grant is the other axis, and moves this one not at all.
    assert!(trial.carve(at, 1));
    let reading = trial.glyphs().unwrap();
    assert!(reading.owns_effect("test:reshape"));
    assert_eq!(
        reading.embodied_glyphs(trial.world(), &table),
        before,
        "carving earned a glyph; it did not grow a trait"
    );

    // A table naming a trait this body does not express is simply silent.
    let absent = wing_glyphs::ExpressionTable::new(wing_glyphs::ExpressionSpec {
        version: 1,
        id: "test:expression".into(),
        canon_revision: 1,
        entries: vec![wing_glyphs::GlyphExpression {
            glyph: "test:unborne".into(),
            traits: vec!["reef:filter".into()],
        }],
        limits: Default::default(),
    })
    .unwrap();
    assert!(reading.embodied_glyphs(trial.world(), &absent).is_empty());

    // And a body that is no longer alive bears nothing, while its journey is
    // untouched: that asymmetry is the whole point of keeping the two apart.
    let mut dead = trial.world().clone();
    for organism in dead.organisms.iter_mut() {
        organism.stage = mesocosm_core::Stage::Carrion;
    }
    assert!(reading.embodied_glyphs(&dead, &table).is_empty());
    assert!(reading.owns_effect("test:reshape"));
}

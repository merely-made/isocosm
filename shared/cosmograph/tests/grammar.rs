// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The growth grammar (lineage-rounds LR6 step 3; cases 1202, 1206): a line
//! without one is as it was, and the plant grown from its tips unfolds into
//! a self-similar tree whose body develops clear of itself.

use cosmograph::{
    bodied,
    development::{Soma, develop},
    geometry::Body,
    rules::{Development, Facing, Policy, Recipe, Rules, Tagma, regrown, unfold},
    schema::*,
};
use std::collections::BTreeSet;

fn rules() -> Rules {
    let mut r = cosmograph::probe::BodyFounding::default().generate().unwrap().genesis.rules;
    r.kinds = bodied::default_kinds();
    r
}

fn run(segments: u8) -> Tagma {
    Tagma {
        chain: vec![],
        segments,
        segment: "kind:block".into(),
        bears: None,
        per_segment: 0,
        parent: None,
        anchor: Default::default(),
        facing: Facing::Back,
        socket: Facing::Right,
        variance: None,
    }
}

fn plain() -> Development {
    let recipe = Recipe { tagmata: vec![run(3)], variance: 0, absence: [0, 1], riff: [0, 1], vary: [0, 1] };
    Development {
        lexicon: recipe.kinds(),
        recipe,
        policy: Policy::default(),
        domain: 0,
        clutch: 1,
        anamorphic: false,
        tracts: vec![],
        systems: Default::default(),
        comfort: 0,
        grammar: None,
    }
}

fn grown(rules: &Rules, d: &Development) -> Body {
    let soma = Soma {
        segments: d.recipe.tagmata.iter().map(|t| t.segments).collect(),
        absent: vec![],
        seed: 0,
    };
    develop(rules, d, &soma).unwrap()
}

#[test]
fn without_a_grammar_a_development_serializes_and_reads_as_it_did() {
    let d = plain();
    let json = serde_json::to_string(&d).unwrap();
    assert!(!json.contains("grammar"), "{json}");
    assert_eq!(serde_json::from_str::<Development>(&json).unwrap(), d);
    // An older document, with no such key, reads as no grammar.
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(value.get("grammar").is_none());
    regrown(&d).unwrap_err();
}

#[test]
fn a_grammar_round_trips() {
    let d = bodied::tips(&rules()).unwrap();
    let json = serde_json::to_string(&d).unwrap();
    assert!(json.contains("grammar"));
    assert_eq!(serde_json::from_str::<Development>(&json).unwrap(), d);
}

fn facing(d: &Development, f: Facing) -> Vec<&Tagma> {
    d.recipe.tagmata.iter().filter(|t| t.facing == f).collect()
}

#[test]
fn the_tips_plant_unfolds_into_a_self_similar_tree() {
    let d = bodied::tips(&rules()).unwrap();
    let grammar = d.grammar.as_ref().unwrap();
    assert_eq!(unfold(grammar).unwrap(), d.recipe.tagmata);
    assert_eq!(d.lexicon, BTreeSet::from(["kind:block".to_string(), "kind:broad".to_string()]));
    // A stem of eight in four pieces; one run per piece above it, each of
    // six in three pieces; one per piece of those, turned right, of four
    // (less what the cap took off the longest).
    let stem = facing(&d, Facing::Back);
    let columns = facing(&d, Facing::Above);
    let twigs = facing(&d, Facing::Right);
    assert_eq!((stem.len(), columns.len(), twigs.len()), (4, 12, 12));
    assert_eq!(stem.iter().map(|t| t.segments).sum::<u8>(), 8);
    // Each column is a head and two more pieces: four runs of six.
    assert_eq!(columns.iter().map(|t| u32::from(t.segments)).sum::<u32>(), 24);
    assert_eq!(columns.iter().filter(|t| t.parent.is_some_and(|p| p < 4)).count(), 4);
    assert!(twigs.iter().all(|t| t.segments < 6 && t.segments >= 3));
    let total: u32 = d.recipe.tagmata.iter().map(|t| u32::from(t.segments)).sum();
    assert_eq!((d.recipe.tagmata.len(), total), (28, 72));
}

#[test]
fn the_tips_plant_develops_clear_of_itself_and_the_same_each_time() {
    let r = rules();
    let d = bodied::tips(&r).unwrap();
    let body = grown(&r, &d);
    let doc = &body.doc;
    let mut situs = BTreeSet::new();
    for p in &doc.parts {
        assert!(situs.insert(p.situs), "a situs twice: {:?}", p.situs);
    }
    assert_eq!(doc.parts.len(), situs.len());
    let mut crossing = vec![];
    for p in &doc.parts {
        for q in doc.parts.iter().filter(|q| q.id > p.id) {
            let (a, b) = (doc.world_pivot(p.id).unwrap(), doc.world_pivot(q.id).unwrap());
            if (0..3).all(|i| (a[i] - b[i]).abs() < p.half_extent[i].abs() + q.half_extent[i].abs()) {
                crossing.push((p.id, q.id));
            }
        }
    }
    assert!(crossing.is_empty(), "{crossing:?}");
    assert_eq!(body, grown(&r, &bodied::tips(&r).unwrap()));
    assert_eq!(body.parts.len(), 72 + 2 * 8 + 24 + 40, "segments and the plates they bear");
}

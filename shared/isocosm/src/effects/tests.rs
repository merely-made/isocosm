// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::embodiment::{bearing_by, embodied_by, traits_by};
use super::*;
use crate::geometry::{Body, Frame};
use crate::schema::{Entity, Part, PartId};
use std::collections::BTreeSet;
use wing_glyphs::{
    Canon, CanonSpec, ExpressionSpec, ExpressionTable, GlyphDefinition, GlyphExpression,
    Provenance, ProvenanceKind, SCHEMA_VERSION,
};

const SUN: &str = "glyph:sun";
const EARTH: &str = "glyph:earth";
const STING: &str = "glyph:sting";

fn table() -> ExpressionTable {
    let entry = |glyph: &str, traits: &[&str]| GlyphExpression {
        glyph: glyph.into(),
        traits: traits.iter().map(|t| t.to_string()).collect(),
    };
    ExpressionTable::new(ExpressionSpec {
        version: SCHEMA_VERSION,
        id: "test:expression".into(),
        canon_revision: 1,
        entries: vec![
            entry(SUN, &["function:fix"]),
            entry(EARTH, &["function:intake", "function:store"]),
            entry(STING, &["function:secrete"]),
        ],
        limits: Default::default(),
    })
    .unwrap()
}

fn part(parent: Option<u32>, functions: &[&str]) -> (Frame, Part) {
    let frame = Frame {
        parent: parent.map(PartId),
        half_extent: [1, 1, 1],
        ..Default::default()
    };
    let functions = functions.iter().map(|f| format!("function:{f}")).collect();
    let part = Part {
        functions,
        ..Default::default()
    };
    (frame, part)
}

/// A probe entity given a root that takes in and a frond that fixes.
fn producer() -> Entity {
    let mut e = crate::probe::BodyFounding::default()
        .generate()
        .expect("a probe world")
        .genesis
        .population
        .groups
        .into_values()
        .next()
        .expect("an entity")
        .entity;
    let body = Body::of([part(None, &["intake"]), part(Some(0), &["fix"])]);
    e.embody(body.expect("a body"));
    e
}

#[test]
fn readings_are_a_disjunction_over_living_parts() {
    let parts = || {
        vec![
            (PartId(0), vec!["function:store".to_string()]),
            (PartId(1), vec!["function:fix".to_string()]),
            (PartId(2), vec![]),
        ]
    };
    let found = embodied_by(parts(), &table());
    assert_eq!(found, BTreeSet::from([EARTH.into(), SUN.into()]));
    assert_eq!(bearing_by(parts(), &table(), SUN), [PartId(1)]);
    assert_eq!(bearing_by(parts(), &table(), EARTH), [PartId(0)]);
    assert!(bearing_by(parts(), &table(), STING).is_empty());
    assert_eq!(traits_by(parts()).len(), 2);
}

#[test]
fn a_native_body_embodies_its_living_parts_functions() {
    let mut e = producer();
    let table = table();
    assert_eq!(
        embodied(&e, &table),
        BTreeSet::from([EARTH.into(), SUN.into()])
    );
    assert_eq!(bearing_parts(&e, &table, SUN), [PartId(1)]);
    assert_eq!(functions::live_parts(7, &e).len(), 2);
    // A part the body no longer holds bears nothing.
    e.parts.remove(&PartId(1));
    assert_eq!(embodied(&e, &table), BTreeSet::from([EARTH.into()]));
    assert!(expressed_traits(&e).contains("function:intake"));
    assert_eq!(functions::live_parts(7, &e).len(), 1);
}

#[test]
fn a_journal_follows_only_a_newer_revision() {
    let glyph = |id: &str, effect: &str| GlyphDefinition {
        id: id.into(),
        display: "#".into(),
        effect: effect.into(),
    };
    let canon = Canon::new(CanonSpec {
        version: SCHEMA_VERSION,
        id: "test:canon".into(),
        revision: 1,
        glyphs: vec![glyph("test:a", "test:one"), glyph("test:b", "test:two")],
        variants: vec![],
        limits: Default::default(),
    })
    .unwrap();
    let mut journal = Journal::new(canon, "test:subject".into(), vec![0, 1]).unwrap();
    let provenance = Provenance {
        kind: ProvenanceKind::Custom("moved".into()),
        evidence: "test/0".into(),
        context: None,
    };
    let (first, revision) = journal.grant("test:a", provenance.clone(), 3);
    assert_eq!((first, revision), (GlyphGrantOutcome::Acquired, 1));
    let (again, _) = journal.grant("test:a", provenance, 4);
    assert_eq!(again, GlyphGrantOutcome::Duplicate);
    assert!(!journal.revise(1, 9));
    assert!(journal.revise(2, 9));
    assert_eq!(journal.live_canon().spec().revision, 2);
    assert_eq!(journal.canon().spec().revision, 1);
    assert_eq!(journal.founding_effect("test:a"), Some("test:one"));
}

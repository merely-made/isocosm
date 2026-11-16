// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The growth grammar (lineage-rounds LR6 step 3; cases 1202, 1206): a line
//! without one is as it was, and the plant grown from its tips unfolds into
//! a self-similar tree whose body develops clear of itself.

use cosmograph::{
    bodied,
    development::{Soma, develop},
    geometry::Body,
    rules::{Accretion, Development, Facing, Policy, Recipe, Rule, Rules, Symmetry, Tagma, Template, regrown, unfold},
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
        symmetry: None,
        harden: None,
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

/// A stem of two segments running up, each bearing one plate at its flank,
/// symmetry named by a rule where `symmetry` is given.
fn stem(symmetry: Option<Symmetry>, bilateral: bool) -> Development {
    let seed = Tagma {
        bears: Some("kind:broad".into()),
        per_segment: 1,
        facing: Facing::Above,
        ..run(2)
    };
    let rules = symmetry.map(|symmetry| Rule::Symmetry { tagma: 0, symmetry }).into_iter().collect();
    let mut d = plain();
    d.policy = Policy { bilateral, ..Policy::default() };
    d.lexicon = ["kind:block", "kind:broad"].map(String::from).into();
    d.grammar = Some(cosmograph::rules::Grammar { seed: vec![seed], rules });
    regrown(&d).unwrap()
}

/// Pairs of parts whose boxes cross.
fn crossings(body: &Body) -> Vec<(PartId, PartId)> {
    let doc = &body.doc;
    let mut crossing = vec![];
    for p in &doc.parts {
        for q in doc.parts.iter().filter(|q| q.id > p.id) {
            let (a, b) = (doc.world_pivot(p.id).unwrap(), doc.world_pivot(q.id).unwrap());
            if (0..3).all(|i| (a[i] - b[i]).abs() < p.half_extent[i].abs() + q.half_extent[i].abs()) {
                crossing.push((p.id, q.id));
            }
        }
    }
    crossing
}

#[test]
fn a_runs_symmetry_places_what_it_bears_once_in_pairs_or_round() {
    let r = rules();
    let parts = |d: &Development| {
        let body = grown(&r, d);
        assert!(crossings(&body).is_empty(), "{:?}", crossings(&body));
        body.parts.len()
    };
    // Two stem segments and a plate on each, doubled or not.
    assert_eq!(parts(&stem(None, false)), 2 + 2);
    assert_eq!(parts(&stem(None, true)), 2 + 4);
    // A named symmetry overrides the policy either way.
    assert_eq!(parts(&stem(Some(Symmetry::None), true)), 2 + 2);
    assert_eq!(parts(&stem(Some(Symmetry::Bilateral), false)), 2 + 4);
    // Round the vertical: the arms the lattice can show, four at most, while
    // the count the rule named is stored whole.
    assert_eq!(parts(&stem(Some(Symmetry::Radial { arms: 3 }), true)), 2 + 2 * 3);
    let five = stem(Some(Symmetry::Radial { arms: 5 }), true);
    assert_eq!(parts(&five), 2 + 2 * 4);
    assert_eq!(five.recipe.tagmata[0].symmetry, Some(Symmetry::Radial { arms: 5 }));
}

#[test]
fn symmetry_leaves_a_recipe_that_names_none_serializing_as_before() {
    let d = stem(None, true);
    assert!(!serde_json::to_string(&d.recipe).unwrap().contains("symmetry"));
    let named = stem(Some(Symmetry::Radial { arms: 5 }), true);
    let json = serde_json::to_string(&named).unwrap();
    assert!(json.contains("Radial"));
    assert_eq!(serde_json::from_str::<Development>(&json).unwrap(), named);
}

/// A stem of four segments with its oldest two hardened to wood, a kind of
/// the block's box that holds cells of nothing, and bearing bark round it.
#[test]
fn accreted_segments_and_coverings_develop_as_parts_that_do_nothing() {
    let mut r = rules();
    let block = r.kinds["kind:block"].clone();
    for kind in ["kind:wood", "kind:bark"] {
        r.kinds.insert(kind.into(), Template { cells: Default::default(), ..block.clone() });
    }
    let mut d = plain();
    d.recipe.tagmata = vec![Tagma { facing: Facing::Above, ..run(4) }];
    let rules_ = [
        Accretion::Harden { oldest: 2, kind: "kind:wood".into() },
        Accretion::Cover { kind: "kind:bark".into(), socket: Facing::Right },
    ];
    let mut rules_: Vec<Rule> = rules_.into_iter().map(|how| Rule::Accrete { tagma: 0, how }).collect();
    rules_.push(Rule::Symmetry { tagma: 0, symmetry: Symmetry::None });
    d.grammar = Some(cosmograph::rules::Grammar { seed: d.recipe.tagmata.clone(), rules: rules_ });
    d.lexicon = ["kind:block", "kind:wood", "kind:bark"].map(String::from).into();
    let d = regrown(&d).unwrap();
    let body = grown(&r, &d);
    // Four segments and a bark each, none overlapping.
    assert_eq!(body.parts.len(), 8);
    assert!(crossings(&body).is_empty());
    let does_nothing = |p: &Part| p.cells.is_empty() && p.functions.is_empty();
    let situs = |id: PartId| body.doc.part(id).and_then(|g| g.situs).unwrap();
    for (id, part) in &body.parts {
        let [_, s, slot] = situs(*id);
        // Wood is the oldest two, bark every covering; the young stay live.
        assert_eq!(does_nothing(part), slot > 0 || s < 2, "{:?}", situs(*id));
    }
}

fn entity_of(body: &Body) -> Entity {
    let mut e = cosmograph::probe::BodyFounding::default().generate().unwrap().genesis.population.groups.values().next().unwrap().entity.as_ref().clone();
    e.embody(body.clone());
    e
}

/// A stem of four with its oldest two wood and bark on each, as above.
fn accreted(r: &mut Rules) -> Development {
    let block = r.kinds["kind:block"].clone();
    for kind in ["kind:wood", "kind:bark"] {
        r.kinds.insert(kind.into(), Template { cells: Default::default(), ..block.clone() });
    }
    let mut d = plain();
    d.recipe.tagmata = vec![Tagma { facing: Facing::Above, ..run(4) }];
    let how = [
        Accretion::Harden { oldest: 2, kind: "kind:wood".into() },
        Accretion::Cover { kind: "kind:bark".into(), socket: Facing::Right },
    ];
    let rules = how.into_iter().map(|how| Rule::Accrete { tagma: 0, how }).collect();
    d.grammar = Some(cosmograph::rules::Grammar { seed: d.recipe.tagmata.clone(), rules });
    d.lexicon = ["kind:block", "kind:wood", "kind:bark"].map(String::from).into();
    regrown(&d).unwrap()
}

#[test]
fn dead_matter_is_left_out_of_the_size_a_sum_over_parts_reads() {
    let mut r = rules();
    let d = accreted(&mut r);
    let e = entity_of(&grown(&r, &d));
    // Four segments and a bark each: the two live segments are the size.
    assert_eq!(e.living().count(), 4 + 4 * 2);
    let sized = e.sized();
    assert_eq!(sized.len(), 2);
    assert!(sized.iter().all(|(_, p)| !p.cells.is_empty()));
    // The same body with nothing accreted is all sized, as it was.
    let plain_body = entity_of(&grown(&r, &stem(None, true)));
    assert_eq!(plain_body.sized().len(), plain_body.living().count());
}

#[test]
fn arms_copied_round_a_vertical_stem_grow_clear_of_one_another() {
    let r = rules();
    let spine = Tagma { facing: Facing::Above, ..run(3) };
    let arm = Tagma { parent: Some(0), facing: Facing::Right, ..run(2) };
    let mut d = plain();
    d.lexicon = ["kind:block"].map(String::from).into();
    d.grammar = Some(cosmograph::rules::Grammar { seed: vec![spine, arm], rules: vec![Rule::Radiate { tagma: 1, arms: 4 }] });
    let d = regrown(&d).unwrap();
    assert_eq!(d.recipe.tagmata.len(), 5);
    let body = grown(&r, &d);
    assert_eq!(body.parts.len(), 3 + 4 * 2);
    assert!(crossings(&body).is_empty(), "{:?}", crossings(&body));
}

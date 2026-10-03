// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Checkpoint 8, step 8b: a body developed from its recipe (rulings 478,
//! 510, 511 and 531). A child's soma is drawn by its seed within its
//! recipe's variance and absence odds, never lacking a kind that feeds, and
//! develops into parts set flush, a borne kind mirrored on a bilateral
//! plan; the same seed develops the same body.

use isocosm::{
    anatomy,
    development::{develop, soma},
    rules::{Development, Facing, Policy, Recipe, Rules, Tagma, Template},
    schema::*,
};
use std::collections::{BTreeMap, BTreeSet};

fn template(half_extent: [i32; 3], cells: &[(&str, u32)]) -> Template {
    Template {
        half_extent,
        cells: cells
            .iter()
            .map(|(f, n)| (format!("function:{f}"), *n))
            .collect(),
        shape: String::new(),
    }
}

fn rules() -> Rules {
    let mut r = isocosm::probe::BodyFounding::default()
        .generate()
        .unwrap()
        .genesis
        .rules;
    r.kinds = BTreeMap::from([
        (
            "kind:lump".into(),
            template([2, 2, 2], &[("intake", 6), ("reproduce", 1)]),
        ),
        ("kind:limb".into(), template([3, 1, 1], &[("contract", 2)])),
        ("kind:mouth".into(), template([2, 1, 1], &[("intake", 2)])),
    ]);
    r
}

fn tagma(segments: u8, bears: Option<&str>, per_segment: u8) -> Tagma {
    Tagma {
        segments,
        segment: "kind:lump".into(),
        bears: bears.map(Into::into),
        per_segment,
        parent: None,
        anchor: Default::default(),
        facing: Facing::Back,
        socket: Facing::Right,
        variance: None,
    }
}

fn development(tagmata: Vec<Tagma>, variance: u8, absence: [u32; 2]) -> Development {
    let recipe = Recipe {
        tagmata,
        variance,
        absence,
    };
    Development {
        lexicon: recipe.kinds(),
        recipe,
        policy: Policy::default(),
        domain: 0,
        clutch: 1,
    }
}

/// Each part's pivot in the root's frame.
fn placed(parts: &BTreeMap<Id, Part>) -> BTreeMap<Id, [i32; 3]> {
    let mut at = BTreeMap::new();
    for (id, p) in parts {
        let base = p.parent.map_or([0; 3], |q| at[&q]);
        at.insert(*id, [0, 1, 2].map(|i| base[i] + p.offset[i]));
    }
    at
}

/// Pairs of parts whose boxes overlap, touching not counted (isometer's
/// test).
fn overlaps(parts: &BTreeMap<Id, Part>) -> Vec<(Id, Id)> {
    let at = placed(parts);
    let mut out = vec![];
    for (a, p) in parts {
        for (b, q) in parts.range(a + 1..) {
            let inside = (0..3).all(|i| {
                (at[a][i] - at[b][i]).abs() < p.half_extent[i].abs() + q.half_extent[i].abs()
            });
            if inside {
                out.push((*a, *b));
            }
        }
    }
    out
}

#[test]
fn the_same_seed_develops_the_same_body_and_variance_makes_siblings_differ() {
    let r = rules();
    let d = development(vec![tagma(4, Some("kind:limb"), 1)], 1, [0, 1]);
    let body = |seed| develop(&r, &d, &soma(&r, &d.recipe, seed)).unwrap();
    for seed in 0..20 {
        assert_eq!(body(seed), body(seed), "seed {seed}");
    }
    let differ = (0..20).map(body).collect::<BTreeSet<_>>().len();
    assert!(differ > 1, "siblings are clones");
    // The control: without variance or absence every seed develops alike.
    let fixed = development(vec![tagma(4, Some("kind:limb"), 1)], 0, [0, 1]);
    let alike = (0..20)
        .map(|s| develop(&r, &fixed, &soma(&r, &fixed.recipe, s)).unwrap())
        .collect::<BTreeSet<_>>();
    assert_eq!(alike.len(), 1);
}

#[test]
fn drift_stays_within_the_variance_and_covers_it() {
    let r = rules();
    let d = development(vec![tagma(4, None, 0), tagma(1, None, 0)], 1, [0, 1]);
    let mut seen = [BTreeSet::new(), BTreeSet::new()];
    for seed in 0..300 {
        let s = soma(&r, &d.recipe, seed);
        for (i, n) in s.segments.iter().enumerate() {
            seen[i].insert(*n);
        }
    }
    assert_eq!(seen[0], BTreeSet::from([3, 4, 5]));
    // A single segment never drifts to none.
    assert_eq!(seen[1], BTreeSet::from([1, 2]));
}

#[test]
fn absence_follows_the_odds_and_never_takes_a_kind_that_feeds() {
    let r = rules();
    let mut mouth = tagma(1, Some("kind:mouth"), 1);
    mouth.socket = Facing::Below;
    let tagmata = vec![tagma(3, Some("kind:limb"), 1), mouth];
    let certain = development(tagmata.clone(), 0, [1, 1]);
    let never = development(tagmata.clone(), 0, [0, 1]);
    let some = development(tagmata, 0, [1, 4]);
    let mut lacking = 0;
    for seed in 0..200 {
        let s = soma(&r, &certain.recipe, seed);
        // Every limb tagma lacks one segment's limbs; the mouth never.
        assert_eq!(s.absent.len(), 1, "seed {seed}");
        assert_eq!(s.absent[0].0, 0);
        assert!(s.absent[0].1 < 3);
        assert!(soma(&r, &never.recipe, seed).absent.is_empty());
        lacking += soma(&r, &some.recipe, seed).absent.len();
    }
    // About a quarter of 200 draws at odds of 1 in 4.
    assert!((30..=70).contains(&lacking), "{lacking}");
    // An absent segment's limbs do not develop; the segment does.
    let s = soma(&r, &certain.recipe, 0);
    let parts = develop(&r, &certain, &s).unwrap();
    let limbs = parts
        .values()
        .filter(|p| p.functions.contains("function:contract"));
    assert_eq!(limbs.count(), 4, "two of three segments' pairs");
    assert_eq!(parts.len(), 4 + 4 + 1, "four segments, four limbs, a mouth");
}

#[test]
fn parts_sit_flush_their_borne_kinds_mirrored() {
    let r = rules();
    let d = development(vec![tagma(2, Some("kind:limb"), 1)], 0, [0, 1]);
    let parts = develop(&r, &d, &soma(&r, &d.recipe, 7)).unwrap();
    let offsets: Vec<(Option<Id>, [i32; 3])> =
        parts.values().map(|p| (p.parent, p.offset)).collect();
    assert_eq!(
        offsets,
        [
            (None, [0, 0, 0]),
            // The limbs: flush right and left of the lump, 2 + 3 out.
            (Some(0), [5, 0, 0]),
            (Some(0), [-5, 0, 0]),
            // The next segment flush behind, 2 + 2 back, and its limbs.
            (Some(0), [0, 0, 4]),
            (Some(3), [5, 0, 0]),
            (Some(3), [-5, 0, 0]),
        ]
    );
    assert!(overlaps(&parts).is_empty(), "{:?}", overlaps(&parts));
    // Each part is its kind's template, read as its box reads.
    let names: Vec<_> = (0..6)
        .map(|id| anatomy::name(&tagged(&parts), id).unwrap())
        .collect();
    assert_eq!(names[0], "part-shape:lump");
    assert_eq!(names[1], "part-shape:rod");
    assert_eq!(parts[&0].cells["function:intake"], 6);
    // The control: borne behind, a limb runs into the next segment.
    let mut back = d.clone();
    back.recipe.tagmata[0].socket = Facing::Back;
    let collide = develop(&r, &back, &soma(&r, &back.recipe, 7)).unwrap();
    assert!(!overlaps(&collide).is_empty());
}

fn tagged(parts: &BTreeMap<Id, Part>) -> Entity {
    let mut e = isocosm::probe::BodyFounding::default()
        .generate()
        .unwrap()
        .genesis
        .population
        .groups
        .values()
        .next()
        .unwrap()
        .entity
        .clone();
    e.parts = parts.clone();
    e
}

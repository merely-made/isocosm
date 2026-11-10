// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The growth grammar's rules (cases 1202, 1206), alone and together.

use super::*;
use crate::{rules::Rules, schema::Lineage};
use std::collections::{BTreeMap, BTreeSet};

fn run(segments: u8) -> Tagma {
    Tagma {
        chain: vec![],
        segments,
        segment: "kind:a".into(),
        bears: None,
        per_segment: 0,
        parent: None,
        anchor: Anchor::Tip,
        facing: Facing::Back,
        socket: Facing::Right,
        variance: None,
    }
}

fn grammar(seed: Vec<Tagma>, rules: Vec<Rule>) -> Grammar {
    Grammar { seed, rules }
}

fn counts(t: &[Tagma]) -> Vec<u8> {
    t.iter().map(|t| t.segments).collect()
}

/// A tagma of `segments` joined to `parent` at its tip, facing `facing`.
fn joined(segments: u8, parent: u8, facing: Facing) -> Tagma {
    Tagma { segments, parent: Some(parent), facing, ..run(0) }
}

fn branching(depth: u8) -> Rule {
    Rule::Branch { tagma: 0, every: 3, depth, taper: 2, anchor: Anchor::Tip, facing: Facing::Above }
}

#[test]
fn no_rules_unfold_to_the_seed() {
    let seed = vec![run(3), joined(2, 0, Facing::Above)];
    assert_eq!(unfold(&grammar(seed.clone(), vec![])).unwrap(), seed);
}

#[test]
fn repeat_lengthens_a_run_and_stops_at_a_byte() {
    let g = grammar(vec![run(3), run(250)], vec![Rule::Repeat { tagma: 0, by: 2 }]);
    assert_eq!(counts(&unfold(&g).unwrap()), [5, 250]);
    let g = grammar(vec![run(3), run(250)], vec![Rule::Repeat { tagma: 1, by: 10 }]);
    assert_eq!(counts(&unfold(&g).unwrap()), [3, 255]);
}

#[test]
fn a_branch_of_one_level_bears_a_tapered_run_at_each_whole_piece() {
    // Six segments cut in threes: two pieces, each bearing a run of 6 - 2.
    let t = unfold(&grammar(vec![run(6)], vec![branching(1)])).unwrap();
    assert_eq!(
        t,
        [
            run(3),
            Tagma { parent: Some(0), ..run(3) },
            Tagma { parent: Some(0), facing: Facing::Above, ..run(4) },
            Tagma { parent: Some(1), facing: Facing::Above, ..run(4) },
        ]
    );
    // Seven segments cut in threes: a short last piece, which bears none.
    let t = unfold(&grammar(vec![run(7)], vec![branching(1)])).unwrap();
    assert_eq!(counts(&t), [3, 3, 1, 5, 5]);
    assert_eq!(t[2].parent, Some(1));
    assert_eq!((t[3].parent, t[4].parent), (Some(0), Some(1)));
    // A run shorter than a piece bears nothing and is not cut.
    let t = unfold(&grammar(vec![run(2)], vec![branching(1)])).unwrap();
    assert_eq!(t, [run(2)]);
    // A run no longer than the taper bears no run of no segments.
    let rule = Rule::Branch { tagma: 0, every: 3, depth: 1, taper: 6, anchor: Anchor::Tip, facing: Facing::Above };
    assert_eq!(counts(&unfold(&grammar(vec![run(6)], vec![rule])).unwrap()), [3, 3]);
}

#[test]
fn a_branch_of_two_levels_turns_a_third_way_and_joins_at_the_anchor() {
    let t = unfold(&grammar(vec![run(6)], vec![branching(2)])).unwrap();
    assert_eq!(
        t,
        [
            run(3),
            Tagma { parent: Some(0), ..run(3) },
            Tagma { parent: Some(0), facing: Facing::Above, ..run(3) },
            Tagma { parent: Some(1), facing: Facing::Above, ..run(3) },
            // The first side run's cut-off end, and the run it bears: 4 - 2
            // segments, turned off both the spine and its bearer.
            Tagma { parent: Some(2), facing: Facing::Above, ..run(1) },
            Tagma { parent: Some(2), facing: Facing::Right, ..run(2) },
            Tagma { parent: Some(3), facing: Facing::Above, ..run(1) },
            Tagma { parent: Some(3), facing: Facing::Right, ..run(2) },
        ]
    );
    // The anchor is the joint it names; the rest is as before.
    let rule = Rule::Branch { tagma: 0, every: 3, depth: 1, taper: 2, anchor: Anchor::Base, facing: Facing::Above };
    let t = unfold(&grammar(vec![run(6)], vec![rule])).unwrap();
    assert_eq!((t[2].anchor, t[3].anchor), (Anchor::Base, Anchor::Base));
}

#[test]
fn a_bearing_run_turns_its_socket_off_the_runs_it_bears() {
    let stem = Tagma { bears: Some("kind:f".into()), per_segment: 1, ..run(6) };
    let t = unfold(&grammar(vec![stem], vec![branching(2)])).unwrap();
    // Spine and pieces keep the flank; a side run along above takes behind,
    // clear of the runs turned right from it, and those keep it.
    assert_eq!((t[0].socket, t[1].socket), (Facing::Right, Facing::Right));
    assert_eq!((t[2].socket, t[5].socket), (Facing::Back, Facing::Back));
}

#[test]
fn differentiate_splits_a_run_and_a_child_keeps_the_near_part() {
    let seed = vec![run(4), joined(2, 0, Facing::Above)];
    let rule = Rule::Differentiate { tagma: 0, at: 3, kind: "kind:b".into() };
    let t = unfold(&grammar(seed.clone(), vec![rule])).unwrap();
    assert_eq!(
        t,
        [
            run(3),
            joined(2, 0, Facing::Above),
            Tagma { segment: "kind:b".into(), parent: Some(0), ..run(1) },
        ]
    );
    for at in [0, 4, 9] {
        let rule = Rule::Differentiate { tagma: 0, at, kind: "kind:b".into() };
        assert_eq!(unfold(&grammar(seed.clone(), vec![rule])).unwrap(), seed, "at {at}");
    }
}

#[test]
fn allocate_shortens_the_longest_first_and_never_below_one() {
    let seed = vec![run(5), run(2), run(4), run(4)];
    let t = unfold(&grammar(seed.clone(), vec![Rule::Allocate { total: 10 }])).unwrap();
    assert_eq!(counts(&t), [3, 2, 3, 2]);
    let t = unfold(&grammar(seed.clone(), vec![Rule::Allocate { total: 15 }])).unwrap();
    assert_eq!(t, seed);
    let t = unfold(&grammar(seed, vec![Rule::Allocate { total: 1 }])).unwrap();
    assert_eq!(counts(&t), [1, 1, 1, 1]);
}

#[test]
fn rules_apply_in_order() {
    let split = Rule::Differentiate { tagma: 0, at: 4, kind: "kind:b".into() };
    let longer = Rule::Repeat { tagma: 0, by: 2 };
    // Lengthened to eight, then split at four: four and four.
    let t = unfold(&grammar(vec![run(6)], vec![longer.clone(), split.clone()])).unwrap();
    assert_eq!(counts(&t), [4, 4]);
    // Split at four, then the near part lengthened: six and two.
    let t = unfold(&grammar(vec![run(6)], vec![split, longer])).unwrap();
    assert_eq!(counts(&t), [6, 2]);
}

#[test]
fn rules_that_cannot_unfold_are_errors() {
    let bad = |seed: Vec<Tagma>, rule| unfold(&grammar(seed, vec![rule])).unwrap_err();
    bad(vec![run(1)], Rule::Repeat { tagma: 1, by: 1 });
    bad(vec![run(1)], Rule::Differentiate { tagma: 3, at: 0, kind: "kind:b".into() });
    bad(vec![run(1)], Rule::Branch { tagma: 2, every: 1, depth: 1, taper: 0, anchor: Anchor::Tip, facing: Facing::Above });
    let rule = |every, depth, facing| Rule::Branch { tagma: 0, every, depth, taper: 0, anchor: Anchor::Tip, facing };
    bad(vec![run(4)], rule(0, 1, Facing::Above));
    // Along its bearer, or three levels deep, a branch meets its sisters.
    bad(vec![run(4)], rule(2, 1, Facing::Front));
    bad(vec![run(4)], rule(2, 3, Facing::Above));
    // Past 255 tagmata: by seed, by splitting and by branching.
    bad(vec![run(1); 256], Rule::Allocate { total: 9 });
    bad(vec![run(2); 255], Rule::Differentiate { tagma: 0, at: 1, kind: "kind:b".into() });
    let wide = unfold(&grammar(vec![run(255)], vec![rule(1, 1, Facing::Above)]));
    assert!(wide.is_err());
    // Without a rule to push it over, 255 are fine.
    assert_eq!(unfold(&grammar(vec![run(1); 255], vec![])).unwrap().len(), 255);
}

fn tips_world() -> (Rules, Development) {
    let mut rules = crate::probe::BodyFounding::default().generate().unwrap().genesis.rules;
    rules.kinds = crate::bodied::default_kinds();
    let d = crate::bodied::tips(&rules).unwrap();
    (rules, d)
}

fn lines(d: &Development) -> BTreeMap<String, Lineage> {
    let line = Lineage {
        parent: None,
        revision: 1,
        traits: BTreeSet::new(),
        kingdom: "kingdom:flora".into(),
        development: Some(d.clone()),
        decides: None,
        policy: None,
        meals: BTreeMap::new(),
        regard: BTreeMap::new(),
        ate: BTreeSet::new(),
    };
    BTreeMap::from([("lineage:t".to_string(), line)])
}

#[test]
fn validation_reads_the_grammar() {
    let (rules, d) = tips_world();
    crate::validation::development(&rules, &lines(&d)).unwrap();
    let refused = |d: &Development, why: &str| {
        let e = crate::validation::development(&rules, &lines(d)).unwrap_err();
        assert!(e.contains(why), "{e}");
    };
    // An empty seed, a seed that is no tagma, a rule that does not unfold.
    let mut v = d.clone();
    v.grammar.as_mut().unwrap().seed.clear();
    refused(&v, "no seed");
    let mut v = d.clone();
    v.grammar.as_mut().unwrap().seed[0].segments = 0;
    refused(&v, "grammar seed");
    let mut v = d.clone();
    v.grammar.as_mut().unwrap().rules.push(Rule::Repeat { tagma: 99, by: 1 });
    refused(&v, "not there");
    // A kind the unfolding names that the lexicon lacks.
    let mut v = d.clone();
    let split = Rule::Differentiate { tagma: 0, at: 1, kind: "kind:rock".into() };
    v.grammar.as_mut().unwrap().rules.push(split);
    refused(&v, "lexicon lacks");
    // The tagmata are the truth: drifted counts are not refused.
    let mut v = d.clone();
    v.recipe.tagmata[0].segments += 1;
    crate::validation::development(&rules, &lines(&v)).unwrap();
}

#[test]
fn regrown_unfolds_afresh_and_needs_a_grammar() {
    let (_, d) = tips_world();
    assert_eq!(regrown(&d).unwrap(), d);
    let mut v = d.clone();
    v.recipe.tagmata[0].segments += 1;
    assert_eq!(regrown(&v).unwrap(), d);
    v.grammar.as_mut().unwrap().rules[0] = Rule::Repeat { tagma: 0, by: 5 };
    assert_ne!(regrown(&v).unwrap().recipe.tagmata, d.recipe.tagmata);
    v.grammar = None;
    regrown(&v).unwrap_err();
}

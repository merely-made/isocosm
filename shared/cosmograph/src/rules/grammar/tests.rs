// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The growth grammar's rules (cases 1202, 1206), alone and together.

use super::*;
use crate::{rules::{Policy, Recipe, Rules, Template}, schema::Lineage};
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
        symmetry: None,
        harden: None,
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

fn radial(arms: u8) -> Symmetry {
    Symmetry::Radial { arms }
}

#[test]
fn symmetry_sets_a_runs_placement_and_a_copy_keeps_it() {
    let rule = Rule::Symmetry { tagma: 1, symmetry: radial(5) };
    let t = unfold(&grammar(vec![run(2), run(3)], vec![rule])).unwrap();
    assert_eq!((t[0].symmetry, t[1].symmetry), (None, Some(radial(5))));
    // The arm count is stored whole, though the lattice shows four.
    assert_eq!(counts(&t), [2, 3]);
    // Later rules copy it into the runs they cut and bear, and a rule that
    // names it again replaces it.
    let branch = Rule::Branch { tagma: 0, every: 2, depth: 1, taper: 0, anchor: Anchor::Tip, facing: Facing::Above };
    let sym = Rule::Symmetry { tagma: 0, symmetry: Symmetry::None };
    let t = unfold(&grammar(vec![run(4)], vec![sym.clone(), branch, Rule::Symmetry { tagma: 0, symmetry: radial(3) }])).unwrap();
    let none = Some(Symmetry::None);
    assert_eq!(t.iter().map(|t| t.symmetry).collect::<Vec<_>>(), [Some(radial(3)), none, none, none]);
    // A rule after a split: the far part is a copy of the near part.
    let split = Rule::Differentiate { tagma: 0, at: 1, kind: "kind:b".into() };
    let t = unfold(&grammar(vec![run(3)], vec![sym, split])).unwrap();
    assert_eq!((t[0].symmetry, t[1].symmetry), (Some(Symmetry::None), Some(Symmetry::None)));
}

#[test]
fn a_radial_run_of_fewer_than_two_arms_is_refused() {
    for arms in [0, 1] {
        let rule = Rule::Symmetry { tagma: 0, symmetry: radial(arms) };
        unfold(&grammar(vec![run(1)], vec![rule])).unwrap_err();
    }
    unfold(&grammar(vec![run(1)], vec![Rule::Symmetry { tagma: 2, symmetry: Symmetry::None }])).unwrap_err();
    // The check a recipe's tagmata meet refuses it too, grammar or none.
    let (rules, mut d) = tips_world();
    d.recipe.tagmata[0].symmetry = Some(radial(1));
    d.grammar = None;
    let e = crate::validation::development(&rules, &lines(&d)).unwrap_err();
    assert!(e.contains("fewer than two arms"), "{e}");
}

#[test]
fn sockets_follow_the_runs_symmetry_or_else_the_policy() {
    let policy = |bilateral| Policy { bilateral, ..Policy::default() };
    let s = |sym, bilateral, socket, run| Symmetry::sockets(sym, &policy(bilateral), socket, run);
    use Facing::*;
    // No symmetry named: the policy decides, as it always did.
    assert_eq!(s(None, true, Right, 1), [Right, Left]);
    assert_eq!(s(None, false, Right, 1), [Right]);
    assert_eq!(s(None, true, Above, 1), [Above]);
    // A named symmetry ignores the policy: bilateral on a flat body,
    // none on a bilateral one.
    assert_eq!(s(Some(Symmetry::Bilateral), false, Left, 2), [Left, Right]);
    assert_eq!(s(Some(Symmetry::None), true, Right, 2), [Right]);
    // Radial goes round the vertical from the socket, up to four, and
    // leaves out what lies along its own run.
    assert_eq!(s(Some(radial(5)), true, Right, 1), [Right, Back, Left, Front]);
    assert_eq!(s(Some(radial(3)), true, Left, 1), [Left, Front, Right]);
    assert_eq!(s(Some(radial(4)), true, Right, 2), [Right, Left]);
    assert_eq!(s(Some(radial(4)), true, Back, 0), [Back, Front]);
    // A vertical socket is already on the axis.
    assert_eq!(s(Some(radial(4)), true, Above, 1), [Above]);
}

fn harden(tagma: u8, oldest: u8) -> Rule {
    Rule::Accrete { tagma, how: Accretion::Harden { oldest, kind: "kind:wood".into() } }
}

fn wood(oldest: u8) -> Option<Hardened> {
    Some(Hardened { oldest, kind: "kind:wood".into() })
}

#[test]
fn harden_marks_the_oldest_segments_of_a_run_and_names_its_kind() {
    let t = unfold(&grammar(vec![run(5), run(3)], vec![harden(0, 2)])).unwrap();
    assert_eq!((t[0].harden.clone(), t[1].harden.clone()), (wood(2), None));
    assert_eq!(counts(&t), [5, 3]);
    assert_eq!(t[0].segment_kind(1), "kind:wood");
    assert_eq!(t[0].segment_kind(2), "kind:a");
    // The kind joins those the recipe names, for the lexicon to hold.
    let recipe = Recipe { tagmata: t, variance: 0, absence: [0, 1], riff: [0, 1], vary: [0, 1] };
    assert!(recipe.kinds().contains("kind:wood"));
    // A run that hardens none of its segments is no rule.
    unfold(&grammar(vec![run(5)], vec![harden(0, 0)])).unwrap_err();
    unfold(&grammar(vec![run(5)], vec![harden(3, 1)])).unwrap_err();
}

#[test]
fn what_hardened_follows_a_cut_and_a_split_but_not_a_side_run() {
    // Six cut in threes, five of them old: the first piece wholly, the
    // second for two segments; the run each whole piece bears is young.
    let rule = Rule::Branch { tagma: 0, every: 3, depth: 1, taper: 2, anchor: Anchor::Tip, facing: Facing::Above };
    let t = unfold(&grammar(vec![run(6)], vec![harden(0, 5), rule])).unwrap();
    let old: Vec<_> = t.iter().map(|t| t.harden.clone()).collect();
    assert_eq!(old, [wood(5), wood(2), None, None]);
    // Split at three: the far part has the old segments the near one lacks.
    let split = Rule::Differentiate { tagma: 0, at: 3, kind: "kind:b".into() };
    let t = unfold(&grammar(vec![run(6)], vec![harden(0, 4), split.clone()])).unwrap();
    assert_eq!((t[0].harden.clone(), t[1].harden.clone()), (wood(4), wood(1)));
    let t = unfold(&grammar(vec![run(6)], vec![harden(0, 3), split])).unwrap();
    assert_eq!((t[0].harden.clone(), t[1].harden.clone()), (wood(3), None));
}

#[test]
fn a_cover_is_one_borne_kind_a_segment_at_its_socket() {
    let cover = |socket| Rule::Accrete { tagma: 0, how: Accretion::Cover { kind: "kind:bark".into(), socket } };
    let t = unfold(&grammar(vec![run(3)], vec![cover(Facing::Left)])).unwrap();
    assert_eq!((t[0].bears.as_deref(), t[0].per_segment, t[0].socket), (Some("kind:bark"), 1, Facing::Left));
    // Round the vertical, it is a ring of bark.
    let ring = Rule::Symmetry { tagma: 0, symmetry: radial(4) };
    let t = unfold(&grammar(vec![run(3)], vec![cover(Facing::Right), ring])).unwrap();
    assert_eq!(t[0].symmetry, Some(radial(4)));
    // A run that bears something has no room for it.
    let bearing = Tagma { bears: Some("kind:f".into()), per_segment: 1, ..run(3) };
    unfold(&grammar(vec![bearing], vec![cover(Facing::Right)])).unwrap_err();
}

#[test]
fn matter_laid_down_must_be_a_kind_that_does_nothing() {
    let (mut rules, mut d) = tips_world();
    let block = rules.kinds["kind:block"].clone();
    rules.kinds.insert("kind:wood".into(), Template { cells: BTreeMap::new(), ..block.clone() });
    rules.kinds.insert("kind:live".into(), block);
    d.lexicon.extend(["kind:wood".to_string(), "kind:live".to_string()]);
    let with = |kind: &str| {
        let mut v = d.clone();
        let how = Accretion::Harden { oldest: 2, kind: kind.into() };
        v.grammar.as_mut().unwrap().rules.push(Rule::Accrete { tagma: 0, how });
        v.recipe.tagmata = unfold(v.grammar.as_ref().unwrap()).unwrap();
        v
    };
    crate::validation::development(&rules, &lines(&with("kind:wood"))).unwrap();
    let e = crate::validation::development(&rules, &lines(&with("kind:live"))).unwrap_err();
    assert!(e.contains("gives cells to a function"), "{e}");
    // Held for a recipe that names it with no grammar too.
    let mut v = with("kind:live");
    v.grammar = None;
    crate::validation::development(&rules, &lines(&v)).unwrap_err();
    v.recipe.tagmata[0].harden.as_mut().unwrap().oldest = 0;
    v.recipe.tagmata[0].harden.as_mut().unwrap().kind = "kind:wood".into();
    let e = crate::validation::development(&rules, &lines(&v)).unwrap_err();
    assert!(e.contains("none of its segments"), "{e}");
}

fn switch(when: Condition, rules: Vec<Rule>) -> Rule {
    Rule::Switch { when, rules }
}

#[test]
fn a_switch_applies_its_rules_only_while_it_holds_and_where_it_stands() {
    let flower = vec![Rule::Repeat { tagma: 0, by: 3 }, harden(0, 1)];
    let seed = vec![run(2)];
    let rules = vec![Rule::Repeat { tagma: 0, by: 1 }, switch(Condition::FullReserves, flower), Rule::Allocate { total: 4 }];
    let g = grammar(seed, rules);
    // Off, as stored: the switch is not there.
    let off = unfold(&g).unwrap();
    assert_eq!((counts(&off), off[0].harden.clone()), (vec![3], None));
    assert_eq!(unfold_on(&g, &[]).unwrap(), off);
    // On: the run grows by three where the switch stands, then the cap.
    let on = unfold_on(&g, &[1]).unwrap();
    assert_eq!((counts(&on), on[0].harden.clone()), (vec![4], wood(1)));
    // A rule is held by where it stands among them, not by the switch's.
    assert_eq!(unfold_on(&g, &[0, 2]).unwrap(), off);
    // A grammar with no switch is unfolded as it was.
    let plain = grammar(vec![run(2)], vec![Rule::Repeat { tagma: 0, by: 1 }]);
    assert_eq!(unfold_on(&plain, &[0]).unwrap(), unfold(&plain).unwrap());
}

#[test]
fn a_switch_cannot_hold_a_switch_or_a_rule_that_does_not_unfold() {
    let inner = switch(Condition::FullReserves, vec![]);
    let g = grammar(vec![run(1)], vec![switch(Condition::Age { ticks: 5 }, vec![inner])]);
    unfold(&g).unwrap_err();
    unfold_on(&g, &[0]).unwrap_err();
    // Off, a switch's rules are not applied; validation unfolds each stage
    // and so finds a bad rule, or a kind the lexicon lacks, at once.
    let bad = switch(Condition::FullReserves, vec![Rule::Repeat { tagma: 4, by: 1 }]);
    let g = grammar(vec![run(1)], vec![bad]);
    unfold(&g).unwrap();
    unfold_on(&g, &[0]).unwrap_err();
    let (rules, mut d) = tips_world();
    let split = Rule::Differentiate { tagma: 0, at: 1, kind: "kind:rock".into() };
    d.grammar.as_mut().unwrap().rules.push(switch(Condition::FullReserves, vec![split]));
    let e = crate::validation::development(&rules, &lines(&d)).unwrap_err();
    assert!(e.contains("lexicon lacks"), "{e}");
    d.grammar.as_mut().unwrap().rules.pop();
    d.grammar.as_mut().unwrap().rules.push(switch(Condition::FullReserves, vec![Rule::Repeat { tagma: 99, by: 1 }]));
    let e = crate::validation::development(&rules, &lines(&d)).unwrap_err();
    assert!(e.contains("not there"), "{e}");
}

#[test]
fn a_switch_reads_the_body_and_its_place() {
    let now = Situation { age: 30, reserves_full: false, crowd: 2 };
    let holds = |c: Condition| c.holds(&now);
    assert!(holds(Condition::Age { ticks: 30 }) && !holds(Condition::Age { ticks: 31 }));
    assert!(!holds(Condition::FullReserves));
    assert!(Condition::FullReserves.holds(&Situation { reserves_full: true, ..now }));
    assert!(holds(Condition::Crowding { bodies: 2 }) && !holds(Condition::Crowding { bodies: 3 }));
    let g = grammar(vec![run(1)], vec![
        Rule::Repeat { tagma: 0, by: 1 },
        switch(Condition::Age { ticks: 10 }, vec![]),
        switch(Condition::Crowding { bodies: 9 }, vec![]),
        switch(Condition::FullReserves, vec![]),
    ]);
    assert_eq!(switches(&g).len(), 3);
    assert_eq!(switched_on(&g, &now), [1]);
    assert_eq!(switched_on(&g, &Situation { reserves_full: true, crowd: 9, ..now }), [1, 2, 3]);
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
        held: BTreeSet::new(),
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

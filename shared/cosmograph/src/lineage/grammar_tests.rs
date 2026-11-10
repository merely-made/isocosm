// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Rule edits at the review (cases 1202, 1206).

use super::tests::learned;
use super::{boundary, grammar, revise, sheet};
use crate::{Session, history::Command, lineage::{Offer, Review}, rules::*, schema::*};
use std::sync::Arc;

fn development(s: &Session, line: &str) -> Development {
    s.sim.state().lineages[line].development.clone().unwrap()
}

/// The kind `learned()` taught its line: in the lexicon, not the recipe.
fn taught(s: &Session, line: &str) -> Key {
    let d = development(s, line);
    let named = d.recipe.kinds();
    d.lexicon.iter().find(|k| !named.contains(*k)).cloned().unwrap()
}

fn ate(s: &mut Session, line: &str, kind: &str, at: Tick) {
    let l = s.sim.state.lineages.get_mut(line).unwrap();
    l.meals.insert(kind.into(), Eaten { count: 1, needed: 1, last: at });
}

fn review(s: &Session, line: &str, since: Option<Tick>) -> Review {
    Review::of_draft_using(s, line, 2, since, &mut boundary::score).unwrap()
}

fn revised(o: &Offer) -> &Development {
    match &o.commands[0] {
        Command::Revise { development, .. } => development,
        other => panic!("not a revision: {other:?}"),
    }
}

/// The line's lexicon and development, the world's copy too.
fn edit(s: &mut Session, line: &str, change: impl Fn(&mut Development)) {
    let lines = [
        s.sim.state.lineages.get_mut(line),
        Arc::make_mut(&mut s.sim.genesis).lineages.get_mut(line),
    ];
    for l in lines.into_iter().flatten() {
        change(l.development.as_mut().unwrap());
    }
}

/// A session whose line grows from a grammar: its recipe as the seed, the
/// longest run lengthened by one, the whole capped well above it; and room
/// for every variant.
fn grammared() -> (Session, Key) {
    let (mut s, line, _) = learned();
    let mut d = development(&s, &line);
    let seed = d.recipe.tagmata.clone();
    let longest = (0..seed.len()).max_by_key(|i| seed[*i].segments).unwrap() as u8;
    d.grammar = Some(Grammar {
        seed,
        rules: vec![Rule::Repeat { tagma: longest, by: 1 }, Rule::Allocate { total: 200 }],
    });
    let d = regrown(&d).unwrap();
    edit(&mut s, &line, |x| *x = d.clone());
    let g = Arc::make_mut(&mut s.sim.genesis);
    g.rules.directing.get_or_insert_with(Directing::default).variants = 64;
    (s, line)
}

fn rule_offers(r: &Review) -> Vec<&Offer> {
    r.offers.iter().filter(|o| o.name.starts_with("revision:rule-")).collect()
}

fn names(o: &[(Key, Development)]) -> Vec<&str> {
    o.iter().map(|(n, _)| n.as_str()).collect()
}

#[test]
fn a_grammar_line_is_offered_each_rule_moved_and_a_differentiation() {
    let (s, line) = grammared();
    let d = development(&s, &line);
    let kind = taught(&s, &line);
    let offers = grammar::rules(&d, 64);
    assert_eq!(
        names(&offers),
        [
            "revision:rule-0-by-up",
            "revision:rule-0-by-down",
            "revision:rule-1-total-up",
            "revision:rule-1-total-down",
            &format!("revision:rule-differentiate-{kind}"),
        ]
    );
    // Each is the grammar edited once and the tagmata unfolded from it.
    for (_, v) in &offers {
        assert_eq!(v.recipe.tagmata, unfold(v.grammar.as_ref().unwrap()).unwrap());
        assert_eq!(v.lexicon, d.lexicon);
    }
    let by = |name: &str| offers.iter().find(|(n, _)| n == name).unwrap().1.clone();
    let Rule::Repeat { by: up, .. } = by("revision:rule-0-by-up").grammar.unwrap().rules[0] else { panic!() };
    let Rule::Repeat { by: down, .. } = by("revision:rule-0-by-down").grammar.unwrap().rules[0] else { panic!() };
    assert_eq!((up, down), (2, 0));
    // The cap, kept to `most`.
    assert_eq!(names(&grammar::rules(&d, 3)), names(&offers)[..3]);
    assert!(grammar::rules(&d, 0).is_empty());
    // The differentiation splits the last run at its midpoint.
    let split = by(&format!("revision:rule-differentiate-{kind}"));
    let (was, now) = (&d.recipe.tagmata, &split.recipe.tagmata);
    let last = was.last().unwrap().segments;
    assert_eq!(now.len(), was.len() + 1);
    assert_eq!(now[was.len() - 1].segments, last / 2);
    assert_eq!((now[was.len()].segment.as_str(), now[was.len()].segments), (kind.as_str(), last - last / 2));
}

#[test]
fn every_number_is_tuned_where_it_stays_valid() {
    let mut rules = crate::probe::BodyFounding::default().generate().unwrap().genesis.rules;
    rules.kinds = crate::bodied::default_kinds();
    let mut d = crate::bodied::tips(&rules).unwrap();
    d.lexicon.insert("kind:rod".into());
    let made = grammar::rules(&d, 64);
    let named = names(&made);
    for name in [
        "revision:rule-0-by-up", "revision:rule-0-by-down",
        "revision:rule-1-every-up", "revision:rule-1-every-down",
        "revision:rule-1-depth-down", "revision:rule-1-taper-up", "revision:rule-1-taper-down",
        "revision:rule-2-total-up", "revision:rule-2-total-down",
        "revision:rule-differentiate-kind:rod",
    ] {
        assert!(named.contains(&name), "{name} in {named:?}");
    }
    // Every is two: down to one is valid, and up; never to none. Depth is
    // two, and a third level would turn back on its sisters.
    assert!(!named.contains(&"revision:rule-1-depth-up"));
    let mut once = d.clone();
    let Rule::Branch { every, .. } = &mut once.grammar.as_mut().unwrap().rules[1] else { panic!() };
    *every = 1;
    assert!(!names(&grammar::rules(&once, 64)).contains(&"revision:rule-1-every-down"));
    // A line with no grammar is offered nothing.
    let mut bare = d.clone();
    bare.grammar = None;
    assert!(grammar::rules(&bare, 64).is_empty());
}

#[test]
fn committing_a_rule_offer_leaves_the_tagmata_its_grammar_unfolds_to() {
    let (mut s, line) = grammared();
    let offers = Review::of(&s, &line, 2).unwrap().offers;
    let at = offers.iter().position(|o| o.name == "revision:rule-0-by-up").expect("offered");
    assert!(offers[at].takeable(), "{:?}", offers[at].why_not);
    s.command(offers[at].commands[0].clone()).unwrap();
    let now = development(&s, &line);
    assert_eq!(now.recipe.tagmata, unfold(now.grammar.as_ref().unwrap()).unwrap());
    assert_eq!(&now, revised(&offers[at]));
    // The line's grammar is the one edited.
    let Rule::Repeat { by, .. } = now.grammar.unwrap().rules[0] else { panic!() };
    assert_eq!(by, 2);
}

#[test]
fn a_line_without_a_grammar_is_offered_no_rules() {
    let (s, line, _) = learned();
    let d = development(&s, &line);
    assert!(grammar::rules(&d, 64).is_empty());
    let all = revise::offered(&s, &line, &d, 64);
    assert!(all.iter().all(|(n, _)| !n.starts_with("revision:rule-")), "{:?}", names(&all));
    assert!(rule_offers(&Review::of(&s, &line, 2).unwrap()).is_empty());
    // A line with one is offered the same others, and its rules after them.
    let (g, gline) = grammared();
    let plain = revise::offered(&g, &gline, &development(&g, &gline), 64);
    let at = plain.iter().position(|(n, _)| n.starts_with("revision:rule-")).unwrap();
    assert!(plain[at..].iter().all(|(n, _)| n.starts_with("revision:rule-")));
    assert!(at > 0);
}

#[test]
fn the_boundary_weighs_rule_edits_of_unplayed_lines() {
    let (s, line) = grammared();
    let candidates = revise::revisions(&s, &line);
    assert!(candidates.iter().any(|c| c.name == "revision:rule-0-by-up"));
}

#[test]
fn a_draft_offers_a_differentiation_only_of_a_kind_eaten_this_round_and_costs_each() {
    let (mut s, line) = grammared();
    let first = taught(&s, &line);
    let second = s.sim.genesis().rules.kinds.keys()
        .find(|k| **k != first && !development(&s, &line).recipe.kinds().contains(*k)).cloned().unwrap();
    edit(&mut s, &line, |d| {
        d.lexicon.insert(second.clone());
    });
    let g = Arc::make_mut(&mut s.sim.genesis);
    g.rules.directing.as_mut().unwrap().draft = Some(Draft { share: [1_000_000, 1] });
    ate(&mut s, &line, &first, 10);
    ate(&mut s, &line, &second, 4);
    let r = review(&s, &line, Some(10));
    let adds: Vec<_> = r.offers.iter().filter(|o| o.name.starts_with("revision:rule-differentiate-")).collect();
    assert_eq!(adds.len(), 1, "{:?}", adds.iter().map(|o| &o.name).collect::<Vec<_>>());
    assert!(adds[0].name.ends_with(&first));
    // Every rule offer carries a cost, tunes unfiltered by meals.
    let rules = rule_offers(&r);
    assert!(rules.len() >= 5);
    assert!(rules.iter().all(|o| o.cost.is_some()), "{rules:?}");
    assert!(rules.iter().any(|o| o.name == "revision:rule-0-by-down"));
    let had = development(&s, &line);
    let by_hand = crate::lineage::draft::cost(&s.sim.genesis().rules, &s.sim.state().lineages[&line].kingdom,
        &had, revised(rules.iter().find(|o| o.name == "revision:rule-0-by-up").unwrap())).unwrap();
    assert_eq!(rules.iter().find(|o| o.name == "revision:rule-0-by-up").unwrap().cost, Some(by_hand));
    assert!(by_hand > 0, "a longer run adds mass");
    // Ate nothing this round: the tunes stay, the differentiation goes.
    let none = review(&s, &line, Some(11));
    assert!(none.offers.iter().all(|o| !o.name.starts_with("revision:rule-differentiate-")));
    assert!(none.offers.iter().any(|o| o.name == "revision:rule-0-by-up"));
    // The budget refuses what costs more than it.
    let g = Arc::make_mut(&mut s.sim.genesis);
    g.rules.directing.as_mut().unwrap().draft = Some(Draft { share: [0, 1] });
    let r = review(&s, &line, Some(10));
    let up = r.offers.iter().find(|o| o.name == "revision:rule-0-by-up").unwrap();
    assert!(up.why_not.as_deref().is_some_and(|w| w.contains("over the budget")), "{:?}", up.why_not);
}

#[test]
fn the_sheet_reads_the_unfolded_body_and_a_longer_run_weighs_more() {
    let (s, line) = grammared();
    let d = development(&s, &line);
    let (rules, lines) = (&s.sim.genesis().rules, &s.sim.state().lineages);
    let sheet_of = |d: &Development| sheet::of(rules, lines, &line, d).unwrap();
    let mut plain = d.clone();
    plain.grammar = None;
    assert_eq!(sheet_of(&d), sheet_of(&plain));
    let longer = grammar::rules(&d, 64).into_iter().find(|(n, _)| n == "revision:rule-0-by-up").unwrap().1;
    assert!(sheet_of(&longer).mass > sheet_of(&d).mass);
}

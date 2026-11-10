// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The player's draft (cases 1200, 1203; lineage-rounds D10 to D12).

use super::tests::learned;
use super::{boundary, draft, revise};
use crate::development::{Soma, develop};
use crate::{Session, history::Command, lineage::{Offer, Review}, rules::*, schema::*};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

fn rule(s: &mut Session, draft: Option<Draft>) {
    let g = Arc::make_mut(&mut s.sim.genesis);
    g.rules.directing.get_or_insert_with(Directing::default).draft = draft;
}

fn development(s: &Session, line: &str) -> Development {
    s.sim.state().lineages[line].development.clone().unwrap()
}

/// The kind `learned()` taught its line: in the lexicon, not the recipe.
fn taught(s: &Session, line: &str) -> Key {
    let d = development(s, line);
    let named = d.recipe.kinds();
    d.lexicon.iter().find(|k| !named.contains(*k)).cloned().unwrap()
}

/// Every kind the world has that the line's recipe lacks.
fn unnamed(s: &Session, line: &str) -> Vec<Key> {
    let named = development(s, line).recipe.kinds();
    s.sim.genesis().rules.kinds.keys().filter(|k| !named.contains(*k)).cloned().collect()
}

fn ate(s: &mut Session, line: &str, kind: &str, at: Tick) {
    let l = s.sim.state.lineages.get_mut(line).unwrap();
    l.meals.insert(kind.into(), Eaten { count: 1, needed: 1, last: at });
}

fn review(s: &Session, line: &str, since: Option<Tick>) -> Review {
    Review::of_draft_using(s, line, 2, since, &mut boundary::score).unwrap()
}

fn bearing(r: &Review) -> Vec<&Offer> {
    r.offers.iter().filter(|o| o.name.contains("-bears-")).collect()
}

/// The adult mass of the body `d` grows into, summed by hand.
fn grown(s: &Session, d: &Development) -> u128 {
    let rules = &s.sim.genesis().rules;
    let soma = Soma {
        segments: d.recipe.tagmata.iter().map(|t| t.segments).collect(),
        absent: vec![],
        seed: 0,
    };
    let body = develop(rules, d, &soma).unwrap();
    body.doc.parts.iter()
        .map(|p| u128::from(crate::anatomy::ceiling(p.half_extent, rules.body()))).sum()
}

fn revised(o: &Offer) -> &Development {
    match &o.commands[0] {
        Command::Revise { development, .. } => development,
        other => panic!("not a revision: {other:?}"),
    }
}

/// Lines `a`, its child `b`, its grandchild `c` and an unrelated `x`, all
/// of one kingdom, and the world's rules with a function-poor kind and a
/// kind of the recipe's own function added.
struct Tree {
    lines: BTreeMap<Key, Lineage>,
    rules: Rules,
    unfamiliar: Key,
    familiar: Key,
}

fn tree() -> Tree {
    let (s, line, _) = learned();
    let mut lines = s.sim.state().lineages.clone();
    let other = lines.keys().find(|k| **k != line).cloned().unwrap();
    let mut base = lines[&line].clone();
    base.parent = None;
    let mut rules = s.sim.genesis().rules.clone();
    let owned = development(&s, &line).recipe.kinds();
    let function = owned.iter().find_map(|k| rules.kinds[k].cells.keys().next().cloned())
        .expect("the recipe's kinds carry a function");
    let template = |f: &str| Template {
        half_extent: [1, 1, 1], cells: BTreeMap::from([(f.to_string(), 1)]),
        shape: String::new(), port: None,
    };
    rules.kinds.insert("kind:unheard".into(), template("function:unheard"));
    rules.kinds.insert("kind:heard".into(), template(&function));
    lines.insert("a".into(), base.clone());
    for (name, parent) in [("b", "a"), ("c", "b")] {
        lines.insert(name.into(), Lineage { parent: Some(parent.into()), ..base.clone() });
    }
    let mut x = base.clone();
    x.development = lines[&other].development.clone();
    lines.insert("x".into(), x);
    Tree { lines, rules, unfamiliar: "kind:unheard".into(), familiar: "kind:heard".into() }
}

impl Tree {
    fn needed(&self, eater: &str, donor: &str, kind: &str) -> u8 {
        draft::needed(&self.lines, &self.rules, eater, donor, kind)
    }
}

#[test]
fn unset_the_first_meal_teaches_and_nothing_is_counted() {
    let (s, line, _) = learned();
    let (a, b) = (review(&s, &line, Some(0)), Review::of(&s, &line, 2).unwrap());
    assert_eq!(a, b, "the draft is lifted with the rule unset");
    assert!(a.budget.is_none() && a.offers.iter().all(|o| o.cost.is_none()));
    let kind = unnamed(&s, &line).pop().unwrap();
    let mut lines = s.sim.state().lineages.clone();
    lines.get_mut(&line).unwrap().development.as_mut().unwrap().lexicon.remove(&kind);
    let rules = s.sim.genesis().rules.clone();
    draft::learn(&mut lines, &rules, &line, "elsewhere", &kind, 7);
    let l = &lines[&line];
    assert!(l.development.as_ref().unwrap().lexicon.contains(&kind));
    assert!(l.meals.is_empty());
}

#[test]
fn needed_moves_by_each_term_alone() {
    let t = tree();
    let (u, f) = (t.unfamiliar.as_str(), t.familiar.as_str());
    assert_eq!(t.needed("a", "a", u), 1, "same line, same kingdom, unfamiliar");
    // Kingdom alone: b is kin (distance 1); only its kingdom differs.
    let mut other = tree();
    other.lines.get_mut("b").unwrap().kingdom = "kingdom:elsewhere".into();
    assert_eq!(t.needed("a", "b", u), 1);
    assert_eq!(other.needed("a", "b", u), 2);
    // Kinship alone: kingdoms equal, kind unfamiliar.
    assert_eq!(t.needed("a", "b", u), 1, "distance 1");
    assert_eq!(t.needed("a", "c", u), 2, "distance 2");
    assert_eq!(t.needed("a", "x", u), 3, "unrelated");
    // Familiarity alone: the same eaters and donors with a known kind.
    assert_eq!(t.needed("a", "c", f), 1, "familiar takes one off");
    assert_eq!(t.needed("a", "x", f), 2);
    // The floor.
    assert_eq!(t.needed("a", "a", f), 1);
    assert_eq!(t.needed("a", "b", f), 1);
    // A missing line is of another kingdom and unrelated; a missing kind is unfamiliar.
    assert_eq!(t.needed("ghost", "a", u), 4);
    assert_eq!(t.needed("a", "a", "kind:ghost"), 1);
    assert_eq!(t.needed("a", "x", "kind:ghost"), 3);
}

#[test]
fn a_kind_needing_three_meals_is_learned_at_the_third() {
    let (s, line, _) = learned();
    let mut rules = s.sim.genesis().rules.clone();
    rules.directing.get_or_insert_with(Directing::default).draft = Some(Draft::default());
    let kind: Key = "kind:unheard".into();
    rules.kinds.insert(kind.clone(), Template {
        half_extent: [1, 1, 1], cells: BTreeMap::from([("function:unheard".to_string(), 1)]),
        shape: String::new(), port: None,
    });
    let mut lines = s.sim.state().lineages.clone();
    let far = lines.keys().find(|k| **k != line).cloned().unwrap();
    lines.get_mut(&far).unwrap().kingdom = lines[&line].kingdom.clone();
    // An unrelated line of one kingdom, a kind its eater cannot know: 3.
    let asked = draft::needed(&lines, &rules, &line, &far, &kind);
    assert_eq!(asked, 3, "unfamiliar kinds from unrelated lines are slow to learn");
    let has = |lines: &BTreeMap<Key, Lineage>| {
        lines[&line].development.as_ref().unwrap().lexicon.contains(&kind)
    };
    draft::learn(&mut lines, &rules, &line, &far, &kind, 5);
    draft::learn(&mut lines, &rules, &line, &far, &kind, 6);
    assert!(!has(&lines), "absent after two meals");
    assert_eq!(lines[&line].meals[&kind], Eaten { count: 2, needed: 3, last: 6 });
    draft::learn(&mut lines, &rules, &line, &far, &kind, 9);
    assert!(has(&lines), "present after the third");
    assert_eq!(lines[&line].meals[&kind], Eaten { count: 3, needed: 3, last: 9 });
}

#[test]
fn the_fewest_meals_asked_is_kept() {
    let (s, line, _) = learned();
    let mut rules = s.sim.genesis().rules.clone();
    rules.directing.get_or_insert_with(Directing::default).draft = Some(Draft::default());
    let kind = unnamed(&s, &line).pop().unwrap();
    let mut lines = s.sim.state().lineages.clone();
    lines.get_mut(&line).unwrap().development.as_mut().unwrap().lexicon.remove(&kind);
    let far = lines.keys().find(|k| **k != line).cloned().unwrap();
    lines.get_mut(&far).unwrap().kingdom = "kingdom:elsewhere".into();
    assert!(draft::needed(&lines, &rules, &line, &far, &kind) > 1);
    draft::learn(&mut lines, &rules, &line, &far, &kind, 3);
    draft::learn(&mut lines, &rules, &line, &line.clone(), &kind, 4);
    assert_eq!(lines[&line].meals[&kind].needed, 1);
    assert!(lines[&line].development.as_ref().unwrap().lexicon.contains(&kind));
}

#[test]
fn a_survival_review_offers_only_this_rounds_kinds() {
    let (mut s, line, _) = learned();
    rule(&mut s, Some(Draft::default()));
    Arc::make_mut(&mut s.sim.genesis).rules.directing.as_mut().unwrap().variants = 64;
    let first = taught(&s, &line);
    let second = unnamed(&s, &line).into_iter().find(|k| *k != first).expect("a second kind");
    let lexicons = [
        s.sim.state.lineages.get_mut(&line),
        Arc::make_mut(&mut s.sim.genesis).lineages.get_mut(&line),
    ];
    for l in lexicons.into_iter().flatten() {
        l.development.as_mut().unwrap().lexicon.insert(second.clone());
    }
    ate(&mut s, &line, &first, 10);
    ate(&mut s, &line, &second, 4);
    let r = review(&s, &line, Some(10));
    let names = bearing(&r);
    assert!(!names.is_empty());
    assert!(names.iter().all(|o| o.name.ends_with(&format!("-bears-{first}"))), "{names:?}");
    assert!(names.iter().all(|o| o.cost.is_some()));
    // The same kinds, both in the lexicon, are offered when nothing is filtered.
    let all = review(&s, &line, Some(0));
    assert!(bearing(&all).iter().any(|o| o.name.ends_with(&format!("-bears-{second}"))));
    // Ate nothing since: no variant.
    let none = review(&s, &line, Some(11));
    assert!(bearing(&none).is_empty());
    assert_eq!(none.offers[0].name, "candidate:stay");
}

#[test]
fn a_variant_has_a_cost_and_a_budget_that_can_refuse_it() {
    let (mut s, line, _) = learned();
    let kind = taught(&s, &line);
    ate(&mut s, &line, &kind, 10);
    let had = development(&s, &line);
    rule(&mut s, Some(Draft::default()));
    let r = review(&s, &line, Some(10));
    let (held, members) = boundary::standing(&s, &line);
    assert_eq!(r.budget, Some(held / (u128::from(members) * 2)));
    let offer = bearing(&r)[0].clone();
    let by_hand = grown(&s, revised(&offer)).saturating_sub(grown(&s, &had));
    assert_eq!(offer.cost, Some(by_hand));
    assert!(by_hand > 0, "bearing a kind adds mass");
    // A budget of nought refuses it, saying how far over.
    rule(&mut s, Some(Draft { share: [0, 1] }));
    let r = review(&s, &line, Some(10));
    assert_eq!(r.budget, Some(0));
    let at = r.offers.iter().position(|o| o.name == offer.name).unwrap();
    let why = format!("costs {by_hand} mg, {by_hand} mg over the budget of 0 mg");
    assert_eq!(r.offers[at].why_not.as_deref(), Some(why.as_str()));
    assert!(r.commit(at).is_none());
    // A budget that covers it leaves it takeable, still carrying its cost.
    rule(&mut s, Some(Draft { share: [1_000_000, 1] }));
    let r = review(&s, &line, Some(10));
    assert!(r.budget.unwrap() >= by_hand);
    let at = r.offers.iter().position(|o| o.name == offer.name).unwrap();
    assert!(r.offers[at].takeable() && r.offers[at].why_not.is_none());
    assert_eq!(r.offers[at].cost, Some(by_hand));
    assert!(r.commit(at).is_some());
    // A budget exactly its cost is not over it.
    let (held, members) = boundary::standing(&s, &line);
    let top = u32::try_from(by_hand * u128::from(members)).unwrap();
    rule(&mut s, Some(Draft { share: [top, u32::try_from(held).unwrap()] }));
    let r = review(&s, &line, Some(10));
    assert_eq!(r.budget, Some(by_hand));
    let at = r.offers.iter().position(|o| o.name == offer.name).unwrap();
    assert!(r.offers[at].takeable());
}

#[test]
fn creative_lifts_the_draft() {
    let (mut s, line, _) = learned();
    let plain = Review::of(&s, &line, 2).unwrap();
    rule(&mut s, Some(Draft { share: [0, 1] }));
    let r = review(&s, &line, None);
    assert_eq!(r, plain, "no filter, no cost, no budget");
    assert!(r.budget.is_none() && r.offers.iter().all(|o| o.cost.is_none()));
    let lexicon = development(&s, &line);
    let lacking: BTreeSet<_> = lexicon.lexicon.difference(&lexicon.recipe.kinds()).cloned().collect();
    assert!(!lacking.is_empty());
    for kind in &lacking {
        assert!(bearing(&r).iter().any(|o| o.name.ends_with(&format!("-bears-{kind}"))));
    }
    assert!(bearing(&r).iter().all(|o| o.takeable()));
}

#[test]
fn unplayed_lines_are_not_drafted() {
    let (mut s, line, _) = learned();
    let kind = taught(&s, &line);
    ate(&mut s, &line, &kind, 10);
    let candidates = |s: &Session| -> Vec<(Key, Vec<Command>)> {
        let others = s.sim.state().lineages.keys().cloned().collect::<Vec<_>>();
        others.iter().flat_map(|l| revise::revisions(s, l)).map(|c| (c.name, c.commands)).collect()
    };
    let before = candidates(&s);
    assert!(!before.is_empty());
    rule(&mut s, Some(Draft { share: [0, 1] }));
    assert_eq!(candidates(&s), before);
}

// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::{Execution, Founding, Session, history::Command, schema::*};

/// A bodied world whose first developing line has learned a kind its
/// recipe lacks, and a living member of that line.
fn learned() -> (Session, Key, Id) {
    let founding = Founding {
        seed: 31,
        lineages: 3,
        bodies: Some(crate::bodied::Bodies::default()),
        ..Default::default()
    };
    let mut g = founding.generate().unwrap();
    let kinds = g.rules.kinds.clone();
    let line = g
        .lineages
        .iter()
        .find(|(_, l)| l.development.is_some())
        .map(|(k, _)| k.clone())
        .unwrap();
    let d = g
        .lineages
        .get_mut(&line)
        .unwrap()
        .development
        .as_mut()
        .unwrap();
    let named = d.recipe.kinds();
    let other = kinds.keys().find(|k| !named.contains(*k)).unwrap();
    d.lexicon.insert(other.clone());
    let s = Session::new(g, Execution::Individuals).unwrap();
    let member = s
        .sim
        .state()
        .population
        .groups
        .iter()
        .find(|(_, g)| g.entity.alive && g.entity.lineage == line)
        .map(|(id, _)| *id)
        .unwrap();
    (s, line, member)
}

fn speciate(s: &mut Session, founder: Id, name: &str) -> Result<String, String> {
    s.command(Command::Speciate {
        founder,
        name: name.into(),
    })
}

#[test]
fn naming_a_founder_splits_its_line() {
    let (mut s, line, founder) = learned();
    let made = speciate(&mut s, founder, "a").unwrap();
    let lines = &s.sim.state().lineages;
    assert_eq!(lines[&made].parent.as_deref(), Some(line.as_str()));
    assert_eq!(s.sim.state().population.get(founder).unwrap().lineage, made);
    assert_eq!(
        tree::ancestry(lines, &made),
        vec![made.clone(), line.clone()]
    );
    assert_eq!(tree::distance(lines, &made, &line), Some(1));
    assert!(tree::descends_from(lines, &made, &line));
    assert!(!tree::descends_from(lines, &line, &made));
    assert_eq!(speciated_at(&s, &made), Some(0));
    // Planted: a taken name and an unknown founder are refused.
    assert!(speciate(&mut s, founder, "a").is_err());
    assert!(speciate(&mut s, Id::MAX - 1, "b").is_err());
}

#[test]
fn founding_lines_are_unrelated() {
    let (s, line, _) = learned();
    let lines = &s.sim.state().lineages;
    let other = lines.keys().find(|k| **k != line).unwrap();
    assert_eq!(tree::common_ancestor(lines, &line, other), None);
    assert_eq!(tree::distance(lines, &line, other), None);
    assert_eq!(tree::distance(lines, &line, &line), Some(0));
}

#[test]
fn a_fork_inherits_its_parents_program_up_to_the_split() {
    let (mut s, line, founder) = learned();
    assert!(program::program(&s, &line).is_empty());
    let first = &review::offers(&s, &line, 1).unwrap()[1];
    s.command(first.commands[0].clone()).unwrap();
    let made = speciate(&mut s, founder, "a").unwrap();
    let inherited = program::program(&s, &made);
    assert_eq!(inherited.len(), 1);
    assert_eq!(inherited[0].lineage, line);
    // The parent's later revision is not the fork's.
    let again = s.sim.state().lineages[&line].development.clone().unwrap();
    s.command(Command::Revise {
        lineage: line.clone(),
        development: again,
    })
    .unwrap();
    assert_eq!(program::program(&s, &line).len(), 2);
    assert_eq!(program::program(&s, &made).len(), 1);
    assert_ne!(program::digest(&program::program(&s, &made)), None);
}

#[test]
fn the_review_offers_the_status_quo_first_and_commits_a_revision() {
    let (s, line, _) = learned();
    let review = Review::of(&s, &line, 2).unwrap();
    assert_eq!(review.offers[0].name, "candidate:stay");
    assert!(review.commit(0).is_none(), "the status quo commits nothing");
    let before = s.sim.state().lineages[&line].revision;
    let i = review.offers.iter().position(Offer::takeable).unwrap();
    let mut after = s.clone();
    for c in review.commit(i).unwrap() {
        after.command(c.clone()).unwrap();
    }
    assert_eq!(after.sim.state().lineages[&line].revision, before + 1);
    // The reading took nothing from the world it read.
    assert_eq!(s.sim.state().lineages[&line].revision, before);
    assert_eq!(Review::of(&s, &line, 2).unwrap(), review);
}

#[test]
fn a_line_that_learned_nothing_has_only_the_status_quo() {
    let (s, line, _) = learned();
    let other = s.sim.state().lineages.keys().find(|k| **k != line).unwrap();
    let offers = review::offers(&s, other, 1).unwrap();
    assert!(offers.iter().all(|o| !o.takeable()), "{offers:?}");
}

#[test]
fn readings_are_never_of_nothing() {
    let (s, line, _) = learned();
    let readings = reckon::readings(&s.sim);
    assert!(readings.iter().all(|r| r.value > 0));
    assert!(
        readings
            .iter()
            .any(|r| r.lineage == line && r.feat == reckon::GROWTH)
    );
}

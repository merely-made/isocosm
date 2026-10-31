// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Controls for rulings 751 to 753: habitability at the start, the native
//! lineage revision, and migration by a move per route.

use super::interim::{Interim, Pace, Start, boundary};
use super::readings::Mode;
use super::tests::{played, world};
use super::*;
use crate::{Execution, Founding, Session, history::Command};

const PACE: Pace = Pace {
    round: 4,
    scoring: 6,
};

fn played_genesis(seed: u64, lineage: u32) -> crate::simulation::Genesis {
    let mut g = Founding {
        seed,
        sites: 4,
        population: 48,
        cohort_size: 4,
        lineages: 3,
        ecology: true,
        played: Some(Played {
            lineage,
            region_sites: 2,
        }),
        ..Default::default()
    }
    .generate()
    .unwrap();
    // Short epochs, so waiting for habitability is cheap.
    g.rules.epoch_ticks = 8;
    g
}

#[test]
fn the_start_waits_for_a_habitable_site() {
    let i = Interim::found(
        played_genesis(4, 1),
        Start::default(),
        Mode::Creative,
        PACE,
        Execution::Individuals,
    )
    .unwrap();
    assert_eq!(i.habitable_at, 0);
    let c = i.critter().unwrap();
    let place = i.session.sim.state().population.get(c).unwrap().place;
    assert!(readings::habitable(&i.session.sim, "lineage:1").contains(&place));
    // Planted: producers need a habitable condition no site meets.
    let mut g = played_genesis(4, 0);
    for site in g.sites.values_mut() {
        site.conditions.insert("world:habitable".into(), 0);
    }
    let start = Start {
        within: 2,
        epochs: 0,
    };
    let refused = Interim::found(g, start, Mode::Creative, PACE, Execution::Individuals);
    assert!(refused.unwrap_err().contains("no habitable site within 2"));
}

/// A bodied world whose lineage 0 has learned a kind its recipe lacks.
fn learned() -> (Session, Key) {
    let founding = Founding {
        seed: 31,
        lineages: 3,
        bodies: Some(crate::bodied::Bodies::default()),
        ..Default::default()
    };
    let mut g = founding.generate().unwrap();
    let rules = g.rules.clone();
    let lines = &mut g.lineages;
    let line = lines
        .keys()
        .find(|k| lines[*k].development.is_some())
        .cloned()
        .unwrap();
    let d = lines.get_mut(&line).unwrap().development.as_mut().unwrap();
    let named = d.recipe.kinds();
    let other = rules
        .kinds
        .keys()
        .find(|k| !named.contains(*k))
        .cloned()
        .unwrap();
    d.lexicon.insert(other);
    let s = Session::new(g, Execution::Individuals).unwrap();
    (s, line)
}

#[test]
fn a_line_revises_its_recipe_from_what_it_learned() {
    let (mut s, line) = learned();
    let offered = revise::revisions(&s, &line);
    assert!(!offered.is_empty(), "a learned kind offers a variant");
    let before = s.sim.state().lineages[&line].revision;
    s.command(offered[0].commands[0].clone()).unwrap();
    assert_eq!(s.sim.state().lineages[&line].revision, before + 1);
    // Planted: a revision reaching past the lexicon is refused.
    let mut d = s.sim.state().lineages[&line].development.clone().unwrap();
    d.lexicon.insert("kind:unlearned".into());
    let reach = Command::Revise {
        lineage: line.clone(),
        development: d,
    };
    assert!(s.command(reach).is_err());
}

#[test]
fn the_boundary_weighs_revisions() {
    let (mut s, line) = learned();
    let turns = boundary::adapt(&mut s, None, &revise::revisions, 6).unwrap();
    let turn = turns
        .iter()
        .find(|t| t.lineage == line)
        .expect("the line took a turn");
    assert!(turn.considered.len() >= 2, "the status quo and a variant");
}

#[test]
fn a_move_per_route_and_members_migrate() {
    let mut s = world(6, Execution::Individuals);
    let rules = &s.sim.genesis().rules;
    let routes: usize = s.sim.state().sites.values().map(|x| x.routes.len()).sum();
    assert_eq!(
        rules
            .processes
            .keys()
            .filter(|k| k.starts_with("move:"))
            .count(),
        routes
    );
    let (p, c) = played(&mut s);
    let here = s.sim.state().population.get(c).unwrap().place;
    let there = s.sim.state().sites[&here].routes[0].to;
    // Hungry, and nudged to attend a neighbour: it goes.
    let e = s.sim.state.population.lift(c).unwrap();
    let body = e.accounts.insert("matter:1-0".into(), 4).unwrap();
    *e.accounts.entry("world:soil".into()).or_default() += body - 4;
    let n = Command::Nudge {
            act: None,
        participant: p,
        critter: c,
        aim: Aim::Attend,
        toward: Toward::Site(there),
    };
    s.command(n).unwrap();
    s.advance(12).unwrap();
    let moved = s.sim.state().nudges[0]
        .answer
        .as_ref()
        .is_some_and(|a| a.process.starts_with("move:"));
    assert!(moved, "{:?}", s.sim.state().nudges[0]);
    s.advance(40).unwrap();
    let travelled = s
        .sim
        .state()
        .population
        .groups
        .values()
        .any(|g| !g.entity.visits.is_empty() && g.entity.method == Method::Reactive);
    assert!(travelled, "hungry members migrate");
}

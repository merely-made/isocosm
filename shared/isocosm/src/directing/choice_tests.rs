// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The deliberative methodology's controls (D2) and the bond's (D1).

use super::tests::{played, world};
use super::*;
use crate::{Execution, Session, history::Command};

/// A living producer sharing the critter's site, holding matter.
fn prey(s: &Session, c: Id) -> Option<Id> {
    let pop = &s.sim.state().population;
    let place = pop.get(c)?.place;
    let found = pop.groups.iter().find(|(_, g)| {
        g.entity.alive && g.entity.place == place && g.entity.lineage == "lineage:0"
    });
    found.map(|(f, _)| *f)
}

fn set_body(s: &mut Session, c: Id, amount: u64) {
    let e = s.sim.state.population.lift(c).unwrap();
    e.accounts.insert("matter:1-0".into(), amount);
}

fn nudge(s: &mut Session, p: Id, c: Id, aim: Aim, toward: Toward) {
    let n = Command::Nudge {
            act: None,
        participant: p,
        critter: c,
        aim,
        toward,
    };
    s.command(n).unwrap();
}

#[test]
fn a_bodied_choice_reads_the_native_need_ceiling() {
    let mut g = crate::Founding {
        seed: 3,
        sites: 1,
        population: 30,
        cohort_size: 1,
        lineages: 3,
        ecology: true,
        bodies: Some(crate::bodied::Bodies::default()),
        played: Some(Played { lineage: 1, region_sites: 1 }),
        ..Default::default()
    }.generate().unwrap();
    let p = g.rules.processes.get_mut("ecology:feed-1").unwrap();
    p.need_account = Some("matter:1-0".into());
    p.need_below = 10;
    let mut s = Session::new(g, Execution::Individuals).unwrap();
    let c = *s.sim.state().population.groups.iter()
        .find(|(_, g)| g.entity.lineage == "lineage:1").unwrap().0;
    assert!(prey(&s, c).is_some());
    assert!(!s.sim.consider(c, false).options.contains_key("ecology:feed-1"));
    let rules = s.sim.genesis().rules.clone();
    crate::anatomy::take(s.sim.state.population.lift(c).unwrap(), &rules,
        "matter:1-0", 1).unwrap().unwrap();
    assert!(s.sim.consider(c, false).options.contains_key("ecology:feed-1"));
}

#[test]
fn an_unnudged_critter_acts_on_its_own_needs() {
    let mut s = world(3, Execution::Individuals);
    let (_, c) = played(&mut s);
    // Tick 15 has both upkeep (every 5) and feeding (every 3) due.
    s.sim.state.tick = 15;
    let fed = s.sim.choose(c);
    assert_eq!(
        fed.process.as_deref(),
        Some("ecology:upkeep-1"),
        "{:?}",
        fed.options
    );
    set_body(&mut s, c, 3);
    if prey(&s, c).is_some() {
        let hungry = s.sim.choose(c);
        assert_eq!(
            hungry.process.as_deref(),
            Some("ecology:feed-1"),
            "{:?}",
            hungry.options
        );
    }
}

#[test]
fn a_nudge_moves_the_choice_by_the_bond_and_a_zero_bond_moves_nothing() {
    let mut ran = 0;
    for seed in 1..10 {
        let mut s = world(seed, Execution::Individuals);
        let (p, c) = played(&mut s);
        let Some(thing) = prey(&s, c) else {
            continue;
        };
        ran += 1;
        nudge(&mut s, p, c, Aim::Act, Toward::Thing(thing));
        s.sim.state.tick = 15;
        let score = |s: &Session| s.sim.choose(c).options["ecology:feed-1"].0;
        let swayed = s.sim.choose(c);
        assert_eq!(swayed.process.as_deref(), Some("ecology:feed-1"));
        assert_eq!(swayed.target, Some(thing), "the nudge names the target");
        let with = score(&s);
        s.sim.set_bond(p, c, 0);
        let without = s.sim.choose(c);
        assert_eq!(
            with - score(&s),
            250,
            "the sway is the bond's share of the most"
        );
        assert_eq!(
            without.process.as_deref(),
            Some("ecology:upkeep-1"),
            "a zero bond sways nothing"
        );
    }
    assert!(ran > 0, "no seed put prey beside the critter");
}

#[test]
fn choice_is_deterministic_under_the_seed() {
    let run = |seed| {
        let mut s = world(seed, Execution::Grouped);
        let (p, c) = played(&mut s);
        for t in 0..4u64 {
            if s.sim.state().population.get(c).is_some_and(|e| e.alive) {
                nudge(&mut s, p, c, Aim::Attend, Toward::Site(t % 4));
            }
            s.advance(8).unwrap();
        }
        s.sim.state_hash()
    };
    assert_eq!(run(21), run(21));
    assert_ne!(run(21), run(22));
}

#[test]
fn outcomes_that_serve_raise_the_bond_and_those_that_harm_lower_it() {
    let mut ran = 0;
    for seed in 1..12 {
        let mut s = world(seed, Execution::Individuals);
        let (p, c) = played(&mut s);
        let Some(thing) = prey(&s, c) else {
            continue;
        };
        ran += 1;
        // Tick 3 has only feeding due: eating what it was pointed at serves it.
        nudge(&mut s, p, c, Aim::Act, Toward::Thing(thing));
        s.advance(3).unwrap();
        assert_eq!(
            s.sim.bond(p, c),
            Some(300),
            "seed {seed}: a meal raised the bond"
        );
        // Tick 5 has only upkeep due: staying put to pay it harms it.
        let home = s.sim.state().population.get(c).unwrap().place;
        nudge(&mut s, p, c, Aim::Attend, Toward::Site(home));
        s.advance(2).unwrap();
        assert_eq!(
            s.sim.bond(p, c),
            Some(250),
            "seed {seed}: a cost lowered the bond"
        );
        let answers: Vec<bool> = s
            .sim
            .state()
            .nudges
            .iter()
            .map(|n| n.answer.as_ref().unwrap().served)
            .collect();
        assert_eq!(answers, [true, false]);
    }
    assert!(ran > 0, "no seed put prey beside the critter");
}

#[test]
fn a_reactive_critter_is_not_deliberated() {
    let mut s = world(3, Execution::Individuals);
    let reactive = s
        .sim
        .state()
        .population
        .groups
        .iter()
        .find(|(_, g)| g.entity.method == Method::Reactive);
    let (r, _) = reactive.unwrap();
    let r = *r;
    let feed = s.sim.genesis().rules.processes["ecology:upkeep-0"].clone();
    assert!(matches!(s.sim.deliberate(r, &feed), Deliberated::Free));
}

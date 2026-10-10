// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A prey can move after the methodology chooses an act, before its feeding
//! pass. Rule 454 binds automatic prey there; a Thing/Act nudge binds identity.

use super::*;
use crate::{Execution, Founding, Session, history::Command, rules::*, simulation::Genesis};

fn fixture() -> (Genesis, Id, Id) {
    let mut g = Founding {
        seed: 3,
        sites: 2,
        population: 30,
        cohort_size: 2,
        lineages: 3,
        ecology: true,
        played: Some(Played {
            lineage: 1,
            region_sites: 2,
        }),
        ..Default::default()
    }
    .generate()
    .unwrap();
    let actor = *g
        .population
        .groups
        .iter()
        .find(|(_, g)| g.entity.lineage == "lineage:1")
        .unwrap()
        .0;
    let producers: Vec<_> = g
        .population
        .groups
        .iter()
        .filter(|(_, g)| g.entity.lineage == "lineage:0")
        .take(2)
        .map(|(id, _)| *id)
        .collect();
    assert_eq!(producers.len(), 2);
    g.population
        .groups
        .retain(|id, _| *id == actor || producers.contains(id));
    for group in g.population.groups.values_mut() {
        group.entity.place = 0;
        if group.entity.lineage == "lineage:1" {
            group.entity.accounts.insert("matter:1-0".into(), 3);
        }
    }
    g.rules
        .processes
        .retain(|id, _| id == "ecology:feed-1" || id == "ecology:upkeep-1");
    for p in g.rules.processes.values_mut() {
        p.period = Some(1);
    }
    let mut early = crate::generate::process(
        "test:early",
        Causation::Choice,
        vec![Effect::Condition {
            key: "test:early".into(),
            delta: 1,
        }],
    );
    early.requires.push(Query::Trait {
        who: Binding::Actor,
        key: "ability:cycle-1".into(),
    });
    early.period = Some(1);
    early.priority = -20;
    g.rules.conditions.insert("test:early".into());
    g.rules.processes.insert(early.id.clone(), early);
    let mut moving = crate::generate::process(
        "test:prey-move",
        Causation::Choice,
        vec![Effect::Move { destination: 1 }],
    );
    moving.requires.push(Query::Trait {
        who: Binding::Actor,
        key: "test:mover".into(),
    });
    moving.period = Some(1);
    moving.priority = -10;
    g.rules.traits.insert("test:mover".into());
    g.rules.processes.insert(moving.id.clone(), moving);
    let mut preview =
        crate::simulation::Simulation::new(g.clone(), Execution::Individuals).unwrap();
    preview.state.tick = 1;
    let choice = preview.choose(actor);
    assert_eq!(choice.process.as_deref(), Some("ecology:feed-1"));
    let moved = choice.target.unwrap();
    g.population
        .lift(moved)
        .unwrap()
        .traits
        .insert("test:mover".into());
    (g, actor, moved)
}

fn joined(g: Genesis, mode: Execution, actor: Id, nudge: Option<(Aim, Id)>) -> (Session, Id) {
    let mut s = Session::new(g, mode).unwrap();
    let (participant, ours) = super::tests::played(&mut s);
    assert_eq!(ours, actor);
    if let Some((aim, thing)) = nudge {
        s.command(Command::Nudge {
            participant,
            critter: actor,
            aim,
            toward: Toward::Thing(thing),
            act: Some("ecology:feed-1".into()),
        })
        .unwrap();
    }
    for id in [
        "test:early",
        "test:prey-move",
        "ecology:feed-1",
        "ecology:upkeep-1",
    ] {
        s.sim.watch(id);
    }
    (s, participant)
}

fn replay(s: &Session) {
    for mode in [Execution::Individuals, Execution::Grouped] {
        assert_eq!(
            Session::load(s.save(), mode).unwrap().sim.state_hash(),
            s.sim.state_hash()
        );
    }
}

#[test]
fn automatic_feeding_draws_after_the_scored_prey_moves() {
    let (g, actor, moved) = fixture();
    let mut hashes = Vec::new();
    for mode in [Execution::Individuals, Execution::Grouped] {
        for _ in 0..2 {
            let (mut s, _) = joined(g.clone(), mode, actor, None);
            let matter = s.sim.matter();
            let flows = s.advance_tick_with_flows().unwrap().flows;
            assert_eq!(s.sim.matter(), matter);
            assert_eq!(s.sim.state().population.get(moved).unwrap().place, 1);
            let watched = s.sim.take_watched();
            let acts: Vec<_> = watched.iter().filter(|w| w.actor == actor).collect();
            assert_eq!(
                acts.len(),
                1,
                "one choice, with no upkeep or early fallback"
            );
            assert_eq!(acts[0].process, "ecology:feed-1");
            let prey = acts[0].target.unwrap();
            assert_ne!(prey, moved);
            assert_eq!(s.sim.state().population.get(prey).unwrap().place, 0);
            assert_eq!(
                s.sim.state().population.get(actor).unwrap().accounts["matter:1-0"],
                4
            );
            assert!(flows.iter().any(|f| f.from.0.body() == Some(prey)
                && f.to.0.body() == Some(actor)
                && f.amount == 1));
            replay(&s);
            hashes.push(s.sim.state_hash());
        }
    }
    assert!(
        hashes.iter().all(|hash| *hash == hashes[0]),
        "deterministic in both modes"
    );
}

#[test]
fn an_explicit_prey_identity_is_not_replaced_or_credited() {
    let (g, actor, moved) = fixture();
    for mode in [Execution::Individuals, Execution::Grouped] {
        let (mut s, participant) = joined(g.clone(), mode, actor, Some((Aim::Act, moved)));
        let matter = s.sim.matter();
        s.advance(1).unwrap();
        assert_eq!(s.sim.matter(), matter);
        assert_eq!(s.sim.state().population.get(moved).unwrap().place, 1);
        assert!(
            !s.sim.take_watched().iter().any(|w| w.actor == actor),
            "no replacement or fallback"
        );
        assert_eq!(
            s.sim.state().population.get(actor).unwrap().accounts["matter:1-0"],
            3
        );
        let answer = s.sim.state().nudges[0].answer.as_ref().unwrap();
        assert!(!answer.served, "the bound prey could not be eaten");
        assert_eq!(
            s.sim.bond(participant, actor),
            Some(200),
            "no credit from another prey"
        );
        replay(&s);
    }
}

#[test]
fn replacement_prey_does_not_answer_attention_to_the_moved_thing() {
    let (g, actor, moved) = fixture();
    for mode in [Execution::Individuals, Execution::Grouped] {
        let (mut s, participant) = joined(g.clone(), mode, actor, Some((Aim::Attend, moved)));
        s.advance(1).unwrap();
        let feed = s
            .sim
            .take_watched()
            .into_iter()
            .find(|w| w.actor == actor)
            .unwrap();
        assert_eq!(feed.process, "ecology:feed-1");
        assert_ne!(feed.target, Some(moved));
        assert!(
            s.sim.state().nudges[0].answer.is_none(),
            "the old cached answer no longer fits"
        );
        assert_eq!(s.sim.bond(participant, actor), Some(250));
        replay(&s);
    }
}

#[test]
fn planning_and_visiting_keep_one_resolved_prey() {
    let (g, actor, moved) = fixture();
    for mode in [Execution::Individuals, Execution::Grouped] {
        let (mut s, _) = joined(g.clone(), mode, actor, None);
        s.sim.state.tick = 1;
        let early = s.sim.genesis().rules.processes["test:early"].clone();
        let feed = s.sim.genesis().rules.processes["ecology:feed-1"].clone();
        assert!(matches!(
            s.sim.deliberate(actor, &early),
            Deliberated::Foregone
        ));
        s.sim.state.population.lift(moved).unwrap().place = 1;
        let planned = s.sim.deliberate(actor, &feed).target().unwrap();
        assert_ne!(planned, moved);
        s.sim.state.population.lift(planned).unwrap().place = 1;
        assert_eq!(
            s.sim.deliberate(actor, &feed).target(),
            Some(planned),
            "no second resolution within the pass"
        );
        assert!(matches!(
            s.sim.deliberate(actor, &early),
            Deliberated::Foregone
        ));
    }
}

#[test]
fn a_nudge_for_another_act_does_not_bind_the_meals_prey() {
    let (g, actor, moved) = fixture();
    for mode in [Execution::Individuals, Execution::Grouped] {
        let (mut s, participant) = joined(g.clone(), mode, actor, None);
        s.command(Command::Nudge {
            participant,
            critter: actor,
            aim: Aim::Act,
            toward: Toward::Thing(moved),
            act: Some("ecology:upkeep-1".into()),
        })
        .unwrap();
        s.advance(1).unwrap();
        let feed = s
            .sim
            .take_watched()
            .into_iter()
            .find(|w| w.actor == actor)
            .unwrap();
        assert_eq!(feed.process, "ecology:feed-1");
        assert_ne!(feed.target, Some(moved));
        assert!(s.sim.state().nudges[0].answer.is_none());
        assert_eq!(s.sim.bond(participant, actor), Some(250));
        replay(&s);
    }
}

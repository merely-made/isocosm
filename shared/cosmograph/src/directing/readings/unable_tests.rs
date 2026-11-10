// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Why not (case 1233) and what a body's parts let it do (cases 1233, 1243).

use super::*;
use crate::{
    Execution,
    rules::{Causation, Query},
    simulation::{Genesis, Outcome},
};
use std::collections::BTreeSet;

fn world(seed: u64) -> Genesis {
    crate::probe::BodyFounding { seed, ..Default::default() }.generate().unwrap().genesis
}

/// A feeding act that needs a part that fixes.
fn with_fixing(mut g: Genesis) -> Genesis {
    let needs = Query::Expresses { function: "function:fix".into() };
    let mut p = crate::generate::process("test:fixing", Causation::Choice, vec![]);
    p.requires.push(needs);
    p.period = Some(1);
    g.rules.processes.insert(p.id.clone(), p);
    g
}

fn first(g: &Genesis, lineage: &str) -> Id {
    *g.population.groups.iter().find(|(_, c)| c.entity.lineage == lineage && c.entity.alive).unwrap().0
}

#[test]
fn a_body_lacking_the_part_is_listed_with_the_act_paths_reason() {
    let g = with_fixing(world(3));
    let (plant, grazer) = (first(&g, "lineage:0"), first(&g, "lineage:1"));
    let mut sim = Simulation::new(g, Execution::Individuals).unwrap();
    let listed = |sim: &Simulation, id| unable(sim, id).into_iter().find(|u| u.process == "test:fixing");
    assert!(listed(&sim, plant).is_none(), "a body with the part is not listed");
    let missing = listed(&sim, grazer).expect("the grazer has no part that fixes");
    let attempt = sim.execute(grazer, missing.target, "test:fixing", None);
    assert_eq!(attempt.outcome, Outcome::Blocked(missing.why.clone()));
    assert!(missing.why.starts_with("missing requirement: Expresses"), "{}", missing.why);
    assert_eq!(sim.execute(plant, None, "test:fixing", None).outcome, Outcome::Accepted);
}

#[test]
fn every_reason_is_the_one_its_act_gives_where_attempted() {
    for seed in [3, 5] {
        let sim = Simulation::new(world(seed), Execution::Individuals).unwrap();
        let ids: Vec<Id> = sim.state().population.groups.keys().copied().take(24).collect();
        for id in ids {
            let all = unable(&sim, id);
            assert!(all.windows(2).all(|w| w[0].process < w[1].process), "in process-key order");
            for u in all {
                let mut tried = sim.clone();
                let receipt = tried.execute(id, u.target, &u.process, None);
                assert_eq!(receipt.outcome, Outcome::Blocked(u.why.clone()), "{seed} {id} {}", u.process);
            }
        }
    }
}

#[test]
fn offered_and_unable_agree_for_every_choice_process() {
    for seed in [3, 5, 7] {
        let sim = Simulation::new(with_fixing(world(seed)), Execution::Individuals).unwrap();
        let ids: Vec<Id> = sim.state().population.groups.keys().copied().collect();
        for id in ids {
            let offered = sim.consider(id, false).options;
            let barred: Vec<Key> = unable(&sim, id).into_iter().map(|u| u.process).collect();
            for p in sim.genesis().rules.processes.values().filter(|p| p.causation == Causation::Choice) {
                if sim.satisfied(id, p) {
                    assert!(!barred.contains(&p.id) && !offered.contains_key(&p.id));
                    continue;
                }
                assert_ne!(offered.contains_key(&p.id), barred.contains(&p.id), "{seed} {id} {}", p.id);
            }
            for s in suggestions(&sim, id) {
                assert!(!barred.contains(&s.process), "a suggestion is never unable");
            }
        }
    }
}

#[test]
fn a_grown_bodys_abilities_are_its_own() {
    let sim = Simulation::new(world(3), Execution::Individuals).unwrap();
    for (&id, c) in &sim.state().population.groups {
        let e = &c.entity;
        let all = abilities(&sim, id);
        assert!(all.windows(2).all(|w| w[0].function < w[1].function));
        let live: BTreeSet<&Key> = e.living().flat_map(|(_, p)| p.functions.iter()).collect();
        assert_eq!(all.iter().map(|a| &a.function).collect::<BTreeSet<_>>(), live);
        for a in &all {
            let cells: u32 = e.living().map(|(_, p)| p.cells.get(&a.function).copied().unwrap_or(0)).sum();
            assert_eq!(a.cells, cells);
            assert!(!a.borrowed && a.from.is_empty(), "{id} {a:?}");
        }
    }
}

#[test]
fn a_part_is_borrowed_when_most_of_its_matter_is_anothers() {
    let g = world(3);
    let plant = first(&g, "lineage:0");
    let read = |own: u64, other: u64| {
        let mut g = g.clone();
        let e = g.population.lift(plant).unwrap();
        for id in e.parts.keys().copied().collect::<Vec<_>>() {
            let p = e.part_mut(&id).unwrap();
            p.matter.clear();
            p.matter.insert("tissue:0".into(), own);
            p.matter.insert("tissue:1".into(), other);
        }
        let sim = Simulation::new(g, Execution::Individuals).unwrap();
        abilities(&sim, plant).into_iter().find(|a| a.function == "function:fix").unwrap()
    };
    let most = read(5, 6);
    assert!(most.borrowed, "most of it is lineage:1's");
    assert_eq!(most.from, BTreeSet::from(["lineage:1".to_string()]));
    assert!(!read(5, 5).borrowed, "half is not most");
    assert!(!read(7, 0).borrowed);
    assert!(!read(0, 0).borrowed, "a part holding no matter is its body's own");
}

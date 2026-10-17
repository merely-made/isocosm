// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The indexed candidate view must agree with expanded original members,
//! even after native acts move, split, kill, spend, and reproduce them.

use super::*;
use crate::{
    Execution, Founding,
    schedule::{Frame, Pass},
    simulation::Genesis,
};

fn fixture(seed: u64, place: Id) -> (Genesis, Id, Id, Id) {
    let mut g = Founding {
        seed,
        sites: 2,
        population: 30,
        lineages: 3,
        cohort_size: 2,
        ecology: true,
        ..Default::default()
    }
    .generate()
    .unwrap();
    let body = |lineage: &str| {
        g.population
            .groups
            .values()
            .find(|c| c.entity.lineage == lineage)
            .unwrap()
            .entity
            .as_ref().clone()
    };
    let mut actor = body("lineage:1");
    let mut prey = body("lineage:0");
    actor.place = place;
    prey.place = 0;
    prey.accounts = Ledger::from([("matter:0-0".into(), 10)]);
    prey.traits.insert("test:edible".into());
    g.rules.traits.insert("test:edible".into());
    g.population = Population::default();
    let actor = g.population.insert(actor, 1).unwrap();
    let first = g.population.insert(prey.clone(), 5).unwrap();
    prey.place = 1;
    let cohort = g.population.insert(prey.clone(), 3).unwrap();
    prey.place = 0;
    prey.alive = false;
    g.population.insert(prey, 2).unwrap();
    g.rules.processes.retain(|id, _| id == "ecology:feed-1");
    let feed = g.rules.processes.get_mut("ecology:feed-1").unwrap();
    feed.requires.push(Query::Trait {
        who: Binding::Target,
        key: "test:edible".into(),
    });
    for (id, effects) in [
        ("test:move", vec![Effect::Move { destination: 1 }]),
        (
            "test:strip",
            vec![Effect::Trait {
                who: Binding::Actor,
                key: "test:edible".into(),
                present: false,
            }],
        ),
        ("test:death", vec![Effect::Death]),
        (
            "test:spend",
            vec![Effect::Transfer {
                from: Binding::Actor,
                to: Binding::Place,
                account: "matter:0-0".into(),
                amount: 7.into(),
            }],
        ),
        (
            "test:birth",
            vec![Effect::Birth {
                provision: Ledger::from([("matter:0-0".into(), 8)]),
            }],
        ),
    ] {
        let p = crate::generate::process(id, Causation::Choice, effects);
        g.rules.processes.insert(id.into(), p);
    }
    (g, actor, first, cohort)
}

/// A separate per-member oracle, without stored-group counts or the index.
fn expanded(s: &Simulation, actor: Id, p: &Process, bound: Id) -> Option<Id> {
    let selector = p.target.as_ref().unwrap();
    let place = s.body_at_start(actor).unwrap().place;
    let part = s.bind_part(actor, p);
    let mut candidates = Vec::new();
    for id in 0..bound {
        if !s.target_matches(actor, Some(id), p)
            || !p
                .requires
                .iter()
                .all(|q| s.query(actor, Some(id), place, part, q).is_ok())
        {
            continue;
        }
        let held = mass(
            &crate::anatomy::books(s.body_at_start(id).unwrap()),
            &s.genesis.rules,
        );
        if !selector.weighted {
            return Some(id);
        }
        if held > 0 {
            candidates.push((id, held));
        }
    }
    let total: u128 = candidates.iter().map(|(_, held)| held).sum();
    if total == 0 {
        return None;
    }
    let draw = crate::draw(
        s.genesis.dynamics_seed(),
        &format!("target:{}", p.id),
        &[s.state.tick, actor],
    );
    let mut pick = (u128::from(draw) * total) >> 64;
    for (id, held) in candidates {
        if pick < held {
            return Some(id);
        }
        pick -= held;
    }
    unreachable!()
}

fn start(s: &mut Simulation) {
    s.state.tick = 1;
    s.targets = Some(Targets::new(&s.state.population));
    s.pass = Some(Pass::new(&s.state.population));
    s.frame = Some(Frame::default());
}

#[test]
fn pass_books_are_reused_for_start_state_then_reset_for_next_pass() {
    let (g, _, first, _) = fixture(17, 0);
    let mut s = Simulation::new(g, Execution::Grouped).unwrap();
    start(&mut s);

    let expected = crate::anatomy::books(s.body_at_start(first).unwrap());
    let cached = s.books_at_start(first).unwrap();
    let again = s.books_at_start(first).unwrap();
    assert!(std::sync::Arc::ptr_eq(&cached, &again), "same pass should reuse one ledger");
    assert_eq!(cached.as_ref(), &expected, "cache agrees with uncached holdings");

    assert!(s.apply(first, None, "test:spend", None, 1).accepted());
    let after_mutation = s.books_at_start(first).unwrap();
    assert!(std::sync::Arc::ptr_eq(&cached, &after_mutation));
    assert_eq!(after_mutation.as_ref(), &expected);
    assert_ne!(crate::anatomy::books(s.state.population.get(first).unwrap()), expected);

    start(&mut s);
    let next_uncached = crate::anatomy::books(s.body_at_start(first).unwrap());
    let next = s.books_at_start(first).unwrap();
    assert_eq!(next.as_ref(), &next_uncached);
    assert!(!std::sync::Arc::ptr_eq(&cached, &next), "next pass must not reuse an old ledger");
}

#[test]
fn moved_split_cohort_dead_and_born_candidates_keep_original_spans() {
    let mut draws = BTreeSet::new();
    for seed in 0..12 {
        for place in [0, 1] {
            for mode in [Execution::Individuals, Execution::Grouped] {
                let (g, actor, first, cohort) = fixture(seed, place);
                let mut s = Simulation::new(g, mode).unwrap();
                start(&mut s);
                let bound = s.state.population.next_id;
                let p = s.genesis.rules.processes["ecology:feed-1"].clone();
                let expected = expanded(&s, actor, &p, bound).unwrap();
                assert_eq!(s.choose_target(actor, &p), Some(expected));
                let pass_books = crate::anatomy::books(s.body_at_start(first).unwrap());
                let cached_pass_books = s.books_at_start(first).unwrap();
                let matter = s.matter();
                for (id, process, count) in [
                    (first + 1, "test:move", 1),
                    (first, "test:strip", 1),
                    (first + 3, "test:death", 1),
                    (first + 2, "test:spend", 1),
                    (first + 4, "test:birth", 1),
                    (cohort, "test:strip", 3),
                ] {
                    assert!(
                        s.apply(id, None, process, None, count).accepted(),
                        "{process}"
                    );
                    assert_eq!(s.matter(), matter);
                    assert_eq!(
                        s.books_at_start(first).unwrap().as_ref(),
                        &pass_books,
                        "cached pass-start holdings after {process}"
                    );
                    assert_eq!(
                        s.choose_target(actor, &p),
                        Some(expected),
                        "seed{seed}/{place}/{process}"
                    );
                    assert_eq!(expanded(&s, actor, &p, bound), Some(expected));
                }
                assert!(std::sync::Arc::ptr_eq(
                    &cached_pass_books,
                    &s.books_at_start(first).unwrap()
                ));
                assert_eq!(
                    s.state.population.next_id,
                    bound + 1,
                    "a genuine native birth"
                );
                assert!(s.state.population.get(bound).unwrap().alive);
                assert!(!s.state.population.get(first + 3).unwrap().alive);
                assert!(s.body_at_start(first + 3).unwrap().alive);
                let mut unweighted = p.clone();
                unweighted.target.as_mut().unwrap().weighted = false;
                assert_eq!(
                    s.choose_target(actor, &unweighted),
                    expanded(&s, actor, &unweighted, bound)
                );
                // The same original eligibility must hold without the acceleration index.
                s.targets = None;
                assert_eq!(s.choose_target(actor, &p), Some(expected));
                draws.insert(expected);
            }
        }
    }
    assert!(
        draws.len() > 3,
        "the declared seeds exercise multiple original slots"
    );
}

#[test]
fn a_split_actor_is_excluded_once_from_the_original_weighted_cohort() {
    for seed in 0..12 {
        for mode in [Execution::Individuals, Execution::Grouped] {
            for offset in [0, 2, 4] {
                let (g, _, first, _) = fixture(seed, 0);
                let mut s = Simulation::new(g, mode).unwrap();
                start(&mut s);
                let actor = first + offset;
                let bound = s.state.population.next_id;
                let mut p = s.genesis.rules.processes["ecology:feed-1"].clone();
                p.requires.retain(|q| {
                    !matches!(
                        q,
                        Query::Trait {
                            who: Binding::Actor,
                            ..
                        }
                    )
                });
                let expected = expanded(&s, actor, &p, bound).unwrap();
                assert_ne!(expected, actor);
                assert!(s.apply(actor, None, "test:move", None, 1).accepted());
                assert_eq!(s.choose_target(actor, &p), Some(expected));
                assert_eq!(expanded(&s, actor, &p, bound), Some(expected));
            }
        }
    }
}

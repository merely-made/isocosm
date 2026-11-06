// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Case 1145: local food per head, with the counts read at pass start.

use cosmograph::{
    Command, Execution, Founding, Session,
    ecology::{Rates, Web},
    population::Population,
    rules::{Binding, Causation, Effect, Query},
    schema::{Entity, Id, Method, Movement, Provenance},
    simulation::{Genesis, Receipt},
};
use std::collections::{BTreeMap, BTreeSet};

const PROCESS: &str = "test:per-head";
const MARK: &str = "test:fed";

fn set(lineage: usize) -> BTreeSet<String> {
    BTreeSet::from([format!("lineage:{lineage}")])
}

fn ratio(at_least: u64, per_members: u64) -> Query {
    Query::MembersPerHead {
        among: set(0),
        per: Some(set(1)),
        at_least,
        per_members,
    }
}

fn member(prototype: &Entity, lineage: usize, place: Id) -> Entity {
    let mut e = prototype.clone();
    e.lineage = format!("lineage:{lineage}");
    e.provenance = Provenance::Intrinsic(e.lineage.clone());
    e.method = Method::Reactive;
    e.place = place;
    e.alive = true;
    e.journey = None;
    e.traits = BTreeSet::from([format!("ability:cycle-{lineage}")]);
    e.accounts = BTreeMap::from([(format!("matter:{lineage}-0"), 20)]);
    e
}

/// Exact counts in contiguous cohorts; no population-changing process runs
/// unless the test explicitly adds one.
fn world(food: u64, heads: u64, query: Query) -> (Genesis, Id) {
    let mut g = Founding {
        sites: 2,
        population: 6,
        cohort_size: 1,
        lineages: 3,
        ..Founding::default()
    }
    .generate()
    .unwrap();
    let prototype = g
        .population
        .groups
        .values()
        .find(|g| g.entity.lineage.starts_with("lineage:"))
        .unwrap()
        .entity
        .as_ref()
        .clone();
    let mut p = g.rules.processes.values().next().unwrap().clone();
    p.id = PROCESS.into();
    p.causation = Causation::Transition;
    let actor_lineage = if heads == 0 { 2 } else { 1 };
    p.requires = vec![
        Query::Alive(Binding::Actor),
        Query::Trait {
            who: Binding::Actor,
            key: format!("ability:cycle-{actor_lineage}"),
        },
        query,
    ];
    p.commitments.clear();
    p.effects = vec![Effect::Trait {
        who: Binding::Actor,
        key: MARK.into(),
        present: true,
    }];
    p.target = None;
    p.period = Some(1);
    p.priority = 0;
    p.need_account = None;
    p.risk = None;
    p.note = true;
    g.rules.processes.clear();
    g.rules.processes.insert(p.id.clone(), p);
    g.rules.traits.insert(MARK.into());
    g.population = Population::default();
    if food > 0 {
        g.population.insert(member(&prototype, 0, 0), food).unwrap();
    }
    let actor = g
        .population
        .insert(member(&prototype, actor_lineage, 0), heads.max(1))
        .unwrap();
    (g, actor)
}

fn act(g: Genesis, actor: Id, mode: Execution) -> Receipt {
    let mut s = Session::new(g, mode).unwrap();
    act_in(&mut s, actor)
}

fn act_in(s: &mut Session, actor: Id) -> Receipt {
    let outcome = s
        .command(Command::Act {
            actor,
            target: None,
            process: PROCESS.into(),
            cause: None,
        })
        .unwrap();
    serde_json::from_str(&outcome).unwrap()
}

#[test]
fn more_heads_close_the_gate_with_the_same_food() {
    for mode in [Execution::Individuals, Execution::Grouped] {
        for (food, heads, accepted) in [
            (30, 12, true),
            (30, 13, false),
            (60, 24, true),
            (5, 2, true),
            (4, 2, false),
        ] {
            let (g, actor) = world(food, heads, ratio(30, 12));
            let receipt = act(g, actor, mode);
            assert_eq!(
                receipt.accepted(),
                accepted,
                "{food}/{heads}, {mode:?}: {receipt:?}"
            );
        }
    }
}

#[test]
fn zero_heads_never_meet_even_a_zero_threshold() {
    let (g, actor) = world(30, 0, ratio(0, 12));
    let receipt = act(g, actor, Execution::Grouped);
    assert!(!receipt.accepted(), "{receipt:?}");
    assert!(
        format!("{:?}", receipt.outcome).contains("30/0"),
        "{receipt:?}"
    );
}

#[test]
fn a_fork_counts_its_own_heads_after_the_origin_dies_out() {
    for mode in [Execution::Individuals, Execution::Grouped] {
        let query = Query::MembersPerHead {
            among: set(0),
            per: None,
            at_least: 3,
            per_members: 1,
        };
        let bytes = postcard::to_allocvec(&query).unwrap();
        assert_eq!(postcard::from_bytes::<Query>(&bytes).unwrap(), query);
        let (mut g, founder) = world(6, 2, query);
        g.rules.processes.get_mut(PROCESS).unwrap().effects = vec![Effect::Birth {
            provision: BTreeMap::from([("matter:1-0".into(), 1)]),
            scatter: None,
        }];
        let mut s = Session::new(g, mode).unwrap();
        let fork = s
            .command(Command::Speciate {
                founder,
                name: "fork".into(),
            })
            .unwrap();
        s.command(Command::Kill {
            entity: founder + 1,
        })
        .unwrap();
        assert!(
            !s.sim
                .state()
                .population
                .groups
                .values()
                .any(|g| g.entity.alive && g.entity.lineage == "lineage:1")
        );
        // The inherited process still requires the original ability, but
        // its denominator follows the actor's actual lineage.
        assert!(
            s.sim
                .state()
                .population
                .get(founder)
                .unwrap()
                .traits
                .contains("ability:cycle-1")
        );
        for _ in 0..2 {
            let receipt = act_in(&mut s, founder);
            assert!(receipt.accepted(), "{mode:?}: {receipt:?}");
        }
        let receipt = act_in(&mut s, founder);
        assert!(
            !receipt.accepted(),
            "six prey cannot support a fourth head: {receipt:?}"
        );
        let heads: u64 = s
            .sim
            .state()
            .population
            .groups
            .values()
            .filter(|g| g.entity.alive && g.entity.lineage == fork)
            .map(|g| g.count)
            .sum();
        assert_eq!(heads, 3, "{mode:?}");
        assert_eq!(Session::load(s.save(), mode).unwrap().save(), s.save());
    }
}

#[test]
fn only_living_residents_of_the_actor_site_count() {
    for extra_lineage in [0, 1] {
        let (mut g, actor) = world(4, 2, ratio(2 + u64::from(extra_lineage == 0), 1));
        let prototype = g.population.get(actor).unwrap().clone();
        let mut dead = member(&prototype, extra_lineage, 0);
        dead.alive = false;
        g.population.insert(dead, 20).unwrap();
        g.population
            .insert(member(&prototype, extra_lineage, 1), 20)
            .unwrap();
        let mut travel = g.rules.processes[PROCESS].clone();
        travel.id = "test:travel".into();
        travel.requires = vec![Query::Alive(Binding::Actor)];
        travel.effects = vec![Effect::Move {
            destination: 1,
            via: None,
        }];
        travel.period = None;
        g.rules.processes.insert(travel.id.clone(), travel);
        let travellers: Vec<_> = (0..3)
            .map(|_| {
                g.population
                    .insert(member(&prototype, extra_lineage, 0), 1)
                    .unwrap()
            })
            .collect();
        let mut s = Session::new_with_movement(g, Execution::Grouped, Movement::Journey).unwrap();
        for id in travellers {
            let out = s
                .command(Command::Act {
                    actor: id,
                    target: None,
                    process: "test:travel".into(),
                    cause: None,
                })
                .unwrap();
            let receipt: Receipt = serde_json::from_str(&out).unwrap();
            assert!(receipt.accepted(), "{receipt:?}");
            assert!(s.sim.state().population.get(id).unwrap().journey.is_some());
        }
        let receipt = act_in(&mut s, actor);
        assert_eq!(receipt.accepted(), extra_lineage == 1, "{receipt:?}");
    }
}

#[test]
fn counts_are_preserved_for_every_birth_in_one_pass() {
    for mode in [Execution::Individuals, Execution::Grouped] {
        let (mut g, _) = world(6, 2, ratio(3, 1));
        g.rules.processes.get_mut(PROCESS).unwrap().effects = vec![Effect::Birth {
            provision: BTreeMap::from([("matter:1-0".into(), 1)]),
            scatter: None,
        }];
        let mut s = Session::new(g, mode).unwrap();
        s.advance(1).unwrap();
        let heads = |s: &Session| {
            s.sim
                .state()
                .population
                .groups
                .values()
                .filter(|g| g.entity.alive && g.entity.lineage == "lineage:1")
                .map(|g| g.count)
                .sum::<u64>()
        };
        assert_eq!(
            heads(&s),
            4,
            "both parents read six prey and two heads: {mode:?}"
        );
        s.advance(1).unwrap();
        assert_eq!(heads(&s), 4, "the next pass sees four heads: {mode:?}");
        let restored = Session::load(s.save(), mode).unwrap();
        assert_eq!(restored.sim.state_hash(), s.sim.state_hash(), "{mode:?}");
    }
}

#[test]
fn the_gate_round_trips_and_refuses_invalid_ratios() {
    let q = ratio(u64::MAX, u64::MAX);
    let bytes = postcard::to_allocvec(&q).unwrap();
    assert_eq!(postcard::from_bytes::<Query>(&bytes).unwrap(), q);
    // Cross multiplication is wider than the population or threshold types.
    let (g, actor) = world(12, 12, q);
    assert!(act(g, actor, Execution::Individuals).accepted());

    for q in [
        ratio(1, 0),
        Query::MembersPerHead {
            among: BTreeSet::new(),
            per: Some(set(1)),
            at_least: 1,
            per_members: 1,
        },
        Query::MembersPerHead {
            among: set(0),
            per: Some(BTreeSet::new()),
            at_least: 1,
            per_members: 1,
        },
    ] {
        let (g, _) = world(6, 2, q);
        assert!(g.validate().unwrap_err().contains("positive denominator"));
    }
}

#[test]
fn only_world_time_breeding_uses_the_provisional_ratio() {
    for rates in [Rates::Stock, Rates::A1, Rates::WorldTime] {
        let g = Founding {
            ecology: true,
            ecology_rates: rates,
            web: Web::Diets,
            lineages: 5,
            population: 60,
            cohort_size: 1,
            ..Founding::default()
        }
        .generate()
        .unwrap();
        for (lineage, numerator, denominator) in [(1, 30, 12), (3, 10, 4), (4, 30, 6)] {
            let birth = &g.rules.processes[&format!("ecology:birth-{lineage}")];
            let found = birth.requires.iter().find_map(|q| match q {
                Query::MembersPerHead {
                    among,
                    per,
                    at_least,
                    per_members,
                } => Some((among, per, *at_least, *per_members)),
                _ => None,
            });
            if rates == Rates::WorldTime {
                let (among, per, n, d) = found.expect("world-time breeding reads food per head");
                assert!(!among.is_empty());
                assert!(per.is_none(), "a fork must count its own current lineage");
                assert_eq!((n, d), (numerator, denominator));
            } else {
                assert!(found.is_none(), "{rates:?}");
            }
        }
    }
}

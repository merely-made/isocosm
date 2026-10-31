// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Seeded movement ties remain a world rule and are exercised through the
//! ordinary deliberation and scheduled-session paths.

use super::*;
use crate::{
    Execution, Founding, Session,
    rules::{Binding, Causation, Directing, Effect, MovementTies, Query},
};
use std::collections::BTreeSet;

fn movement_fixture(seed: u64) -> (crate::simulation::Genesis, Id, Vec<Key>) {
    let mut genesis = Founding {
        seed,
        sites: 4,
        population: 48,
        cohort_size: 4,
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
    let (actor, home) = genesis
        .population
        .groups
        .iter()
        .find(|(_, group)| group.entity.method == Method::Deliberative)
        .map(|(&id, group)| (id, group.entity.place))
        .expect("played cohort exists");
    assert!(
        genesis.population.groups[&actor].count >= 2,
        "fixture must retain a real cohort"
    );
    let sites: Vec<_> = genesis.sites.keys().copied().collect();
    let destinations: Vec<_> = sites
        .iter()
        .copied()
        .filter(|site| *site != home)
        .take(2)
        .collect();
    assert_eq!(destinations.len(), 2, "fixture needs two other sites");
    // Make the tiny route graph deterministic for every founding seed. The
    // reciprocal edges also let a cohort complete native journeys.
    for &from in &sites {
        for &to in &sites {
            if from == to
                || genesis.sites[&from]
                    .routes
                    .iter()
                    .any(|route| route.to == to)
            {
                continue;
            }
            genesis.sites.get_mut(&from).unwrap().routes.push(Route {
                to,
                travel: 1,
                transmission: 1_000_000,
                border: None,
                authored: None,
            });
        }
    }
    for site in genesis.sites.values_mut() {
        for route in &mut site.routes {
            route.travel = 1;
        }
    }
    let mut selected: Vec<_> = destinations
        .iter()
        .map(|to| format!("move:{home}-{to}"))
        .collect();
    selected.sort();
    let home_key: Key = format!("site:{home}");
    genesis
        .sites
        .get_mut(&home)
        .unwrap()
        .conditions
        .insert(home_key.clone(), 1);
    genesis.rules.conditions.insert(home_key.clone());
    let entity = genesis.population.groups.get_mut(&actor).unwrap().entity_mut();
    let stock = entity.accounts.insert("matter:1-0".into(), 4).unwrap_or(4);
    *entity.accounts.entry("world:soil".into()).or_default() += stock.saturating_sub(4);
    genesis
        .rules
        .processes
        .retain(|_, process| process.causation != Causation::Choice);
    for id in &selected {
        let process = crate::generate::process(
            id,
            Causation::Choice,
            vec![Effect::Move {
                destination: destinations
                    .iter()
                    .copied()
                    .find(|to| format!("move:{home}-{to}") == *id)
                    .unwrap(),
            }],
        );
        let mut process = process;
        process.requires.push(Query::Condition {
            key: home_key.clone(),
            at_least: 1,
        });
        process.requires.push(Query::MoodBelow { amount: 0 });
        process.period = Some(1);
        process.priority = 0;
        genesis.rules.processes.insert(id.clone(), process);
    }
    let directing = genesis
        .rules
        .directing
        .get_or_insert_with(Directing::default);
    directing.movement_ties = MovementTies::Seeded;
    (genesis, actor, selected)
}

fn selected_id(sim: &Simulation, actor: Id) -> Key {
    sim.choose(actor)
        .process
        .expect("at least one feasible move")
}

#[test]
fn movement_ties_repeat_and_vary_with_seed_actor_and_tick() {
    let (genesis, actor, menu) = movement_fixture(19);
    let mut session = Session::new(genesis.clone(), Execution::Individuals).unwrap();
    session.sim.state.tick = 1;
    let first = selected_id(&session.sim, actor);
    assert_eq!(selected_id(&session.sim, actor), first);
    assert!(menu.contains(&first));

    let count = session.sim.state.population.groups[&actor].count;
    let actors: Vec<_> = (actor..actor + count).collect();
    assert!(
        actors.len() >= 2,
        "fixture needs a real multi-member cohort"
    );
    let by_actor: BTreeSet<_> = actors
        .iter()
        .map(|&id| {
            menu.iter()
                .position(|key| key == &selected_id(&session.sim, id))
                .unwrap()
        })
        .collect();
    let by_tick: BTreeSet<_> = (1..=16)
        .map(|tick| {
            session.sim.state.tick = tick;
            menu.iter()
                .position(|key| key == &selected_id(&session.sim, actor))
                .unwrap()
        })
        .collect();
    let by_seed: BTreeSet<_> = (11..=26)
        .map(|seed| {
            let mut g = genesis.clone();
            g.dynamics = Some(seed);
            let mut s = Session::new(g, Execution::Individuals).unwrap();
            s.sim.state.tick = 1;
            menu.iter()
                .position(|key| key == &selected_id(&s.sim, actor))
                .unwrap()
        })
        .collect();
    assert!(by_actor.len() > 1, "actor identity contributes to the draw");
    assert!(by_tick.len() > 1, "tick contributes to the draw");
    assert!(by_seed.len() > 1, "dynamics seed contributes to the draw");
}

#[test]
fn identity_policy_nonmove_and_mixed_ties_keep_identity_order() {
    let (mut genesis, actor, menu) = movement_fixture(29);
    genesis.rules.directing.as_mut().unwrap().movement_ties = MovementTies::Identity;
    let mut session = Session::new(genesis, Execution::Individuals).unwrap();
    session.sim.state.tick = 1;
    assert_eq!(selected_id(&session.sim, actor), menu[0]);

    let (mut genesis, actor, menu) = movement_fixture(31);
    let mut nonmove = crate::generate::process(
        "aaa:nonmove",
        Causation::Choice,
        vec![Effect::Trait {
            who: Binding::Actor,
            key: "test:mark".into(),
            present: true,
        }],
    );
    nonmove.requires = genesis.rules.processes[&menu[0]].requires.clone();
    nonmove.period = Some(1);
    nonmove.priority = 0;
    genesis.rules.traits.insert("test:mark".into());
    genesis.rules.processes.insert(nonmove.id.clone(), nonmove);
    let mut session = Session::new(genesis, Execution::Individuals).unwrap();
    session.sim.state.tick = 1;
    let choice = session.sim.choose(actor);
    assert_eq!(choice.options["aaa:nonmove"].0, choice.options[&menu[0]].0);
    assert_eq!(choice.options[&menu[1]].0, choice.options[&menu[0]].0);
    assert_eq!(choice.process.as_deref(), Some("aaa:nonmove"));

    let (mut genesis, actor, _) = movement_fixture(32);
    let requirement = genesis
        .rules
        .processes
        .values()
        .find(|process| process.causation == Causation::Choice)
        .unwrap()
        .requires
        .clone();
    genesis
        .rules
        .processes
        .retain(|_, process| process.causation != Causation::Choice);
    for (id, trait_key) in [("bbb:nonmove", "test:other"), ("aaa:nonmove", "test:mark")] {
        let mut process = crate::generate::process(
            id,
            Causation::Choice,
            vec![Effect::Trait {
                who: Binding::Actor,
                key: trait_key.into(),
                present: true,
            }],
        );
        process.requires = requirement.clone();
        process.period = Some(1);
        process.priority = 0;
        genesis.rules.traits.insert(trait_key.into());
        genesis.rules.processes.insert(process.id.clone(), process);
    }
    let mut session = Session::new(genesis, Execution::Individuals).unwrap();
    session.sim.state.tick = 1;
    assert_eq!(selected_id(&session.sim, actor), "aaa:nonmove");
}

#[test]
fn due_feasible_menu_and_legacy_policy_wire_are_preserved() {
    let (mut genesis, actor, menu) = movement_fixture(37);
    let move_a = genesis.rules.processes.get_mut(&menu[0]).unwrap();
    move_a.period = Some(2);
    let mut infeasible = crate::generate::process(
        "move:invalid-destination",
        Causation::Choice,
        vec![Effect::Move {
            destination: genesis.population.get(actor).unwrap().place,
        }],
    );
    infeasible.period = Some(1);
    infeasible.priority = 0;
    genesis
        .rules
        .processes
        .insert(infeasible.id.clone(), infeasible);
    let mut session = Session::new(genesis, Execution::Individuals).unwrap();
    session.sim.state.tick = 1;
    let considered = session.sim.consider(actor, false);
    assert!(considered.options.contains_key(&menu[0]));
    assert!(
        !session.sim.choose(actor).options.contains_key(&menu[0]),
        "off-cadence movement is excluded"
    );
    assert!(
        !session
            .sim
            .choose(actor)
            .options
            .contains_key("move:invalid-destination"),
        "an infeasible movement is excluded from the scored menu"
    );

    let mut value = serde_json::to_value(Directing::default()).unwrap();
    value.as_object_mut().unwrap().remove("movement_ties");
    let old: Directing = serde_json::from_value(value).unwrap();
    assert_eq!(old.movement_ties, MovementTies::Identity);
    assert!(
        !serde_json::to_string(&old)
            .unwrap()
            .contains("movement_ties")
    );
    let mut seeded = old;
    seeded.movement_ties = MovementTies::Seeded;
    let wire = serde_json::to_string(&seeded).unwrap();
    assert!(wire.contains("\"movement_ties\":\"Seeded\""));
    assert_eq!(serde_json::from_str::<Directing>(&wire).unwrap(), seeded);
    let generated = Founding {
        seed: 43,
        sites: 4,
        population: 48,
        cohort_size: 4,
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
    assert_eq!(
        generated.rules.directing().movement_ties,
        MovementTies::Seeded
    );
}

#[test]
fn cohort_movement_matches_execution_modes_and_native_session_replay() {
    let (genesis, _, menu) = movement_fixture(41);
    let run = |mode| {
        let mut session = Session::new(genesis.clone(), mode).unwrap();
        for id in &menu {
            session.sim.watch(id);
        }
        let mut hashes = Vec::new();
        let mut acts = Vec::new();
        for _ in 0..24 {
            session.advance(1).unwrap();
            hashes.push(session.sim.state_hash());
            acts.extend(session.sim.take_watched());
        }
        assert!(
            !acts.is_empty(),
            "at least one movement act must be accepted"
        );
        assert!(
            acts.iter()
                .all(|act| menu.contains(&act.process) && act.count == 1)
        );
        assert!(
            acts.iter()
                .map(|act| act.actor)
                .collect::<BTreeSet<_>>()
                .len()
                >= 2,
            "movement is visited for multiple members of the real cohort"
        );
        let replay = Session::load(session.save(), mode).unwrap();
        assert_eq!(replay.sim.state_hash(), session.sim.state_hash());
        (hashes, acts)
    };
    let individuals = run(Execution::Individuals);
    let grouped = run(Execution::Grouped);
    assert_eq!(individuals, grouped);
}

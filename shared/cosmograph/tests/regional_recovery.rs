// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Controlled regional recovery through native movement, with an otherwise
//! identical disconnected control. This qualifies dispersal and the observer,
//! not an unattended ecosystem's long-term stability or physical terrain.

use cosmograph::{
    Command, Execution, Founding, Session, anatomy,
    population::Population,
    rules::{AccountKind, Binding, Causation, Effect, Query},
    schema::{Id, Movement},
    simulation::Genesis,
};
use std::collections::{BTreeMap, BTreeSet};

const MIGRANT: &str = "test:migrant";
const SOURCE: &str = "test:source";
const RETURN: &str = "test:regional-return";
const LEVELS: [&str; 3] = ["life:producer", "life:consumer", "life:decomposer"];

fn world(connected: bool) -> (Genesis, Vec<Id>) {
    let mut g = Founding {
        seed: 9,
        sites: 2,
        population: 30,
        cohort_size: 1,
        lineages: 3,
        ecology: true,
        ..Founding::default()
    }
    .generate()
    .unwrap();
    let prototypes: Vec<_> = (0..3)
        .map(|i| {
            g.population
                .groups
                .values()
                .find(|group| group.entity.lineage == format!("lineage:{i}"))
                .unwrap()
                .entity
                .as_ref()
                .clone()
        })
        .collect();
    let mut migration = g.rules.processes.values().next().unwrap().clone();
    migration.id = RETURN.into();
    migration.causation = Causation::Transition;
    migration.period = Some(1);
    migration.risk = None;
    migration.target = None;
    migration.port_feeding = None;
    migration.need_account = None;
    migration.commitments.clear();
    migration.requires = vec![
        Query::Alive(Binding::Actor),
        Query::Trait {
            who: Binding::Actor,
            key: MIGRANT.into(),
        },
        Query::Condition {
            key: SOURCE.into(),
            at_least: 1,
        },
    ];
    migration.effects = vec![Effect::Move {
        destination: 0,
        via: None,
    }];
    g.rules.processes = BTreeMap::from([(RETURN.into(), migration)]);
    g.rules.traits.insert(MIGRANT.into());
    g.rules.conditions.insert(SOURCE.into());
    for (&id, site) in &mut g.sites {
        site.conditions.insert(SOURCE.into(), i64::from(id == 1));
        if !connected {
            site.routes.clear();
        }
    }
    assert!(connected == !g.sites[&1].routes.is_empty());
    g.population = Population::default();
    let mut local = Vec::new();
    for mut e in prototypes {
        e.place = 0;
        e.method = cosmograph::schema::Method::Reactive;
        local.push(g.population.insert(e.clone(), 1).unwrap());
        e.place = 1;
        // The stationary source remains viable when this one moves.
        g.population.insert(e.clone(), 1).unwrap();
        e.traits.insert(MIGRANT.into());
        g.population.insert(e, 1).unwrap();
    }
    (g, local)
}

fn levels(s: &Session, sites: &[Id]) -> BTreeSet<String> {
    s.sim
        .state()
        .population
        .groups
        .values()
        .filter(|g| g.entity.alive && sites.contains(&g.entity.place) && g.entity.journey.is_none())
        .flat_map(|g| {
            LEVELS
                .iter()
                .filter(|level| g.entity.traits.contains(**level))
                .map(|s| s.to_string())
        })
        .collect()
}

fn matter(s: &Session) -> u128 {
    let rules = &s.sim.genesis().rules;
    let mass = |ledger: &BTreeMap<String, u64>| {
        ledger
            .iter()
            .filter(|(key, _)| matches!(rules.accounts.get(*key), Some(AccountKind::Matter { .. })))
            .map(|(_, n)| u128::from(*n))
            .sum::<u128>()
    };
    s.sim
        .state()
        .population
        .groups
        .values()
        .map(|g| mass(&anatomy::books(&g.entity)) * u128::from(g.count))
        .sum::<u128>()
        + s.sim
            .state()
            .sites
            .values()
            .map(|site| mass(&site.accounts))
            .sum::<u128>()
}

#[test]
fn local_loss_recovers_only_through_a_live_connected_source() {
    let all: BTreeSet<_> = LEVELS.into_iter().map(String::from).collect();
    for mode in [Execution::Individuals, Execution::Grouped] {
        for connected in [false, true] {
            let (g, locals) = world(connected);
            let mut s = Session::new_with_movement(g, mode, Movement::ImmediateV3).unwrap();
            let initial_matter = matter(&s);
            assert_eq!(levels(&s, &[0]), all);
            for id in locals {
                s.command(Command::Kill { entity: id }).unwrap();
            }
            assert!(
                levels(&s, &[0]).is_empty(),
                "actual local loss must precede return"
            );
            assert_eq!(levels(&s, &[1]), all, "the regional source survives");
            let living = |s: &Session| {
                s.sim
                    .state()
                    .population
                    .groups
                    .values()
                    .filter(|g| g.entity.alive)
                    .map(|g| g.count)
                    .sum::<u64>()
            };
            assert_eq!(living(&s), 6);
            let sources: BTreeSet<_> = s
                .sim
                .state()
                .population
                .groups
                .iter()
                .filter(|(_, g)| g.entity.alive && g.entity.traits.contains(MIGRANT))
                .map(|(&id, _)| id)
                .collect();
            s.advance(2).unwrap();
            assert_eq!(living(&s), 6, "recovery moves existing population");
            assert_eq!(
                matter(&s),
                initial_matter,
                "movement and death conserve matter"
            );
            assert_eq!(
                levels(&s, &[1]),
                all,
                "recovery does not consume the source"
            );
            assert_eq!(
                levels(&s, &[0, 1]),
                all,
                "regional function survives local loss"
            );
            if connected {
                assert_eq!(levels(&s, &[0]), all);
                let arrivals: BTreeSet<_> = s
                    .sim
                    .state()
                    .population
                    .groups
                    .iter()
                    .filter(|(_, g)| g.entity.alive && g.entity.place == 0)
                    .map(|(&id, _)| id)
                    .collect();
                assert_eq!(
                    arrivals, sources,
                    "the original native migrants provide recovery"
                );
            } else {
                assert!(
                    levels(&s, &[0]).is_empty(),
                    "no route means no invented replacement"
                );
            }
            let restored = Session::load(s.save(), mode).unwrap();
            assert_eq!(restored.sim.state_hash(), s.sim.state_hash());
        }
    }
}

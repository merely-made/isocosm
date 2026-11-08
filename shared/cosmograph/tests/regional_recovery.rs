// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Controlled regional recovery through native movement, with disconnected
//! and absent-source controls. The fixture uses two sites and an authored
//! movement process. Its scope is dispersal, residence and replay.

use cosmograph::{
    Command, Execution, Founding, Session, anatomy,
    population::Population,
    rules::{AccountKind, Binding, Causation, Effect, Query},
    schema::{Id, Journey, Movement},
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

fn residents(session: &Session, site: Id) -> BTreeSet<Id> {
    session
        .sim
        .state()
        .population
        .groups
        .iter()
        .filter(|(_, group)| group.entity.alive && group.entity.resident_site() == Some(site))
        .flat_map(|(&first, group)| first..first + group.count)
        .collect()
}

#[test]
fn routed_recovery_counts_only_actual_arrivals_and_replays_in_transit() {
    const TRAVEL: u64 = 3;

    for mode in [Execution::Individuals, Execution::Grouped] {
        for (connected, has_source) in [(true, true), (false, true), (true, false)] {
            let (mut genesis, locals) = world(connected);
            let border = if connected {
                let route = genesis
                    .sites
                    .get_mut(&1)
                    .unwrap()
                    .routes
                    .iter_mut()
                    .find(|route| route.to == 0)
                    .unwrap();
                route.travel = TRAVEL;
                route.border
            } else {
                None
            };
            let departure = genesis.started + 1;
            let mut session = Session::new(genesis, mode).unwrap();
            assert_eq!(session.sim.state().movement, Movement::Routed);
            let initial_matter = matter(&session);
            let first_unused_id = session.sim.state().population.next_id;

            for id in locals {
                session.command(Command::Kill { entity: id }).unwrap();
            }
            if !has_source {
                for id in residents(&session, 1) {
                    session.command(Command::Kill { entity: id }).unwrap();
                }
            }
            assert!(
                residents(&session, 0).is_empty(),
                "the local population died out"
            );
            let source = residents(&session, 1);
            assert_eq!(source.len(), if has_source { 6 } else { 0 });
            let donors: BTreeSet<_> = source
                .iter()
                .copied()
                .filter(|&id| {
                    session
                        .sim
                        .state()
                        .population
                        .get(id)
                        .unwrap()
                        .traits
                        .contains(MIGRANT)
                })
                .collect();
            assert_eq!(donors.len(), if has_source { 3 } else { 0 });
            let expected = if connected { donors } else { BTreeSet::new() };
            let remaining: BTreeSet<_> = source.difference(&expected).copied().collect();

            session.sim.watch(RETURN);
            assert_eq!(session.advance(1).unwrap().arrivals, 0);
            let acts = session.sim.take_watched();
            assert_eq!(
                acts.len(),
                expected.len(),
                "one accepted departure per donor"
            );
            let departed: BTreeSet<_> = acts
                .iter()
                .map(|act| {
                    assert_eq!(
                        (act.process.as_str(), act.tick, act.count),
                        (RETURN, departure, 1)
                    );
                    act.actor
                })
                .collect();
            assert_eq!(departed, expected);
            for &id in &expected {
                let entity = session.sim.state().population.get(id).unwrap();
                assert_eq!(entity.resident_site(), None);
                assert_eq!(
                    entity.journey,
                    Some(Journey {
                        from: 1,
                        to: 0,
                        departed: departure,
                        arrives: departure + TRAVEL,
                        border
                    }),
                );
            }
            assert!(residents(&session, 0).is_empty());
            assert_eq!(residents(&session, 1), remaining);
            assert_eq!(matter(&session), initial_matter);

            let mut replayed = Session::load(session.save(), mode).unwrap();
            assert_eq!(replayed.sim.state_hash(), session.sim.state_hash());
            assert_eq!(replayed.advance(TRAVEL - 1).unwrap().arrivals, 0);
            assert!(
                residents(&replayed, 0).is_empty(),
                "transit is not recovery"
            );
            assert_eq!(residents(&replayed, 1), remaining);
            assert_eq!(matter(&replayed), initial_matter);

            let arrivals = replayed.advance(1).unwrap().arrivals;
            assert_eq!(arrivals, expected.len() as u64);
            assert_eq!(residents(&replayed, 0), expected);
            assert_eq!(residents(&replayed, 1), remaining);
            assert_eq!(matter(&replayed), initial_matter);
            assert_eq!(replayed.sim.state().population.next_id, first_unused_id);
            session.advance(TRAVEL).unwrap();
            assert_eq!(replayed.sim.state_hash(), session.sim.state_hash());
            eprintln!(
                "{mode:?} connected={connected} source={has_source}: source members={source:?}, \
                 departures={departed:?}, arrivals={arrivals} on route 1→0 by tick {}, matter={initial_matter}",
                replayed.sim.state().tick,
            );
        }
    }
}

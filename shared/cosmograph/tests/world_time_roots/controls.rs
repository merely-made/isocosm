// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use cosmograph::rules::{Binding, Effect, Query};

#[test]
fn healthy_plants_are_not_death_evaluated_and_last_part_loss_files_death() {
    for mode in [Execution::Individuals, Execution::Grouped] {
        for foreign in [false, true] {
            let (mut g, plant, grazer) = fixture_genesis(u64::from(!foreign), 0);
            let mut drain = g.rules.processes[if foreign {
                "ecology:feed-1"
            } else {
                "ecology:upkeep-0"
            }]
            .clone();
            drain.period = None;
            if foreign {
                let e = g.population.groups.get_mut(&plant).unwrap().entity_mut();
                let part = *e.parts.keys().find(|&&id| !store(e, id)).unwrap();
                e.part_mut(&part)
                    .unwrap()
                    .matter
                    .insert("matter:1-0".into(), 1);
                drain.id = "test:drain-foreign-part".into();
                drain.port_feeding = None;
                drain.requires = vec![
                    Query::Alive(Binding::Actor),
                    Query::Trait {
                        who: Binding::Actor,
                        key: "ability:cycle-1".into(),
                    },
                    Query::Holds {
                        who: Binding::Target,
                        at_least: 1,
                    },
                ];
                drain.effects = vec![Effect::Eat {
                    from: Binding::Target,
                    amount: 1.into(),
                    into: "matter:1-0".into(),
                    of: vec!["matter:1-0".into()],
                    whole: false,
                }];
            }
            let process = drain.id.clone();
            // Only the ordinary death definition runs on the clock. The
            // nonperiodic native act removes the last held unit on demand.
            g.rules.processes.retain(|id, _| id == "ecology:death-0");
            g.rules.processes.insert(process.clone(), drain);
            let mut s = Session::new(g, mode).unwrap();
            let before = s.sim.matter();
            let work = s.advance(10).unwrap();
            assert_eq!(
                work.evaluations, 0,
                "healthy body must be filed out, {mode:?}, foreign={foreign}: {work:?}"
            );
            assert!(s.sim.state().population.get(plant).unwrap().alive);
            let actor = if foreign { grazer } else { plant };
            let r = act(&mut s, actor, foreign.then_some(plant), &process);
            assert!(r.accepted(), "last part loss, foreign={foreign}: {r:?}");
            assert_eq!(at(&s, plant), (0, 0));
            let work = s.advance(1).unwrap();
            assert_eq!((work.evaluations, work.accepted), (1, 1), "{work:?}");
            assert!(!s.sim.state().population.get(plant).unwrap().alive);
            assert_eq!(s.advance(10).unwrap().evaluations, 0);
            assert_eq!(s.sim.matter(), before);
            assert_eq!(Session::load(s.save(), mode).unwrap().save(), s.save());
        }
    }
}

#[test]
fn foreign_matter_in_a_part_keeps_an_own_tissue_empty_plant_alive() {
    for mode in [Execution::Individuals, Execution::Grouped] {
        let (mut g, plant, _) = fixture_genesis(0, 0);
        let e = g.population.groups.get_mut(&plant).unwrap().entity_mut();
        let root = e.body.as_ref().unwrap().root;
        e.part_mut(&root)
            .unwrap()
            .matter
            .insert("matter:1-0".into(), 1);
        let mut s = Session::new(g, mode).unwrap();
        assert_eq!(at(&s, plant), (0, 0));
        let r = act(&mut s, plant, None, "ecology:death-0");
        assert!(!r.accepted(), "an actual part still holds matter: {r:?}");
        assert!(s.sim.state().population.get(plant).unwrap().alive);
        assert_eq!(Session::load(s.save(), mode).unwrap().save(), s.save());
    }
}

#[test]
fn unrooted_rate_models_omit_the_contract_and_round_trip_their_digest() {
    for rates in [Rates::Stock, Rates::A1] {
        let g = Founding {
            seed: 9,
            sites: 4,
            population: 60,
            lineages: 5,
            ecology: true,
            ecology_rates: rates,
            experimental_food_contract: rates == Rates::WorldTime,
            web: Web::Diets,
            bodies: Some(Bodies::default()),
            ..Founding::default()
        }
        .generate()
        .unwrap();
        assert!(g.rules.root_stores.is_empty(), "{rates:?}");
        let json = serde_json::to_string(&g).unwrap();
        assert!(!json.contains("\"root_stores\""), "{rates:?}");
        let back: Genesis = serde_json::from_str(&json).unwrap();
        assert_eq!(cosmograph::digest(&back), cosmograph::digest(&g));
        assert_eq!(back.rules.revision(), g.rules.revision());
        for mode in [Execution::Individuals, Execution::Grouped] {
            let s = Session::new(back.clone(), mode).unwrap();
            let saved = s.save();
            let restored = Session::load(saved.clone(), mode).unwrap().save();
            assert_eq!(cosmograph::digest(&restored), cosmograph::digest(&saved));
        }
    }
}

#[test]
fn a_forced_sensing_founding_rejects_an_unset_smell_radius() {
    let result = Founding {
        seed: 9,
        sites: 4,
        population: 60,
        lineages: 5,
        ecology: true,
        ecology_rates: Rates::WorldTime,
        experimental_food_contract: true,
        web: Web::Diets,
        bodies: Some(Bodies {
            sensing: false,
            smell: [0, 0],
            ..Bodies::default()
        }),
        ..Founding::default()
    }
    .generate();
    assert!(
        matches!(result, Err(ref why) if why.contains("smell radius")),
        "the effective sensing bounds must return an error: {result:?}"
    );
}

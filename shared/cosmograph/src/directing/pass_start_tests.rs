// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Choice reads the current pass's start, independent of actor order (454).
//! A later pass is a new start: empty choices can be reconsidered (813),
//! while any selected process stays committed for the tick.

use super::*;
use crate::{
    Execution, Founding, Session, history::Command, population::Population, rules::*,
    simulation::Genesis,
};
use std::collections::BTreeMap;

fn fixture(first_actor: bool, incoming: bool, non_move: bool) -> (Genesis, Id, Id) {
    fixture_sites(first_actor, incoming, non_move, 2)
}

fn fixture_sites(
    first_actor: bool,
    incoming: bool,
    non_move: bool,
    sites: u32,
) -> (Genesis, Id, Id) {
    let g = Founding {
        seed: 3,
        sites,
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
    let mut g = super::tests::unpaced(g);
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
    actor.place = Id::from(incoming);
    actor.accounts = Ledger::from([("matter:1-0".into(), 3)]);
    prey.place = 0;
    prey.accounts = Ledger::from([("matter:0-0".into(), 10)]);
    prey.traits.insert("test:edible".into());
    g.rules.traits.insert("test:edible".into());
    g.population = Population::default();
    let (actor, prey) = if first_actor {
        let actor = g.population.insert(actor, 1).unwrap();
        (actor, g.population.insert(prey, 2).unwrap())
    } else {
        let prey = g.population.insert(prey, 2).unwrap();
        (g.population.insert(actor, 1).unwrap(), prey)
    };
    g.rules.processes.retain(|id, _| id == "ecology:feed-1");
    let feed = g.rules.processes.get_mut("ecology:feed-1").unwrap();
    feed.period = Some(1);
    feed.requires.push(Query::Trait {
        who: Binding::Target,
        key: "test:edible".into(),
    });
    let effects = if non_move {
        vec![Effect::Trait {
            who: Binding::Actor,
            key: "test:edible".into(),
            present: false,
        }]
    } else {
        vec![Effect::Move { destination: 1 }]
    };
    let mut early = crate::generate::process("test:early", Causation::Choice, effects);
    early.requires.push(Query::Condition {
        key: "site:0".into(),
        at_least: 1,
    });
    early.period = Some(1);
    early.priority = -20;
    g.rules.processes.insert(early.id.clone(), early);
    (g, actor, prey)
}

fn replay(s: &Session) {
    for mode in [Execution::Individuals, Execution::Grouped] {
        assert_eq!(
            Session::load(s.save(), mode).unwrap().sim.state_hash(),
            s.sim.state_hash()
        );
    }
}

fn watched(g: Genesis, mode: Execution) -> Session {
    let mut s = Session::new(g, mode).unwrap();
        s.sim.state.movement = Movement::ImmediateV3;
    s.sim.watch("test:early");
    s.sim.watch("ecology:feed-1");
    s
}

#[test]
fn same_pass_arrivals_wait_for_a_later_pass_before_selecting_a_process() {
    for first_actor in [true, false] {
        let (g, actor, prey) = fixture(first_actor, true, false);
        let mut hashes = Vec::new();
        for mode in [Execution::Individuals, Execution::Grouped] {
            for _ in 0..2 {
                let mut s = watched(g.clone(), mode);
                let matter = s.sim.matter();
                let flows = s.advance_tick_with_flows().unwrap().flows;
                assert_eq!(s.sim.matter(), matter);
                assert!(
                    flows
                        .iter()
                        .any(|flow| flow.to.0.body() == Some(actor) && flow.amount == 1)
                );
                assert_eq!(s.sim.state().population.get(prey).unwrap().place, 1);
                assert_eq!(
                    s.sim.state().population.get(actor).unwrap().accounts["matter:1-0"],
                    4
                );
                let acts: Vec<_> = s
                    .sim
                    .take_watched()
                    .into_iter()
                    .filter(|w| w.actor == actor)
                    .collect();
                assert_eq!(acts.len(), 1, "only the later feeding pass selects an act");
                assert_eq!(acts[0].process, "ecology:feed-1");
                assert_eq!(acts[0].tick, 1);
                assert_eq!(s.sim.state().population.get(actor).unwrap().place, 1);
                replay(&s);
                hashes.push(s.sim.state_hash());
            }
        }
        assert!(hashes.iter().all(|h| *h == hashes[0]));
    }
}

mod empty;

#[test]
fn same_pass_departures_do_not_make_the_last_actor_select_a_second_process() {
    for first_actor in [true, false] {
        let (g, actor, prey) = fixture(first_actor, false, false);
        for mode in [Execution::Individuals, Execution::Grouped] {
            let mut s = watched(g.clone(), mode);
            let matter = s.sim.matter();
            s.advance(1).unwrap();
            assert_eq!(s.sim.matter(), matter);
            assert_eq!(s.sim.state().population.get(prey).unwrap().place, 1);
            assert_eq!(
                s.sim.state().population.get(actor).unwrap().place,
                0,
                "feed chosen at pass start"
            );
            assert!(
                !s.sim.take_watched().iter().any(|w| w.actor == actor),
                "feeding blocked in its later pass; no move fallback"
            );
            replay(&s);
        }
    }
}

#[test]
fn targetless_non_move_choice_preserves_other_options_target_requirements() {
    for first_actor in [true, false] {
        let (g, actor, prey) = fixture(first_actor, false, true);
        for mode in [Execution::Individuals, Execution::Grouped] {
            let mut s = watched(g.clone(), mode);
            let matter = s.sim.matter();
            s.advance(1).unwrap();
            assert_eq!(s.sim.matter(), matter);
            assert!(
                !s.sim
                    .state()
                    .population
                    .get(prey)
                    .unwrap()
                    .traits
                    .contains("test:edible")
            );
            assert!(
                !s.sim.take_watched().iter().any(|w| w.actor == actor),
                "one scored meal; no eligibility-strip fallback"
            );
            replay(&s);
        }
    }
}

#[test]
fn an_explicit_prey_nudge_stays_bound_after_a_same_pass_departure() {
    for first_actor in [true, false] {
        let (g, actor, prey) = fixture(first_actor, false, false);
        for mode in [Execution::Individuals, Execution::Grouped] {
            let mut s = watched(g.clone(), mode);
            let (participant, played) = super::tests::played(&mut s);
            assert_eq!(played, actor);
            s.command(Command::Nudge {
                participant,
                critter: actor,
                aim: Aim::Act,
                toward: Toward::Thing(prey),
                act: Some("ecology:feed-1".into()),
            })
            .unwrap();
            let matter = s.sim.matter();
            s.advance(1).unwrap();
            assert_eq!(s.sim.matter(), matter);
            assert_eq!(s.sim.state().population.get(actor).unwrap().place, 0);
            assert!(!s.sim.take_watched().iter().any(|w| w.actor == actor));
            assert!(!s.sim.state().nudges[0].answer.as_ref().unwrap().served);
            assert_eq!(s.sim.bond(participant, actor), Some(200));
            replay(&s);
        }
    }
}

#[test]
fn attending_to_a_thing_scores_its_pass_start_location() {
    for first_actor in [true, false] {
        let (mut g, actor, prey) = fixture(first_actor, false, false);
        // Without hunger, the nudge's site can decide between feed and move.
        g.population
            .lift(actor)
            .unwrap()
            .accounts
            .insert("matter:1-0".into(), 10);
        for mode in [Execution::Individuals, Execution::Grouped] {
            let mut s = watched(g.clone(), mode);
            let (participant, _) = super::tests::played(&mut s);
            s.command(Command::Nudge {
                participant,
                critter: actor,
                aim: Aim::Attend,
                toward: Toward::Thing(prey),
                act: None,
            })
            .unwrap();
            let matter = s.sim.matter();
            s.advance(1).unwrap();
            assert_eq!(s.sim.matter(), matter);
            assert_eq!(s.sim.state().population.get(prey).unwrap().place, 1);
            assert_eq!(
                s.sim.state().population.get(actor).unwrap().place,
                0,
                "the old thing location favors feeding in both actor orders"
            );
            assert!(!s.sim.take_watched().iter().any(|w| w.actor == actor));
            assert!(
                s.sim.state().nudges[0].answer.is_none(),
                "the later feeding pass cannot credit attention to moved prey"
            );
            assert_eq!(s.sim.bond(participant, actor), Some(250));
            replay(&s);
        }
    }
}

fn frame(s: &mut Session) {
    s.sim.state.tick = 1;
    s.sim.pass = Some(crate::schedule::Pass::new(&s.sim.state.population));
    s.sim.frame = Some(crate::schedule::Frame::default());
    s.sim.targets = Some(crate::targets::Targets::new(&s.sim.state.population));
}

#[test]
fn framed_need_scoring_keeps_old_body_and_site_but_wellbeing_reads_live() {
    for mode in [Execution::Individuals, Execution::Grouped] {
        let (mut g, actor, prey) = fixture(false, false, false);
        g.population
            .lift(actor)
            .unwrap()
            .accounts
            .insert("matter:1-0".into(), 9);
        let mut s = watched(g, mode);
        frame(&mut s);
        let score = s.sim.choose(actor).options["ecology:feed-1"].0;
        assert_eq!(s.sim.wellbeing(actor).0, -10);
        let matter = s.sim.matter();
        assert!(
            s.sim
                .apply(actor, Some(prey), "ecology:feed-1", None, 1)
                .accepted()
        );
        assert_eq!(s.sim.matter(), matter);
        assert_eq!(
            s.sim.choose(actor).options["ecology:feed-1"].0,
            score,
            "choice still scores the hungry pass-start body"
        );
        assert_eq!(
            s.sim.wellbeing(actor).0,
            0,
            "landed food satisfied live hunger"
        );

        let (mut g, actor, prey) = fixture(false, false, false);
        g.population
            .lift(actor)
            .unwrap()
            .accounts
            .insert("matter:1-0".into(), 10);
        g.rules.conditions.insert("test:pressure".into());
        for site in g.sites.values_mut() {
            site.conditions.insert("test:pressure".into(), 0);
        }
        g.rules.mind.as_mut().unwrap().needs = vec![Need {
            traits: BTreeSet::from(["ability:cycle-1".into()]),
            query: Query::Condition {
                key: "test:pressure".into(),
                at_least: 1,
            },
            weight: -10,
        }];
        g.rules.processes.get_mut("test:early").unwrap().effects = vec![Effect::Condition {
            key: "test:pressure".into(),
            delta: 1,
        }];
        let mut s = watched(g, mode);
        frame(&mut s);
        let score = s.sim.choose(actor).options["ecology:feed-1"].0;
        assert_eq!(s.sim.wellbeing(actor).0, 0);
        assert!(s.sim.apply(prey, None, "test:early", None, 1).accepted());
        assert_eq!(
            s.sim.choose(actor).options["ecology:feed-1"].0,
            score,
            "choice still scores the unpressured pass-start site"
        );
        assert_eq!(s.sim.wellbeing(actor).0, -10, "live site pressure is real");
    }
}

#[test]
fn answered_bond_credits_live_mood_relief_despite_its_matter_cost() {
    let (mut g, actor, _) = fixture(false, false, false);
    g.population
        .lift(actor)
        .unwrap()
        .accounts
        .insert("matter:1-0".into(), 10);
    g.rules.conditions.insert("test:pressure".into());
    for site in g.sites.values_mut() {
        site.conditions.insert("test:pressure".into(), 1);
    }
    g.rules.mind.as_mut().unwrap().needs = vec![Need {
        traits: BTreeSet::from(["ability:cycle-1".into()]),
        query: Query::Condition {
            key: "test:pressure".into(),
            at_least: 1,
        },
        weight: -10,
    }];
    let mut relief = crate::generate::process(
        "test:relief",
        Causation::Choice,
        vec![
            Effect::Transform {
                who: Binding::Actor,
                take: Ledger::from([("matter:1-0".into(), 1)]),
                give: Ledger::from([("world:soil".into(), 1)]),
                conversion: None,
            },
            Effect::Transfer {
                from: Binding::Actor,
                to: Binding::Place,
                account: "world:soil".into(),
                amount: 1.into(),
            },
            Effect::Condition {
                key: "test:pressure".into(),
                delta: -1,
            },
        ],
    );
    relief.requires.push(Query::Trait {
        who: Binding::Actor,
        key: "ability:cycle-1".into(),
    });
    relief.requires.push(Query::Condition {
        key: "test:pressure".into(),
        at_least: 1,
    });
    relief.period = Some(1);
    g.rules.processes = BTreeMap::from([(relief.id.clone(), relief)]);
    for mode in [Execution::Individuals, Execution::Grouped] {
        let mut s = Session::new(g.clone(), mode).unwrap();
        s.sim.state.movement = Movement::ImmediateV3;
        let (participant, _) = super::tests::played(&mut s);
        s.command(Command::Nudge {
            participant,
            critter: actor,
            aim: Aim::Act,
            toward: Toward::Site(0),
            act: Some("test:relief".into()),
        })
        .unwrap();
        assert_eq!(s.sim.wellbeing(actor), (-10, 10));
        let matter = s.sim.matter();
        s.advance(1).unwrap();
        assert_eq!(s.sim.matter(), matter);
        assert_eq!(s.sim.wellbeing(actor), (0, 9));
        assert!(s.sim.state().nudges[0].answer.as_ref().unwrap().served);
        assert_eq!(
            s.sim.bond(participant, actor),
            Some(300),
            "live mood improves despite cost; framed scoring must not freeze credit"
        );
        replay(&s);
    }
}

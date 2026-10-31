// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! An empty cache can change only at a later pass start (813).
//! Scoring still considers every due Choice; selected acts stay committed.

use super::*;

#[test]
fn an_empty_choice_is_reused_until_the_next_pass_begins() {
    for mode in [Execution::Individuals, Execution::Grouped] {
        let (g, actor, prey) = fixture(false, true, false);
        let mut s = watched(g, mode);
        let feed = s.sim.genesis().rules.processes["ecology:feed-1"].clone();
        frame(&mut s);
        assert!(matches!(
            s.sim.deliberate(actor, &feed),
            Deliberated::Foregone
        ));

        // Isolate the cache boundary: a native move between frames changes
        // feasibility, but installing a view alone does not invalidate None.
        // These unlogged unit calls are separate from the Session/replay controls.
        s.sim.pass = None;
        s.sim.frame = None;
        let matter = s.sim.matter();
        assert!(s.sim.apply(prey, None, "test:early", None, 1).accepted());
        assert_eq!(s.sim.matter(), matter);
        frame(&mut s);
        assert_eq!(
            s.sim.choose(actor).process.as_deref(),
            Some("ecology:feed-1")
        );
        assert!(
            matches!(s.sim.deliberate(actor, &feed), Deliberated::Foregone),
            "planning and visiting reuse the same empty result until invalidation"
        );
        s.sim.chosen.begin_pass();
        let target = s.sim.deliberate(actor, &feed).target().unwrap();
        assert_eq!(target, prey);
        assert_eq!(
            s.sim.deliberate(actor, &feed).target(),
            Some(target),
            "planning and visiting share the newly selected prey"
        );
        s.sim.chosen.begin_pass();
        assert_eq!(
            s.sim.deliberate(actor, &feed).target(),
            Some(target),
            "a selected process remains committed at later pass starts"
        );
    }
}

#[test]
fn an_actor_can_reconsider_after_more_than_one_empty_pass() {
    for first_actor in [true, false] {
        let (mut g, actor, prey) = fixture_sites(first_actor, false, false, 3);
        g.population.lift(actor).unwrap().place = 2;
        let mut later = crate::generate::process(
            "test:later",
            Causation::Choice,
            vec![Effect::Move { destination: 2 }],
        );
        later.requires.push(Query::Condition {
            key: "site:1".into(),
            at_least: 1,
        });
        later.period = Some(1);
        later.priority = -10;
        g.rules.processes.insert(later.id.clone(), later);
        let mut hashes = Vec::new();
        for mode in [Execution::Individuals, Execution::Grouped] {
            for _ in 0..2 {
                let mut s = watched(g.clone(), mode);
                s.sim.watch("test:later");
                let matter = s.sim.matter();
                let flows = s.advance_tick_with_flows().unwrap().flows;
                assert_eq!(s.sim.matter(), matter);
                assert_eq!(s.sim.state().population.get(prey).unwrap().place, 2);
                assert_eq!(s.sim.state().population.get(actor).unwrap().place, 2);
                assert_eq!(
                    s.sim.state().population.get(actor).unwrap().accounts["matter:1-0"],
                    4
                );
                let watched = s.sim.take_watched();
                let acts: Vec<_> = watched.iter().filter(|w| w.actor == actor).collect();
                assert_eq!(acts.len(), 1);
                assert_eq!(acts[0].process, "ecology:feed-1");
                for process in ["test:early", "test:later"] {
                    assert_eq!(
                        watched
                            .iter()
                            .filter(|w| w.process == process)
                            .map(|w| w.count)
                            .sum::<u64>(),
                        2
                    );
                }
                assert!(
                    flows
                        .iter()
                        .any(|f| f.to.0.body() == Some(actor) && f.amount == 1)
                );
                replay(&s);
                hashes.push(s.sim.state_hash());
            }
        }
        assert!(hashes.iter().all(|h| *h == hashes[0]));
    }
}

#[test]
fn a_later_selected_meal_keeps_the_explicit_nudged_identity_and_credit() {
    for first_actor in [true, false] {
        let (g, actor, prey) = fixture(first_actor, true, false);
        for mode in [Execution::Individuals, Execution::Grouped] {
            let mut s = watched(g.clone(), mode);
            let (participant, ours) = super::super::tests::played(&mut s);
            assert_eq!(ours, actor);
            s.command(Command::Nudge {
                participant,
                critter: actor,
                aim: Aim::Act,
                toward: Toward::Thing(prey),
                act: Some("ecology:feed-1".into()),
            })
            .unwrap();
            let matter = s.sim.matter();
            let flows = s.advance_tick_with_flows().unwrap().flows;
            assert_eq!(s.sim.matter(), matter);
            let acts: Vec<_> = s
                .sim
                .take_watched()
                .into_iter()
                .filter(|w| w.actor == actor)
                .collect();
            assert_eq!(acts.len(), 1);
            assert_eq!(acts[0].process, "ecology:feed-1");
            assert_eq!(acts[0].target, Some(prey));
            assert!(flows.iter().any(|f| f.from.0.body() == Some(prey)
                && f.to.0.body() == Some(actor)
                && f.amount == 1));
            let answer = s.sim.state().nudges[0].answer.as_ref().unwrap();
            assert_eq!(answer.process, "ecology:feed-1");
            assert!(answer.served);
            assert_eq!(s.sim.bond(participant, actor), Some(300));
            replay(&s);
        }
    }
}

#[test]
fn a_selected_process_expires_at_the_next_tick_inside_one_advance() {
    for first_actor in [true, false] {
        let (g, actor, _) = fixture(first_actor, true, false);
        let mut hashes = Vec::new();
        for mode in [Execution::Individuals, Execution::Grouped] {
            let mut s = watched(g.clone(), mode);
            let matter = s.sim.matter();
            s.advance(2).unwrap();
            assert_eq!(s.sim.matter(), matter);
            let acts: Vec<_> = s
                .sim
                .take_watched()
                .into_iter()
                .filter(|w| w.actor == actor)
                .collect();
            assert_eq!(
                acts.iter()
                    .map(|w| (w.tick, w.process.as_str()))
                    .collect::<Vec<_>>(),
                [(1, "ecology:feed-1"), (2, "ecology:feed-1")]
            );
            assert_eq!(
                s.sim.state().population.get(actor).unwrap().accounts["matter:1-0"],
                5
            );
            replay(&s);
            hashes.push(s.sim.state_hash());
        }
        assert_eq!(hashes[0], hashes[1]);
    }
}

#[test]
fn later_passes_still_score_due_processes_whose_pass_already_ran() {
    for first_actor in [true, false] {
        let (mut g, actor, _) = fixture(first_actor, true, false);
        let feed = g.rules.processes.get_mut("ecology:feed-1").unwrap();
        feed.priority = -30;
        let mut last = feed.clone();
        last.id = "test:last".into();
        last.priority = 20;
        last.commitments.clear();
        last.effects = vec![Effect::Trait {
            who: Binding::Actor,
            key: "test:last".into(),
            present: true,
        }];
        g.rules.traits.insert("test:last".into());
        g.rules.processes.insert(last.id.clone(), last);
        for mode in [Execution::Individuals, Execution::Grouped] {
            let mut s = watched(g.clone(), mode);
            s.sim.watch("test:last");
            let matter = s.sim.matter();
            let flows = s.advance_tick_with_flows().unwrap().flows;
            assert_eq!(s.sim.matter(), matter);
            assert!(flows.is_empty());
            let actor_body = s.sim.state().population.get(actor).unwrap();
            assert_eq!(actor_body.accounts["matter:1-0"], 3);
            assert!(!actor_body.traits.contains("test:last"));
            assert!(
                !s.sim.take_watched().iter().any(|w| w.actor == actor),
                "the higher-scored due meal stays selected even though its pass ran earlier"
            );
            replay(&s);
        }
    }
}

#[test]
fn a_selected_blocked_meal_does_not_fall_back_to_a_later_feasible_process() {
    for first_actor in [true, false] {
        let (mut g, actor, _) = fixture(first_actor, false, false);
        let mut last = crate::generate::process(
            "test:last",
            Causation::Choice,
            vec![Effect::Trait {
                who: Binding::Actor,
                key: "test:last".into(),
                present: true,
            }],
        );
        last.requires.push(Query::Trait {
            who: Binding::Actor,
            key: "ability:cycle-1".into(),
        });
        last.period = Some(1);
        last.priority = 30;
        g.rules.traits.insert("test:last".into());
        g.rules.processes.insert(last.id.clone(), last);
        for mode in [Execution::Individuals, Execution::Grouped] {
            let mut s = watched(g.clone(), mode);
            s.sim.watch("test:last");
            let matter = s.sim.matter();
            let flows = s.advance_tick_with_flows().unwrap().flows;
            assert_eq!(s.sim.matter(), matter);
            assert!(flows.is_empty());
            assert!(
                !s.sim
                    .state()
                    .population
                    .get(actor)
                    .unwrap()
                    .traits
                    .contains("test:last"),
                "blocked feeding cannot fall back to the later feasible trait write"
            );
            assert!(!s.sim.take_watched().iter().any(|w| w.actor == actor));
            replay(&s);
        }
    }
}

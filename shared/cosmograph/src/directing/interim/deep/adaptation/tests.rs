// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::{
    Command, Execution,
    directing::policy::{LineagePolicy, UNIT},
    rules::Causation,
};
use std::cell::Cell;

fn session() -> Session {
    session_with_epoch(1)
}

fn session_with_epoch(epoch: Tick) -> Session {
    let mut genesis = crate::directing::tests::founding(3);
    genesis.rules.epoch_ticks = epoch;
    for process in genesis.rules.processes.values_mut() {
        if process.causation == Causation::Choice && process.period.is_some() {
            process.period = Some(1);
        }
    }
    Session::new(genesis, Execution::Individuals).unwrap()
}

fn physical_tick(session: &mut Session) -> Result<()> {
    session.command(Command::AdvanceGeometry {
        ticks: 1,
        resolutions: vec![],
    })?;
    Ok(())
}

#[test]
fn adaptive_deep_time_adopts_a_better_policy_and_replays() {
    let mut session = session();
    let lineage = "lineage:1";
    let needs = crate::meaning::needs(&session.sim.genesis().rules).len();
    let good = LineagePolicy::founding(needs);
    let bad = LineagePolicy {
        drives: vec![-UNIT; needs],
        ..good.clone()
    };
    session
        .command(Command::Policy {
            lineage: lineage.into(),
            policy: bad,
        })
        .unwrap();
    let calls = Cell::new(0);
    let candidates = |_: &Session, line: &str| {
        if line != lineage {
            return vec![];
        }
        calls.set(calls.get() + 1);
        vec![Candidate {
            name: "policy:good".into(),
            commands: vec![Command::Policy {
                lineage: lineage.into(),
                policy: good.clone(),
            }],
        }]
    };
    let (handover, boundaries) = run_adapting(
        &mut session,
        DeepTimeSpan { epochs: 2 },
        30,
        &candidates,
        &mut run_deep_time,
        &mut boundary::score,
    )
    .unwrap();
    assert_eq!((handover.from_tick, handover.to_tick), (0, 2));
    assert_eq!(
        calls.get(),
        2,
        "only the two source boundaries offer candidates"
    );
    let Happening::Boundary { tick, turns } = &boundaries[0] else {
        panic!("boundary");
    };
    assert_eq!(*tick, 1);
    assert_eq!(
        turns[0].committed.as_deref(),
        Some("policy:good"),
        "{turns:?}"
    );
    assert_eq!(
        session.sim.state().lineages[lineage].policy.as_ref(),
        Some(&good)
    );
    let policies: Vec<_> = session
        .entries
        .iter()
        .filter(|entry| matches!(entry.command, Command::Policy { .. }))
        .collect();
    assert_eq!(
        policies.len(),
        2,
        "one initial policy and one admitted replacement"
    );
    assert_eq!(policies[1].tick, 1);
    let replayed = Session::load(session.save(), Execution::Individuals).unwrap();
    assert_eq!(replayed.save(), session.save());
}

#[test]
fn native_adaptive_entrypoint_matches_separate_clock_and_boundary_calls() {
    let mut actual = session_with_epoch(4);
    actual.advance(3).unwrap();
    let mut expected = actual.clone();
    let (handover, boundaries) =
        run_deep_time_adapting(&mut actual, DeepTimeSpan { epochs: 2 }, 3).unwrap();
    let mut readings = Vec::new();
    for _ in 0..2 {
        run_deep_time(&mut expected, DeepTimeSpan { epochs: 1 }).unwrap();
        let turns = boundary::adapt(&mut expected, None, &revise::revisions, 3).unwrap();
        readings.push(Happening::Boundary {
            tick: expected.sim.state().tick,
            turns,
        });
    }
    assert_eq!((handover.from_tick, handover.to_tick), (3, 8));
    assert_eq!(
        boundaries
            .iter()
            .map(|event| match event {
                Happening::Boundary { tick, .. } => *tick,
                _ => panic!("boundary"),
            })
            .collect::<Vec<_>>(),
        [4, 8]
    );
    assert_eq!(boundaries, readings);
    assert_eq!(actual.save(), expected.save());
}

#[test]
fn an_empty_span_never_advances_or_scores_and_a_zero_horizon_is_refused() {
    let mut session = session();
    let before = session.save();
    let mut advance = |_: &mut Session| -> Result<()> {
        panic!("empty span advanced");
    };
    let mut score = |_: &Session, _: &str, _: &Candidate, _: Tick| -> Result<Score> {
        panic!("empty span scored");
    };
    let (handover, boundaries) = run_deep_time_adapting_using(
        &mut session,
        DeepTimeSpan::default(),
        0,
        &mut advance,
        &mut score,
    )
    .unwrap();
    assert_eq!(handover.to_tick, before.tick);
    assert!(boundaries.is_empty());
    assert_eq!(session.save(), before);
    assert!(run_deep_time_adapting(&mut session, DeepTimeSpan { epochs: 1 }, 0).is_err());
    assert_eq!(session.save(), before);
}

#[test]
fn physical_adaptive_time_keeps_clock_inputs_separate_from_adaptation() {
    let mut session = session();
    let (_, boundaries) = run_deep_time_adapting_using(
        &mut session,
        DeepTimeSpan { epochs: 2 },
        3,
        &mut physical_tick,
        &mut |source, lineage, candidate, ticks| {
            boundary::score_using(source, lineage, candidate, ticks, &mut |copy, ticks| {
                for _ in 0..ticks {
                    physical_tick(copy)?;
                }
                Ok(())
            })
        },
    )
    .unwrap();
    assert_eq!(boundaries.len(), 2);
    assert_eq!(session.entries.len(), 2);
    assert!(
        session
            .entries
            .iter()
            .all(|entry| matches!(entry.command, Command::AdvanceGeometry { ticks: 1, .. }))
    );
    assert_eq!(
        Session::load(session.save(), Execution::Individuals)
            .unwrap()
            .save(),
        session.save()
    );
}

#[test]
fn physical_time_admits_a_policy_between_epochs_then_replays() {
    let mut session = session();
    let lineage = "lineage:1";
    let needs = crate::meaning::needs(&session.sim.genesis().rules).len();
    let good = LineagePolicy::founding(needs);
    session
        .command(Command::Policy {
            lineage: lineage.into(),
            policy: LineagePolicy {
                drives: vec![-UNIT; needs],
                ..good.clone()
            },
        })
        .unwrap();
    let candidates = |_: &Session, line: &str| {
        if line == lineage {
            vec![Candidate {
                name: "policy:good".into(),
                commands: vec![Command::Policy {
                    lineage: lineage.into(),
                    policy: good.clone(),
                }],
            }]
        } else {
            vec![]
        }
    };
    let (_, boundaries) = run_adapting(
        &mut session,
        DeepTimeSpan { epochs: 2 },
        30,
        &candidates,
        &mut |source, span| run_deep_time_using(source, span, &mut physical_tick),
        &mut |source, line, candidate, ticks| {
            boundary::score_using(source, line, candidate, ticks, &mut |copy, ticks| {
                for _ in 0..ticks {
                    physical_tick(copy)?;
                }
                Ok(())
            })
        },
    )
    .unwrap();
    let Happening::Boundary { tick, turns } = &boundaries[0] else {
        panic!("boundary");
    };
    assert_eq!(*tick, 1);
    assert_eq!(
        turns[0].committed.as_deref(),
        Some("policy:good"),
        "{turns:?}"
    );
    let kinds: Vec<_> = session
        .entries
        .iter()
        .map(|entry| match entry.command {
            Command::AdvanceGeometry { ticks: 1, .. } => (entry.tick, "clock"),
            Command::Policy { .. } => (entry.tick, "policy"),
            _ => panic!("unexpected command"),
        })
        .collect();
    assert_eq!(
        kinds,
        [(0, "policy"), (0, "clock"), (1, "policy"), (1, "clock")]
    );
    assert_eq!(
        session.sim.state().lineages[lineage].policy.as_ref(),
        Some(&good)
    );
    assert_eq!(
        Session::load(session.save(), Execution::Individuals)
            .unwrap()
            .save(),
        session.save()
    );
}

#[test]
fn adaptation_does_not_let_a_physical_provider_insert_commands_or_skip_ticks() {
    for skip in [false, true] {
        let mut session = session();
        let mut clock = |session: &mut Session| -> Result<()> {
            if skip {
                session.advance(2)?;
            } else {
                physical_tick(session)?;
                session.command(Command::Join)?;
            }
            Ok(())
        };
        let error = run_deep_time_adapting_using(
            &mut session,
            DeepTimeSpan { epochs: 1 },
            1,
            &mut clock,
            &mut boundary::score,
        )
        .unwrap_err();
        assert!(
            error.contains(if skip {
                "one-tick boundary"
            } else {
                "permits only clocks"
            }),
            "{error}"
        );
    }
}

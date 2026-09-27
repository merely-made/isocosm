// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Ruling 371: each requested tick hands over its own moves. The same
//! checks see honest records and deliberate cross-tick contamination.

use super::*;

fn only_tick<T>(record: &FlowResult<T>) -> bool {
    record.flows.iter().all(|flow| flow.tick == record.tick)
}

#[test]
fn successive_results_reconcile_without_accumulating_or_mixing_ticks() {
    for mode in [Execution::Individuals, Execution::Grouped] {
        let mut session = new_session(ecology(7), mode);
        let mut earlier = None;
        let mut controls = 0;
        for tick in 1..=30 {
            let before = books(&session);
            let record = session.advance_tick_with_flows().unwrap();
            let after = books(&session);
            assert_eq!(record.tick, tick);
            assert!(only_tick(&record));
            reconcile(&before, &after, &record.flows, "one handed-over tick").unwrap();
            if let Some(previous) = &earlier {
                let previous: &FlowResult<isocosm::simulation::Work> = previous;
                assert_eq!(previous.tick + 1, record.tick);
                assert!(only_tick(previous), "the caller's old record stays intact");
                if let Some(old_move) = previous.flows.first() {
                    let mut mixed = record.clone();
                    mixed.flows.push(old_move.clone());
                    assert!(
                        !only_tick(&mixed),
                        "a previous tick's move must be detected"
                    );
                    controls += 1;
                }
            }
            earlier = Some(record);
        }
        assert!(
            controls > 0,
            "real earlier moves exercised the mixing control"
        );
    }
}

#[test]
fn commands_handoff_immediately_without_repeating_same_tick_moves() {
    for mode in [Execution::Individuals, Execution::Grouped] {
        let mut session = new_session(ecology(7), mode);
        for amount in [7, 9] {
            let before = books(&session);
            let record = session
                .command_with_flows(Command::PlaceMatter {
                    site: 0,
                    account: "world:soil".into(),
                    amount,
                })
                .unwrap();
            assert_eq!(record.tick, 0);
            assert_eq!(record.flows.len(), 1);
            assert_eq!(record.flows[0].amount, amount);
            reconcile(&before, &books(&session), &record.flows, "one command").unwrap();
        }
        let empty = session.command_with_flows(Command::Collect).unwrap();
        assert_eq!(empty.tick, 0);
        assert!(
            empty.flows.is_empty(),
            "earlier commands were already handed over"
        );
        assert!(
            session
                .command_with_flows(Command::PlaceMatter {
                    site: u64::MAX,
                    account: "world:soil".into(),
                    amount: 7,
                })
                .is_err()
        );
        assert!(
            session
                .command_with_flows(Command::Collect)
                .unwrap()
                .flows
                .is_empty()
        );
    }
}

#[test]
fn recording_is_requested_per_call_and_empty_ticks_are_returned() {
    for mode in [Execution::Individuals, Execution::Grouped] {
        let mut session = new_session(ecology(7), mode);
        let first = session.advance_tick_with_flows().unwrap();
        let mut witness = session.clone();
        let mut unrequested_moves = 0;
        for _ in 0..12 {
            session.advance(1).unwrap();
            unrequested_moves += witness.advance_tick_with_flows().unwrap().flows.len();
        }
        assert!(unrequested_moves > 0, "the opt-out interval had real moves");
        assert_eq!(session.sim.state_hash(), witness.sim.state_hash());
        let before = books(&session);
        let next = session.advance_tick_with_flows().unwrap();
        assert_eq!(next.tick, first.tick + 13);
        assert!(only_tick(&next));
        reconcile(
            &before,
            &books(&session),
            &next.flows,
            "after an unrecorded interval",
        )
        .unwrap();

        let mut idle = ecology(7);
        for process in idle.rules.processes.values_mut() {
            process.period = None;
        }
        let mut idle = new_session(idle, mode);
        for tick in 1..=3 {
            let record = idle.advance_tick_with_flows().unwrap();
            assert_eq!(record.tick, tick);
            assert!(record.flows.is_empty());
        }
    }
}

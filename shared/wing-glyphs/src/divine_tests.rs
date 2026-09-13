// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

const LIMITS: WindowLimits = WindowLimits {
    min_ticks: 1,
    max_ticks: 100,
};

fn choice(start_tick: u64, duration_ticks: u64) -> DomainChoice {
    DomainChoice {
        metric_id: "world.fire.ignitions/v1".into(),
        unit: "occurrences".into(),
        start_tick,
        duration_ticks,
    }
}

fn window(start: u64, duration: u64) -> DomainWindow {
    DomainWindow::new(choice(start, duration), LIMITS).unwrap()
}

#[test]
fn partitioning_evidence_changes_latency_not_total() {
    let evidence = [0, 3, 0, 8, 2, 0];
    let mut long = window(10, 6);
    for (i, amount) in evidence.into_iter().enumerate() {
        long.observe("world.fire.ignitions/v1", 10 + i as u64, amount)
            .unwrap();
    }
    let mut partition_total = 0;
    for (chunk, amounts) in evidence.chunks(2).enumerate() {
        let start = 10 + chunk as u64 * 2;
        let mut short = window(start, 2);
        for (i, amount) in amounts.iter().enumerate() {
            short
                .observe("world.fire.ignitions/v1", start + i as u64, *amount)
                .unwrap();
        }
        let receipt = short.settle(start + 2).unwrap();
        assert_eq!((receipt.start_tick, receipt.end_tick), (start, start + 2));
        partition_total += receipt.total;
    }
    assert_eq!(partition_total, 13);
    assert_eq!(long.settle(16).unwrap().total, partition_total);
}

#[test]
fn explicit_zeros_are_required_and_settle_to_zero() {
    let mut w = window(7, 2);
    assert_eq!(
        w.settle(9),
        Err(WindowError::Incomplete {
            next_tick: 7,
            end_tick: 9
        })
    );
    w.observe("world.fire.ignitions/v1", 7, 0).unwrap();
    w.observe("world.fire.ignitions/v1", 8, 0).unwrap();
    assert_eq!(w.observed_ticks(), 2);
    assert_eq!(
        w.settle(8),
        Err(WindowError::TooEarly {
            at_tick: 8,
            end_tick: 9
        })
    );
    assert!(!w.is_settled());
    assert_eq!(
        w.settle(100).unwrap(),
        DomainReceipt {
            metric_id: "world.fire.ignitions/v1".into(),
            unit: "occurrences".into(),
            start_tick: 7,
            end_tick: 9,
            total: 0,
        }
    );
    assert!(w.is_settled());
    assert_eq!(w.settle(100), Err(WindowError::AlreadySettled));
    assert_eq!(
        w.observe("world.fire.ignitions/v1", 8, 1),
        Err(WindowError::AlreadySettled)
    );
}

#[test]
fn bad_observations_and_early_settlement_are_transactional() {
    let mut w = window(4, 3);
    let mut unchanged = window(4, 3);
    assert!(matches!(
        w.observe("different", 4, 2),
        Err(WindowError::WrongMetric { .. })
    ));
    assert_eq!(w, unchanged);
    assert!(matches!(
        w.observe("world.fire.ignitions/v1", 3, 2),
        Err(WindowError::OutsideWindow { .. })
    ));
    assert!(matches!(
        w.observe("world.fire.ignitions/v1", 7, 2),
        Err(WindowError::OutsideWindow { .. })
    ));
    assert_eq!(
        w.observe("world.fire.ignitions/v1", 5, 2),
        Err(WindowError::OutOfOrder {
            expected_tick: 4,
            supplied_tick: 5
        })
    );
    assert_eq!(w, unchanged);
    for target in [&mut w, &mut unchanged] {
        target.observe("world.fire.ignitions/v1", 4, 2).unwrap();
    }
    assert_eq!(
        w.observe("world.fire.ignitions/v1", 4, 9),
        Err(WindowError::OutOfOrder {
            expected_tick: 5,
            supplied_tick: 4
        })
    );
    assert_eq!(
        w.settle(6),
        Err(WindowError::TooEarly {
            at_tick: 6,
            end_tick: 7
        })
    );
    assert_eq!(
        w.settle(7),
        Err(WindowError::Incomplete {
            next_tick: 5,
            end_tick: 7
        })
    );
    assert_eq!(w, unchanged);
}

#[test]
fn overflow_does_not_consume_tick_or_corrupt_sum() {
    let mut w = window(u64::MAX - 2, 2);
    w.observe("world.fire.ignitions/v1", u64::MAX - 2, u64::MAX)
        .unwrap();
    assert_eq!(
        w.observe("world.fire.ignitions/v1", u64::MAX - 1, 1),
        Err(WindowError::TotalOverflow)
    );
    assert_eq!(w.next_tick(), u64::MAX - 1);
    assert_eq!(w.total(), u64::MAX);
    w.observe("world.fire.ignitions/v1", u64::MAX - 1, 0)
        .unwrap();
    assert_eq!(w.settle(u64::MAX).unwrap().total, u64::MAX);
}

#[test]
fn selection_and_limits_are_validated_without_reinterpreting_units() {
    for limits in [
        WindowLimits {
            min_ticks: 0,
            max_ticks: 1,
        },
        WindowLimits {
            min_ticks: 2,
            max_ticks: 1,
        },
    ] {
        assert_eq!(
            DomainWindow::new(choice(0, 1), limits),
            Err(WindowError::InvalidLimits)
        );
    }
    for duration in [0, 101] {
        assert!(matches!(
            DomainWindow::new(choice(0, duration), LIMITS),
            Err(WindowError::DurationOutsideLimits { .. })
        ));
    }
    assert_eq!(
        DomainWindow::new(choice(u64::MAX, 1), LIMITS),
        Err(WindowError::EndOverflow)
    );
    let mut c = choice(0, 1);
    c.metric_id = " \n".into();
    assert_eq!(DomainWindow::new(c, LIMITS), Err(WindowError::EmptyMetric));
    let mut c = choice(0, 1);
    c.unit = "\t".into();
    assert_eq!(DomainWindow::new(c, LIMITS), Err(WindowError::EmptyUnit));
    let mut selected = choice(20, 1);
    selected.unit = "burning-entity-ticks".into();
    let mut w = DomainWindow::new(selected.clone(), LIMITS).unwrap();
    selected.metric_id = "replacement".into();
    selected.unit = "occurrences".into();
    assert_eq!(
        w.choice(),
        &DomainChoice {
            unit: "burning-entity-ticks".into(),
            ..choice(20, 1)
        }
    );
    w.observe("world.fire.ignitions/v1", 20, 3).unwrap();
    let receipt = w.settle(21).unwrap();
    assert_eq!(receipt.unit, "burning-entity-ticks");
    assert_eq!(receipt.total, 3);
}

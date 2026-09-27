// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Mesocosm's two transport tests, ported: its four material channels are
//! four accounts of one ledger.

use super::scalar_reference::ScalarSoil as Soil;
use super::*;

const ACCOUNTS: [&str; 4] = ["matter:a", "matter:b", "matter:c", "matter:d"];

fn amount(column: &Ledger, account: &str) -> u64 {
    column.get(account).copied().unwrap_or(0)
}

fn totals(columns: &[Ledger]) -> [u128; 4] {
    ACCOUNTS.map(|a| columns.iter().map(|c| u128::from(amount(c, a))).sum())
}

#[test]
fn ledger_transport_matches_four_independent_scalar_soils() {
    let extent = 2;
    let side = 5;
    let mut columns = vec![Ledger::new(); side * side];
    let mut scalar = std::array::from_fn::<_, 4, _>(|_| Soil::seeded(extent, 0));
    let at = |index: usize| {
        [
            (index % side) as i32 - extent,
            0,
            (index / side) as i32 - extent,
        ]
    };
    for (index, column) in columns.iter_mut().enumerate() {
        let amounts = [
            index as u64 * 17 + 1,
            index as u64 * 3,
            71 - index as u64,
            9,
        ];
        for (a, (account, value)) in ACCOUNTS.iter().zip(amounts).enumerate() {
            column.insert(account.to_string(), value);
            let soil = &mut scalar[a];
            soil.deposit(soil.column_at(at(index)), value);
        }
    }
    let before = totals(&columns);
    for _ in 0..100 {
        percolate(&mut columns, side, 8).unwrap();
        for soil in &mut scalar {
            soil.percolate();
        }
        for (index, column) in columns.iter().enumerate() {
            for (a, account) in ACCOUNTS.iter().enumerate() {
                let soil = &scalar[a];
                assert_eq!(
                    amount(column, account),
                    soil.matter_mg(soil.column_at(at(index)))
                );
            }
        }
        assert_eq!(totals(&columns), before);
    }
}

#[test]
fn failures_are_atomic_and_single_column_has_nowhere_to_shed() {
    let full = Ledger::from([("matter:c".into(), u64::MAX)]);
    let mut columns = vec![full; 9];
    let original = columns.clone();
    let err = |r: Result<()>| r.unwrap_err();
    assert!(err(percolate(&mut columns, 2, 8)).contains("square"));
    assert!(err(percolate(&mut columns, 3, 0)).contains("zero divisor"));
    assert!(err(percolate(&mut columns, 3, 1)).contains("overflows"));
    assert_eq!(columns, original);
    let mut single = [ACCOUNTS
        .iter()
        .map(|a| (a.to_string(), u64::MAX))
        .collect::<Ledger>()];
    let before = single.clone();
    percolate(&mut single, 1, 1).unwrap();
    assert_eq!(single, before);
}

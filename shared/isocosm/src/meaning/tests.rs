// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! `mesocosm-core`'s stock arithmetic, ported (ruling 263): its four fixed
//! material channels are four matter accounts of one ledger, their keys
//! sorting in the channels' order, so its tie order is key order. Five of
//! the six are here; refusing a partial removal belongs to an act, whose
//! stage is dropped whole, and is in `tests/conversions.rs`.

use super::*;
use crate::Founding;

const UNTYPED: &str = "a:untyped";
const PRODUCER: &str = "b:producer";
const CONSUMER: &str = "c:consumer";
const DECOMPOSER: &str = "d:decomposer";
const CHANNELS: [&str; 4] = [UNTYPED, PRODUCER, CONSUMER, DECOMPOSER];

/// Rules declaring the four channels as matter.
fn rules() -> Rules {
    let mut rules = Founding::default().generate().unwrap().rules;
    for key in CHANNELS {
        let kind = AccountKind::Matter {
            lineage: "world:ground".into(),
            reserve: false,
            provision: false,
        };
        rules.accounts.insert(key.into(), kind);
    }
    rules
}

/// A ledger from amounts in channel order, zeros included.
fn stock(amounts: [u64; 4]) -> Ledger {
    CHANNELS
        .iter()
        .map(|k| k.to_string())
        .zip(amounts)
        .collect()
}

/// What `ledger` keeps once `taken` is removed.
fn left(ledger: &Ledger, taken: &Ledger) -> Ledger {
    let mut left = ledger.clone();
    for (key, amount) in taken {
        debit(&mut left, key, *amount).unwrap();
    }
    left
}

fn total(ledger: &Ledger) -> u128 {
    ledger.values().map(|v| u128::from(*v)).sum()
}

#[test]
fn credit_reports_the_overflowing_account() {
    let mut ledger = Ledger::from([(CONSUMER.into(), u64::MAX)]);
    let error = credit(&mut ledger, CONSUMER, 1).unwrap_err();
    assert_eq!(error, format!("account overflow: {CONSUMER}"));
    assert_eq!(ledger[CONSUMER], u64::MAX);
}

#[test]
fn an_odd_proportional_take_breaks_equal_remainders_in_key_order() {
    let original = stock([1, 1, 1, 0]);
    let taken = share(&original, &rules(), 2);
    assert_eq!(
        taken,
        Ledger::from([(UNTYPED.into(), 1), (PRODUCER.into(), 1)])
    );
    assert_eq!(left(&original, &taken), stock([0, 0, 1, 0]));
    assert_eq!(
        total(&taken) + total(&left(&original, &taken)),
        total(&original)
    );
}

#[test]
fn a_take_handles_nothing_and_the_full_u64_range() {
    let rules = rules();
    assert!(share(&Ledger::new(), &rules, 7).is_empty());
    let one_full = Ledger::from([(UNTYPED.into(), u64::MAX)]);
    assert_eq!(share(&one_full, &rules, u64::MAX), one_full);
    let original = stock([u64::MAX; 4]);
    let taken = share(&original, &rules, u64::MAX);
    assert_eq!(total(&taken), u128::from(u64::MAX));
    assert_eq!(
        total(&taken) + total(&left(&original, &taken)),
        total(&original)
    );
    assert_eq!(share(&original, &rules, u64::MAX), taken);
}

#[test]
fn taking_more_than_is_held_takes_every_account() {
    let original = stock([2, 3, 5, 7]);
    assert_eq!(share(&original, &rules(), u64::MAX), original);
}

#[test]
fn a_ledger_round_trips_with_all_four_accounts() {
    let original = stock([0, 3, 5, 8]);
    let encoded = serde_json::to_string(&original).unwrap();
    let decoded: Ledger = serde_json::from_str(&encoded).unwrap();
    assert_eq!(decoded, original);
    assert_eq!(decoded.len(), 4);
}

// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::legacy::mesocosm::snapshot::{decode, encode};

#[test]
fn a_ruleset_digest_is_stable_across_calls() {
    assert_eq!(WorldRules::native(), WorldRules::native());
    assert_eq!(WorldRules::native().processes, Registry::native().digest());
}

/// The epoch rule and its budget are folded into the record's identity. (PE3)
#[test]
fn the_epoch_rule_is_digested() {
    let native = WorldRules::native();
    assert_eq!(
        (native.epoch, native.epoch_ticks),
        (EpochRule::Timed, 1_000)
    );
    let brisk = native.timed(500);
    assert_ne!(brisk.digest(), native.digest(), "a budget is rule-bearing");
    assert_eq!(brisk.processes, native.processes);
    assert_ne!(native.scoring_over(10).digest(), native.digest());
}

/// A timed world encodes as it did when the budget lived in the rule.
#[test]
fn a_timed_world_keeps_its_encoding() {
    let bytes = encode(&(EpochRule::Timed, 1_000_u64)).unwrap();
    assert_eq!(bytes, [0, 0xe8, 0x07]);
}

#[test]
fn the_trophic_grammar_is_digested() {
    let native = WorldRules::native();
    assert_eq!(native.trophic_grammar, TROPHIC_GRAMMAR_REVISION);
    let pre_ports = WorldRules {
        trophic_grammar: 0,
        ..native
    };
    assert_ne!(pre_ports.digest(), native.digest());
}

#[test]
fn the_soil_completion_rate_is_saved_and_digested() {
    let native = WorldRules::native();
    assert_eq!(native.soil_mineralization_mg_per_column_per_tick, 1);
    for dose in [0, 3, u64::MAX] {
        let configured = WorldRules {
            soil_mineralization_mg_per_column_per_tick: dose,
            ..native
        };
        assert_ne!(configured.digest(), native.digest());
        let decoded: WorldRules = decode(&encode(&configured).unwrap()).unwrap();
        assert_eq!(decoded, configured);
    }
}

/// The deep-time span is zero unless given, saved, digested, and handed to
/// the hagiograph as the same count. (D7a)
#[test]
fn the_deep_time_span_is_zero_by_default_saved_and_digested() {
    assert_eq!(WorldRules::native().deep_time.epochs, 0);
    assert_eq!(WorldRules::default().deep_time.epochs, 0);
    let native = WorldRules::native();
    for epochs in [1, 6, u32::MAX] {
        let spanned = WorldRules {
            deep_time: DeepTimeSpan { epochs },
            ..native
        };
        assert_ne!(spanned.digest(), native.digest());
        let decoded: WorldRules = decode(&encode(&spanned).unwrap()).unwrap();
        assert_eq!(decoded, spanned);
        assert_eq!(
            hagiograph::DeepTime::from(spanned.deep_time),
            hagiograph::DeepTime { epochs }
        );
    }
}

/// Only a timed rule with a budget ends an epoch on the clock.
#[test]
fn only_the_timed_rule_spends_a_budget() {
    let timed = WorldRules::native().timed(3);
    assert!(!timed.epoch_spent(2));
    assert!(timed.epoch_spent(3));
    assert!(!WorldRules::native().timed(0).epoch_spent(u64::MAX));
    for other in [EpochRule::Gated, EpochRule::PlayerTriggered] {
        assert!(!timed.ending(other).epoch_spent(u64::MAX));
    }
}

#[test]
fn the_three_epoch_rules_still_have_three_identities() {
    let native = WorldRules::native();
    let gated = native.ending(EpochRule::Gated);
    let demanded = native.ending(EpochRule::PlayerTriggered);
    assert_ne!(gated.digest(), demanded.digest());
    assert_ne!(native.digest(), demanded.digest());
    assert_eq!(
        demanded.digest(),
        demanded
            .timed(7)
            .ending(EpochRule::PlayerTriggered)
            .digest(),
        "an unread budget is not rule-bearing"
    );
}

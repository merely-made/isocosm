// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Checkpoint 6's step 4: the epoch rule, deep time and the founding
//! presets as rules data (rulings 272, 451 and 452).

use isocosm::{Execution, Founding, Session, preset::*, rules::*, simulation::Genesis};

fn founding(seed: u64) -> Founding {
    Founding {
        seed,
        sites: 3,
        population: 24,
        cohort_size: 8,
        lineages: 2,
        ..Default::default()
    }
}

fn genesis(seed: u64) -> Genesis {
    founding(seed).generate().unwrap()
}

fn round_trip(rules: &Rules) -> Rules {
    serde_json::from_str(&serde_json::to_string(rules).unwrap()).unwrap()
}

#[test]
fn a_year_counts_the_worlds_own_unit() {
    assert_eq!(year_ticks(DEFAULT_TICK_MICROSECONDS), 525_960);
    assert_eq!(year_ticks(1_000_000), 31_557_600);
    assert_eq!(year_ticks(YEAR_MICROSECONDS), 1);
    assert_eq!(year_ticks(YEAR_MICROSECONDS * 2), 1);
}

#[test]
fn generated_worlds_take_a_year_and_serialize_as_before() {
    let rules = genesis(1).rules;
    assert_eq!(rules.epoch_ticks, 525_960);
    assert_eq!(rules.epoch, EpochRule::Timed);
    assert!(rules.deep_time.is_bare());
    let json = serde_json::to_string(&rules).unwrap();
    assert!(!json.contains("\"epoch\""), "a timed rule adds no key");
    assert!(!json.contains("\"deep_time\""), "a bare span adds no key");
}

#[test]
fn the_rule_and_the_span_round_trip_and_move_the_digest() {
    let timed = genesis(2).rules;
    let mut seen = vec![timed.revision()];
    for rule in [EpochRule::Gated, EpochRule::PlayerTriggered] {
        let mut rules = timed.clone();
        rules.epoch = rule;
        assert_eq!(round_trip(&rules), rules);
        seen.push(rules.revision());
    }
    let mut spanned = timed.clone();
    spanned.deep_time = DeepTimeSpan { epochs: 100 };
    assert_eq!(round_trip(&spanned), spanned);
    seen.push(spanned.revision());
    seen.sort();
    seen.dedup();
    assert_eq!(seen.len(), 4, "each rule and span is its own world");
}

#[test]
fn deep_time_counts_epochs_with_one_more_as_its_ceiling() {
    let year = 525_960;
    let century = DeepTimeSpan { epochs: 100 };
    assert_eq!(
        deep_time_ceiling(EpochRule::Timed, year, century),
        Ok(101 * year)
    );
    assert_eq!(
        deep_time_ceiling(EpochRule::Timed, u64::MAX, century),
        Ok(u64::MAX)
    );
    for rule in [EpochRule::Gated, EpochRule::PlayerTriggered] {
        assert_eq!(
            deep_time_ceiling(rule, year, DeepTimeSpan::default()),
            Ok(0)
        );
        assert!(deep_time_ceiling(rule, year, century).is_err(), "{rule:?}");
    }
    assert!(deep_time_ceiling(EpochRule::Timed, 0, century).is_err());
}

#[test]
fn a_world_whose_epochs_never_close_cannot_hold_a_past() {
    let mut refused = genesis(3);
    refused.rules.epoch = EpochRule::PlayerTriggered;
    refused.rules.deep_time = DeepTimeSpan { epochs: 2 };
    assert!(Session::new(refused.clone(), Execution::Grouped).is_err());
    refused.rules.epoch = EpochRule::Timed;
    assert!(Session::new(refused, Execution::Grouped).is_ok());
}

#[test]
fn the_rules_predicates_are_mesocosms() {
    assert!(EpochRule::Timed.built() && EpochRule::Timed.admits_demand());
    assert!(!EpochRule::Gated.built() && !EpochRule::Gated.admits_demand());
    assert!(EpochRule::PlayerTriggered.built() && EpochRule::PlayerTriggered.admits_demand());
    let mut rules = genesis(4).rules;
    assert_eq!(rules.epoch_budget(), Some(525_960));
    rules.epoch = EpochRule::PlayerTriggered;
    assert_eq!(rules.epoch_budget(), None);
}

#[test]
fn only_a_timed_epoch_checkpoints_on_ticks() {
    for mode in [Execution::Grouped, Execution::Individuals] {
        let mut timed = genesis(5);
        timed.rules.epoch_ticks = 4;
        let mut untimed = timed.clone();
        untimed.rules.epoch = EpochRule::PlayerTriggered;

        let mut a = Session::new(timed, mode).unwrap();
        a.advance(18).unwrap();
        let ticks: Vec<u64> = a.checkpoints.iter().map(|c| c.tick).collect();
        assert_eq!(ticks, [4, 8, 12, 16], "{mode:?}");

        let mut b = Session::new(untimed.clone(), mode).unwrap();
        b.advance(18).unwrap();
        assert!(b.checkpoints.is_empty(), "{mode:?}");
        // The epoch rule decides boundaries, never what the world does. The
        // state hash folds in the rules' revision, so states are compared.
        assert!(normalized(&a) == normalized(&b), "{mode:?}");
        // The comparison can fail: the same rule over another seed's world.
        let mut other = genesis(6);
        other.rules.epoch_ticks = 4;
        other.rules.epoch = EpochRule::PlayerTriggered;
        let mut c = Session::new(other, mode).unwrap();
        c.advance(18).unwrap();
        assert!(normalized(&b) != normalized(&c), "{mode:?}");
        let again = Session::load(b.save(), mode).unwrap();
        assert_eq!(again.sim.state_hash(), b.sim.state_hash(), "{mode:?}");
    }
}

fn normalized(session: &Session) -> isocosm::simulation::State {
    let mut state = session.sim.state().clone();
    state.population = state.population.normalized();
    state
}

#[test]
fn a_preset_sets_every_site_and_declares_its_pressures() {
    for preset in Preset::ALL {
        let world = Founding {
            preset: Some(preset),
            ..founding(6)
        }
        .generate()
        .unwrap();
        for key in PRESSURES {
            assert!(world.rules.conditions.contains(key), "{preset:?} {key}");
        }
        assert!(!world.sites.is_empty());
        for site in world.sites.values() {
            for key in PRESSURES {
                assert_eq!(site.conditions.get(key), Some(&preset.strength(key)));
            }
        }
        assert!(Session::new(world, Execution::Grouped).is_ok());
    }
    assert_eq!(Preset::HeavyDeep.strength("pressure:gravity"), 9);
    assert_eq!(Preset::HeavyDeep.strength("pressure:cold"), 0);
}

#[test]
fn a_founding_without_a_preset_carries_no_pressure() {
    let world = genesis(7);
    assert!(
        PRESSURES
            .iter()
            .all(|k| !world.rules.conditions.contains(*k))
    );
    assert!(
        world
            .sites
            .values()
            .all(|s| s.conditions.keys().all(|k| !k.starts_with("pressure:")))
    );
    let json = serde_json::to_string(&founding(7)).unwrap();
    assert!(!json.contains("\"preset\""));
    let preset = Founding {
        preset: Some(Preset::LongYear),
        ..founding(7)
    };
    let back: Founding = serde_json::from_str(&serde_json::to_string(&preset).unwrap()).unwrap();
    assert_eq!(back, preset);
}

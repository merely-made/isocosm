// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Native observer controls. Short paired runs qualify observation neutrality,
//! never ecology or long-term persistence.

use super::*;

#[test]
fn diagnostic_simulations_remain_send_and_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<cosmograph::simulation::Simulation>();
}

#[test]
fn diagnostic_founding_has_the_bounded_seed9_native_structure() {
    let s = native_world();
    assert_bounded_initial(&Report::new(&s));
}

#[test]
fn enabling_reading_draining_and_disabling_changes_no_native_authority() {
    let mut s = native_world();
    let saved = s.save();
    let hash = s.sim.state_hash();
    enable(&mut s);
    assert_eq!(s.sim.state_hash(), hash);
    assert_eq!(s.save(), saved);
    let first = snapshot(&s);
    assert_eq!(snapshot(&s), first, "read-only snapshot is repeatable");
    assert_eq!(take(&mut s), first);
    assert_eq!(s.sim.state_hash(), hash);
    assert_eq!(s.save(), saved);
    assert!(s.sim.end_food_diagnostic().is_some());
    assert!(s.sim.food_diagnostic().is_none());
    assert_eq!(s.sim.state_hash(), hash);
    assert_eq!(s.save(), saved);
}

#[test]
fn enabled_native_meal_observations_preserve_save_hash_and_trajectory() {
    let mut ordinary = native_world();
    let mut observed = ordinary.clone();
    let mut ordinary_report = Report::new(&ordinary);
    let mut observed_report = Report::new(&observed);
    enable(&mut observed);
    for ticks in [1, 59, 300, 361] {
        ordinary.advance(ticks).unwrap();
        observed.advance(ticks).unwrap();
        assert_eq!(ordinary.sim.state_hash(), observed.sim.state_hash());
        assert_eq!(ordinary.save(), observed.save());
        ordinary_report.observe(&ordinary);
        observed_report.observe(&observed);
        assert_eq!(ordinary_report.samples, observed_report.samples);
        let saved = observed.save();
        let snapshot = snapshot(&observed);
        assert_eq!(snapshot, super::snapshot(&observed));
        assert_eq!(observed.save(), saved);
    }
    let saved = observed.save();
    let observations = take(&mut observed);
    assert!(
        observations["feeds"]
            .as_array()
            .is_some_and(|rows| !rows.is_empty()),
        "neutrality control must exercise native feed observations"
    );
    assert_eq!(observed.save(), saved);
    assert_eq!(ordinary.sim.state_hash(), observed.sim.state_hash());
    assert_eq!(ordinary_report.outcome, Outcome::Inconclusive);
    assert_eq!(observed_report.outcome, Outcome::Inconclusive);
    assert!(
        snapshot(&observed)["feeds"]
            .as_array()
            .is_some_and(Vec::is_empty),
        "monthly drain keeps an empty enabled collector"
    );
    assert!(observed.sim.end_food_diagnostic().is_some());
    assert_eq!(ordinary.save(), observed.save());
}

#[test]
fn a_refused_native_advance_restores_its_observations_with_the_world() {
    // An instrument control only: isolate a due native feed and force an
    // operation-budget refusal after native planning/attempts have begun.
    let mut g = founding().generate().unwrap();
    g.rules.processes.retain(|id, _| id == "ecology:feed-1");
    let feed = g.rules.processes.get_mut("ecology:feed-1").unwrap();
    feed.causation = cosmograph::rules::Causation::Transition;
    feed.period = Some(1);
    g.rules.limits.events_per_advance = 1_000_000;
    let mut ample = Session::new(g.clone(), Execution::Grouped).unwrap();
    enable(&mut ample);
    ample.advance(1).unwrap();
    assert!(
        snapshot(&ample)["feeds"]
            .as_array()
            .is_some_and(|rows| !rows.is_empty()),
        "positive control must exercise native feed observation"
    );

    g.rules.limits.events_per_advance = 1;
    let mut bounded = Session::new(g, Execution::Grouped).unwrap();
    enable(&mut bounded);
    let saved = bounded.save();
    let before = snapshot(&bounded);
    let why = bounded
        .advance(1)
        .expect_err("native operation-budget control");
    assert!(
        why.contains("operation budget"),
        "unexpected refusal: {why}"
    );
    assert_eq!(bounded.save(), saved, "failed native work is put back");
    assert_eq!(
        snapshot(&bounded),
        before,
        "failed native work leaves no observational attempts or life changes"
    );
}

#[test]
fn an_enabled_clone_owns_its_observations_without_sharing_the_sink() {
    // A one-tick native feed is sufficient to make a shared sink detectable:
    // draining or disabling the copy must leave the original's rows intact.
    let mut g = founding().generate().unwrap();
    g.rules.processes.retain(|id, _| id == "ecology:feed-1");
    let feed = g.rules.processes.get_mut("ecology:feed-1").unwrap();
    feed.causation = cosmograph::rules::Causation::Transition;
    feed.period = Some(1);
    let mut original = Session::new(g, Execution::Grouped).unwrap();
    enable(&mut original);
    original.advance(1).unwrap();
    let before = snapshot(&original);
    assert!(
        before["feeds"]
            .as_array()
            .is_some_and(|rows| !rows.is_empty()),
        "clone control needs actual captured native feed attempts"
    );
    let mut copy = original.clone();
    assert_eq!(snapshot(&copy), before);
    assert_eq!(take(&mut copy), before);
    assert!(
        snapshot(&copy)["feeds"]
            .as_array()
            .is_some_and(Vec::is_empty)
    );
    assert_eq!(
        snapshot(&original),
        before,
        "draining a copy must not drain the original"
    );
    assert!(copy.sim.end_food_diagnostic().is_some());
    assert_eq!(
        snapshot(&original),
        before,
        "disabling a copy must not disable the original"
    );
    assert_eq!(copy.sim.state_hash(), original.sim.state_hash());
    assert_eq!(copy.save(), original.save());
}

#[test]
fn native_ingesta_readings_separate_intake_carry_from_assimilation_and_death() {
    // An instrument control: one-unit admitted takes cannot yet complete the
    // declared three-unit digestion batch. Native ledgers must retain them.
    let mut g = founding().generate().unwrap();
    g.rules.processes.retain(|id, _| id == "ecology:feed-1");
    let feed = g.rules.processes.get_mut("ecology:feed-1").unwrap();
    feed.causation = cosmograph::rules::Causation::Transition;
    feed.period = Some(1);
    let meal = feed.port_feeding.as_ref().unwrap().meal.clone().unwrap();
    assert_eq!((meal.units, meal.kept), (3, 1));
    let cosmograph::rules::Effect::Eat { amount, .. } = &mut feed.effects[0] else {
        panic!("aligned single-take meal")
    };
    *amount = cosmograph::rules::Amount::Fixed(1);
    let mut s = Session::new(g, Execution::Grouped).unwrap();
    let matter = s.sim.matter();
    s.advance(1).unwrap();
    let before = ingesta::read(&s);
    let held = &before["lineage:1"][&meal.ingesta];
    assert!(held.living_part_matter > 0);
    assert_eq!(held.living_entity_ledger_matter, 0);
    assert_eq!(held.living_unassigned_cohort_pool, 0);
    assert_eq!(held.dead_part_matter, 0);
    assert!(!held.living_part_locations.is_empty());
    let (id, stored) = s
        .sim
        .state()
        .population
        .groups
        .iter()
        .find_map(|(&id, group)| {
            if group.entity.lineage != "lineage:1" {
                return None;
            }
            // Foreign ingesta is physical matter in a part, not the body's
            // owned physiological tissue read by anatomy::held.
            let stored: u64 = group
                .entity
                .parts
                .values()
                .map(|part| part.matter.get(&meal.ingesta).copied().unwrap_or(0))
                .sum();
            (stored > 0).then_some((id, stored))
        })
        .unwrap();
    assert_eq!(stored, 1, "one actual intake remains undigested");
    s.command(cosmograph::Command::Kill { entity: id }).unwrap();
    let after = ingesta::read(&s);
    let now = &after["lineage:1"][&meal.ingesta];
    assert_eq!(now.dead_part_matter, u128::from(stored));
    assert_eq!(
        now.living_part_matter + now.dead_part_matter,
        held.living_part_matter
    );
    assert_eq!(s.sim.matter(), matter);
}

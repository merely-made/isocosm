// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Fast controls for the observer and its acceptance predicate. These remove
//! automatic processes deliberately; their stationary years qualify the
//! instrument, never the ecology. All samples read actual native bodies.

use super::*;
use cosmograph::{
    history::Command,
    rules::{Binding, Causation, Effect, Query},
    schema::Movement,
};

fn stationary() -> Session {
    let mut g = Founding {
        seed: 3,
        sites: 2,
        population: 60,
        cohort_size: 1,
        lineages: 5,
        ecology: true,
        web: Web::Diets,
        ecology_rates: Rates::WorldTime,
        bodies: Some(Bodies::default()),
        ..Default::default()
    }
    .generate()
    .unwrap();
    let mut movement = g.rules.processes["ecology:upkeep-0"].clone();
    movement.id = "test:recolonize".into();
    movement.causation = Causation::Transition;
    movement.period = None;
    movement.risk = None;
    movement.requires = vec![Query::Alive(Binding::Actor)];
    movement.effects = vec![Effect::Move {
        destination: 0,
        via: None,
    }];
    let mut leaving = movement.clone();
    leaving.id = "test:leave".into();
    leaving.effects = vec![Effect::Move {
        destination: 1,
        via: None,
    }];
    g.rules.processes.clear();
    g.rules.processes.insert(movement.id.clone(), movement);
    g.rules.processes.insert(leaving.id.clone(), leaving);
    Session::new_with_movement(g, Execution::Individuals, Movement::ImmediateV3).unwrap()
}

fn months(s: &mut Session, report: &mut Report, n: usize) {
    for _ in 0..n {
        s.advance(report.month_ticks).unwrap();
        report.observe(s);
    }
}

fn kill_line(s: &mut Session, line: &str, at: Option<u64>) {
    let ids: Vec<_> = s
        .sim
        .state()
        .population
        .groups
        .iter()
        .filter(|(_, g)| {
            g.entity.alive
                && g.entity.lineage == line
                && at.is_none_or(|site| g.entity.place == site)
        })
        .map(|(&id, _)| id)
        .collect();
    assert!(
        !ids.is_empty(),
        "native mortality control needs living members"
    );
    for id in ids {
        s.command(Command::Kill { entity: id }).unwrap();
    }
}

#[test]
fn a_short_bodied_observation_is_inconclusive_and_changes_no_native_state() {
    let mut s = stationary();
    let before = s.sim.state_hash();
    let mut report = Report::new(&s);
    assert_eq!(
        s.sim.state_hash(),
        before,
        "reading must not change the world"
    );
    let first = &report.samples[0];
    assert!(first.lineages.values().any(|t| t.bodied_members > 0));
    assert_eq!(first.resident_sites.len(), 2);
    assert_eq!(first.fixed_geographic_regions.len(), 1);
    months(&mut s, &mut report, 1);
    assert_eq!(report.outcome, Outcome::Inconclusive);
    assert!(report.first_year.is_none());
    assert!(!report.root_store_qualified);
    let json = serde_json::to_value(&report).unwrap();
    assert_eq!(json["outcome"], "inconclusive");
    assert_eq!(json["root_store_qualified"], false);
    assert_eq!(json["founding"]["population"], 60);
    assert_eq!(json["founding"]["bodies"]["sensing"], true);
    assert_eq!(json["tick_microseconds"], 60_000_000u64);
    assert_eq!(report.genesis_digest, cosmograph::digest(s.sim.genesis()));
}

#[test]
fn twenty_four_native_months_complete_the_stationary_observer_control() {
    let mut s = stationary();
    let mut report = Report::new(&s);
    months(&mut s, &mut report, 24);
    assert_eq!(report.samples.len(), 25);
    assert_eq!(report.samples[24].elapsed_ticks, 24 * report.month_ticks);
    assert_eq!(report.outcome, Outcome::Passed);
    assert!(
        report
            .first_year
            .as_ref()
            .unwrap()
            .iter()
            .all(|v| v.alive && v.within_fourfold && v.no_third_loss)
    );
    assert_eq!(report.samples[0].lineages, report.samples[24].lineages);
    assert!(
        !report.root_store_qualified,
        "a stationary trajectory cannot qualify the missing body contract"
    );
}

#[test]
fn a_year_two_extinction_fails_even_when_the_original_year_bar_passed() {
    let mut s = stationary();
    let mut report = Report::new(&s);
    months(&mut s, &mut report, 13);
    kill_line(&mut s, "lineage:0", None);
    months(&mut s, &mut report, 11);
    assert!(
        report
            .first_year
            .as_ref()
            .unwrap()
            .iter()
            .all(|v| v.alive && v.within_fourfold && v.no_third_loss)
    );
    assert!(report.second_year_extinctions.contains("lineage:0"));
    assert_eq!(
        report
            .observed_regional_losses
            .iter()
            .filter(|loss| loss.level == "life:producer")
            .count(),
        1
    );
    assert_eq!(report.outcome, Outcome::Failed);
}

#[test]
fn the_original_year_bar_still_refuses_a_sharp_loss_with_survivors() {
    let mut s = stationary();
    let mut report = Report::new(&s);
    months(&mut s, &mut report, 3);
    let ids: Vec<_> = s
        .sim
        .state()
        .population
        .groups
        .iter()
        .filter(|(_, group)| group.entity.alive && group.entity.lineage == "lineage:0")
        .map(|(&id, _)| id)
        .collect();
    assert!(
        ids.len() >= 5,
        "the loss control needs a measurable founding population"
    );
    for id in ids.into_iter().skip(1) {
        s.command(Command::Kill { entity: id }).unwrap();
    }
    months(&mut s, &mut report, 21);
    let lineage = report
        .first_year
        .as_ref()
        .unwrap()
        .iter()
        .find(|verdict| verdict.lineage == "lineage:0")
        .unwrap();
    assert!(lineage.alive);
    assert!(!lineage.within_fourfold);
    assert!(!lineage.no_third_loss);
    assert!(report.second_year_extinctions.is_empty());
    assert_eq!(report.outcome, Outcome::Failed);
}

#[test]
fn a_local_loss_can_recolonize_while_its_geographic_region_persists() {
    let mut s = stationary();
    let mut report = Report::new(&s);
    let first = &report.samples[0];
    let line = first.resident_sites[&0]
        .keys()
        .find(|line| {
            first.resident_sites[&1]
                .get(*line)
                .is_some_and(|t| t.members >= 2)
        })
        .unwrap()
        .clone();
    kill_line(&mut s, &line, Some(0));
    s.advance(1).unwrap();
    report.observe(&s);
    assert_eq!(report.observed_local_losses.len(), 1);
    assert!(
        report.observed_regional_losses.is_empty(),
        "a local loss with survivors is not a regional loss"
    );
    assert!(report.samples.last().unwrap().fixed_geographic_regions[0].lineages[&line].members > 0);
    assert!(!report.samples.last().unwrap().resident_sites[&0].contains_key(&line));
    let donor = s
        .sim
        .state()
        .population
        .groups
        .iter()
        .find(|(_, g)| g.entity.alive && g.entity.lineage == line && g.entity.place == 1)
        .map(|(&id, _)| id)
        .unwrap();
    s.command(Command::Act {
        actor: donor,
        target: None,
        process: "test:recolonize".into(),
        cause: None,
    })
    .unwrap();
    s.advance(1).unwrap();
    report.observe(&s);
    assert_eq!(
        report.observed_recolonizations.len(),
        1,
        "one native return produces one observation"
    );
    assert_eq!(report.observed_recolonizations[0].lineage, line);
    assert_eq!(report.observed_recolonizations[0].site, 0);
    let restored = Session::load_json(
        &serde_json::to_vec(&s.save()).unwrap(),
        Execution::Individuals,
    )
    .unwrap();
    assert_eq!(restored.sim.state_hash(), s.sim.state_hash());
    let replayed = Report::new(&restored);
    let actual = report.samples.last().unwrap();
    assert_eq!(replayed.samples[0].lineages, actual.lineages);
    assert_eq!(replayed.samples[0].resident_sites, actual.resident_sites);
    assert_eq!(
        replayed.samples[0].fixed_geographic_regions,
        actual.fixed_geographic_regions
    );
}

#[test]
fn a_lineage_split_after_observation_can_be_lost_and_recolonize() {
    let mut s = stationary();
    let mut report = Report::new(&s);
    let counts = &report.samples[0].resident_sites[&0];
    let founder = s
        .sim
        .state()
        .population
        .groups
        .iter()
        .find(|(_, g)| {
            g.entity.alive
                && g.entity.place == 0
                && g.entity.kingdom != "kingdom:world"
                && counts[&g.entity.lineage].members >= 2
        })
        .map(|(&id, _)| id)
        .unwrap();
    s.command(Command::Speciate {
        founder,
        name: "new-line".into(),
    })
    .unwrap();
    report.observe(&s);
    s.command(Command::Act {
        actor: founder,
        target: None,
        process: "test:leave".into(),
        cause: None,
    })
    .unwrap();
    s.advance(1).unwrap();
    report.observe(&s);
    assert_eq!(
        report
            .observed_local_losses
            .iter()
            .filter(|change| change.lineage == "lineage:new-line" && change.site == 0)
            .count(),
        1
    );
    s.command(Command::Act {
        actor: founder,
        target: None,
        process: "test:recolonize".into(),
        cause: None,
    })
    .unwrap();
    s.advance(1).unwrap();
    report.observe(&s);
    assert_eq!(
        report
            .observed_recolonizations
            .iter()
            .filter(|change| change.lineage == "lineage:new-line" && change.site == 0)
            .count(),
        1
    );
}

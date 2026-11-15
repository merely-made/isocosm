// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use crate::{
    Execution, Founding, Session, band::Banding, ecology::Rates,
    lineage::{drift, offspring}, schema::Relation, simulation::Genesis,
};
use std::time::Instant;

const BIRTH: &str = "ecology:birth-0";

fn founding(population: u64, counted: bool) -> Genesis {
    let mut genesis = Founding {
        seed: 9,
        sites: 1,
        lineages: 1,
        population,
        cohort_size: population,
        ecology: true,
        ecology_rates: Rates::A1,
        population_bands: Some(Banding { age_band: 1 }),
        cohort_births: counted,
        ..Default::default()
    }.generate().unwrap();
    for group in genesis.population.groups.values_mut().filter(|g| g.entity.lineage == "lineage:0") {
        group.entity_mut().accounts.insert("matter:0-0".into(), 24);
    }
    genesis.rules.processes.retain(|id, _| id == BIRTH);
    genesis.rules.processes.get_mut(BIRTH).unwrap().period = Some(1);
    genesis
}

fn newborns(session: &Session) -> u64 {
    session.sim.state().population.groups.values()
        .filter(|group| group.entity.born > 0)
        .map(|group| group.count).sum()
}

#[test]
fn counted_births_keep_matter_and_selection_without_personal_history() {
    let mut outcomes = Vec::new();
    for counted in [false, true] {
        let mut session = Session::new(founding(600, counted), Execution::Individuals).unwrap();
        let matter = session.sim.matter();
        let start = Instant::now();
        session.advance(1).unwrap();
        let elapsed = start.elapsed();
        let state = session.sim.state();
        let births = newborns(&session);
        let selection = drift::weights(&session.sim, "lineage:0", 1).values().sum::<u64>();
        assert!(births > 0);
        assert_eq!(session.sim.matter(), matter);
        assert_eq!(selection, 600 + 2 * births);
        assert_eq!(state.events.len() as u64, if counted { 0 } else { births });
        assert_eq!(state.relations.len() as u64, if counted { 0 } else { 2 * births });
        assert_eq!(state.notes.len() as u64, if counted { 0 } else { births });
        let totals = state.population.totals().unwrap();
        let rows = state.population.groups.len();
        let restored = Session::load_json(
            &serde_json::to_vec(&session.save()).unwrap(), Execution::Individuals,
        ).unwrap();
        assert_eq!(restored.sim.state_hash(), session.sim.state_hash());
        let mut split = session.clone();
        session.advance(3).unwrap();
        for _ in 0..3 { split.advance(1).unwrap(); }
        if counted {
            assert_eq!(session.sim.state_hash(), split.sim.state_hash());
            assert!(Session::load(session.save(), Execution::Individuals).is_ok());
        }
        eprintln!("counted={counted} births={births} rows={rows} tick={elapsed:?}");
        outcomes.push((births, rows, totals));
    }
    assert_eq!(outcomes[0].0, outcomes[1].0);
    assert!(outcomes[1].1 < outcomes[0].1);
    assert_eq!(outcomes[0].2, outcomes[1].2);
}

#[test]
fn direct_birth_still_has_exact_family_identity() {
    let genesis = founding(60, true);
    let (&parent, _) = genesis.population.groups.iter()
        .find(|(_, group)| group.entity.lineage == "lineage:0").unwrap();
    let child = genesis.population.next_id;
    let mut session = Session::new(genesis, Execution::Individuals).unwrap();
    let receipt = session.sim.execute(parent, None, BIRTH, None);
    assert!(receipt.accepted(), "{receipt:?}");
    let state = session.sim.state();
    assert!(state.relations.contains(&Relation::new(parent, "sim:child", child)));
    assert!(state.relations.contains(&Relation::new(child, "sim:parent", parent)));
    assert!(state.population.get(child).is_some());
    assert!(state.population.groups[&parent].offspring.is_none());
}

#[test]
fn lifting_and_regrouping_preserve_counted_births() {
    let mut population = founding(60, true).population;
    let (&first, _) = population.groups.iter().find(|(_, g)| g.count > 3).unwrap();
    let counts = &mut population.groups.get_mut(&first).unwrap().offspring;
    for tick in [1, 1, 2, 2, 2, 3] { offspring::record(counts, tick); }
    let original = counts.clone();
    population.lift_shared(first + 1).unwrap();
    let total = |tick| population.groups.values()
        .map(|row| offspring::since(row.offspring.as_ref(), tick)).sum::<u64>();
    assert_eq!(total(1), 6);
    assert_eq!(total(2), 4);
    population.restrict(&Default::default());
    assert_eq!(population.groups[&first].offspring, original);
}

#[test]
fn counted_births_roll_back_with_the_advance() {
    let mut genesis = founding(600, true);
    genesis.rules.limits.events_per_advance = 10;
    let mut session = Session::new(genesis, Execution::Individuals).unwrap();
    let before = session.sim.state_hash();
    assert!(session.advance(1).unwrap_err().contains("operation budget"));
    assert_eq!(session.sim.state_hash(), before);
}

#[test]
fn a_record_holder_keeps_scheduled_family_identity() {
    let genesis = founding(60, true);
    let (&parent, _) = genesis.population.groups.iter()
        .find(|(_, group)| group.entity.lineage == "lineage:0").unwrap();
    let child = genesis.population.next_id;
    let mut session = Session::new(genesis, Execution::Individuals).unwrap();
    session.sim.state.record.note("mark:example".into(), 1, parent);
    session.advance(1).unwrap();
    let state = session.sim.state();
    assert!(state.relations.contains(&Relation::new(parent, "sim:child", child)));
    assert_eq!(state.events.len(), 1);
    assert_eq!(state.notes.len(), 1);
    assert!(state.population.get(child).is_some());
    assert_eq!(state.population.groups[&parent].count, 1);
}

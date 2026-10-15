// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Ruling 891: every generated world declares its global collapse predicate as
//! rules data (visible, saved, part of the digest), and it fires on extinction.

use isocosm::{Command, Execution, Founding, Session, collapse::WorldReading, digest, rules::Expr,
    schema::{Id, Method}, simulation::Genesis};

fn world() -> Genesis {
    Founding { population: 8, cohort_size: 4, ..Default::default() }.generate().unwrap()
}

fn organisms(session: &Session) -> Vec<Id> {
    session.sim.state().population.groups.iter()
        .filter(|(_, g)| g.entity.alive && g.entity.method != Method::Inert)
        .flat_map(|(&first, g)| first..first + g.count).collect()
}

#[test]
fn the_declared_predicate_is_visible_rules_data_naming_living_organisms() {
    let genesis = world();
    let predicate = genesis.rules.global_collapse.as_ref().expect("generated worlds declare their collapse");
    assert_eq!(predicate.inputs.len(), 1);
    let WorldReading::Population { selector } = predicate.inputs.values().next().unwrap() else { panic!("not a census") };
    assert!(selector.alive && !selector.methods.contains(&Method::Inert));
    assert!(matches!(predicate.condition, Expr::AtLeast(..)));
    predicate.validate_genesis(&genesis).unwrap();
}

#[test]
fn a_healthy_world_does_not_collapse_and_the_last_organism_does() {
    let mut session = Session::new(world(), Execution::Grouped).unwrap();
    let reading = session.sim.global_collapse().unwrap().unwrap();
    assert!(!reading.collapsed);
    let living = organisms(&session);
    assert!(living.len() >= 2);
    for &id in &living[..living.len() - 1] { session.command(Command::Kill { entity: id }).unwrap(); }
    assert!(!session.sim.global_collapse().unwrap().unwrap().collapsed, "one organism remains");
    session.command(Command::Kill { entity: *living.last().unwrap() }).unwrap();
    let reading = session.sim.global_collapse().unwrap().unwrap();
    assert!(reading.collapsed && reading.result == 1);
    assert_eq!(reading.inputs.values().copied().collect::<Vec<_>>(), [0]);
}

#[test]
fn the_world_ground_never_counts_as_an_organism() {
    let session = Session::new(world(), Execution::Grouped).unwrap();
    assert!(session.sim.state().population.groups.values().any(|g| g.entity.method == Method::Inert));
    let reading = session.sim.global_collapse().unwrap().unwrap();
    let census: i64 = reading.inputs.values().sum();
    assert_eq!(census as usize, organisms(&session).len());
}

#[test]
fn changing_the_predicate_changes_the_digest() {
    let genesis = world();
    let mut other = genesis.clone();
    let predicate = other.rules.global_collapse.as_mut().unwrap();
    predicate.condition = Expr::Const(0);
    assert_ne!(digest(&genesis), digest(&other));
    let mut none = genesis.clone();
    none.rules.global_collapse = None;
    assert_ne!(digest(&genesis), digest(&none));
    assert_eq!(digest(&genesis), digest(&world()), "generation is deterministic");
}

#[test]
fn save_and_replay_keep_the_predicate_and_the_collapse() {
    let mut session = Session::new(world(), Execution::Grouped).unwrap();
    session.advance(3).unwrap();
    for id in organisms(&session) { session.command(Command::Kill { entity: id }).unwrap(); }
    let before = session.sim.global_collapse().unwrap().unwrap();
    assert!(before.collapsed);
    let saved = session.save();
    let json = serde_json::to_vec(&saved).unwrap();
    let loaded = Session::load_json(&json, Execution::Grouped).unwrap();
    assert_eq!(loaded.sim.genesis().rules.global_collapse, session.sim.genesis().rules.global_collapse);
    assert_eq!(loaded.sim.global_collapse().unwrap(), Some(before));
    assert_eq!(loaded.sim.state_hash(), session.sim.state_hash());
    assert_eq!(loaded.save(), saved);
}

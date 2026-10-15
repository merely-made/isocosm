// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The society feature gates operations, never state (rulings 886 and 893).
//! Runs in both builds: `cargo test` has the feature, and
//! `cargo test --no-default-features` is the featureless build, which must
//! refuse every society operation and still read the one save format.

#[path = "owned_stock/fixture.rs"]
mod fixture;
use fixture::*;
use isocosm::{SOCIETY, custody, goals, party, institutions::Change as Institution, simulation::State};

const OFF: &str = "the society feature is off in this build";

fn commands(w: &World) -> Vec<(&'static str, Command)> {
    let [a, b, _] = w.bodies;
    vec![
        ("project", act(a, a, RESERVE, None, &["lot:none"])),
        ("project carry", Command::Project(isocosm::projects::Change::Carry {
            participant: a, actor: a, project: ProjectId(0), carried: true })),
        ("party", Command::Party(party::Change::Join { participant: a, party: "party:none".into(), member: b })),
        ("goal", Command::Goal(goals::Change::Adopt { actor: a, goal: "goal:none".into(),
            event: "event:none".into(), text: String::new() })),
        ("institution", Command::Institution(Institution::Apply { participant: a, actor: a,
            proposal: "proposal:none".into(), selections: Vec::new() })),
        ("custody", Command::Custody(custody::Change { participant: a, actor: a, item: b,
            provider: "provider:none".into(), action: custody::Action::Carry, cause: None,
            selections: Vec::new() })),
        ("owned act", Command::ActOwned { participant: a, actor: a, target: None, process: RESERVE.into(),
            cause: None, selections: Vec::new() }),
        ("stock ownership", Command::Assert(Assertion::StockOwnership(lot("lot:x", StockSource::Site(w.home), 1, &[])))),
    ]
}

#[test]
fn society_operations_are_refused_in_a_build_without_the_feature() {
    let w = world(91);
    let mut s = Session::new(w.genesis.clone(), Execution::Individuals).unwrap();
    let (hash, projects) = (s.sim.state_hash(), s.sim.state().projects.clone());
    for (name, command) in commands(&w) {
        let why = s.command(command).expect_err(name);
        assert_eq!(why == OFF, !SOCIETY, "{name}: {why}");
        assert_eq!(s.sim.state_hash(), hash, "{name} left a trace");
    }
    assert_eq!(s.sim.state().projects, projects);
    assert!(s.entries.is_empty(), "a refused command is not recorded");
}

#[test]
fn society_effects_and_queries_are_refused_without_the_feature() {
    let w = world(92);
    let [a, _, _] = w.bodies;
    let mut s = Session::new(w.genesis.clone(), Execution::Individuals).unwrap();
    let why = s.sim.project_process_readiness(a, None, RESERVE).expect_err("no lot is asserted");
    assert_eq!(why == OFF, !SOCIETY, "{why}");
    let why = s.sim.project_reservation_readiness(a, None, RESERVE, &["lot:none".into()]).expect_err("absent lot");
    assert_eq!(why == OFF, !SOCIETY, "{why}");
    let hash = s.sim.state_hash();
    let receipt = s.command(Command::Act { actor: a, target: None, process: RESERVE.into(), cause: None }).unwrap();
    let receipt: isocosm::simulation::Receipt = serde_json::from_str(&receipt).unwrap();
    assert!(!receipt.accepted(), "{:?}", receipt.outcome);
    assert!(s.sim.state().projects.is_empty());
    let _ = hash;
}

/// State fields stay in every build, so a save written with the layer in use reads in either.
#[test]
fn society_state_is_kept_and_reads_in_every_build() {
    let w = world(93);
    let s = Session::new(w.genesis.clone(), Execution::Individuals).unwrap();
    let mut json = serde_json::to_value(s.sim.state()).unwrap();
    let lot = serde_json::json!({ "lot": { "key": "lot:kept", "source": { "Site": w.home }, "account": TIMBER,
        "amount": 5, "owners": [{ "Entity": w.bodies[0] }] }, "at": "Source" });
    json["projects"] = serde_json::json!({ "records": {}, "bindings": {}, "ownership": { "lot:kept": lot } });
    let state: State = serde_json::from_value(json.clone()).unwrap();
    assert_eq!(state.projects.ownership["lot:kept"].lot.amount, 5);
    assert_eq!(serde_json::to_value(&state).unwrap(), json, "the society fields round-trip untouched");
    let plain = serde_json::to_value(s.sim.state()).unwrap();
    assert!(plain.get("projects").is_none() && plain.get("institutions").is_none(),
        "unused society fields stay empty and out of the save");
}

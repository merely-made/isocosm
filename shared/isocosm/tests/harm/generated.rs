// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The agentless hazard in generated worlds (the 2026-10-18 note to ruling
//! 227): the probe's definition, drawn per world, on bodied lineages only.

use isocosm::{Execution, Founding, Session, bodied::Bodies, harm::hazard, simulation::Genesis};
use std::collections::BTreeSet;

fn world(seed: u64, bodies: Option<Bodies>) -> Genesis {
    Founding {
        seed,
        sites: 2,
        population: 40,
        cohort_size: 1,
        lineages: 3,
        ecology: true,
        bodies,
        ..Default::default()
    }
    .generate()
    .unwrap()
}

fn wounded(s: &Session) -> usize {
    s.sim
        .state()
        .population
        .groups
        .values()
        .filter(|g| g.entity.parts.values().any(|p| !p.lost.is_empty()))
        .count()
}

#[test]
fn the_hazard_joins_bodied_worlds_on_their_bodied_lineages_only() {
    let g = world(3, Some(Bodies::default()));
    let p = &g.rules.processes[hazard::ID];
    let bodied: BTreeSet<_> = g
        .lineages
        .iter()
        .filter(|(_, l)| l.development.is_some())
        .map(|(k, _)| k.clone())
        .collect();
    assert_eq!(bodied.len(), 2, "flora and fauna have bodies, myco has none");
    assert_eq!(p.target.as_ref().unwrap().among, bodied);
    assert!(!world(3, None).rules.processes.contains_key(hazard::ID));
    let control = Bodies {
        hazard: [0, 0],
        wound: [0, 0],
        ..Bodies::default()
    };
    assert!(!world(3, Some(control)).rules.processes.contains_key(hazard::ID));
}

#[test]
fn a_world_draws_its_rate_within_the_range_and_the_probe_defines_the_process() {
    let drawn = |seed| {
        let g = world(seed, Some(Bodies::default()));
        serde_json::to_string(&g.rules.processes[hazard::ID]).unwrap()
    };
    let rates: BTreeSet<_> = (0..12).map(drawn).collect();
    assert!(rates.len() > 1, "seeds draw different rates or cells");
    let probe = isocosm::probe::BodyFounding {
        harm: Some(isocosm::probe::HarmFounding::default()),
        ..Default::default()
    }
    .generate()
    .unwrap();
    let theirs = &probe.genesis.rules.processes[hazard::ID];
    let mut ours = world(3, Some(Bodies::default())).rules.processes[hazard::ID].clone();
    ours.target.as_mut().unwrap().among.clear();
    let (a, b) = (serde_json::to_value(&ours).unwrap(), serde_json::to_value(theirs).unwrap());
    // The same definition: only the drawn amounts differ.
    assert_eq!(a["causation"], b["causation"]);
    assert_eq!(a["requires"], b["requires"]);
    assert_eq!(a["target"], b["target"]);
    assert_eq!(a["period"], b["period"]);
}

#[test]
fn the_hazard_wounds_generated_worlds_and_a_rate_of_nought_never_does() {
    let hits = Bodies {
        hazard: [64, 64],
        wound: [1, 2],
        ..Bodies::default()
    };
    let control = Bodies {
        hazard: [0, 0],
        wound: [0, 0],
        ..Bodies::default()
    };
    let mut hashes = Vec::new();
    for mode in [Execution::Individuals, Execution::Grouped] {
        let mut s = Session::new(world(5, Some(hits.clone())), mode).unwrap();
        let matter = s.sim.matter();
        assert_eq!(wounded(&s), 0);
        s.advance(24).unwrap();
        assert!(wounded(&s) > 0, "{mode:?}: a hit every tick wounds someone");
        assert_eq!(s.sim.matter(), matter, "{mode:?}: spills stay in the world");
        hashes.push(s.sim.state_hash());
        let mut s = Session::new(world(5, Some(control.clone())), mode).unwrap();
        s.advance(24).unwrap();
        assert_eq!(wounded(&s), 0, "{mode:?}: the control wounds no one");
    }
    assert_eq!(hashes[0], hashes[1], "grouped and individual runs agree");
}

#[test]
fn a_saved_founding_without_the_hazard_is_the_control_and_is_not_rewritten() {
    let old = serde_json::json!({ "variance": [1, 2], "absence": [12, 12], "clutch": [1, 4] });
    let loaded: Bodies = serde_json::from_value(old.clone()).unwrap();
    assert_eq!((loaded.hazard, loaded.wound), ([0, 0], [0, 0]));
    assert_eq!(
        serde_json::to_value(&loaded).unwrap(),
        old,
        "a founding without a hazard saves as before"
    );
    let new = serde_json::to_value(Bodies::default()).unwrap();
    assert_eq!((new["hazard"].clone(), new["wound"].clone()), ([2, 8].into(), [1, 8].into()));
    let invalid = |b: Bodies| b.check().is_err();
    assert!(invalid(Bodies { hazard: [9, 3], ..Bodies::default() }));
    assert!(invalid(Bodies { hazard: [1, 65], ..Bodies::default() }));
    assert!(invalid(Bodies { wound: [0, 0], ..Bodies::default() }), "a hit takes a cell");
}

// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The agentless hazard in generated worlds (the 2026-10-18 note to ruling
//! 227): the probe's definition, drawn per world, on bodied lineages only.

use isocosm::{Execution, Founding, Session, bodied::Bodies, harm::{hazard, rest}, simulation::Genesis};
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

/// Mesocosm's bodies: the default with the probe's hazard (704).
fn on() -> Bodies {
    Bodies {
        hazard: hazard::RATE,
        wound: hazard::CELLS,
        ..Bodies::default()
    }
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
fn the_hazard_and_rest_join_bodied_worlds_that_ask_for_them_on_bodied_lineages_only() {
    let g = world(3, Some(on()));
    let p = &g.rules.processes[hazard::ID];
    let bodied: BTreeSet<_> = g
        .lineages
        .iter()
        .filter(|(_, l)| l.development.is_some())
        .map(|(k, _)| k.clone())
        .collect();
    assert_eq!(bodied.len(), 2, "flora and fauna have bodies, myco has none");
    assert_eq!(p.target.as_ref().unwrap().among, bodied);
    assert!(g.rules.processes.contains_key(rest::ID), "a world that wounds also rests");
    assert!(g.rules.processes[rest::ID].target.is_none(), "any lineage rests");
    // The generator's own default has none, and bare worlds have no bodies.
    for none in [world(3, None), world(3, Some(Bodies::default()))] {
        assert!(!none.rules.processes.contains_key(hazard::ID));
        assert!(!none.rules.processes.contains_key(rest::ID));
    }
}

#[test]
fn a_world_draws_its_rate_within_the_range_and_the_probe_defines_the_process() {
    let drawn = |seed| {
        let g = world(seed, Some(on()));
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
    let mut ours = world(3, Some(on())).rules.processes[hazard::ID].clone();
    ours.target.as_mut().unwrap().among.clear();
    let (a, b) = (serde_json::to_value(&ours).unwrap(), serde_json::to_value(theirs).unwrap());
    // The same definition: only the drawn amounts differ.
    assert_eq!(a["causation"], b["causation"]);
    assert_eq!(a["requires"], b["requires"]);
    assert_eq!(a["target"], b["target"]);
    assert_eq!(a["period"], b["period"]);
}

#[test]
fn the_hazard_wounds_generated_worlds_rest_heals_them_and_a_rate_of_nought_never_does() {
    let hits = Bodies {
        hazard: [64, 64],
        wound: [1, 2],
        ..Bodies::default()
    };
    let without_rest = |bodies| {
        let mut g = world(5, Some(bodies));
        g.rules.processes.remove(rest::ID);
        g
    };
    let mut hashes = Vec::new();
    for mode in [Execution::Individuals, Execution::Grouped] {
        // Wounds with no rest to heal them stay.
        let mut s = Session::new(without_rest(hits.clone()), mode).unwrap();
        let matter = s.sim.matter();
        assert_eq!(wounded(&s), 0);
        s.advance(24).unwrap();
        assert!(wounded(&s) > 0, "{mode:?}: a hit every tick wounds someone");
        assert_eq!(s.sim.matter(), matter, "{mode:?}: spills stay in the world");
        hashes.push(s.sim.state_hash());
        // With it, every body that can pay a quarter of a cell rests.
        let mut s = Session::new(world(5, Some(hits.clone())), mode).unwrap();
        s.sim.watch(rest::ID);
        let matter = s.sim.matter();
        s.advance(24).unwrap();
        assert!(!s.sim.take_watched().is_empty(), "{mode:?}: the wounded rest");
        assert_eq!(s.sim.matter(), matter, "{mode:?}: and rest pays the soil");
        hashes.push(s.sim.state_hash());
        // A rate of nought is the control: nothing wounded, nothing rests.
        let mut s = Session::new(world(5, Some(Bodies::default())), mode).unwrap();
        s.sim.watch(rest::ID);
        s.advance(24).unwrap();
        assert_eq!(wounded(&s), 0, "{mode:?}: the control wounds no one");
        assert!(s.sim.take_watched().is_empty());
    }
    assert_eq!(hashes[0], hashes[2], "grouped and individual runs agree");
    assert_eq!(hashes[1], hashes[3], "with rest as well");
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
    let plain = serde_json::to_value(Bodies::default()).unwrap();
    assert!(plain.get("hazard").is_none() && plain.get("wound").is_none(), "the default names none");
    let new = serde_json::to_value(on()).unwrap();
    assert_eq!((new["hazard"].clone(), new["wound"].clone()), ([2, 8].into(), [1, 8].into()));
    let invalid = |b: Bodies| b.check().is_err();
    assert!(invalid(Bodies { hazard: [9, 3], ..on() }));
    assert!(invalid(Bodies { hazard: [1, 65], ..on() }));
    assert!(invalid(Bodies { wound: [0, 0], ..on() }), "a hit takes a cell");
}

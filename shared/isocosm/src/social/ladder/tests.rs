// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The ladder's invariants: its readings change nothing, and a world founded
//! by a quick-start pack replays to the same state and the same readings.

use super::*;
use crate::{Session, simulation::Execution};

/// Seeds nobody picked: draws under the test's own domain (Method 1).
fn seeds() -> impl Iterator<Item = u64> {
    (0..3).map(|i| crate::draw(0, "ladder-tests", &[i]))
}

fn everything(sim: &Simulation) -> String {
    let places = sim.place_readings().unwrap();
    let people = sim.denizens().unwrap();
    crate::digest(&(places, people))
}

#[test]
fn reading_the_ladder_changes_nothing() {
    for seed in seeds() {
        let s = crate::quickstart::session("mill-ford", seed, Execution::Individuals).unwrap();
        let before = crate::digest(s.sim.state());
        let first = everything(&s.sim);
        assert_eq!(before, crate::digest(s.sim.state()), "a reading wrote to the state");
        assert_eq!(first, everything(&s.sim), "a reading depends on reading");
        assert!(!s.sim.place_readings().unwrap().is_empty(), "the pack's location reads");
    }
}

#[test]
fn a_quick_start_world_replays_to_the_same_hash_and_readings() {
    for seed in seeds() {
        let s = crate::quickstart::session("mill-ford", seed, Execution::Individuals).unwrap();
        let back = Session::load(s.save(), Execution::Individuals).unwrap();
        assert_eq!(crate::digest(s.sim.state()), crate::digest(back.sim.state()));
        assert_eq!(everything(&s.sim), everything(&back.sim));
    }
}

#[test]
fn tiers_rise_and_start_at_one_resident() {
    let (t, _) = tiers();
    assert_eq!(t.tier(0), None);
    assert_eq!(t.tier(t.hamlet), Some(Tier::Hamlet));
    assert_eq!(t.tier(t.town), Some(Tier::Town));
    assert_eq!(t.tier(t.city), Some(Tier::City));
}


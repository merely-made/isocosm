// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Out of view a forager travels (case 1084): a hungry forager four cells
//! from its meal leaves with its route's travel time at its speed and lands
//! touching it, the game drawing the arrival; no step is asked of the game,
//! and the record replays.
#![allow(dead_code)]

use cosmograph::{Execution, Session, spatial as native};
use mesocosm_runtime::{
    Founded,
    spatial::{Admission, Game},
};
#[path = "foraging_play/fixtures.rs"]
mod fixtures;
use fixtures::*;

#[test]
fn a_hungry_forager_out_of_view_travels_to_its_meal_without_steps() {
    let mut f = travelling();
    assert!(f.session.sim.genesis().rules.speed.is_some());
    assert_eq!(f.distance(), 4);
    let mut ticks = 0;
    while f.distance() > 0 && ticks < 8 {
        f.tick();
        ticks += 1;
    }
    assert_eq!(f.distance(), 0, "the forager stands at its meal after {ticks} ticks");
    assert_eq!(f.steps(), 0, "no step was asked of the game");
    let sim = &f.session.sim;
    assert_eq!(f.game.members[&f.actor].pose, sim.state().spatial_poses[&f.actor].pose,
        "the game drew the arrival");
    assert!(sim.state().spatial_poses[&f.actor].travel.is_none());
    let restored = Session::load(f.session.save(), Execution::Grouped).unwrap();
    assert_eq!(restored.sim.state_hash(), sim.state_hash(), "travel replays from the record");
}


// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The tier line's physical side (E5, D32): a camera's frame roots the
//! members posed in it; posed members outside it and every root lose their
//! pose at the end of an advance.

use super::tests::{played, world};
use crate::{
    Execution,
    attention::{Region, WindowAttention},
    schema::*,
    spatial::{Pose, foraging::Resident},
};

#[test]
fn a_frame_keeps_its_members_posed_and_the_rest_leave_the_physical_path() {
    let mut s = world(3, Execution::Individuals);
    let (p, c) = played(&mut s);
    let site = s.sim.state().population.get(c).unwrap().resident_site().unwrap();
    // Every member at the critter's site posed in a row, one voxel apart.
    let members: Vec<(Id, Tick)> = s.sim.state().population.groups.iter()
        .filter(|(_, g)| g.entity.alive && g.entity.resident_site() == Some(site))
        .flat_map(|(&first, g)| (first..first + g.count).map(move |id| (id, g.entity.arrived)))
        .collect();
    assert!(members.len() > 4, "a site with members to frame");
    for (x, (id, arrived)) in members.iter().enumerate() {
        let pose = Pose { at: [x as i64, 0, 0], yaw: Default::default(), feet: Default::default() };
        s.sim.state.spatial_poses.insert(*id, Resident { site, arrived: *arrived, pose });
    }
    let frame = Region::Frame { site, min: [0, 0], max: [2, 1] };
    s.attach_window(p, "mesocosm".into(), WindowAttention { examined: Some(frame), ..Default::default() }).unwrap();
    s.advance(1).unwrap();
    let posed: Vec<Id> = s.sim.state().spatial_poses.keys().copied().collect();
    let framed: Vec<Id> = members.iter().take(2).map(|(id, _)| *id).collect();
    for id in &framed {
        assert!(posed.contains(id), "a member in the frame keeps its pose");
    }
    assert!(posed.iter().all(|id| framed.contains(id) || *id == c), "outside the frame only roots stay posed: {posed:?}");
    assert!(posed.len() < members.len(), "some members left the physical path");
}

#[test]
fn a_far_forager_finds_food_more_often_where_prey_is_plentiful() {
    let mut s = world(3, Execution::Individuals);
    let feed = "ecology:feed-1";
    assert!(s.sim.far_meal(Id::MAX - 1, feed), "with no far-food table, a far forager eats");
    let forager = *s.sim.state().population.groups.iter()
        .find(|(_, g)| g.entity.alive && g.entity.lineage == "lineage:1").unwrap().0;
    let site = s.sim.state().population.get(forager).unwrap().resident_site().unwrap();
    let rate = |s: &mut crate::Session, prey: u128| {
        s.sim.far_food = Some(crate::spatial::FarFood::from([((site, feed.to_string()), (prey, 1))]));
        (0..2000).filter(|t| { s.sim.state.tick = *t; s.sim.far_meal(forager, feed) }).count()
    };
    let (scarce, plenty) = (rate(&mut s, 1), rate(&mut s, 1_000_000));
    assert!(scarce < 1000 && plenty > 1900, "scarce {scarce}, plentiful {plenty}");
    assert_eq!(rate(&mut s, 0), 0, "no prey, no meal");
}

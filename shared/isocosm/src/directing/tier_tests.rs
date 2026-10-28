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

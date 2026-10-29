// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The tier line's physical side in a generated world (the execution plan's
//! E5, D32): a camera's frame draws members of its site in by the arrival
//! policy, and outside it only roots keep a pose.

use isocosm::{
    Command, Execution, Founding, Session,
    attention::{Region, WindowAttention},
    bodied::Bodies,
    directing::Played,
    map::{Grid, Layout},
    schema::Method,
};

#[test]
fn a_frame_draws_members_in_and_only_roots_stay_posed_outside_it() {
    let g = Founding {
        seed: 7,
        population: 256,
        sites: 6,
        ecology: true,
        bodies: Some(Bodies::default()),
        map: Some(Layout::Grid(Grid { width: 3, height: 2, ..Grid::drawn(7) })),
        played: Some(Played { lineage: 1, region_sites: 2 }),
        ..Default::default()
    }
    .generate()
    .unwrap();
    let t = g.rules.spatial_templates.clone().expect("map and bodies declare templates");
    let mut s = Session::new(g, Execution::Individuals).unwrap();
    let joined = s.command(Command::Join).unwrap();
    let p: u64 = joined.strip_prefix("participant:").unwrap().parse().unwrap();
    let pop = &s.sim.state().population;
    let critter = pop.groups.iter().find(|(_, g)| g.entity.method == Method::Deliberative && g.entity.alive)
        .map(|(id, _)| *id).unwrap();
    s.command(Command::Take { participant: p, critter }).unwrap();
    let site = s.sim.state().population.get(critter).unwrap().resident_site().unwrap();
    let mid = [0, 1].map(|k| (t.window.min[k] + t.window.max[k]) / 2);
    let frame = Region::Frame { site, min: t.window.min, max: mid };
    let inside = |at: [i64; 3]| (0..2).all(|k| t.window.min[k] <= at[2 * k] && at[2 * k] < mid[k]);
    let framed = |s: &Session| s.sim.state().spatial_poses.values().filter(|r| r.site == site && inside(r.pose.at)).count();
    let before = framed(&s);
    let members = s.sim.state().population.groups.values()
        .filter(|g| g.entity.alive && g.entity.body.is_some() && g.entity.journey.is_none() && g.entity.resident_site() == Some(site))
        .map(|g| g.count).sum::<u64>();
    s.attach_window(p, "mesocosm".into(), WindowAttention { examined: Some(frame.clone()), ..Default::default() }).unwrap();
    // How full a frame gets depends on its ground; this world's quarter has
    // few stands, so the test asks only that drawing adds members. A draw
    // stops at its first member with no stance (D40), so a frame fills over
    // several advances.
    let mut after = before;
    for _ in 0..8 {
        s.advance(1).unwrap();
        after = framed(&s);
        if after > before {
            break;
        }
    }
    assert!(members > 0 && after > before, "the frame draws members in: {before} then {after} of {members}");
    let state = s.sim.state();
    let at_site: Vec<_> = state.spatial_poses.iter().filter(|(_, r)| r.site == site).collect();
    for (id, r) in &at_site {
        assert!(inside(r.pose.at) || **id == critter, "outside the frame only roots keep a pose: {id}");
    }
}

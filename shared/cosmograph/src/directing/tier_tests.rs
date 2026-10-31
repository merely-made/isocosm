// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The tier line's physical side (E5, D32): a camera's frame roots the
//! members posed in it; posed members outside it and every root lose their
//! pose at the end of an advance. Far foragers eat by an encounter rate on
//! the prey their intake admits at their site (D35, D37), and wait there for
//! their feed while it holds food (D36).

use super::tests::{played, world};
use crate::{
    Execution, Session,
    attention::{Region, WindowAttention},
    rules::Process,
    schema::*,
    operations::Window,
    spatial::{FarFood, Pose, Templates, encounter, foraging::{Foraging, Resident}},
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
    let s = world(3, Execution::Individuals);
    assert!(s.sim.far_meal(Id::MAX - 1, "ecology:feed-1"), "with no far-food table, a far forager eats");
    let (scarce, plenty) = (encounter(1, 6), encounter(1_000_000, 6));
    assert!(scarce < 500_000 && plenty > 950_000, "scarce {scarce}, plentiful {plenty}");
    assert_eq!(encounter(0, 6), 0, "no prey, no meal");
}

/// A far forager at a site with prey its intake admits, its feed, and the
/// session with a far-food table naming its site.
fn far_forager(s: &mut Session) -> (Id, Id, Process) {
    let feed = s.sim.genesis().rules.processes["ecology:feed-1"].clone();
    let forager = s.sim.state().population.groups.iter()
        .filter(|(_, g)| g.entity.alive && g.entity.lineage == "lineage:1")
        .map(|(&first, _)| first)
        .find(|&id| s.sim.prey_matter(id, &feed) > 0)
        .expect("a forager with prey at its site");
    let site = s.sim.state().population.get(forager).unwrap().resident_site().unwrap();
    s.sim.far_food = Some(FarFood::from([((site, feed.id.clone()), 1)]));
    (forager, site, feed)
}

/// Moves every living body at `site` but the forager's own kind elsewhere.
fn only_its_kind(s: &mut Session, site: Id) {
    let away = *s.sim.state().sites.keys().find(|k| **k != site).unwrap();
    for g in s.sim.state.population.groups.values_mut() {
        if g.entity.resident_site() == Some(site) && g.entity.lineage != "lineage:1" {
            std::sync::Arc::make_mut(&mut g.entity).place = away;
        }
    }
}

#[test]
fn a_far_forager_counts_only_the_prey_its_intake_admits() {
    let mut s = world(3, Execution::Individuals);
    let (forager, site, feed) = far_forager(&mut s);
    assert!((0..200).any(|t| { s.sim.state.tick = t; s.sim.far_meal(forager, &feed.id) }));
    only_its_kind(&mut s, site);
    let kin: u128 = s.sim.state().population.groups.values()
        .filter(|g| g.entity.alive && g.entity.resident_site() == Some(site))
        .map(|g| crate::meaning::mass(&crate::anatomy::books(&g.entity), &s.sim.genesis().rules) * u128::from(g.count))
        .sum();
    assert!(kin > 0, "its own kind still holds matter there");
    assert_eq!(s.sim.prey_matter(forager, &feed), 0, "its own kind is no prey");
    assert!(!(0..200).any(|t| { s.sim.state.tick = t; s.sim.far_meal(forager, &feed.id) }));
}

#[test]
fn a_far_forager_waits_for_its_feed_where_its_site_holds_food() {
    let mut s = world(3, Execution::Individuals);
    let (forager, site, feed) = far_forager(&mut s);
    // Hungry: below what its body was founded with.
    let group = s.sim.state.population.groups.get_mut(&forager).unwrap();
    std::sync::Arc::make_mut(&mut group.entity).accounts.insert("matter:1-0".into(), 1);
    // Foraging templates without terrain: no member here is posed.
    let walk = Foraging { feed: "ecology:feed-1".into(), reach: 0, search: 1, nodes: 1, sight: false, reachable: false, far_half: 1 };
    std::sync::Arc::make_mut(&mut s.sim.genesis).rules.spatial_templates = Some(Templates {
        version: 3,
        window: Window { min: [0, 0], max: [1, 1], depth: 1 },
        places: Default::default(),
        eye: [0; 3],
        stands: Default::default(),
        crossings: vec![],
        foraging: [("body:walk:ability:cycle-1".into(), walk)].into(),
        body_spacing: 0,
        walking: None,
    });
    let period = feed.period.unwrap();
    assert!(period > 1, "a feed with ticks between its turns");
    s.sim.state.tick = s.sim.state.tick / period * period + 1;
    assert!(s.sim.state().spatial_poses.get(&forager).is_none(), "the forager is far");
    let waits = s.sim.foraging_wait_score(forager);
    assert!(waits > 0, "food at its site holds it there");
    assert_eq!(waits, s.sim.food_need_score(forager, &feed));
    only_its_kind(&mut s, site);
    assert_eq!(s.sim.foraging_wait_score(forager), 0, "no food, no waiting");
    s.sim.far_food = None;
    assert_eq!(s.sim.foraging_wait_score(forager), 0, "only the frame tier's far foragers wait at site grain");
}

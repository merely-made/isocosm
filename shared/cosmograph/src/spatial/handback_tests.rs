// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! An accepted handback ends the body's pending journey (E10, D52): the
//! journey would otherwise land the body again at its due tick.

use super::*;
use crate::{Command, Execution, Founding, Session, attention::{Region, WindowAttention}, bodied::Bodies,
    directing::Played, map::{Grid, Layout}, schema::Method};
use foraging::Travel;

/// The foreground test world (`tests/foreground_1080.rs`) at walking speed:
/// the session, the played critter and another body posed at its site.
fn world(walks: bool) -> (Session, Id, Id) {
    let mut g = Founding {
        seed: 8, population: 256, sites: 6, ecology: true, bodies: Some(Bodies::default()),
        map: Some(Layout::Grid(Grid { width: 3, height: 2, ..Grid::drawn(8) })),
        played: Some(Played { lineage: 1, region_sites: 2 }), ..Default::default()
    }.generate().unwrap();
    embodiment::declare(&mut g).unwrap();
    g.rules.speed = Some(crate::speed::Speed::froude(g.world.base_unit_micrometres));
    g.rules.foreground_walks = walks;
    let t = g.rules.spatial_templates.clone().unwrap();
    let mut s = Session::new(g, Execution::Individuals).unwrap();
    let p: u64 = s.command(Command::Join).unwrap().strip_prefix("participant:").unwrap().parse().unwrap();
    let critter = s.sim.state().spatial_poses.keys().copied()
        .find(|id| s.sim.state().population.get(*id).is_some_and(|e| e.alive && e.method == Method::Deliberative))
        .expect("a posed deliberative founder");
    s.command(Command::Take { participant: p, critter }).unwrap();
    let at = s.sim.state().spatial_poses[&critter].clone();
    let w = embodiment::site_window(&t, at.pose.at);
    let frame = Region::Frame { site: at.site, min: w.min, max: w.max };
    s.attach_window(p, "mesocosm".into(), WindowAttention { examined: Some(frame), ..Default::default() }).unwrap();
    let other = s.sim.state().spatial_poses.iter().find(|(id, r)| **id != critter && r.site == at.site)
        .map(|(id, _)| *id).expect("another body at the site");
    (s, critter, other)
}

/// Sends the critter on a journey to `other`, due next tick.
fn depart(s: &mut Session, critter: Id, other: Id) {
    let at = s.sim.state().spatial_poses[&other].pose.at;
    let arrives = s.sim.state().tick + 1;
    s.sim.state.spatial_poses.get_mut(&critter).unwrap().travel = Some(Travel { target: other, at, arrives });
}

fn minute(s: &mut Session, handbacks: &[Handback]) {
    let (mut births, mut steps, mut terrain) = (Vec::new(), Vec::new(), Vec::new());
    loop {
        match s.try_advance_round_spatial(1, &terrain, &steps, &[], &births, handbacks).unwrap() {
            SpatialAdvance::Advanced(_) => return,
            SpatialAdvance::Birth(p) => births.push(BirthResolution { request: p.request, geometry: Err("test".into()) }),
            SpatialAdvance::Step(p) => steps.push(Resolution { request: p.request, geometry: Err("test".into()) }),
            SpatialAdvance::Terrain(p) => terrain.push(TerrainResolution { request: p.request, geometry: Err("test".into()) }),
        }
    }
}

fn moved(s: &Session, critter: Id) -> Handback {
    let sim = &s.sim;
    let t = sim.genesis().rules.spatial_templates.clone().unwrap();
    let held = sim.state().spatial_poses[&critter].clone();
    let placed: BTreeMap<Id, _> = sim.state().spatial_poses.iter()
        .filter(|(id, r)| **id != critter && r.site == held.site).map(|(id, r)| (*id, r.pose.clone())).collect();
    let pose = embodiment::place_within(sim, &t, critter, held.site, &t.window, &placed).unwrap();
    let e = sim.state().population.get(critter).unwrap();
    Handback {
        actor: critter, site: held.site, body_revision: e.body_revision, terrain_revision: sim.terrain_revision(),
        geometry: ArrivalGeometry { window: embodiment::site_window(&t, pose.at),
            eye: std::array::from_fn(|k| pose.at[k] + t.eye[k]), pose, places: t.places },
        contacts: Vec::new(),
    }
}

fn pose_of(s: &Session, id: Id) -> Pose { s.sim.state().spatial_poses[&id].pose.clone() }

#[test]
fn an_accepted_handback_ends_the_journey_a_refused_one_keeps_it() {
    for walks in [true, false] {
        let (base, critter, other) = world(walks);
        // Control: with no handback the journey lands the body at its due tick.
        let mut control = base.clone();
        depart(&mut control, critter, other);
        let start = pose_of(&control, critter);
        minute(&mut control, &[]);
        assert!(control.sim.state().spatial_poses[&critter].travel.is_none(), "walks {walks}: the journey came due");
        assert_ne!(pose_of(&control, critter), start, "walks {walks}: the control body was landed by its journey");

        // Accepted: the body keeps the handed pose and its journey is gone.
        let mut accepted = base.clone();
        depart(&mut accepted, critter, other);
        let h = moved(&accepted, critter);
        minute(&mut accepted, std::slice::from_ref(&h));
        assert!(accepted.sim.spatial_handbacks()[0].patch.is_ok(), "{:?}", accepted.sim.spatial_handbacks());
        assert!(accepted.sim.state().spatial_poses[&critter].travel.is_none());
        assert_eq!(pose_of(&accepted, critter), h.geometry.pose, "walks {walks}: not moved again when due");
        minute(&mut accepted, &[]);
        assert_eq!(pose_of(&accepted, critter), h.geometry.pose, "walks {walks}: still there a tick later");

        // Refused: the journey stands as the sim had it, then lands.
        let mut refused = base.clone();
        depart(&mut refused, critter, other);
        let mut bad = moved(&refused, critter);
        bad.body_revision += 1;
        let before = refused.sim.state().spatial_poses[&critter].travel.clone();
        minute(&mut refused, std::slice::from_ref(&bad));
        assert!(refused.sim.spatial_handbacks()[0].patch.is_err());
        assert_eq!(pose_of(&refused, critter), pose_of(&control, critter), "walks {walks}: the journey ran as without it");
        assert!(before.is_some());
    }
}

#[test]
fn the_world_rule_is_absent_from_a_world_without_it() {
    let rules = |walks| {
        let mut g = Founding { seed: 8, population: 16, sites: 2, ..Default::default() }.generate().unwrap();
        g.rules.foreground_walks = walks;
        serde_json::to_string(&g.rules).unwrap()
    };
    let (off, on) = (rules(false), rules(true));
    assert!(!off.contains("foreground_walks"), "an absent rule writes no field");
    assert!(on.contains("\"foreground_walks\":true"));
    assert_eq!(on.replace(",\"foreground_walks\":true", ""), off, "the rule adds only its field");
}

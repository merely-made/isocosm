// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Case 1057: on a torus two sites high, each of a pair's two routes lands
//! its traveller at the edge of the destination that route enters by. With
//! `MESOCOSM_1057_PICTURE` set, the landings are written there as data for
//! a top-down picture.

use cosmograph::{Command, Execution, Session, rules::{Effect, Query}, spatial::embodiment};
use mesocosm_runtime::{Founded, spatial::Game};
use std::collections::BTreeMap;

/// Seed 14 draws a torus whose sites 0 and 3 both have ground; earlier torus
/// seeds (1, 1793482854536437400 among them) seat no one on one of a pair.
const SEED: u64 = 14;

struct Trip { founded: Founded, from: u64, to: u64, travellers: Vec<u64>, acts: Vec<(String, u8)>, side: i64 }

/// New play's founding (A1 rates, a spaced roster of varied bodies), each
/// of a seated site's two parallel routes given an act with no mood gate.
fn trip() -> Trip {
    let mut founded = Founded::generated(SEED, 60).unwrap();
    let mut domain = founded.genesis.founding.clone().unwrap();
    domain.ecology_rates = cosmograph::ecology::Rates::A1;
    let bodies = domain.bodies.as_mut().unwrap();
    bodies.roster = true; bodies.founding = Some("SpacedRoster".into()); bodies.proportions = true;
    bodies.limb_scale = [1, 2];
    founded.genesis = domain.generate().unwrap();
    founded.genesis.rules.epoch_ticks = 60;
    embodiment::declare_site(&mut founded.genesis).unwrap();
    assert_eq!(founded.genesis.rules.spatial_templates.as_ref().unwrap().version, embodiment::ENTERING);
    let mut s = Session::new(founded.genesis.clone(), Execution::Grouped).unwrap();
    let mut game = Game::default();
    game.found_generated(&mut s).unwrap();
    let mut seated = BTreeMap::<u64, Vec<u64>>::new();
    for (id, p) in &game.members { seated.entry(p.site.0).or_default().push(*id); }
    let g = &mut founded.genesis;
    let partner = |from: u64| g.sites[&from].routes.iter().map(|r| r.to)
        .find(|to| g.sites[&from].routes.iter().filter(|r| r.to == *to).count() == 2);
    // A destination with no ground seats no one and lands no one.
    let (&from, ids) = seated.iter().find(|(from, ids)| ids.len() >= 2
        && partner(**from).is_some_and(|to| seated.contains_key(&to))).expect("a seated torus pair");
    let to = partner(from).unwrap();
    let prefix = format!("move:{from}-{to}-");
    let routes: Vec<_> = g.rules.processes.values().filter_map(|p| p.effects.iter().find_map(|e| match e {
        Effect::Move { destination, via: Some(side) } if *destination == to && p.id.starts_with(&prefix) => Some((*side, p.clone())),
        _ => None,
    })).collect();
    assert_eq!(routes.len(), 2, "each parallel route has its own act");
    let mut acts = Vec::new();
    for (side, mut p) in routes {
        p.id = format!("test:route-{side}");
        p.requires.retain(|q| !matches!(q, Query::MoodBelow { .. }));
        let enters = g.sites[&from].routes.iter().find(|r| r.to == to && r.border.is_some_and(|b| b.side == side))
            .unwrap().border.unwrap().enters;
        g.rules.processes.insert(p.id.clone(), p);
        acts.push((format!("test:route-{side}"), enters));
    }
    let side = g.world.footprint.unwrap().side as i64;
    Trip { founded, from, to, travellers: ids[..2].to_vec(), acts, side }
}

fn check(trip: &Trip, landed: &BTreeMap<u64, [i64; 3]>) {
    let Trip { from, to, side, .. } = *trip;
    for (&actor, (_, enters)) in trip.travellers.iter().zip(&trip.acts) {
        let at = *landed.get(&actor).unwrap_or_else(|| panic!("traveller {actor} lands physically"));
        let z = at[2];
        eprintln!("traveller {actor} {from}->{to} entered by side {enters}: lands at {at:?} of a {side}-wide site");
        match enters {
            0 => assert!(z < side / 4, "entering by low z lands near it: {z}"),
            2 => assert!(z >= side - side / 4, "entering by high z lands near it: {z}"),
            other => panic!("a torus row pair enters by side 0 or 2, not {other}"),
        }
    }
}

#[test]
fn each_torus_route_arrives_at_the_edge_it_enters_by() {
    let trip = trip();
    let mut session = Session::new(trip.founded.genesis.clone(), Execution::Grouped).unwrap();
    let mut game = Game::default();
    game.found_generated(&mut session).unwrap();
    for (&actor, (process, _)) in trip.travellers.iter().zip(&trip.acts) {
        session.command(Command::Act { actor, target: None, process: process.clone(), cause: None }).unwrap();
        let j = session.sim.state().population.get(actor).unwrap().journey.unwrap();
        assert!(j.border.is_some(), "routed movement records the border crossed");
    }
    // Each landing is read as it happens: a native mover may set out again.
    let mut landed = BTreeMap::new();
    for _ in 0..12 {
        let _ = game.advance_clock(&mut session).unwrap();
        for &id in &trip.travellers {
            let resident = session.sim.state().population.get(id).and_then(|e| e.resident_site());
            if let Some(p) = game.members.get(&id).filter(|p| p.site.0 == trip.to && resident == Some(trip.to)) {
                landed.entry(id).or_insert(p.pose.at);
            }
        }
        if landed.len() == trip.travellers.len() { break; }
    }
    check(&trip, &landed);
    if let Ok(path) = std::env::var("MESOCOSM_1057_PICTURE") { picture(&trip, &session, &game, &path); }
    game.validate_replay(&session).unwrap();
    let restored = Session::load_json(&serde_json::to_vec(&session.save()).unwrap(), Execution::Grouped).unwrap();
    assert_eq!(restored.sim.state_hash(), session.sim.state_hash());
}

/// The landings as data for a top-down picture: each traveller's terrain
/// window (ground tops, water marked), its body's boxes and the edge it
/// entered by.
fn picture(trip: &Trip, session: &Session, game: &Game, path: &str) {
    let t = session.sim.genesis().rules.spatial_templates.as_ref().unwrap();
    let mut panels = Vec::new();
    for (&id, (_, enters)) in trip.travellers.iter().zip(&trip.acts) {
        let p = &game.members[&id];
        let v = session.sim.site_volume(trip.to, &embodiment::site_window(t, p.pose.at)).unwrap();
        let m = v.materials;
        let mut tops = Vec::new();
        for z in v.min[1]..v.max[1] { for x in v.min[0]..v.max[0] {
            let top = (v.datum..v.ceiling()).rev().find(|&y| v.material([x, y, z]).is_some_and(|c| c != m.air));
            tops.push(top.map_or(-1, |y| if v.material([x, y, z]) == Some(m.water) { -2 - y } else { y }));
        } }
        let doc = session.sim.state().population.get(id).unwrap().body.clone().unwrap();
        panels.push(serde_json::json!({ "id": id, "enters": enters, "at": p.pose.at, "min": v.min, "max": v.max,
            "tops": tops, "boxes": embodiment::boxes(&doc, &p.pose).unwrap() }));
    }
    let out = serde_json::json!({ "from": trip.from, "to": trip.to, "side": trip.side, "panels": panels });
    std::fs::write(path, serde_json::to_vec(&out).unwrap()).unwrap();
}

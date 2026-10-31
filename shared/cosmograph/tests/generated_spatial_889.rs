// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Ruling 889: a generated world declares spawn and arrival templates, derived
//! from its realized terrain, and declares no walking, terrain or move process.

use cosmograph::{Execution, Founding, Session, bodied::Bodies, digest, map::{Grid, Layout},
    rules::Effect, simulation::{Genesis, Simulation}, spatial::{STANDS_PER_SITE, Stand, Templates}};
use isometer_space::places::Places;
use std::sync::OnceLock;

fn founding(seed: u64) -> Founding {
    Founding { seed, population: 256, sites: 6, ecology: true, bodies: Some(Bodies::default()),
        map: Some(Layout::Grid(Grid { width: 3, height: 2, ..Grid::drawn(seed) })), ..Default::default() }
}

fn world() -> &'static Genesis {
    static WORLD: OnceLock<Genesis> = OnceLock::new();
    WORLD.get_or_init(|| founding(7).generate().unwrap())
}

fn templates() -> &'static Templates { world().rules.spatial_templates.as_ref().expect("map and bodies declare templates") }

#[test]
fn worlds_without_terrain_or_bodies_declare_none() {
    let bare = Founding { population: 8, ..Default::default() }.generate().unwrap();
    assert!(bare.rules.spatial_templates.is_none());
    let bodiless = Founding { bodies: None, ..founding(7) }.generate().unwrap();
    assert!(bodiless.rules.spatial_templates.is_none());
}

#[test]
fn stands_are_clear_ground_in_one_realized_patch() {
    let sim = Simulation::new(world().clone(), Execution::Grouped).unwrap();
    let templates = templates();
    assert_eq!(templates.stands.len(), world().sites.len());
    assert!(templates.stands.values().any(|stands| !stands.is_empty()), "some site has ground to stand on");
    for (&site, stands) in &templates.stands {
        assert!(stands.len() <= STANDS_PER_SITE);
        if stands.is_empty() { continue; }
        let w = &templates.window;
        let volume = sim.volume(site, w.min, w.max, w.depth).unwrap();
        let places = Places::derive(&volume, templates.places).unwrap();
        let m = volume.materials;
        for Stand { at, .. } in stands {
            let eye: [i64; 3] = std::array::from_fn(|k| at[k] + templates.eye[k]);
            assert_eq!(volume.material(*at), Some(m.air));
            assert_eq!(volume.material(eye), Some(m.air));
            let below = volume.material([at[0], at[1] - 1, at[2]]).unwrap();
            assert!(below != m.air && below != m.water, "a stand rests on solid ground");
            assert!(places.place_at(*at).is_some() && places.place_at(*at) == places.place_at(eye));
        }
    }
}

#[test]
fn every_crossing_follows_a_native_route_to_a_declared_stand() {
    let templates = templates();
    assert!(!templates.crossings.is_empty());
    for crossing in &templates.crossings {
        assert!(world().sites[&crossing.from].routes.iter().any(|r| r.to == crossing.to));
        assert!(templates.stands[&crossing.to].contains(&crossing.stand));
    }
    let routed = world().sites.values().map(|s| s.routes.len()).sum::<usize>();
    assert!(templates.crossings.len() <= routed);
}

#[test]
fn the_templates_add_no_walk_terrain_or_birth_declaration() {
    let rules = &world().rules;
    assert!(rules.step_profiles.is_empty() && rules.birth_profiles.is_empty() && rules.pace_profiles.is_empty());
    assert!(!rules.processes.values().flat_map(|p| &p.effects).any(|e|
        matches!(e, Effect::Step { .. } | Effect::Terrain { .. } | Effect::MovePaced { .. })));
}

#[test]
fn templates_are_deterministic_saved_and_part_of_the_digest() {
    assert_eq!(&founding(7).generate().unwrap(), world());
    let json = serde_json::to_vec(world()).unwrap();
    let back: Genesis = serde_json::from_slice(&json).unwrap();
    assert_eq!(&back, world());
    let mut moved = world().clone();
    let stands = moved.rules.spatial_templates.as_mut().unwrap().stands.values_mut().find(|s| !s.is_empty()).unwrap();
    stands[0].at[0] += 1;
    assert_ne!(digest(&moved), digest(world()));
    let mut gone = world().clone();
    gone.rules.spatial_templates = None;
    assert_ne!(digest(&gone), digest(world()));
    let session = Session::new(world().clone(), Execution::Grouped).unwrap();
    let loaded = Session::load_json(&serde_json::to_vec(&session.save()).unwrap(), Execution::Grouped).unwrap();
    assert_eq!(loaded.sim.genesis().rules.spatial_templates, world().rules.spatial_templates);
}

#[test]
fn tampered_templates_are_refused() {
    let tamper = |edit: &dyn Fn(&mut Templates)| {
        let mut genesis = world().clone();
        edit(genesis.rules.spatial_templates.as_mut().unwrap());
        genesis.validate().is_err()
    };
    assert!(tamper(&|t| { t.version += 1; }));
    assert!(tamper(&|t| { t.stands.insert(u64::MAX, vec![]); }));
    assert!(tamper(&|t| { let s = t.stands.values_mut().find(|s| !s.is_empty()).unwrap(); s[0].at[0] = t.window.max[0]; }));
    assert!(tamper(&|t| { let c = &mut t.crossings[0]; c.to = c.from; }));
    assert!(!tamper(&|_| {}));
}

#[test]
fn every_move_of_a_played_world_lands_on_a_declared_crossing() {
    let played = Founding { played: Some(cosmograph::directing::Played { lineage: 1, region_sites: 2 }), ..founding(7) }
        .generate().unwrap();
    let templates = played.rules.spatial_templates.as_ref().unwrap();
    let mut moves = 0;
    for process in played.rules.processes.values() {
        for effect in &process.effects {
            let Effect::Move { destination } = effect else { continue };
            let from: u64 = process.id.trim_start_matches("move:").split('-').next().unwrap().parse().unwrap();
            moves += 1;
            let seated = !templates.stands[destination].is_empty();
            assert_eq!(templates.crossings.iter().any(|c| c.from == from && c.to == *destination), seated, "{}", process.id);
        }
    }
    assert!(moves > 0);
}

// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A round's handbacks through Mesocosm's clock (case 1080): an accepted one
//! is drawn into the game, kept in its record and replays to the recorded
//! hash; a refused one leaves the source unchanged and is reported.
#![allow(dead_code)]

use cosmograph::{Execution, Session, spatial as native};
use mesocosm_runtime::{Founded, spatial::{Admission, Game}};
#[path = "foraging_play/fixtures.rs"]
mod fixtures;
use fixtures::*;
use std::collections::BTreeMap;

/// The actor's handback to another supported stance in its site.
fn handback(f: &Fixture) -> native::Handback {
    let sim = &f.session.sim;
    let t = sim.genesis().rules.spatial_templates.clone().unwrap();
    let held = sim.state().spatial_poses[&f.actor].clone();
    let placed: BTreeMap<u64, _> = sim.state().spatial_poses.iter()
        .filter(|(id, r)| **id != f.actor && r.site == held.site).map(|(id, r)| (*id, r.pose.clone())).collect();
    let pose = native::embodiment::place_within(sim, &t, f.actor, held.site, &t.window, &placed).unwrap();
    let e = sim.state().population.get(f.actor).unwrap();
    native::Handback {
        actor: f.actor, site: held.site, body_revision: e.body_revision, terrain_revision: sim.terrain_revision(),
        geometry: native::ArrivalGeometry { window: native::embodiment::site_window(&t, pose.at),
            eye: std::array::from_fn(|k| pose.at[k] + t.eye[k]), pose, places: t.places },
        contacts: Vec::new(),
    }
}

/// The game as a save writes it, read back against a session loaded from the
/// session's own save.
fn reload(f: &Fixture) -> (Game, Session) {
    let session = Session::load_json(&serde_json::to_vec(&f.session.save()).unwrap(), Execution::Grouped).unwrap();
    let game: Game = serde_json::from_slice(&serde_json::to_vec(&f.game).unwrap()).unwrap();
    (game, session)
}

#[test]
fn a_round_with_an_accepted_handback_replays_to_its_hash() {
    let mut f = control(false, 256, true);
    let h = handback(&f);
    assert_ne!(h.geometry.pose, f.game.members[&f.actor].pose, "the handback moves the body");
    f.game.advance_clock_round(&mut f.session, std::slice::from_ref(&h)).unwrap();
    let reports = f.session.sim.spatial_handbacks();
    assert!(reports.len() == 1 && reports[0].patch.is_ok(), "{reports:?}");
    assert_eq!(f.game.members[&f.actor].pose, h.geometry.pose, "the accepted pose is drawn into the game");
    assert_eq!(f.session.sim.state().spatial_poses[&f.actor].pose, h.geometry.pose);
    let record = f.game.driver.as_ref().unwrap().records.last().unwrap();
    assert_eq!(record.handbacks, vec![h.clone()]);
    assert_eq!(record.handback_reports, reports.to_vec());
    assert!(f.session.entries.last().unwrap().outcome.contains("handbacks"));
    f.tick();
    f.game.validate_replay(&f.session).unwrap();
    let (game, loaded) = reload(&f);
    assert_eq!(loaded.sim.state_hash(), f.session.sim.state_hash(), "the round replays to its hash");
    game.validate_replay(&loaded).unwrap();
    assert_eq!(game, f.game);
    let mut tampered = game.clone();
    tampered.driver.as_mut().unwrap().records[0].handbacks.clear();
    assert!(tampered.validate_replay(&loaded).is_err(), "the record must name the handback the entry carries");
}

#[test]
fn a_refused_handback_leaves_the_source_unchanged_and_is_reported() {
    let mut plain = control(false, 256, true);
    let mut f = control(false, 256, true);
    let mut stale = handback(&f);
    stale.body_revision += 1;
    plain.tick();
    f.game.advance_clock_round(&mut f.session, std::slice::from_ref(&stale)).unwrap();
    let reports = f.session.sim.spatial_handbacks();
    assert!(reports.len() == 1 && reports[0].patch.as_ref().is_err_and(|e| e.contains("source changed")), "{reports:?}");
    assert_eq!(f.session.sim.state_hash(), plain.session.sim.state_hash(), "a refusal changes nothing in the sim");
    assert_eq!(f.game.members, plain.game.members, "nor in the game");
    let record = f.game.driver.as_ref().unwrap().records.last().unwrap();
    assert_eq!(record.handbacks, vec![stale]);
    assert_eq!(record.handback_reports, reports.to_vec(), "the refusal is in the record");
    f.game.validate_replay(&f.session).unwrap();
    let (game, loaded) = reload(&f);
    assert_eq!(loaded.sim.state_hash(), f.session.sim.state_hash());
    game.validate_replay(&loaded).unwrap();
}

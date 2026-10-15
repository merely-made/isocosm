// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Ruling 889: a generated world's spawn and arrival templates reach the
//! existing spatial provider and door. Every pose below is one the generator
//! declared; a body with no declared template acquires none.

use isocosm::{Command, Execution, Session, schema::Id, spatial::Templates};
use mesocosm_runtime::{Founded, spatial::Game};

fn founded() -> Founded { Founded::generated(7, 256).unwrap() }

fn templates(session: &Session) -> Templates {
    session.sim.genesis().rules.spatial_templates.clone().expect("a generated world declares templates")
}

fn seated(founded: &Founded) -> (Session, Game) {
    let mut session = Session::new(founded.genesis.clone(), Execution::Grouped).unwrap();
    let mut game = Game::default();
    assert!(game.found_generated(&mut session).unwrap());
    (session, game)
}

#[test]
fn spawn_poses_are_the_declared_stands_and_the_world_is_saved_with_them() {
    let (session, game) = seated(&founded());
    let templates = templates(&session);
    assert!(!game.members.is_empty());
    for (id, at) in &game.members {
        let stands = &templates.stands[&at.site.0];
        let stand = stands.iter().find(|s| s.at == at.pose.at).expect("a declared stand");
        assert_eq!(at.pose.yaw, stand.yaw);
        assert_eq!(at.eye, std::array::from_fn(|k| stand.at[k] + templates.eye[k]));
        assert_eq!(at.window, templates.window);
        assert_eq!(session.sim.state().population.get(*id).unwrap().patch, Some(at.patch));
    }
    game.validate_replay(&session).unwrap();
    let saved: Game = serde_json::from_slice(&serde_json::to_vec(&game).unwrap()).unwrap();
    assert_eq!(saved, game);
    // At most the declared stands per site are used, in id order.
    for site in templates.stands.keys() {
        let at_site: Vec<Id> = game.members.iter().filter(|(_, a)| a.site.0 == *site).map(|(id, _)| *id).collect();
        assert!(at_site.len() <= templates.stands[site].len());
    }
}

#[test]
fn a_world_without_templates_seats_nobody() {
    let mut genesis = founded().genesis;
    genesis.rules.spatial_templates = None;
    let mut session = Session::new(genesis, Execution::Grouped).unwrap();
    let mut game = Game::default();
    assert!(!game.found_generated(&mut session).unwrap());
    assert!(game.members.is_empty() && game.driver.is_none());
}

#[test]
fn an_arrival_takes_its_declared_stand_and_an_unseated_body_takes_none() {
    let founded = founded();
    let (probe, seats) = seated(&founded);
    let templates = templates(&probe);
    // A seated member whose site has a declared crossing, and an unseated body
    // of that site that takes the same route.
    let (traveller, crossing) = seats.members.iter().find_map(|(id, at)| templates.crossings.iter()
        .find(|c| c.from == at.site.0).map(|c| (*id, c.clone()))).expect("a seated member has a crossing");
    // The played world's own move process per route (753) is mood-gated; the
    // test forces the same effect, since the generator declares no other mover.
    let route = format!("move:{}-{}", crossing.from, crossing.to);
    let mut forced = probe.sim.genesis().rules.processes[&route].clone();
    forced.id = "test:forced-move".into();
    forced.requires.retain(|q| !matches!(q, isocosm::rules::Query::MoodBelow { .. }));
    let mut genesis = probe.sim.genesis().clone();
    genesis.rules.processes.insert(forced.id.clone(), forced);
    let mut session = Session::new(genesis, Execution::Grouped).unwrap();
    let mut game = Game::default();
    assert!(game.found_generated(&mut session).unwrap());
    let stranger = session.sim.state().population.groups.iter()
        .find(|(first, g)| !game.members.contains_key(first) && g.entity.alive && g.entity.body.is_some()
            && g.entity.resident_site() == Some(crossing.from)).map(|(first, _)| *first).expect("an unseated body");
    let act = |session: &mut Session, actor| session.command(Command::Act {
        actor, target: None, process: "test:forced-move".into(), cause: None });
    act(&mut session, traveller).unwrap();
    let _ = act(&mut session, stranger);
    for _ in 0..12 {
        game.advance_clock(&mut session).unwrap();
        let here = session.sim.state().population.get(traveller).unwrap();
        if here.journey.is_none() && here.place == crossing.to { break; }
    }
    let here = session.sim.state().population.get(traveller).unwrap();
    assert_eq!((here.place, here.journey.is_none()), (crossing.to, true), "the forced move arrived");
    let driver = game.driver.as_ref().unwrap();
    let arrivals: Vec<_> = driver.records.iter().flat_map(|r| &r.changes)
        .filter(|c| format!("{:?}", c.admission) == "Arrival").collect();
    assert!(arrivals.iter().any(|c| c.actor == traveller && c.after.site.0 == crossing.to
        && c.after.pose.at == crossing.stand.at && c.after.pose.yaw == crossing.stand.yaw),
        "the traveller's first arrival takes the declared crossing stand");
    for change in &arrivals {
        let at = &change.after;
        assert!(templates.crossings.iter().any(|c| c.to == at.site.0 && c.stand.at == at.pose.at),
            "every arrival pose is a declared stand");
    }
    assert!(!game.members.contains_key(&stranger) && arrivals.iter().all(|c| c.actor != stranger),
        "no pose is invented for a body without a template");
    game.validate_replay(&session).unwrap();
}

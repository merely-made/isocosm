// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Interrupted work stays at creation; carrying is an explicit choice (840).
use super::*;
use isocosm::projects::Custody;

fn carry(participant: Id, actor: Id, project: ProjectId, carried: bool) -> Command {
    Command::Project(Change::Carry { participant, actor, project, carried })
}

fn go(r: &mut Reserved, w: &World, actor: Id) {
    let receipt = r.s.command(Command::Act { actor, target: None, process: GO.into(), cause: None }).unwrap();
    let receipt: isocosm::simulation::Receipt = serde_json::from_str(&receipt).unwrap();
    assert!(receipt.accepted(), "{:?}", receipt.outcome);
    while r.s.sim.state().population.get(actor).unwrap().resident_site() != Some(w.away) {
        r.s.advance(1).unwrap();
    }
}

#[test]
fn interrupted_work_stays_at_its_creation_site() {
    let w = world(41);
    let [a, b, c] = w.bodies;
    let mut r = reserved(&w);
    grant(&mut r, &w, a);
    acted(r.s.command(act(r.players[0], a, WORK, Some(r.id), &[])));
    go(&mut r, &w, a);
    let project = r.s.sim.state().projects.get(r.id).unwrap();
    assert_eq!((project.site, project.carrier), (w.home, None));
    assert_eq!(r.s.sim.project_site(project), Some(w.home));
    refused(&mut r.s, act(r.players[0], a, WORK, Some(r.id), &[]));
    grant(&mut r, &w, c);
    refused(&mut r.s, act(r.players[2], c, WORK, Some(r.id), &[]));
    grant(&mut r, &w, b);
    acted(r.s.command(act(r.players[1], b, WORK, Some(r.id), &[])));
    let project = r.s.sim.state().projects.get(r.id).unwrap();
    assert_eq!((project.site, project.progress), (w.home, 2), "changing worker does not move it");
    exact(&r.s, r.mass);
}

#[test]
fn carried_work_follows_its_carrier_and_rests_where_dropped() {
    let w = world(42);
    let [a, b, _] = w.bodies;
    let mut r = reserved(&w);
    refused(&mut r.s, carry(r.players[0], a, r.id, true));
    grant(&mut r, &w, a);
    grant(&mut r, &w, b);
    let hash = r.s.sim.state_hash();
    let held: Custody = serde_json::from_str(&r.s.command(carry(r.players[0], a, r.id, true)).unwrap()).unwrap();
    assert_ne!(r.s.sim.state_hash(), hash, "custody is in the replay witness");
    assert_eq!((held.carrier, held.site, held.matter), (Some(a), w.home, u128::from(RECIPE_MG)));
    refused(&mut r.s, carry(r.players[1], b, r.id, true));
    refused(&mut r.s, carry(r.players[1], b, r.id, false));
    acted(r.s.command(act(r.players[0], a, WORK, Some(r.id), &[])));
    go(&mut r, &w, a);
    let project = r.s.sim.state().projects.get(r.id).unwrap();
    assert_eq!((project.site, project.carrier), (w.home, Some(a)), "the record keeps its creation site");
    assert_eq!(r.s.sim.project_site(project), Some(w.away));
    refused(&mut r.s, act(r.players[1], b, WORK, Some(r.id), &[]));
    acted(r.s.command(act(r.players[0], a, WORK, Some(r.id), &[])));
    exact(&r.s, r.mass);
    refused(&mut r.s, act(r.players[0], a, WORK, Some(r.id), &[]));
    let dropped: Custody = serde_json::from_str(&r.s.command(carry(r.players[0], a, r.id, false)).unwrap()).unwrap();
    assert_eq!((dropped.carrier, dropped.site), (None, w.away));
    let out = acted(r.s.command(act(r.players[0], a, WORK, Some(r.id), &[])));
    let made = out.output.unwrap();
    assert_eq!(r.s.sim.state().population.get(made).unwrap().resident_site(), Some(w.away));
    assert_eq!(timber(&r.s, StockSource::Entity(made)), RECIPE_MG);
    exact(&r.s, r.mass);
}

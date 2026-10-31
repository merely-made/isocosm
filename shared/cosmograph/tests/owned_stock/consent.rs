// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Input owners grant together; any eligible authorized subject resumes (838, 839).
use super::*;
use cosmograph::projects::Consented;

fn decided(outcome: cosmograph::Result<String>) -> Consented { serde_json::from_str(&outcome.unwrap()).unwrap() }

#[test]
fn owners_of_the_reserved_inputs_grant_access_together() {
    let w = world(31);
    let [a, b, c] = w.bodies;
    let mut r = reserved(&w);
    assert!(!r.s.sim.project_authorized(r.s.sim.state().projects.get(r.id).unwrap(), a),
        "making and reserving grant the maker nothing");
    refused(&mut r.s, consent(r.players[2], c, r.id, a, true));
    refused(&mut r.s, consent(r.players[0], b, r.id, a, true));
    let first = decided(r.s.command(consent(r.players[0], a, r.id, a, true)));
    assert_eq!((first.authorized, first.round), (false, 1));
    refused(&mut r.s, act(r.players[0], a, WORK, Some(r.id), &[]));
    let second = decided(r.s.command(consent(r.players[1], b, r.id, a, true)));
    assert_eq!((second.authorized, second.round), (true, 1));
    exact(&r.s, r.mass);
    // A revocation is a new proposal, and the standing grant holds until both decide.
    let first = decided(r.s.command(consent(r.players[0], a, r.id, a, false)));
    assert_eq!((first.authorized, first.round), (true, 2));
    acted(r.s.command(act(r.players[0], a, WORK, Some(r.id), &[])));
    let second = decided(r.s.command(consent(r.players[1], b, r.id, a, false)));
    assert_eq!((second.authorized, second.round), (false, 2));
    refused(&mut r.s, act(r.players[0], a, WORK, Some(r.id), &[]));
    // Disagreeing owners reopen the proposal; nothing is granted.
    r.s.command(consent(r.players[0], a, r.id, a, true)).unwrap();
    let split = decided(r.s.command(consent(r.players[1], b, r.id, a, false)));
    assert!(!split.authorized);
    refused(&mut r.s, act(r.players[0], a, WORK, Some(r.id), &[]));
    exact(&r.s, r.mass);
}

#[test]
fn any_eligible_authorized_subject_resumes_anothers_work() {
    let w = world(32);
    let [a, b, _] = w.bodies;
    let mut r = reserved(&w);
    grant(&mut r, &w, a);
    acted(r.s.command(act(r.players[0], a, WORK, Some(r.id), &[])));
    refused(&mut r.s, act(r.players[1], b, WORK, Some(r.id), &[]));
    grant(&mut r, &w, b);
    for _ in 0..2 { acted(r.s.command(act(r.players[1], b, WORK, Some(r.id), &[]))); }
    let project = r.s.sim.state().projects.get(r.id).unwrap();
    assert_eq!((project.maker, project.progress), (a, 3));
    let made = project.output.expect("the resuming subject completes the same native project");
    assert_eq!(held(&r.s, "lot:bench"), (RECIPE_MG, LotAt::Output(made)));
    exact(&r.s, r.mass);
}

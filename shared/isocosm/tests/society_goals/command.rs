// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! An explicit goal record names an actual accepted event of the recording actor.
use super::*;
use isocosm::goals::{Change, Ending};

#[test]
fn an_explicit_record_binds_to_that_actors_accepted_event() {
    let w = goal_world(161);
    let [a, b, _] = w.bodies;
    let mut s = Session::new(w.genesis.clone(), Execution::Individuals).unwrap();
    let mine = perform(&mut s, a, IDLE);
    let theirs = perform(&mut s, b, IDLE);
    let adopt = |actor, event: &str| record(Change::Adopt { actor, goal: ROOF.into(), event: event.into(), text: "roof".into() });
    refused(&mut s, adopt(a, "event:none"));
    refused(&mut s, adopt(a, &theirs.id));
    refused(&mut s, record(Change::Adopt { actor: a, goal: String::new(), event: mine.id.clone(), text: String::new() }));
    s.command(adopt(a, &mine.id)).unwrap();
    assert_eq!(goal(&s, ROOF).adoption.id, mine.id);
    refused(&mut s, adopt(a, &mine.id));
    s.advance(1).unwrap();
    let later = perform(&mut s, a, IDLE);
    let end = |actor, event: &str| record(Change::End { actor, goal: ROOF.into(), event: event.into(), ending: Ending::Abandoned });
    let b_event = perform_id(&mut s, b);
    refused(&mut s, end(b, &b_event));
    s.command(end(a, &later.id)).unwrap();
    assert_eq!(goal(&s, ROOF).ending.map(|e| e.ending), Some(Ending::Abandoned));
    refused(&mut s, end(a, &later.id));
    exact(&s, s.sim.matter());
}

fn perform_id(s: &mut Session, actor: Id) -> String { perform(s, actor, IDLE).id }

#[test]
fn only_a_body_that_knows_the_adoption_serves_anothers_goal() {
    let w = goal_world(162);
    let [a, b, c] = w.bodies;
    let mut s = Session::new(w.genesis.clone(), Execution::Individuals).unwrap();
    perform(&mut s, a, ADOPT_HEARTH);
    s.advance(1).unwrap();
    let serve = |actor, event: &str| record(Change::Participate { actor, event: event.into(),
        goals: BTreeSet::from([HEARTH.to_string()]) });
    // c lives at the other site and never learned of the adoption; b stood beside it.
    let far = perform(&mut s, c, IDLE);
    let why = refused(&mut s, serve(c, &far.id));
    assert!(why.contains("know"), "{why}");
    let near = perform(&mut s, b, IDLE);
    s.command(serve(b, &near.id)).unwrap();
    assert_eq!(goal(&s, HEARTH).participation.iter().map(|p| p.actor).collect::<Vec<_>>(), [b]);
    exact(&s, s.sim.matter());
}

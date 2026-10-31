// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A process's authored directives record, with the accepted act, every goal it served (866).
use super::*;

#[test]
fn an_accepted_act_adopts_a_goal_under_its_own_event() {
    let w = goal_world(151);
    let [a, _, _] = w.bodies;
    let mut s = Session::new(w.genesis.clone(), Execution::Individuals).unwrap();
    let adopted = perform(&mut s, a, ADOPT_HEARTH);
    assert!(adopted.accepted(), "{:?}", adopted.outcome);
    let thread = goal(&s, HEARTH);
    assert_eq!((thread.holder, thread.adoption.id.clone(), thread.text.as_str()), (a, adopted.id.clone(), "keep a hearth"));
    assert!(thread.participation.is_empty() && thread.ending.is_none() && thread.holder_alive);
    // A goal identity is adopted once; the same directive again is refused and changes nothing.
    s.advance(1).unwrap();
    let hash = s.sim.state_hash();
    assert!(!perform(&mut s, a, ADOPT_HEARTH).accepted());
    assert_eq!(s.sim.state_hash(), hash);
    exact(&s, s.sim.matter());
}

#[test]
fn one_act_participates_in_every_goal_it_served() {
    let w = goal_world(152);
    let [a, _, _] = w.bodies;
    let mut s = Session::new(w.genesis.clone(), Execution::Individuals).unwrap();
    let hash = s.sim.state_hash();
    assert!(!perform(&mut s, a, SERVE_BOTH).accepted(), "serving a goal needs its adoption first");
    assert_eq!(s.sim.state_hash(), hash);
    perform(&mut s, a, ADOPT_HEARTH);
    s.advance(1).unwrap();
    perform(&mut s, a, ADOPT_ROOF);
    s.advance(1).unwrap();
    let served = perform(&mut s, a, SERVE_BOTH);
    assert!(served.accepted(), "{:?}", served.outcome);
    for key in [HEARTH, ROOF] {
        let thread = goal(&s, key);
        assert_eq!(thread.participation.iter().map(|p| (p.actor, p.event.id.clone())).collect::<Vec<_>>(),
            [(a, served.id.clone())], "{key} records the one act, with no primary goal chosen");
    }
    assert_eq!(s.sim.goal_threads().unwrap().len(), 2);
    exact(&s, s.sim.matter());
}

#[test]
fn an_ended_goal_takes_no_further_participation() {
    let w = goal_world(153);
    let [a, _, _] = w.bodies;
    let mut s = Session::new(w.genesis.clone(), Execution::Individuals).unwrap();
    perform(&mut s, a, ADOPT_HEARTH);
    s.advance(1).unwrap();
    let ended = perform(&mut s, a, END_HEARTH);
    assert!(ended.accepted(), "{:?}", ended.outcome);
    assert_eq!(goal(&s, HEARTH).ending.map(|end| (end.ending, end.event.id)), Some((cosmograph::goals::Ending::Met, ended.id)));
    s.advance(1).unwrap();
    let hash = s.sim.state_hash();
    assert!(!perform(&mut s, a, SERVE_LATE).accepted());
    assert!(!perform(&mut s, a, END_HEARTH).accepted());
    assert_eq!(s.sim.state_hash(), hash);
    exact(&s, s.sim.matter());
}

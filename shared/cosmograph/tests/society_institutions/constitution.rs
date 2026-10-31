// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Amendment and dissolution run the authored deciding process and keep the polity's identity.
use super::*;
use cosmograph::institutions::Request;

fn all_approve(s: &mut Session, key: &str, p: [Id; 3], [a, b, _]: [Id; 3]) {
    decided(s.command(approve(key, p[0], a)));
    assert!(decided(s.command(approve(key, p[1], b))).ready);
}

#[test]
fn an_amendment_changes_only_what_the_deciding_process_applies() {
    let w = charter_world(131);
    let [a, b, c] = w.bodies;
    let (mut s, p, id) = founded(&w);
    let was = constitution(&s, id);
    let amended = |edit: &dyn Fn(&mut cosmograph::schema::Constitution)| {
        let mut next = was.clone();
        edit(&mut next);
        Request::Amend { constitution: next, target: None, cause: None }
    };
    refused(&mut s, propose("p:cause", p[0], a, id, amended(&|n| n.founded_by = "cause:forged".into())));
    refused(&mut s, propose("p:method", p[0], a, id, amended(&|n| n.governance = "charter:none".into())));
    refused(&mut s, propose("p:account", p[0], a, id, amended(&|n| n.support_account = "account:none".into())));
    refused(&mut s, propose("p:host", p[0], a, id, amended(&|n| n.host = Some(c))));
    refused(&mut s, propose("p:participant", p[0], a, id, amended(&|n| { n.members.insert(p[2]); })));
    decided(s.command(propose("p:old", p[0], a, id, request("op:feast"))));
    decided(s.command(approve("p:old", p[0], a)));
    decided(s.command(propose("p:focus", p[0], a, id, amended(&|n| { n.focus.insert("focus:hearth".into()); }))));
    all_approve(&mut s, "p:focus", p, w.bodies);
    assert_eq!(constitution(&s, id), was, "approval alone changes nothing");
    let applied = decided(s.command(apply("p:focus", p[0], a))).applied.expect("applied");
    assert_eq!(applied.act.process, DECIDE);
    assert_eq!(constitution(&s, id).focus, BTreeSet::from(["focus:hearth".to_string()]));
    assert_eq!(constitution(&s, id).members, was.members);
    // A proposal made under the old constitution cannot proceed after it changed.
    refused(&mut s, approve("p:old", p[1], b));
    refused(&mut s, apply("p:old", p[0], a));
    decided(s.command(propose("p:fresh", p[0], a, id, request("op:feast"))));
    exact(&s, s.sim.matter());
}

#[test]
fn dissolution_ends_the_polity_by_its_deciding_event() {
    let w = charter_world(132);
    let [a, _, _] = w.bodies;
    let (mut s, p, id) = founded(&w);
    decided(s.command(propose("p:end", p[0], a, id, Request::End { target: None, cause: None })));
    all_approve(&mut s, "p:end", p, w.bodies);
    let applied = decided(s.command(apply("p:end", p[0], a))).applied.expect("applied");
    assert_eq!(s.sim.state().polities[&id.0].ended_by.as_deref(), Some(applied.act.event.as_str()));
    refused(&mut s, propose("p:after", p[0], a, id, request("op:feast")));
    exact(&s, s.sim.matter());
}

#[test]
fn a_changed_electorate_needs_a_fresh_proposal() {
    let w = charter_world(133);
    let [a, b, _] = w.bodies;
    let (mut s, p, id) = founded(&w);
    decided(s.command(propose("p:old", p[0], a, id, request("op:feast"))));
    decided(s.command(approve("p:old", p[0], a)));
    s.command(Command::Kill { entity: b }).unwrap();
    refused(&mut s, approve("p:old", p[1], b));
    refused(&mut s, apply("p:old", p[0], a));
    exact(&s, s.sim.matter());
}

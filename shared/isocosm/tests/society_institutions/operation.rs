// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A constitutional operation is proposed, decided by its electorate and applied by its executor.
use super::*;
use isocosm::{institutions::Change, projects::{DebitSelection, LotQuantity}};

#[test]
fn unanimous_members_decide_and_the_executor_applies_the_exact_operation() {
    let w = charter_world(121);
    let [a, b, c] = w.bodies;
    let (mut s, p, id) = founded(&w);
    let mass = s.sim.matter();
    refused(&mut s, propose("p:no-member", p[2], c, id, request("op:feast")));
    refused(&mut s, propose("p:no-control", p[1], a, id, request("op:feast")));
    refused(&mut s, propose("p:no-method", p[0], a, id, request("op:none")));
    let why = refused(&mut s, propose("p:no-support", p[0], a, id, request("op:rich")));
    assert!(why.contains("support"), "{why}");
    let proposed = decided(s.command(propose("p:feast", p[0], a, id, request("op:feast"))));
    assert!((proposed.ready, proposed.applied.is_none()) == (false, true));
    refused(&mut s, propose("p:feast", p[0], a, id, request("op:feast")));
    refused(&mut s, apply("p:feast", p[0], a));
    refused(&mut s, approve("p:feast", p[2], c));
    assert!(!decided(s.command(approve("p:feast", p[0], a))).ready, "one of two members is not unanimous");
    refused(&mut s, approve("p:feast", p[0], a));
    refused(&mut s, apply("p:feast", p[0], a));
    assert!(decided(s.command(approve("p:feast", p[1], b))).ready);
    refused(&mut s, apply("p:feast", p[1], b));
    let site = timber(&s, StockSource::Site(w.home));
    let applied = decided(s.command(apply("p:feast", p[0], a))).applied.expect("the executor applies it");
    assert_eq!((applied.act.actor, applied.act.process.as_str(), applied.act.polity), (a, FEAST, id));
    assert_eq!(timber(&s, StockSource::Site(w.home)), site + FEAST_MG);
    refused(&mut s, apply("p:feast", p[0], a));
    assert!(s.sim.state().institutions.proposals["p:feast"].applied.is_some());
    exact(&s, mass);
}

#[test]
fn authored_representatives_decide_for_the_polity_and_others_cannot_vote() {
    let w = charter_world(122);
    let [a, b, _] = w.bodies;
    let (mut s, p, id) = founded(&w);
    decided(s.command(propose("p:rep", p[0], a, id, request("op:rep"))));
    refused(&mut s, approve("p:rep", p[1], b));
    assert!(decided(s.command(approve("p:rep", p[0], a))).ready, "one authorized representative suffices");
    assert!(decided(s.command(apply("p:rep", p[0], a))).applied.is_some());
    exact(&s, s.sim.matter());
}

#[test]
fn an_applied_operation_debiting_titled_stock_needs_an_exact_selection() {
    let w = charter_world(123);
    let [a, _, c] = w.bodies;
    let (mut s, p, id) = founded(&w);
    let mass = s.sim.matter();
    assert_lot(&mut s, lot("lot:feast", StockSource::Entity(a), BODY_MG, &[c])).unwrap();
    decided(s.command(propose("p:rep", p[0], a, id, request("op:rep"))));
    decided(s.command(approve("p:rep", p[0], a)));
    refused(&mut s, apply("p:rep", p[0], a));
    let preview = s.sim.prepare_owned_act(a, None, FEAST, None).unwrap();
    let [debit] = preview.requests.as_slice() else { panic!("one exact debit") };
    assert_eq!((debit.source, debit.amount), (StockSource::Entity(a), FEAST_MG));
    let apply_with = |amount| institution(Change::Apply { participant: p[0], actor: a, proposal: "p:rep".into(),
        selections: vec![DebitSelection { request: debit.clone(), unowned: 0,
            lots: vec![LotQuantity { key: "lot:feast".into(), amount }] }] });
    refused(&mut s, apply_with(FEAST_MG - 1));
    assert!(decided(s.command(apply_with(FEAST_MG))).applied.is_some());
    assert_eq!(held(&s, "lot:feast"), (BODY_MG - FEAST_MG, LotAt::Source));
    exact(&s, mass);
}

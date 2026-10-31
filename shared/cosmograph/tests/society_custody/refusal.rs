// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Custody needs native control, contact, a provider, a free item and room to carry.
use super::*;
use cosmograph::custody::Action;

#[test]
fn custody_refuses_without_control_contact_provider_or_a_free_item() {
    let w = custody_world(71);
    let [a, b, c] = w.bodies;
    let (mut s, players) = session(&w);
    let item = arrive_item(&mut s, &w, w.home, ITEM_MG);
    let mass = s.sim.matter();
    refused(&mut s, custody(change(players[1], a, item, Action::Carry)));
    refused(&mut s, custody(change(players[2], c, item, Action::Carry)));
    refused(&mut s, custody(change(players[0], a, a, Action::Carry)));
    refused(&mut s, custody(change(players[0], a, b, Action::Carry)));
    let mut bad = change(players[0], a, item, Action::Carry);
    bad.provider = "provider:none".into();
    refused(&mut s, custody(bad));
    refused(&mut s, custody(change(players[0], a, item, Action::Drop)));
    refused(&mut s, custody(change(players[0], a, item, Action::Equip)));
    custody_acted(s.command(custody(change(players[0], a, item, Action::Carry))));
    refused(&mut s, custody(change(players[0], a, item, Action::Carry)));
    refused(&mut s, custody(change(players[1], b, item, Action::Carry)));
    refused(&mut s, custody(change(players[1], b, item, Action::Drop)));
    assert_eq!(s.sim.item_carrier(item).unwrap(), Some(a));
    exact(&s, mass);
}

#[test]
fn burden_counts_every_carried_item_and_project_against_capacity() {
    let w = custody_world(72);
    let [a, _, _] = w.bodies;
    let mut r = reserved(&w);
    let (one, two) = (arrive_item(&mut r.s, &w, w.home, ITEM_MG), arrive_item(&mut r.s, &w, w.home, ITEM_MG));
    let mass = r.s.sim.matter();
    custody_acted(r.s.command(custody(change(r.players[0], a, one, Action::Carry))));
    assert_eq!(r.s.sim.item_burden(a).unwrap(), u128::from(ITEM_MG));
    let why = refused(&mut r.s, custody(change(r.players[0], a, two, Action::Carry)));
    assert!(why.contains("capacity"), "{why}");
    custody_acted(r.s.command(custody(change(r.players[0], a, one, Action::Drop))));
    // A carried project's reserved matter is burden too, and keeps the next item out.
    grant(&mut r, &w, a);
    let carried = Command::Project(cosmograph::projects::Change::Carry {
        participant: r.players[0], actor: a, project: r.id, carried: true });
    r.s.command(carried).unwrap();
    assert_eq!(r.s.sim.item_burden(a).unwrap(), u128::from(RECIPE_MG));
    refused(&mut r.s, custody(change(r.players[0], a, two, Action::Carry)));
    exact(&r.s, mass);
}

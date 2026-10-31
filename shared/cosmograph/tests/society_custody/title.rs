// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Titles stay with the exact stock they name when its holder travels (845, 846, 850).
use super::*;
use cosmograph::custody::Action;
use cosmograph::projects::{DebitSelection, LotQuantity};

#[test]
fn a_titled_item_keeps_its_owners_through_carry_journey_and_drop() {
    let w = custody_world(81);
    let [a, b, _] = w.bodies;
    let (mut s, players) = session(&w);
    let item = arrive_item(&mut s, &w, w.home, ITEM_MG);
    let mass = s.sim.matter();
    assert_lot(&mut s, lot("lot:item", StockSource::Entity(item), 80, &[b])).unwrap();
    custody_acted(s.command(custody(change(players[0], a, item, Action::Carry))));
    depart(&mut s, a);
    arrive(&mut s, a, w.away);
    custody_acted(s.command(custody(change(players[0], a, item, Action::Drop))));
    let titled: Vec<_> = s.sim.available_lots().map(|(at, l)| (at, l.amount, l.owners.clone())).collect();
    assert_eq!(titled, [(StockSource::Entity(item), 80, BTreeSet::from([cosmograph::projects::Owner::Entity(b)]))]);
    assert_eq!(timber(&s, StockSource::Entity(item)), ITEM_MG);
    exact(&s, mass);
}

#[test]
fn an_equip_that_spends_titled_stock_needs_an_exact_selection_and_a_disposition() {
    let w = custody_world(82);
    let [a, b, _] = w.bodies;
    let (mut s, players) = session(&w);
    let item = arrive_item(&mut s, &w, w.home, ITEM_MG);
    let mass = s.sim.matter();
    assert_lot(&mut s, lot("lot:wearer", StockSource::Entity(a), BODY_MG, &[b])).unwrap();
    custody_acted(s.command(custody(change(players[0], a, item, Action::Carry))));
    let wear = change(players[0], a, item, Action::Equip);
    refused(&mut s, custody(wear.clone()));
    let preview = s.sim.prepare_owned_custody(&wear).unwrap();
    let [request] = preview.requests.as_slice() else { panic!("one exact debit of the wearer's timber") };
    assert_eq!(request.operation, 1, "the staged Relate is operation 0, the Spend operation 1");
    assert_eq!((request.source, request.amount), (StockSource::Entity(a), WEAR_MG));
    let mut short = wear.clone();
    short.selections = vec![DebitSelection { request: request.clone(), unowned: 0,
        lots: vec![LotQuantity { key: "lot:wearer".into(), amount: WEAR_MG - 1 }] }];
    refused(&mut s, custody(short));
    let mut exact_pick = wear;
    exact_pick.selections = vec![DebitSelection { request: request.clone(), unowned: 0,
        lots: vec![LotQuantity { key: "lot:wearer".into(), amount: WEAR_MG }] }];
    custody_acted(s.command(custody(exact_pick)));
    assert_eq!(held(&s, "lot:wearer"), (BODY_MG - WEAR_MG, LotAt::Source), "the unselected portion stays titled");
    assert_eq!(timber(&s, StockSource::Site(w.home)), SITE_MG + WEAR_MG);
    exact(&s, mass);
}

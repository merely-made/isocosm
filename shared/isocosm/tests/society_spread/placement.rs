// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Placing owned site stock into an identity retains or closes exact claims (846, 849, 850).
use super::*;

#[test]
fn placing_owned_stock_needs_an_exact_selection_and_a_disposition() {
    let sp = spread_world(191, Some(retaining(PLACED_MG, 0)));
    let (w, identity) = (&sp.w, &sp.identity);
    let [_, b, _] = w.bodies;
    let mut s = Session::new(w.genesis.clone(), Execution::Individuals).unwrap();
    let mass = s.sim.matter();
    assert_lot(&mut s, lot("lot:site", StockSource::Site(w.home), 300, &[b])).unwrap();
    let command = place(w, identity, PLACED_MG);
    refused(&mut s, command.clone());
    let request = quote(&s, &command);
    assert_eq!((request.source, request.amount), (StockSource::Site(w.home), PLACED_MG));
    refused(&mut s, owned_place(command.clone(), Vec::new()));
    refused(&mut s, owned_place(command.clone(), vec![pick(&request, PLACED_MG - 1)]));
    s.command(owned_place(command, vec![pick(&request, PLACED_MG)])).unwrap();
    assert_eq!(held(&s, "lot:site"), (200, LotAt::Source), "the unselected portion stays");
    assert_eq!(timber(&s, StockSource::Site(w.home)), SITE_MG - PLACED_MG);
    let claims: Vec<_> = s.sim.available_lots().filter(|(_, l)| l.account == identity.key())
        .map(|(at, l)| (at, l.amount, l.owners.clone())).collect();
    assert_eq!(claims, [(StockSource::Site(w.home), PLACED_MG, BTreeSet::from([Owner::Entity(b)]))],
        "the placed matter keeps its owners as the identity's claim");
    assert_eq!(s.sim.stock_balance(StockSource::Site(w.home), &identity.key()).unwrap(), PLACED_MG);
    exact(&s, mass);
}

#[test]
fn a_placement_closes_the_portion_its_disposition_closes() {
    let sp = spread_world(192, Some(retaining(60, 40)));
    let (w, identity) = (&sp.w, &sp.identity);
    let [_, b, _] = w.bodies;
    let mut s = Session::new(w.genesis.clone(), Execution::Individuals).unwrap();
    let mass = s.sim.matter();
    assert_lot(&mut s, lot("lot:site", StockSource::Site(w.home), 300, &[b])).unwrap();
    let command = place(w, identity, PLACED_MG);
    let request = quote(&s, &command);
    s.command(owned_place(command, vec![pick(&request, PLACED_MG)])).unwrap();
    let kept: Vec<_> = s.sim.available_lots().filter(|(_, l)| l.account == identity.key()).map(|(_, l)| l.amount).collect();
    assert_eq!(kept, [60], "40 mg of the identity is physically placed but its claim is closed");
    assert_eq!(s.sim.stock_balance(StockSource::Site(w.home), &identity.key()).unwrap(), PLACED_MG);
    exact(&s, mass);
}

#[test]
fn a_placement_without_a_consistent_disposition_refuses() {
    for disposition in [None, Some(retaining(PLACED_MG - 10, 0)), Some(retaining(PLACED_MG, 10))] {
        let sp = spread_world(193, disposition);
        let (w, identity) = (&sp.w, &sp.identity);
        let [_, b, _] = w.bodies;
        let mut s = Session::new(w.genesis.clone(), Execution::Individuals).unwrap();
        assert_lot(&mut s, lot("lot:site", StockSource::Site(w.home), 300, &[b])).unwrap();
        let command = place(w, identity, PLACED_MG);
        let request = quote(&s, &command);
        refused(&mut s, owned_place(command, vec![pick(&request, PLACED_MG)]));
        assert_eq!(held(&s, "lot:site"), (300, LotAt::Source));
        exact(&s, s.sim.matter());
    }
}

// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Ownership names exact quantity lots; reservation selects them whole (843, 845).
use super::*;

#[test]
fn ownership_names_exact_lots_that_never_exceed_actual_matter() {
    let w = world(11);
    let [a, b, _] = w.bodies;
    let mut s = Session::new(w.genesis.clone(), Execution::Individuals).unwrap();
    let participant = join_take(&mut s, a);
    let home = StockSource::Site(w.home);
    let mass = s.sim.matter();
    for bad in [lot("lot:a", home, SITE_MG + 1, &[a]), lot("lot:a", home, 0, &[a]),
        lot("lot:a", home, 10, &[]), lot("lot:a", home, 10, &[participant])] {
        refused(&mut s, Command::Assert(Assertion::StockOwnership(bad)));
    }
    assert_lot(&mut s, lot("lot:a", home, RECIPE_MG, &[a])).unwrap();
    refused(&mut s, Command::Assert(Assertion::StockOwnership(lot("lot:b", home, SITE_MG - RECIPE_MG + 1, &[b]))));
    refused(&mut s, Command::Assert(Assertion::StockOwnership(lot("lot:a", home, RECIPE_MG - 1, &[a]))));
    let hash = s.sim.state_hash();
    assert_lot(&mut s, lot("lot:a", home, RECIPE_MG, &[a])).unwrap();
    assert_eq!(s.sim.state_hash(), hash, "the identical assertion is idempotent");
    assert_lot(&mut s, lot("lot:b", home, SITE_MG - RECIPE_MG, &[b])).unwrap();
    assert_eq!(held(&s, "lot:a"), (RECIPE_MG, LotAt::Source));
    assert_eq!(timber(&s, home), SITE_MG, "claims record no second physical ledger");
    exact(&s, mass);
}

#[test]
fn later_incoming_matter_acquires_no_earlier_owners() {
    let w = world(12);
    let [a, b, _] = w.bodies;
    let mut s = Session::new(w.genesis.clone(), Execution::Individuals).unwrap();
    let home = StockSource::Site(w.home);
    assert_lot(&mut s, lot("lot:a", home, SITE_MG, &[a])).unwrap();
    refused(&mut s, Command::Assert(Assertion::StockOwnership(lot("lot:b", home, 1, &[b]))));
    s.command(Command::PlaceMatter { site: w.home, account: TIMBER.into(), amount: 500 }).unwrap();
    assert_eq!(timber(&s, home), SITE_MG + 500);
    let lots: Vec<_> = s.sim.source_lots().map(|lot| (lot.key.clone(), lot.amount)).collect();
    assert_eq!(lots, [("lot:a".to_string(), SITE_MG)], "the refill stays unowned");
    assert_lot(&mut s, lot("lot:b", home, 500, &[b])).unwrap();
    refused(&mut s, Command::Assert(Assertion::StockOwnership(lot("lot:c", home, 1, &[a]))));
    exact(&s, s.sim.matter());
}

#[test]
fn reservation_requires_explicit_owned_lots_matching_the_recipe_exactly() {
    let w = world(13);
    let [a, b, _] = w.bodies;
    let mut s = Session::new(w.genesis.clone(), Execution::Individuals).unwrap();
    let mass = s.sim.matter();
    let pa = join_take(&mut s, a);
    let home = StockSource::Site(w.home);
    refused(&mut s, act(pa, a, RESERVE, None, &[]));
    refused(&mut s, act(pa, a, RESERVE, None, &["lot:absent"]));
    assert_lot(&mut s, lot("lot:small", home, RECIPE_MG - 100, &[a])).unwrap();
    assert_lot(&mut s, lot("lot:body", StockSource::Entity(a), 100, &[b])).unwrap();
    refused(&mut s, act(pa, a, RESERVE, None, &["lot:small"]));
    refused(&mut s, act(pa, a, RESERVE, None, &["lot:small", "lot:small"]));
    refused(&mut s, act(pa, a, RESERVE, None, &["lot:small", "lot:body"]));
    assert!(s.sim.project_reservation_readiness(a, None, RESERVE, &["lot:small".into()]).is_err());
    assert_lot(&mut s, lot("lot:rest", home, 100, &[b])).unwrap();
    s.sim.project_reservation_readiness(a, None, RESERVE, &["lot:small".into(), "lot:rest".into()]).unwrap();
    let out = acted(s.command(act(pa, a, RESERVE, None, &["lot:small", "lot:rest"])));
    let id = out.project.unwrap();
    let project = s.sim.state().projects.get(id).unwrap().clone();
    assert_eq!(project.owners, BTreeSet::from([Owner::Entity(a), Owner::Entity(b)]));
    assert_eq!(project.reserved, Ledger::from([(TIMBER.into(), RECIPE_MG)]));
    assert_eq!((project.maker, project.site), (a, w.home));
    for key in ["lot:small", "lot:rest"] { assert_eq!(held(&s, key).1, LotAt::Project(id)); }
    assert_eq!(held(&s, "lot:body").1, LotAt::Source);
    assert_eq!(timber(&s, home), SITE_MG - RECIPE_MG);
    exact(&s, mass);
}

#[test]
fn reservation_needs_current_craft_eligibility() {
    let mut w = world(14);
    let catalogue = w.genesis.rules.projects.as_mut().unwrap();
    catalogue.definitions.get_mut(BENCH).unwrap().work = Work::new(CRAFT, 1, 0);
    w.genesis.validate().unwrap();
    let [a, _, _] = w.bodies;
    let mut s = Session::new(w.genesis.clone(), Execution::Individuals).unwrap();
    let pa = join_take(&mut s, a);
    assert_lot(&mut s, lot("lot:bench", StockSource::Site(w.home), RECIPE_MG, &[a])).unwrap();
    refused(&mut s, act(pa, a, RESERVE, None, &["lot:bench"]));
}

// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Owned debits select exact lot portions and author their disposition (846, 849, 850).
use super::*;
use isocosm::projects::{DebitRequest, DebitSelection, Destination, Disposition, DispositionRule,
    LotQuantity, OperationRule, OutputClaim, OwnedActed, StockRules};

fn owned(participant: Id, actor: Id, process: &str, selection: DebitSelection) -> Command {
    Command::ActOwned { participant, actor, target: None, process: process.into(), cause: None,
        selections: vec![selection] }
}

fn quote(s: &Session, actor: Id, process: &str) -> DebitRequest {
    let preview = s.sim.prepare_owned_act(actor, None, process, None).unwrap();
    assert!(preview.refusal.is_none(), "{:?}", preview.refusal);
    let [request] = preview.requests.as_slice() else { panic!("one exact native debit is quoted") };
    request.clone()
}

fn pick(request: &DebitRequest, lots: &[(&str, u64)], unowned: u64) -> DebitSelection {
    DebitSelection { request: request.clone(), unowned,
        lots: lots.iter().map(|(key, amount)| LotQuantity { key: key.to_string(), amount: *amount }).collect() }
}

/// An owned command answers with its receipt; a refusal changes nothing.
fn admit(s: &mut Session, command: Command) -> Option<OwnedActed> {
    let hash = s.sim.state_hash();
    let out: OwnedActed = serde_json::from_str(&s.command(command).unwrap()).unwrap();
    if out.receipt.accepted() { return Some(out); }
    assert_eq!(s.sim.state_hash(), hash, "{:?}", out.receipt.outcome);
    None
}

#[test]
fn an_owned_debit_selects_an_exact_portion_of_a_named_lot() {
    let w = world(51);
    let [a, b, _] = w.bodies;
    let mut s = Session::new(w.genesis.clone(), Execution::Individuals).unwrap();
    let mass = s.sim.matter();
    let pa = join_take(&mut s, a);
    let (home, body) = (StockSource::Site(w.home), StockSource::Entity(a));
    assert_lot(&mut s, lot("lot:site", home, RECIPE_MG, &[a, b])).unwrap();
    let hash = s.sim.state_hash();
    let plain = s.command(Command::Act { actor: a, target: None, process: HAUL.into(), cause: None }).unwrap();
    let plain: isocosm::simulation::Receipt = serde_json::from_str(&plain).unwrap();
    assert!(!plain.accepted(), "an unselected owned debit is not allocated automatically");
    assert_eq!(s.sim.state_hash(), hash);
    let request = quote(&s, a, HAUL);
    assert_eq!((request.source, request.amount), (home, HAUL_MG));
    for bad in [pick(&request, &[("lot:site", HAUL_MG + 1)], 0), pick(&request, &[("lot:site", HAUL_MG - 1)], 0),
        pick(&request, &[("lot:absent", HAUL_MG)], 0), pick(&request, &[], SITE_MG - RECIPE_MG + 1)] {
        assert!(admit(&mut s, owned(pa, a, HAUL, bad)).is_none());
    }
    let out = admit(&mut s, owned(pa, a, HAUL, pick(&request, &[("lot:site", HAUL_MG)], 0))).unwrap();
    assert_eq!(held(&s, "lot:site"), (RECIPE_MG - HAUL_MG, LotAt::Source), "the unselected portion stays");
    let [moved] = out.claims.as_slice() else { panic!("the selected portion travels as one exact claim") };
    let claim = &s.sim.state().projects.ownership[moved];
    assert_eq!((claim.lot.source, claim.lot.amount, claim.at), (body, HAUL_MG, LotAt::Source));
    assert_eq!(claim.lot.owners, BTreeSet::from([Owner::Entity(a), Owner::Entity(b)]));
    assert_eq!(timber(&s, body), BODY_MG + HAUL_MG);
    exact(&s, mass);
    assert!(admit(&mut s, owned(pa, a, HAUL, pick(&request, &[("lot:site", HAUL_MG)], 0))).is_none(),
        "a quote from before the last accepted act is stale");
    let request = quote(&s, a, HAUL);
    let out = admit(&mut s, owned(pa, a, HAUL, pick(&request, &[], HAUL_MG))).unwrap();
    assert!(out.claims.is_empty());
    assert_eq!(held(&s, "lot:site"), (RECIPE_MG - HAUL_MG, LotAt::Source));
    assert_eq!(timber(&s, home), SITE_MG - 2 * HAUL_MG);
    exact(&s, mass);
}

fn disposed(seed: u64, disposition: Option<Disposition>) -> (World, Session, Id, u128) {
    let mut w = world(seed);
    w.genesis.rules.owned_stock = disposition.map(|disposition| StockRules { selections: Vec::new(),
        dispositions: vec![DispositionRule { operation: OperationRule::Process { process: DROP.into(), ordinal: 0 },
            disposition }] });
    w.genesis.validate().unwrap();
    let mut s = Session::new(w.genesis.clone(), Execution::Individuals).unwrap();
    let mass = s.sim.matter();
    let pa = join_take(&mut s, w.bodies[0]);
    assert_lot(&mut s, lot("lot:carried", StockSource::Entity(w.bodies[0]), BODY_MG, &[w.bodies[0]])).unwrap();
    (w, s, pa, mass)
}

fn retain(amount: u64, closed: u64) -> Disposition {
    Disposition::Retain { outputs: vec![OutputClaim { input: "lot:carried".into(), amount,
        destination: Destination::Binding(Binding::Place), account: TIMBER.into() }],
        closed: (closed > 0).then(|| LotQuantity { key: "lot:carried".into(), amount: closed }).into_iter().collect() }
}

#[test]
fn consumption_without_an_authored_disposition_refuses() {
    for disposition in [None, Some(retain(DROP_MG - 40, 0))] {
        let (w, mut s, pa, mass) = disposed(52, disposition);
        let a = w.bodies[0];
        let request = quote(&s, a, DROP);
        assert!(admit(&mut s, owned(pa, a, DROP, pick(&request, &[("lot:carried", DROP_MG)], 0))).is_none());
        assert_eq!(held(&s, "lot:carried"), (BODY_MG, LotAt::Source));
        exact(&s, mass);
    }
}

#[test]
fn an_authored_disposition_closes_or_retains_each_consumed_portion() {
    let (w, mut s, pa, mass) = disposed(53, Some(Disposition::CloseConsumed));
    let a = w.bodies[0];
    let request = quote(&s, a, DROP);
    let out = admit(&mut s, owned(pa, a, DROP, pick(&request, &[("lot:carried", DROP_MG)], 0))).unwrap();
    assert!(out.claims.is_empty());
    assert_eq!(held(&s, "lot:carried"), (BODY_MG - DROP_MG, LotAt::Source));
    assert_eq!(timber(&s, StockSource::Site(w.home)), SITE_MG + DROP_MG, "closure leaves the matter conserved");
    assert!(s.sim.available_lots().all(|(at, _)| at == StockSource::Entity(a)));
    exact(&s, mass);

    let (w, mut s, pa, mass) = disposed(54, Some(retain(DROP_MG - 40, 40)));
    let request = quote(&s, a, DROP);
    let out = admit(&mut s, owned(pa, a, DROP, pick(&request, &[("lot:carried", DROP_MG)], 0))).unwrap();
    let [kept] = out.claims.as_slice() else { panic!("one authored retained output claim") };
    let kept = &s.sim.state().projects.ownership[kept];
    assert_eq!((kept.lot.source, kept.lot.amount), (StockSource::Site(w.home), DROP_MG - 40));
    assert_eq!(kept.lot.owners, BTreeSet::from([Owner::Entity(a)]));
    assert_eq!(held(&s, "lot:carried"), (BODY_MG - DROP_MG, LotAt::Source));
    exact(&s, mass);
}

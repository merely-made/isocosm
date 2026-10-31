// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A collective owner needs an explicit, valid and unambiguous strategy (844).
use super::*;
use cosmograph::projects::DecisionPolicy;

#[test]
fn a_collective_owner_must_name_its_strategy_validly() {
    let w = collective_world(181);
    let (mut s, p, id) = founded(&w);
    let mut bare = polity_lot("lot:bare", w.home, 100, id.0, DecisionPolicy::UnanimousMembers);
    bare.collectives.clear();
    refused(&mut s, Command::Assert(Assertion::StockOwnership(bare)));
    let mut stray = polity_lot("lot:stray", w.home, 100, id.0, DecisionPolicy::UnanimousMembers);
    stray.collectives.insert(id.0 + 100, DecisionPolicy::UnanimousMembers);
    refused(&mut s, Command::Assert(Assertion::StockOwnership(stray)));
    for invalid in [DecisionPolicy::Governance { process: FEAST.into() },
        DecisionPolicy::Representatives { actors: BTreeSet::new(), approvals: 1 },
        DecisionPolicy::Representatives { actors: BTreeSet::from([w.bodies[0]]), approvals: 2 }] {
        refused(&mut s, Command::Assert(Assertion::StockOwnership(polity_lot("lot:bad", w.home, 100, id.0, invalid))));
    }
    refused(&mut s, Command::Assert(Assertion::StockOwnership(StockLot { owners: BTreeSet::from([Owner::Polity(w.bodies[2])]),
        ..polity_lot("lot:ghost", w.home, 100, w.bodies[2], DecisionPolicy::UnanimousMembers) })));
    assert_lot(&mut s, polity_lot("lot:one", w.home, 300, id.0, DecisionPolicy::UnanimousMembers)).unwrap();
    assert_lot(&mut s, polity_lot("lot:two", w.home, 300, id.0,
        DecisionPolicy::Representatives { actors: BTreeSet::from([w.bodies[0]]), approvals: 1 })).unwrap();
    // Two lots of one polity under different strategies cannot be reserved together.
    let why = refused(&mut s, act(p[0], w.bodies[0], RESERVE, None, &["lot:one", "lot:two"]));
    assert!(why.contains("policy") || why.contains("ambiguous"), "{why}");
}

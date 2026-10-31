// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The bench world with one charter: feast, amend and end by unanimous members.
use super::*;
use cosmograph::{institutions::{self, Change, Operation, Outcome, PolityId, Request, Support},
    projects::{DecisionPolicy, Disposition, DispositionRule, OperationRule, StockRules},
    rules::Target, schema::Constitution};

pub const CHARTER: &str = "charter:band";
pub const FOUND: &str = "test:found";
pub const FEAST: &str = "test:feast";
pub const DECIDE: &str = "test:decide";
pub const FEAST_MG: u64 = 20;

fn operation(process: &str, decision: DecisionPolicy, support: Vec<Support>) -> Operation {
    Operation { process: process.into(), decision, support }
}

/// `a` and `b` found a polity `CHARTER`; its methods are `op:feast` (unanimous), `op:rep` (a alone),
/// `op:rich` (unanimous, needing a million mg of support), amend and end (unanimous, deciding).
pub fn charter_world(seed: u64) -> World {
    let mut w = world(seed);
    let [a, _, _] = w.bodies;
    let g = &mut w.genesis;
    let target = Target { same_place: true, alive: None, lineage: None, among: BTreeSet::new(), weighted: false };
    let mut found = process(FOUND, vec![], vec![Effect::FoundPolity { governance: CHARTER.into(),
        focus: BTreeSet::new(), support: TIMBER.into() }]);
    found.target = Some(target);
    let feast = process(FEAST, vec![], vec![Effect::Spend { from: vec![TIMBER.into()], to: Binding::Place,
        amount: FEAST_MG.into(), into: None }]);
    let decide = process(DECIDE, vec![Query::InstitutionDecision], vec![Effect::InstitutionDecision]);
    for p in [found, feast, decide] { g.rules.processes.insert(p.id.clone(), p); }
    let unanimous = || DecisionPolicy::UnanimousMembers;
    g.rules.institutions.insert(CHARTER.into(), institutions::Definition {
        operations: BTreeMap::from([
            ("op:feast".into(), operation(FEAST, unanimous(), vec![])),
            ("op:rep".into(), operation(FEAST, DecisionPolicy::Representatives { actors: BTreeSet::from([a]), approvals: 1 }, vec![])),
            ("op:rich".into(), operation(FEAST, unanimous(), vec![Support { source: Owner::Entity(a),
                account: TIMBER.into(), minimum: 1_000_000 }])),
        ]),
        amend: Some(operation(DECIDE, unanimous(), vec![])),
        end: Some(operation(DECIDE, unanimous(), vec![])),
    });
    g.rules.owned_stock = Some(StockRules { selections: Vec::new(), dispositions: vec![DispositionRule {
        operation: OperationRule::Process { process: FEAST.into(), ordinal: 0 }, disposition: Disposition::CloseConsumed }] });
    g.validate().unwrap();
    w
}

/// Three players and the polity `a` founds with `b`; its id is the founder's body.
pub fn founded(w: &World) -> (Session, [Id; 3], PolityId) {
    let [a, b, _] = w.bodies;
    let mut s = Session::new(w.genesis.clone(), Execution::Individuals).unwrap();
    let players = w.bodies.map(|body| join_take(&mut s, body));
    let receipt: cosmograph::simulation::Receipt = serde_json::from_str(&s.command(
        Command::Act { actor: a, target: Some(b), process: FOUND.into(), cause: None }).unwrap()).unwrap();
    assert!(receipt.accepted(), "{:?}", receipt.outcome);
    (s, players, PolityId(a))
}

pub fn institution(change: Change) -> Command { Command::Institution(change) }

pub fn propose(key: &str, participant: Id, actor: Id, polity: PolityId, request: Request) -> Command {
    institution(Change::Propose { key: key.into(), participant, actor, polity, request })
}

pub fn request(operation: &str) -> Request { Request::Act { operation: operation.into(), target: None, cause: None } }

pub fn approve(key: &str, participant: Id, actor: Id) -> Command {
    institution(Change::Approve { participant, actor, proposal: key.into(), process: None, target: None, cause: None,
        selections: Vec::new() })
}

pub fn apply(key: &str, participant: Id, actor: Id) -> Command {
    institution(Change::Apply { participant, actor, proposal: key.into(), selections: Vec::new() })
}

pub fn decided(result: cosmograph::Result<String>) -> Outcome { serde_json::from_str(&result.unwrap()).unwrap() }

pub fn constitution(s: &Session, polity: PolityId) -> Constitution {
    s.sim.state().polities[&polity.0].constitution.clone()
}

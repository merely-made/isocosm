// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The bench world with one spread identity and owned site timber to place into it.
use super::*;
use isocosm::{projects::{DebitSelection, Destination, Disposition, DispositionRule, LotQuantity, OperationRule,
    OutputClaim, DirectOperation, SpreadOutput, RetainedOutput, StockRules},
    spread::{self, Identity, Placement, Substrate, Supply}};

pub const PLACED_MG: u64 = 100;

pub struct Spread { pub w: World, pub identity: Identity }

/// `rules` is the authored stock disposition for a placement, absent for none.
pub fn spread_world(seed: u64, disposition: Option<Disposition>) -> Spread {
    let mut w = world(seed);
    let lineage = w.genesis.population.groups[&w.bodies[0]].entity.lineage.clone();
    let identity = Identity::Surface(lineage);
    w.genesis.spreads.identities = BTreeSet::from([identity.clone()]);
    w.genesis.rules.owned_stock = disposition.map(|disposition| StockRules { selections: Vec::new(),
        dispositions: vec![DispositionRule { operation: OperationRule::Command(DirectOperation::SpreadPlace), disposition }] });
    w.genesis.validate().unwrap();
    Spread { w, identity }
}

pub fn place(world: &World, identity: &Identity, amount: u64) -> Command {
    Command::Spread(spread::Command::Place(Placement {
        source: Supply::Matter { from: Substrate::Site(world.home), account: TIMBER.into() },
        identity: identity.clone(), at: Substrate::Site(world.home), amount }))
}

pub fn owned_place(command: Command, selections: Vec<DebitSelection>) -> Command {
    Command::Owned { command: Box::new(command), selections }
}

/// The placement retaining `kept` of the selected claim as the identity and closing `closed`.
pub fn retaining(kept: u64, closed: u64) -> Disposition {
    Disposition::Outputs { outputs: vec![RetainedOutput { input: "lot:site".into(), amount: kept,
        output: SpreadOutput::Placement }],
        closed: (closed > 0).then(|| LotQuantity { key: "lot:site".into(), amount: closed }).into_iter().collect() }
}

pub fn pick(request: &isocosm::projects::DebitRequest, amount: u64) -> DebitSelection {
    DebitSelection { request: request.clone(), unowned: 0, lots: vec![LotQuantity { key: "lot:site".into(), amount }] }
}

pub fn quote(s: &Session, command: &Command) -> isocosm::projects::DebitRequest {
    let preview = s.prepare_owned_command(command).unwrap();
    assert!(preview.refusal.is_none(), "{:?}", preview.refusal);
    let [request] = preview.requests.as_slice() else { panic!("one exact debit is quoted") };
    request.clone()
}

#[allow(dead_code)]
fn unused(_: OutputClaim, _: Destination) {}

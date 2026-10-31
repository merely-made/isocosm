// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The bench world with an authored item custody provider and a stock-costing equip.
use super::*;
use isometer_core::VolumeRef;
use cosmograph::geometry::Body;
use cosmograph::{arrival::Arrival, schema::BodyDocument, custody::{self, Action, Attachment, Yaw},
    projects::{Disposition, DispositionRule, OperationRule, StockRules},
    rules::{Expr, Target}, simulation::Receipt};

pub const PROVIDER: &str = "provider:hands";
pub const CARRIES: &str = "relation:carries";
pub const TAKE: &str = "test:take";
pub const PUT: &str = "test:put";
pub const WEAR: &str = "test:wear";
pub const DOFF: &str = "test:doff";
pub const CAPACITY: i64 = 200;
pub const ITEM_MG: u64 = 120;
pub const WEAR_MG: u64 = 10;

/// A one-part physical body: founded critters carry none until embodied.
pub fn body() -> BodyDocument { BodyDocument::new(VolumeRef([0; 32]), [1, 1, 1]) }

fn custody_process(id: &str, present: bool, effects: Vec<Effect>) -> Process {
    let mut p = process(id, vec![], [vec![Effect::Relate { kind: CARRIES.into(), present }], effects].concat());
    p.target = Some(Target { same_place: true, alive: None, lineage: None, among: BTreeSet::new(), weighted: false });
    p
}

/// `world` plus hands that carry up to `CAPACITY` mg, and a wear that spends `WEAR_MG` of the actor's timber.
pub fn custody_world(seed: u64) -> World {
    let mut w = world(seed);
    let g = &mut w.genesis;
    let root = body().root;
    // Each played body gets its physical document at founding; the first holds its timber in its part.
    let matter: BTreeSet<_> = g.rules.accounts.iter()
        .filter(|(_, kind)| matches!(kind, AccountKind::Matter { .. })).map(|(key, _)| key.clone()).collect();
    for id in w.bodies {
        let e = g.population.groups.get_mut(&id).unwrap().entity_mut();
        let mut part = cosmograph::schema::Part::default();
        for key in &matter { if let Some(held) = e.accounts.remove(key) { part.matter.insert(key.clone(), held); } }
        e.embody(Body { doc: body(), parts: BTreeMap::from([(root, part)]) });
    }
    g.rules.relations.insert(CARRIES.into());
    let wear = Effect::Spend { from: vec![TIMBER.into()], to: Binding::Place, amount: WEAR_MG.into(), into: None };
    for p in [custody_process(TAKE, true, vec![]), custody_process(PUT, false, vec![]),
        custody_process(WEAR, true, vec![wear]), custody_process(DOFF, true, vec![])] {
        g.rules.processes.insert(p.id.clone(), p);
    }
    g.rules.item_custody = Some(custody::Catalogue { providers: BTreeMap::from([(PROVIDER.into(), custody::Provider {
        relation: CARRIES.into(), capacity: Expr::Const(CAPACITY), carry: TAKE.into(), drop: PUT.into(),
        equip: WEAR.into(), unequip: DOFF.into(),
        mounts: BTreeSet::from([Attachment { parent: root, offset: [0, 0, 0], yaw: Yaw::Zero }]) })]) });
    g.rules.owned_stock = Some(StockRules { selections: Vec::new(), dispositions: vec![DispositionRule {
        operation: OperationRule::Process { process: WEAR.into(), ordinal: 1 }, disposition: Disposition::CloseConsumed }] });
    g.validate().unwrap();
    w
}

/// An inert, embodied item of `mg` timber at `site`, issued through the native door.
pub fn arrive_item(s: &mut Session, w: &World, site: Id, mg: u64) -> Id {
    let lineage = s.sim.state().population.get(w.bodies[0]).unwrap().lineage.clone();
    let out = s.command(Command::Arrive(Arrival { lineage, site, method: Method::Inert,
        accounts: Ledger::from([(TIMBER.into(), mg)]), traits: BTreeSet::new(), skills: BTreeMap::new(),
        disposition: [0; 5], character: None })).unwrap();
    let item: Id = out.strip_prefix("entity:").unwrap().parse().unwrap();
    s.command(Command::Embody { entity: item, body: body() }).unwrap();
    item
}

/// Body 0 is controlled by the first participant, body 1 by the second, body 2 by the third.
pub fn session(w: &World) -> (Session, [Id; 3]) {
    let mut s = Session::new(w.genesis.clone(), Execution::Individuals).unwrap();
    let players = w.bodies.map(|body| join_take(&mut s, body));
    (s, players)
}

pub fn change(participant: Id, actor: Id, item: Id, action: Action) -> custody::Change {
    custody::Change { participant, actor, item, provider: PROVIDER.into(), action, cause: None, selections: Vec::new() }
}

pub fn custody(change: custody::Change) -> Command { Command::Custody(change) }

pub fn custody_acted(outcome: cosmograph::Result<String>) -> custody::CustodyActed {
    serde_json::from_str(&outcome.unwrap()).unwrap()
}

/// The actor starts a native journey to `away` and the clock runs it to arrival.
pub fn depart(s: &mut Session, actor: Id) {
    let receipt: Receipt = serde_json::from_str(&s.command(
        Command::Act { actor, target: None, process: GO.into(), cause: None }).unwrap()).unwrap();
    assert!(receipt.accepted(), "{:?}", receipt.outcome);
}

pub fn arrive(s: &mut Session, actor: Id, site: Id) {
    while s.sim.state().population.get(actor).unwrap().resident_site() != Some(site) { s.advance(1).unwrap(); }
}

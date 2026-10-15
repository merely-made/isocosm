// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The charter world plus a project-deciding process, and a polity-owned bench reservation.
use super::*;
use isocosm::{projects::{DecisionPolicy, Permission}, rules::Target};

pub const PROJECT_DECIDE: &str = "test:project-decide";

pub fn collective_world(seed: u64) -> World {
    let mut w = charter_world(seed);
    let mut decide = process(PROJECT_DECIDE, vec![Query::ProjectDecision], vec![Effect::ProjectDecision]);
    decide.target = Some(Target { same_place: true, alive: None, lineage: None, among: BTreeSet::new(), weighted: false });
    w.genesis.rules.processes.insert(decide.id.clone(), decide);
    w.genesis.validate().unwrap();
    w
}

/// A lot of timber owned by the polity under `policy`.
pub fn polity_lot(key: &str, site: Id, amount: u64, polity: Id, policy: DecisionPolicy) -> StockLot {
    StockLot { key: key.into(), source: StockSource::Site(site), account: TIMBER.into(), amount,
        owners: BTreeSet::from([Owner::Polity(polity)]), collectives: BTreeMap::from([(polity, policy)]) }
}

/// The polity `a` founded with `b`, and its bench reserved from two polity lots under `policy`.
pub struct Owned { pub s: Session, pub p: [Id; 3], pub polity: Id, pub project: ProjectId, pub mass: u128 }

pub fn owned(w: &World, policy: DecisionPolicy) -> Owned {
    let [a, _, _] = w.bodies;
    let (mut s, p, id) = founded(w);
    let mass = s.sim.matter();
    for key in ["lot:half-1", "lot:half-2"] {
        assert_lot(&mut s, polity_lot(key, w.home, RECIPE_MG / 2, id.0, policy.clone())).unwrap();
    }
    let out = acted(s.command(act(p[0], a, RESERVE, None, &["lot:half-1", "lot:half-2"])));
    assert!(out.receipt.accepted(), "{:?}", out.receipt.outcome);
    Owned { s, p, polity: id.0, project: out.project.unwrap(), mass }
}

pub fn polity_consent(project: ProjectId, polity: Id, participant: Id, actor: Id, worker: Id, process: Option<&str>) -> Command {
    Command::Project(isocosm::projects::Change::Consent { participant, actor, project,
        owner: Owner::Polity(polity), worker, grant: true, process: process.map(String::from), cause: None })
}

pub fn authorized(o: &Owned, worker: Id) -> bool {
    o.s.sim.project_authorized(o.s.sim.state().projects.get(o.project).unwrap(), worker)
}

pub fn permission(o: &Owned, worker: Id) -> Option<Permission> {
    o.s.sim.state().projects.get(o.project).unwrap().permissions.get(&worker).cloned()
}

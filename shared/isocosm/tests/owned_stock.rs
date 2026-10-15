// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Native owned stock and projects (rulings 837 to 850), under the stock/title
//! plan. Values are explicit test-world inputs, not production defaults.
//! `owned_stock/controls.py` plants the source faults these tests must catch.

use isocosm::{Command, Execution, Founding, Session, asserted::Assertion,
    projects::{Acted, Catalogue, Change, Definition, LotAt, Owner, Phase, ProjectId, Step, StockLot,
        StockSource},
    rules::{AccountKind, Binding, Causation, Effect, Process, Query},
    schema::{Id, Ledger, Method, Route}, simulation::Genesis, social::{DeedRule, Vocabulary, Work}};
use std::collections::{BTreeMap, BTreeSet};

#[path = "owned_stock/ownership.rs"]
mod ownership;
#[path = "owned_stock/work.rs"]
mod work;
#[path = "owned_stock/consent.rs"]
mod consent;
#[path = "owned_stock/placement.rs"]
mod placement;
#[path = "owned_stock/debit.rs"]
mod debit;

const TIMBER: &str = "test:timber";
const CRAFT: &str = "craft:joinery";
const BENCH: &str = "project:bench";
const RECIPE: &str = "recipe:bench";
const RESERVE: &str = "test:reserve";
const WORK: &str = "test:work";
const HAUL: &str = "test:haul";
const DROP: &str = "test:drop";
const GO: &str = "test:go";
/// One authored step's spend; three steps make the recipe.
const STEP: u64 = 200;
const RECIPE_MG: u64 = 3 * STEP;
const SITE_MG: u64 = 1_000;
const BODY_MG: u64 = 300;
const HAUL_MG: u64 = 150;
const DROP_MG: u64 = 100;

/// Bodies 0 and 1 live at `home`, body 2 at `away`; body 0 carries timber.
struct World { genesis: Genesis, home: Id, away: Id, bodies: [Id; 3] }

fn process(id: &str, requires: Vec<Query>, effects: Vec<Effect>) -> Process {
    Process { id: id.into(), causation: Causation::Choice, requires, commitments: vec![], effects,
        risk: None, target: None, period: None, priority: 0, need_account: None, need_below: 0,
        glyphs: BTreeSet::new(), invariants: BTreeSet::from(["sim:matter".into(), "sim:nonnegative".into()]),
        note: true, port_feeding: None, goal_directives: Vec::new() }
}

fn world(seed: u64) -> World {
    let mut g = Founding { seed, sites: 2, population: 8, lineages: 1, cohort_size: 1, ..Default::default() }
        .generate().unwrap();
    g.rules.processes.clear();
    g.rules.shapes = isocosm::rules::default_shapes();
    g.rules.functions = isocosm::rules::default_functions();
    let mut sites = g.sites.keys().copied();
    let (home, away) = (sites.next().unwrap(), sites.next().unwrap());
    let critters: Vec<Id> = g.population.groups.iter()
        .filter(|(_, group)| group.entity.kingdom != "kingdom:world").map(|(id, _)| *id).take(3).collect();
    let bodies = [critters[0], critters[1], critters[2]];
    let lineage = g.population.groups[&bodies[0]].entity.lineage.clone();
    for (id, at) in bodies.into_iter().zip([home, home, away]) {
        let body = &mut g.population.groups.get_mut(&id).unwrap().entity;
        body.place = at;
        body.method = Method::Normative;
    }
    g.rules.accounts.insert(TIMBER.into(), AccountKind::Matter { lineage: lineage.clone(), reserve: false, provision: false });
    g.population.groups.get_mut(&bodies[0]).unwrap().entity.accounts.insert(TIMBER.into(), BODY_MG);
    g.sites.get_mut(&home).unwrap().accounts.insert(TIMBER.into(), SITE_MG);
    for site in g.sites.values_mut() { site.routes.clear(); }
    g.sites.get_mut(&home).unwrap().routes =
        vec![Route { to: away, travel: 1, transmission: 0, border: None, authored: None }];
    let deeds = isocosm::social::RECORDED.iter()
        .map(|k| (k.to_string(), DeedRule { trust: 0, affinity: 0, phrase: k.to_string() })).collect();
    let mut social = g.rules.social.clone().unwrap_or(Vocabulary { crafts: BTreeSet::new(), deeds });
    social.crafts.insert(CRAFT.into());
    g.rules.social = Some(social);
    g.rules.material_recipes.insert(RECIPE.into(), isocosm::material::Recipe {
        lineage, scale: "scale:meso".into(), stocks: Ledger::from([(TIMBER.into(), RECIPE_MG)]) });
    let step = Step { progress: 1, spend: Ledger::from([(TIMBER.into(), STEP)]) };
    g.rules.projects = Some(Catalogue { limit: 4, definitions: BTreeMap::from([(BENCH.into(), Definition {
        recipe: RECIPE.into(), work: Work::new(CRAFT, 0, 0), total: 3, steps: vec![step; 3], output_custody: None })]) });
    let gate = |phase, id: &str| Query::Project { definition: BENCH.into(), phase, process: id.into() };
    let processes = [
        process(RESERVE, vec![gate(Phase::Reserve, RESERVE)], vec![Effect::ReserveProject {
            definition: BENCH.into(), from: Binding::Place, authority: Binding::Actor, lots: vec![] }]),
        process(WORK, vec![gate(Phase::Work, WORK)], vec![Effect::WorkProject]),
        process(HAUL, vec![], vec![Effect::Transfer { from: Binding::Place, to: Binding::Actor,
            account: TIMBER.into(), amount: HAUL_MG.into() }]),
        process(DROP, vec![], vec![Effect::Spend { from: vec![TIMBER.into()], to: Binding::Place,
            amount: DROP_MG.into(), into: None }]),
        process(GO, vec![], vec![Effect::Move { destination: away }]),
    ];
    for p in processes { g.rules.processes.insert(p.id.clone(), p); }
    g.validate().unwrap();
    World { genesis: g, home, away, bodies }
}

fn join_take(session: &mut Session, body: Id) -> Id {
    let receipt = session.command(Command::Join).unwrap();
    let participant = receipt.strip_prefix("participant:").unwrap().parse().unwrap();
    session.command(Command::Take { participant, critter: body }).unwrap();
    participant
}

fn lot(key: &str, source: StockSource, amount: u64, owners: &[Id]) -> StockLot {
    StockLot { key: key.into(), source, account: TIMBER.into(), amount,
        owners: owners.iter().map(|id| Owner::Entity(*id)).collect(), collectives: BTreeMap::new() }
}

fn assert_lot(session: &mut Session, lot: StockLot) -> isocosm::Result<String> {
    session.command(Command::Assert(Assertion::StockOwnership(lot)))
}

fn act(participant: Id, actor: Id, process: &str, project: Option<ProjectId>, lots: &[&str]) -> Command {
    Command::Project(Change::Act { participant, actor, target: None, process: process.into(), project,
        cause: None, lots: lots.iter().map(|k| k.to_string()).collect() })
}

fn consent(participant: Id, actor: Id, project: ProjectId, worker: Id, grant: bool) -> Command {
    Command::Project(Change::Consent { participant, actor, project, owner: Owner::Entity(actor), worker,
        grant, process: None, cause: None })
}

fn acted(outcome: isocosm::Result<String>) -> Acted { serde_json::from_str(&outcome.unwrap()).unwrap() }

/// A refusal is an error that leaves every saved and project byte as it was.
fn refused(session: &mut Session, command: Command) -> String {
    let (hash, projects) = (session.sim.state_hash(), session.sim.state().projects.clone());
    let why = session.command(command).expect_err("this native command must refuse");
    assert_eq!(session.sim.state_hash(), hash, "refused: {why}");
    assert_eq!(session.sim.state().projects, projects);
    why
}

/// Milligrams across every site, body, place, transit and project ledger,
/// then a save replayed from genesis reaching the identical state hash.
fn exact(session: &Session, mass: u128) {
    assert_eq!(session.sim.matter(), mass, "reserve, construction and output count once");
    let json = serde_json::to_vec(&session.save()).unwrap();
    for execution in [Execution::Individuals, Execution::Grouped] {
        let restored = Session::load_json(&json, execution).unwrap();
        assert_eq!(restored.sim.state_hash(), session.sim.state_hash());
        assert_eq!(restored.sim.state().projects, session.sim.state().projects);
        assert_eq!(restored.sim.matter(), mass);
    }
}

fn timber(session: &Session, source: StockSource) -> u64 {
    session.sim.stock_balance(source, TIMBER).unwrap()
}

fn held(session: &Session, key: &str) -> (u64, LotAt) {
    let held = &session.sim.state().projects.ownership[key];
    (held.lot.amount, held.at)
}

/// Both owners of a fresh reservation, played, with the bench reserved by body 0.
struct Reserved { s: Session, mass: u128, id: ProjectId, players: [Id; 3] }

fn reserved(w: &World) -> Reserved {
    let mut s = Session::new(w.genesis.clone(), Execution::Individuals).unwrap();
    let mass = s.sim.matter();
    let players = w.bodies.map(|body| join_take(&mut s, body));
    let [a, b, _] = w.bodies;
    assert_lot(&mut s, lot("lot:bench", StockSource::Site(w.home), RECIPE_MG, &[a, b])).unwrap();
    let out = acted(s.command(act(players[0], a, RESERVE, None, &["lot:bench"])));
    assert!(out.receipt.accepted(), "{:?}", out.receipt.outcome);
    Reserved { s, mass, id: out.project.unwrap(), players }
}

/// Both owners grant `worker`; the second decision completes the grant.
fn grant(r: &mut Reserved, w: &World, worker: Id) {
    let [a, b, _] = w.bodies;
    r.s.command(consent(r.players[0], a, r.id, worker, true)).unwrap();
    r.s.command(consent(r.players[1], b, r.id, worker, true)).unwrap();
    assert!(r.s.sim.project_authorized(r.s.sim.state().projects.get(r.id).unwrap(), worker));
}

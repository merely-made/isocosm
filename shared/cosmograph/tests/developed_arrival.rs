// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use cosmograph::{Command, Execution, Founding, Session, anatomy};
use cosmograph::arrival::Arrival;
use cosmograph::rules::*;
use cosmograph::schema::*;
use std::collections::BTreeMap;

const LINE: &str = "lineage:arrival-test";
const TISSUE: &str = "arrival-test:tissue";
const RESERVE: &str = "arrival-test:reserve";
const ENERGY: &str = "arrival-test:energy";

fn world(stores: u32) -> Session {
    let mut g = Founding { seed: 818, sites: 1, population: 1, lineages: 1,
        cohort_size: 1, ..Default::default() }.generate().unwrap();
    g.rules.functions.extend(default_functions());
    g.rules.shapes.extend(default_shapes());
    g.rules.accounts.extend([
        (TISSUE.into(), AccountKind::Matter { lineage: LINE.into(), reserve: false, provision: false }),
        (RESERVE.into(), AccountKind::Matter { lineage: LINE.into(), reserve: true, provision: false }),
        (ENERGY.into(), AccountKind::Energy),
    ]);
    g.rules.kinds.insert("kind:arrival-root".into(), Template {
        half_extent: [4; 3], shape: String::new(), port: None, cells: BTreeMap::from([
            ("function:store".into(), stores), ("function:intake".into(), 27-stores),
        ]),
    });
    let recipe = Recipe { tagmata: vec![Tagma {
        chain: Vec::new(),
        segments: 1, segment: "kind:arrival-root".into(), bears: None, per_segment: 0,
        parent: None, anchor: Anchor::Base, facing: Facing::Back, socket: Facing::Right, variance: None,
    }], variance: 0, absence: [0,1], riff: [0,1], vary: [0,1] };
    g.lineages.insert(LINE.into(), Lineage { parent: None, revision: 1,
        kingdom: "kingdom:fauna".into(), traits: Default::default(),
        development: Some(Development { lexicon: recipe.kinds(), recipe, policy: Policy::default(),
            domain: 0, clutch: 1, anamorphic: false, tracts: vec![], systems: Default::default() }),
        decides: None,
    });
    Session::new(g, Execution::Grouped).unwrap()
}

fn arrive(s: &mut Session, tissue: u64, reserve: u64) -> Id {
    let a = Arrival { lineage: LINE.into(), site: *s.sim.state().sites.keys().next().unwrap(),
        method: Method::Normative, accounts: BTreeMap::from([
            (TISSUE.into(), tissue), (RESERVE.into(), reserve), (ENERGY.into(), 100),
        ]), traits: Default::default(), skills: Default::default(), disposition: [0;5], character: None };
    s.command(Command::Arrive(a)).unwrap().strip_prefix("entity:").unwrap().parse().unwrap()
}

#[test]
fn supplied_matter_moves_to_declared_parts_without_issuing_or_losing_stock() {
    let mut s = world(5);
    let id = arrive(&mut s, 583, 100);
    let held = s.sim.state().population.get(id).unwrap();
    let before = anatomy::books(held);
    let conserved = s.sim.matter();
    let revision = held.body_revision;
    s.command(Command::Develop { entity: id, seed: 77 }).unwrap();
    let e = s.sim.state().population.get(id).unwrap();
    assert_eq!(anatomy::books(e), before);
    assert_eq!(s.sim.matter(), conserved);
    assert_eq!(e.parts[&PartId(0)].matter.get(RESERVE).copied(), Some(100), "supplied reserve must live in declared stores");
    assert_eq!(e.parts[&PartId(0)].matter[TISSUE], 583);
    assert!(!e.accounts.contains_key(TISSUE));
    assert!(!e.accounts.contains_key(RESERVE));
    assert_eq!(e.accounts[ENERGY], 100);
    assert_eq!(e.body_revision, revision + 1);
}

fn refusal(stores: u32, tissue: u64, reserve: u64, account: &str) {
    let mut s = world(stores);
    let id = arrive(&mut s, tissue, reserve);
    let hash = s.sim.state_hash();
    let saved = s.save();
    let before = s.sim.state().clone();
    let matter = s.sim.matter();
    let error = s.command(Command::Develop { entity: id, seed: 77 }).unwrap_err();
    assert!(error.contains("no room in the body"), "native overflow reason: {error}");
    assert!(error.contains(account), "native overflow names its account: {error}");
    assert_eq!(s.sim.state(), &before);
    assert_eq!(s.sim.state_hash(), hash);
    assert_eq!(s.sim.matter(), matter);
    assert_eq!(s.save(), saved, "refusal must not append an accepted command");
}

#[test]
fn tissue_overflow_is_atomic_instead_of_clipped() { refusal(5, 584, 100, TISSUE); }
#[test]
fn reserve_overflow_is_atomic_instead_of_clipped() { refusal(5, 583, 106, RESERVE); }
#[test]
fn a_recipe_without_stores_cannot_admit_supplied_reserve() { refusal(0, 583, 1, RESERVE); }

#[test]
fn existing_body_cannot_be_redeveloped_or_have_its_stock_issued_twice() {
    let mut s = world(5);
    let id = arrive(&mut s, 583, 100);
    s.command(Command::Develop { entity: id, seed: 77 }).unwrap();
    let saved = s.save();
    assert!(s.command(Command::Develop { entity: id, seed: 77 }).is_err());
    assert_eq!(s.save(), saved);
}

#[test]
fn native_json_save_replays_the_logged_recipe_and_all_supplied_stock() {
    let mut s = world(5);
    let id = arrive(&mut s, 583, 100);
    s.command(Command::Develop { entity: id, seed: 77 }).unwrap();
    let bytes = serde_json::to_vec(&s.save()).unwrap();
    let restored = Session::load_json(&bytes, Execution::Individuals).unwrap();
    assert_eq!(restored.sim.witness(), s.sim.witness());
    assert_eq!(restored.entries, s.entries);
    assert_eq!(restored.sim.state().population.get(id), s.sim.state().population.get(id));
}

// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::{Founding, Session, Execution, schema::*, simulation::Genesis};
use isometer_space::places::PlaceId;
use std::collections::BTreeSet;

fn fixture() -> (Genesis, Id, Identity, Identity, PlaceId) {
    let mut g = Founding { seed: 17, sites: 3, population: 12, lineages: 1,
        cohort_size: 3, ecology: true, ..Default::default() }.generate().unwrap();
    g.rules.processes.clear();
    g.rules.epoch_ticks = 2;
    g.rules.shapes = crate::rules::default_shapes();
    g.rules.functions = crate::rules::default_functions();
    let id = *g.population.groups.keys().next().unwrap();
    let lineage = g.population.get(id).unwrap().lineage.clone();
    g.rules.traits.insert("function:fix".into());
    g.lineages.get_mut(&lineage).unwrap().traits.insert("function:fix".into());
    let territory = Identity::Territory(id);
    let surface = Identity::Surface(lineage);
    g.spreads.identities = BTreeSet::from([territory.clone(), surface.clone()]);
    for site in g.sites.values_mut() { site.routes.clear(); }
    g.sites.get_mut(&0).unwrap().routes = vec![
        Route { to:1, travel:3, transmission:500_000, border:None, authored:None },
        Route { to:2, travel:5, transmission:800_000, border:None, authored:None },
    ];
    g.sites.get_mut(&0).unwrap().accounts.insert(territory.key(), 100);
    g.sites.get_mut(&0).unwrap().accounts.insert(surface.key(), 50);
    g.population.lift(id).unwrap().accounts.insert(surface.key(), 7);
    let place = PlaceId { site:0, cell:[1,-2,3] };
    g.spreads.places.push(PlaceLedger { place, accounts:Ledger::from([(surface.key(),11)]) });
    g.spreads.transit.push(Transfer { identity:territory.clone(), from:0, to:1,
        departed:0, arrives:3, amount:25 });
    (g,id,territory,surface,place)
}

fn replay(session: &Session) {
    let json = serde_json::to_vec(&session.save()).unwrap();
    for mode in [Execution::Individuals, Execution::Grouped] {
        let loaded = Session::load_json(&json, mode).unwrap();
        assert_eq!(loaded.sim.state_hash(), session.sim.state_hash());
        assert_eq!(loaded.sim.entity_witness(), session.sim.entity_witness());
        assert_eq!(loaded.entries, session.entries);
        assert_eq!(loaded.checkpoints, session.checkpoints);
        assert_eq!(loaded.sim.matter(), session.sim.matter());
    }
}

#[test]
fn site_host_fine_place_and_transit_save_as_typed_json_without_duplicate_mass() {
    let (g,id,territory,surface,place) = fixture();
    let mut ordinary = g.clone();
    ordinary.spreads = Spreads::default();
    ordinary.sites.get_mut(&0).unwrap().accounts.remove(&territory.key());
    ordinary.sites.get_mut(&0).unwrap().accounts.remove(&surface.key());
    ordinary.population.lift(id).unwrap().accounts.remove(&surface.key());
    let baseline = Session::new(ordinary, Execution::Individuals).unwrap().sim.matter();
    let s = Session::new(g, Execution::Individuals).unwrap();
    assert_eq!(s.sim.matter(), baseline + 100 + 50 + 7 + 11 + 25);
    assert_eq!(held(s.sim.state(), Substrate::Place(place), &surface).unwrap(),11);
    assert_eq!(held(s.sim.state(), Substrate::Host(id), &surface).unwrap(),7);
    assert_eq!(functions(s.sim.state(), &s.sim.genesis().rules, Substrate::Site(0), &territory)
        .unwrap()["function:fix"],100);
    let state_json = serde_json::to_vec(s.sim.state()).unwrap();
    let decoded: crate::simulation::State = serde_json::from_slice(&state_json).unwrap();
    assert_eq!(decoded, *s.sim.state(), "structured PlaceId remains a JSON entry, not a map key");
    replay(&s);
}

#[test]
fn conserved_arrivals_and_checkpoints_match_both_tick_engines() {
    let (g,_,territory,_,_) = fixture();
    let mut hashes = Vec::new();
    for mode in [Execution::Individuals,Execution::Grouped] {
        let mut s = Session::new(g.clone(),mode).unwrap();
        let total = s.sim.matter();
        s.advance(2).unwrap();
        assert_eq!(held(s.sim.state(),Substrate::Site(1),&territory).unwrap(),0);
        assert_eq!(s.sim.state().spreads.transit.len(),1);
        replay(&s);
        s.advance(1).unwrap();
        assert_eq!(held(s.sim.state(),Substrate::Site(1),&territory).unwrap(),25);
        assert!(s.sim.state().spreads.transit.is_empty());
        assert_eq!(s.sim.matter(),total);
        s.advance(3).unwrap();
        replay(&s);
        hashes.push(s.sim.state_hash());
    }
    assert_eq!(hashes[0],hashes[1]);
}

#[test]
fn native_strain_selection_is_seeded_local_conserved_and_replayable() {
    let (g,id,_,surface,place) = fixture();
    let parent = surface.key();
    let traits = BTreeSet::from(["function:fix".into()]);
    let mut keys = Vec::new();
    for mode in [Execution::Individuals,Execution::Grouped] {
        let mut s = Session::new(g.clone(),mode).unwrap();
        let total = s.sim.matter();
        let key = s.command(crate::Command::Spread(Command::ForkStrain {
            parent:parent.clone(), at:Substrate::Host(id), amount:3, traits:traits.clone(),
        })).unwrap();
        assert_eq!(key,strain_key(g.dynamics_seed(),&parent,Substrate::Host(id),&traits));
        let child = Identity::Surface(key.clone());
        assert_eq!(s.sim.state().lineages[&key].parent,Some(parent.clone()));
        assert_eq!(held(s.sim.state(),Substrate::Host(id),&surface).unwrap(),4);
        assert_eq!(held(s.sim.state(),Substrate::Host(id),&child).unwrap(),3);
        assert_eq!(held(s.sim.state(),Substrate::Place(place),&surface).unwrap(),11);
        assert_eq!(functions(s.sim.state(),&g.rules,Substrate::Host(id),&child)
            .unwrap()["function:fix"],3);
        assert_ne!(key,strain_key(g.dynamics_seed(),&parent,Substrate::Place(place),&traits));
        assert_eq!(s.sim.matter(),total);
        s.advance(4).unwrap();
        replay(&s);
        keys.push(key);
    }
    assert_eq!(keys[0],keys[1]);
}

#[test]
fn invalid_identity_strain_and_arrival_credit_refuse_atomically() {
    let (mut g,id,territory,surface,_) = fixture();
    g.spreads.identities.remove(&surface);
    g.sites.get_mut(&0).unwrap().accounts.remove(&surface.key());
    g.population.lift(id).unwrap().accounts.remove(&surface.key());
    g.spreads.places.clear();
    let mut s = Session::new(g,Execution::Grouped).unwrap();
    let before = s.save();
    assert!(s.command(crate::Command::Spread(Command::Register(Identity::Territory(u64::MAX)))).is_err());
    assert!(s.command(crate::Command::Spread(Command::Register(Identity::Surface("lineage:absent".into())))).is_err());
    assert_eq!(s.save(),before);
    s.command(crate::Command::Spread(Command::Register(surface))).unwrap();
    replay(&s);
    let (g,id,_,surface,_) = fixture();
    let mut s = Session::new(g,Execution::Individuals).unwrap();
    let before = s.save();
    assert!(s.command(crate::Command::Spread(Command::ForkStrain { parent:surface.key(),
        at:Substrate::Host(id), amount:8, traits:BTreeSet::new() })).is_err());
    assert_eq!(s.save(),before);
    let (mut g,_,_,_,_) = fixture();
    g.sites.get_mut(&1).unwrap().accounts.insert(territory.key(),u64::MAX);
    for mode in [Execution::Individuals,Execution::Grouped] {
        let mut s = Session::new(g.clone(),mode).unwrap();
        let before = s.save();
        assert!(s.advance(4).is_err());
        assert_eq!(s.save(),before,"arrival refusal restores tick, queue, ledgers and checkpoints");
    }
}

#[test]
fn pure_overflow_kernel_retains_untransmitted_remainders_and_never_duplicates_routes() {
    let (g,_,territory,_,_) = fixture();
    let mut state = Session::new(g.clone(),Execution::Individuals).unwrap().sim.state().clone();
    let before = crate::simulation::matter(&state,&g.rules);
    assert_eq!(overflow(&mut state,&territory,0,20,&[
        Dispatch {route:0,offered:31}, Dispatch {route:1,offered:49},
    ]).unwrap(),54);
    assert_eq!(held(&state,Substrate::Site(0),&territory).unwrap(),46);
    assert_eq!(crate::simulation::matter(&state,&g.rules),before);
    assert_eq!(settle(&mut state,3).unwrap(),2);
    assert_eq!(held(&state,Substrate::Site(1),&territory).unwrap(),40);
    assert_eq!(crate::simulation::matter(&state,&g.rules),before);
    let snapshot = state.clone();
    assert!(overflow(&mut state,&territory,0,0,&[
        Dispatch {route:0,offered:1}, Dispatch {route:0,offered:1},
    ]).is_err());
    assert_eq!(state,snapshot);
    assert!(overflow(&mut state,&territory,0,0,&[Dispatch {route:1,offered:47}]).is_err());
    assert_eq!(state,snapshot);
}

#[test]
fn founding_refuses_account_aliases_and_territory_host_duplication() {
    let (mut g,id,territory,_,_) = fixture();
    g.population.lift(id).unwrap().accounts.insert(territory.key(),1);
    assert!(g.validate().is_err());
    let (mut g,_,territory,_,_) = fixture();
    g.rules.accounts.insert(territory.key(),crate::rules::AccountKind::Matter {
        lineage:g.population.get(id).unwrap().lineage.clone(), reserve:false, provision:false,
    });
    assert!(g.validate().is_err());
    let (mut g,_,_,_,_) = fixture();
    g.spreads.places.push(g.spreads.places[0].clone());
    assert!(g.validate().is_err());
}

#[test]
fn a_full_existing_strain_ledger_does_not_publish_a_partial_fork() {
    let (mut g,id,_,surface,_) = fixture();
    let parent = surface.key();
    let traits = BTreeSet::from(["function:fix".into()]);
    let key = strain_key(g.dynamics_seed(),&parent,Substrate::Host(id),&traits);
    let original = g.lineages[&parent].clone();
    g.lineages.insert(key.clone(),Lineage { parent:Some(parent.clone()), traits:traits.clone(),
        revision:original.revision + 1, ..original });
    g.spreads.identities.insert(Identity::Surface(key.clone()));
    g.population.lift(id).unwrap().accounts.insert(key,u64::MAX);
    let mut s = Session::new(g,Execution::Grouped).unwrap();
    let before = s.save();
    assert!(s.command(crate::Command::Spread(Command::ForkStrain {
        parent,at:Substrate::Host(id),amount:1,traits,
    })).is_err());
    assert_eq!(s.save(),before,"failed credit must restore the selected parent stock and history");
}

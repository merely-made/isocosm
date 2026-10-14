// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Independent C11 histogram/stock clock against public native Session.
//! These are not a renaming of Execution::Grouped.

#[path="checkpoint11_crowd/fixtures.rs"]
mod fixtures;
#[path="checkpoint11_crowd/direction.rs"]
mod direction;
use fixtures::*;
use isocosm::{Execution,Session,probe::{Crowd,Variant,readings::spreads},schema::Route,
    spread::{self,CapacitySource,Identity,Substrate,Supply,Placement}};
use std::collections::BTreeSet;

#[test]
fn ordinary_crowd_refuses_spread_founding_instead_of_dropping_it() {
    let mut w = world(17);
    assert!(Crowd::new(&w.probe,w.probe.genesis.dynamics_seed(),Variant::Histogram)
        .err().unwrap().contains("spread founding"));
    w.probe.genesis.spreads = Default::default();
    let ordinary = Crowd::new(&w.probe,w.probe.genesis.dynamics_seed(),Variant::Histogram).unwrap().run().unwrap();
    let independent = Crowd::for_spreads(&w.probe,w.probe.genesis.dynamics_seed(),&BTreeSet::new()).unwrap().run().unwrap();
    assert_eq!(ordinary.bins,independent.bins);
    assert_eq!(ordinary.sites,independent.sites);
    assert_eq!(ordinary.tick,independent.tick);
}

#[test]
fn independent_territory_split_preserves_zero_route_and_rounding_residues() {
    let w = world(17);
    let mut p = Pair::new(&w,&BTreeSet::new());
    p.place(&w,&w.territory,Substrate::Site(w.from),101);
    let read = spreads::crowd(&p.crowd).unwrap();
    assert_eq!(p.crowd.spread_held(Substrate::Site(w.from),&w.territory).unwrap(),41);
    assert_eq!(read.travelling.iter().map(|t| t.amount).sum::<u128>(),60);
    assert_eq!(read.travelling.iter().map(|t| t.to).collect::<BTreeSet<_>>(),BTreeSet::from([w.to[0],w.to[1]]));
    assert_eq!(read.stocks.iter().find(|s| s.identity == w.territory).unwrap().functions["function:fix"],41);
    p.replay();
}

#[test]
fn independent_surface_delivery_credits_only_after_authored_travel() {
    let w = world(23);
    let mut p = Pair::new(&w,&BTreeSet::new());
    p.place(&w,&w.surface,Substrate::Site(w.from),101);
    p.advance(1);
    for to in w.to {assert_eq!(p.crowd.spread_held(Substrate::Site(to),&w.surface).unwrap(),0);}
    p.advance(1);
    assert_eq!(p.crowd.spread_held(Substrate::Site(w.to[0]),&w.surface).unwrap(),20);
    assert_eq!(p.crowd.spread_held(Substrate::Site(w.to[1]),&w.surface).unwrap(),0);
    p.advance(1);
    assert_eq!(p.crowd.spread_held(Substrate::Site(w.to[1]),&w.surface).unwrap(),40);
    assert!(spreads::crowd(&p.crowd).unwrap().travelling.is_empty());
    p.replay();
}

#[test]
fn independent_host_stock_is_local_and_not_a_second_site_ledger() {
    let w = world(41);
    let mut p = Pair::new(&w,&BTreeSet::from([w.host+1]));
    p.place(&w,&w.surface,Substrate::Host(w.host),61);
    assert_eq!(p.crowd.spread_held(Substrate::Host(w.host),&w.surface).unwrap(),31);
    assert_eq!(p.crowd.spread_held(Substrate::Host(w.host+1),&w.surface).unwrap(),0);
    assert_eq!(p.crowd.spread_held(Substrate::Site(w.from),&w.surface).unwrap(),0);
    let read = spreads::crowd(&p.crowd).unwrap();
    assert_eq!(read.travelling.iter().map(|t| t.amount).sum::<u128>(),30);
    p.advance(3);
    p.replay();
}

#[test]
fn counted_hosts_round_each_offer_before_multiplying_the_bin() {
    let mut w = world(17);
    let original = w.probe.genesis.population.get(w.host).unwrap().clone();
    w.probe.genesis.population.groups.values_mut().find(|g| g.entity == original).unwrap()
        .entity.accounts.insert(w.surface.key(),51);
    w.probe.genesis.spreads.modifiers.push(modifier("modifier:world",CapacitySource::World,None,100));
    w.probe.genesis.validate().unwrap();
    let mut p = Pair::new(&w,&BTreeSet::new());
    p.command(spread::Command::RemoveModifier {key:"modifier:world".into()});
    let read = spreads::crowd(&p.crowd).unwrap();
    let host_bin = read.hosts.iter().find(|(e,_)| e.accounts.contains_key(&w.surface.key())).unwrap();
    assert_eq!((host_bin.0.accounts[&w.surface.key()],host_bin.1),(29,w.cohort-1));
    assert_eq!(read.travelling.iter().map(|t| t.amount).sum::<u128>(),22*u128::from(w.cohort));
    p.advance(3);
    p.replay();
}

#[test]
fn addressed_host_modifier_does_not_average_over_equal_hosts() {
    let mut w = world(17);
    let original = w.probe.genesis.population.get(w.host).unwrap().clone();
    w.probe.genesis.population.groups.values_mut().find(|g| g.entity == original).unwrap()
        .entity.accounts.insert(w.surface.key(),51);
    w.probe.genesis.spreads.modifiers = vec![
        modifier("modifier:host",CapacitySource::Substrate(Substrate::Host(w.host+1)),Some(w.surface.clone()),100),
        modifier("modifier:world",CapacitySource::World,None,100),
    ];
    w.probe.genesis.validate().unwrap();
    let mut p = Pair::new(&w,&BTreeSet::new());
    p.command(spread::Command::RemoveModifier {key:"modifier:world".into()});
    assert_eq!(p.crowd.spread_held(Substrate::Host(w.host+1),&w.surface).unwrap(),51);
    assert_eq!(p.crowd.spread_held(Substrate::Host(w.host),&w.surface).unwrap(),29);
    p.advance(3);
    p.replay();
}

#[test]
fn lower_capacity_spills_once_across_clock_partitions() {
    let w = world(23);
    let mut one = Pair::new(&w,&BTreeSet::new());
    let mut split = Pair::new(&w,&BTreeSet::new());
    for p in [&mut one,&mut split] {
        p.command(spread::Command::SetModifier(modifier("modifier:world",CapacitySource::World,None,100)));
        p.place(&w,&w.surface,Substrate::Site(w.from),101);
        assert!(spreads::crowd(&p.crowd).unwrap().travelling.is_empty());
        p.command(spread::Command::RemoveModifier {key:"modifier:world".into()});
    }
    one.advance(7);
    for _ in 0..7 {split.advance(1);}
    assert_eq!(spreads::crowd(&one.crowd).unwrap(),spreads::crowd(&split.crowd).unwrap());
    assert_eq!(one.crowd.spread_held(Substrate::Site(w.from),&w.surface).unwrap(),41);
    one.replay();
    split.replay();
}

#[test]
fn receiving_overflow_departs_at_delivery_tick() {
    let mut w = world(41);
    w.probe.genesis.sites.get_mut(&w.from).unwrap().routes[0].transmission = 1_000_000;
    w.probe.genesis.sites.get_mut(&w.to[0]).unwrap().routes.push(Route {
        to:w.to[2],travel:2,transmission:1_000_000,border:None,authored:None});
    let mut p = Pair::new(&w,&BTreeSet::new());
    p.place(&w,&w.surface,Substrate::Site(w.from),101);
    p.advance(2);
    let read = spreads::crowd(&p.crowd).unwrap();
    let onward = read.travelling.iter().find(|t| t.from == w.to[0]).unwrap();
    assert_eq!((onward.departed,onward.arrives,onward.amount),(2,4,20));
    p.advance(1);
    assert_eq!(p.crowd.spread_held(Substrate::Site(w.to[2]),&w.surface).unwrap(),0);
    p.advance(1);
    assert_eq!(p.crowd.spread_held(Substrate::Site(w.to[2]),&w.surface).unwrap(),20);
    p.replay();
}

#[test]
fn seeded_strain_forks_only_selected_host_and_retains_local_functions() {
    let mut identities = BTreeSet::new();
    for seed in [17,23,41] {
        let w = world(seed);
        let mut p = Pair::new(&w,&BTreeSet::from([w.host+1]));
        p.place(&w,&w.surface,Substrate::Host(w.host),61);
        p.place(&w,&w.surface,Substrate::Host(w.host+1),11);
        let parent = w.surface.key();
        let traits = BTreeSet::from([GENE.into(),"function:fix".into()]);
        let child = p.command(spread::Command::ForkStrain {parent:parent.clone(),at:Substrate::Host(w.host),amount:7,traits:traits.clone()});
        assert_eq!(child,spread::strain_key(w.probe.genesis.dynamics_seed(),&parent,Substrate::Host(w.host),&traits));
        let strain = Identity::Surface(child.clone());
        assert_eq!(p.crowd.spread_held(Substrate::Host(w.host),&strain).unwrap(),7);
        assert_eq!(p.crowd.spread_held(Substrate::Host(w.host+1),&strain).unwrap(),0);
        assert_eq!(p.crowd.spread_held(Substrate::Host(w.host+1),&w.surface).unwrap(),11);
        let read = spreads::crowd(&p.crowd).unwrap();
        assert_eq!(read.lineages[&child].parent.as_ref(),Some(&parent));
        assert_eq!(read.stocks.iter().find(|s| s.identity == strain).unwrap().functions["function:fix"],7);
        assert!(identities.insert(child));
        p.advance(5);
        p.replay();
    }
}

#[test]
fn modifier_and_local_supply_refusals_publish_no_partial_candidate() {
    let w = world(17);
    let mut p = Pair::new(&w,&BTreeSet::new());
    let mut invalid = modifier("modifier:absent",CapacitySource::World,None,1);
    invalid.layer = "capacity:unknown".into();
    p.refused(spread::Command::SetModifier(invalid));
    p.refused(spread::Command::Place(Placement {source:Supply::Matter {
        from:Substrate::Site(w.from),account:SUPPLY.into()},identity:w.surface.clone(),
        at:Substrate::Site(w.to[0]),amount:1}));
    p.refused(spread::Command::ForkStrain {parent:w.surface.key(),at:Substrate::Host(w.host),
        amount:1,traits:BTreeSet::from([GENE.into()])});
    assert_eq!(p.native.sim.state().sites[&w.from].accounts[SUPPLY],500);
}

#[test]
fn unsupported_fine_place_founding_refuses_without_a_lossy_projection() {
    use isometer_space::places::PlaceId;
    let mut w = world(17);
    w.probe.genesis.spreads.places.push(spread::PlaceLedger {
        place:PlaceId {site:w.from,cell:[1,2,3]},accounts:Default::default()});
    w.probe.genesis.validate().unwrap();
    assert!(Session::new(w.probe.genesis.clone(),Execution::Individuals).is_ok());
    assert!(Crowd::for_spreads(&w.probe,w.probe.genesis.dynamics_seed(),&BTreeSet::new())
        .err().unwrap().contains("site grain"));
}

#[test]
fn unsupported_titled_transit_refuses_without_discarding_the_mark() {
    let mut w = world(17);
    w.probe.genesis.spreads.transit.push(spread::Transfer {identity:w.surface.clone(),from:w.from,
        to:w.to[0],departed:0,arrives:2,amount:3});
    w.probe.genesis.spreads.transit_marks.push(spread::TransitMark {
        id:spread::TransitId(0),index:0,source:Substrate::Site(w.from),route:0});
    w.probe.genesis.spreads.next_transit = 1;
    w.probe.genesis.validate().unwrap();
    assert!(Session::new(w.probe.genesis.clone(),Execution::Individuals).is_ok());
    assert!(Crowd::for_spreads(&w.probe,w.probe.genesis.dynamics_seed(),&BTreeSet::new()).is_err());
}

#[test]
fn failed_delivery_keeps_clock_queue_and_all_stock_unchanged() {
    let mut w = world(23);
    w.probe.genesis.sites.get_mut(&w.to[0]).unwrap().accounts.insert(w.surface.key(),u64::MAX);
    w.probe.genesis.validate().unwrap();
    let mut p = Pair::new(&w,&BTreeSet::new());
    p.place(&w,&w.surface,Substrate::Site(w.from),101);
    p.advance(1);
    let native = p.native.save();
    let crowd = spreads::crowd(&p.crowd).unwrap();
    assert!(p.native.advance(1).is_err());
    assert!(p.crowd.advance_spreads(1).is_err());
    assert_eq!(p.native.save(),native);
    assert_eq!(spreads::crowd(&p.crowd).unwrap(),crowd);
    p.compare();
}

#[test]
fn site_strain_is_local_and_its_changed_traits_resolve_native_capacity() {
    let w = world(41);
    let mut p = Pair::new(&w,&BTreeSet::new());
    p.place(&w,&w.surface,Substrate::Site(w.from),19);
    let child = p.command(spread::Command::ForkStrain {parent:w.surface.key(),at:Substrate::Site(w.from),
        amount:7,traits:BTreeSet::from(["function:fix".into()])});
    let child = Identity::Surface(child);
    assert_eq!(p.crowd.spread_capacity(Substrate::Site(w.from),&child).unwrap(),0);
    assert_eq!(p.crowd.spread_held(Substrate::Site(w.from),&child).unwrap(),3);
    assert_eq!(p.crowd.spread_held(Substrate::Site(w.from),&w.surface).unwrap(),12);
    p.advance(3);
    p.replay();
}

#[test]
fn drawn_stock_timelines_compare_against_actual_native_individuals() {
    for seed in 0..32 {
        let w = world(seed);
        let mut p = Pair::new(&w,&BTreeSet::from([w.host+1]));
        let site_amount = 21+seed*3;
        let host_amount = 11+seed;
        p.place(&w,&w.territory,Substrate::Site(w.from),site_amount);
        p.place(&w,&w.surface,Substrate::Host(w.host+1),host_amount);
        p.command(spread::Command::ForkStrain {parent:w.surface.key(),at:Substrate::Host(w.host+1),
            amount:3,traits:BTreeSet::from([GENE.into(),"function:fix".into()])});
        for _ in 0..5 {p.advance(1);}
        p.replay();
    }
}

#[test]
fn scheduled_body_founding_is_not_silently_treated_as_a_stock_only_clock() {
    let mut w = world(17);
    let mut authored = isocosm::Founding {seed:17,sites:4,population:24,lineages:1,
        cohort_size:8,..Default::default()}.generate().unwrap().rules.processes;
    let (key,mut process) = authored.pop_first().unwrap();
    process.period = Some(1);
    w.probe.genesis.rules.processes.insert(key,process);
    w.probe.genesis.validate().unwrap();
    assert!(Session::new(w.probe.genesis.clone(),Execution::Individuals).is_ok());
    assert!(Crowd::for_spreads(&w.probe,w.probe.genesis.dynamics_seed(),&BTreeSet::new())
        .err().unwrap().contains("scheduled body"));
}

#[test]
fn overlapping_replace_modifier_refuses_the_entire_candidate() {
    let mut w = world(17);
    w.probe.genesis.rules.spread_capacity.as_mut().unwrap().layers[0].stacking = spread::Stacking::Replace;
    let mut p = Pair::new(&w,&BTreeSet::new());
    p.command(spread::Command::SetModifier(modifier("modifier:world",CapacitySource::World,None,100)));
    p.refused(spread::Command::SetModifier(modifier("modifier:site",
        CapacitySource::Substrate(Substrate::Site(w.from)),Some(w.surface.clone()),1)));
}

#[test]
fn autonomous_territory_direction_runs_exact_against_native_choice() {
    use isocosm::schema::Method;
    let (mut deliberative,mut reactive) = (0,0);
    for seed in 0..24 {
        deliberative += direction::timeline(seed,Method::Deliberative).len();
        reactive += direction::timeline(seed,Method::Reactive).len();
    }
    assert!(deliberative > 0 && reactive > deliberative,"{deliberative} chosen, {reactive} free");
}

#[test]
fn a_deliberative_territory_takes_the_first_feasible_option_of_highest_priority() {
    use isocosm::schema::Method;
    // The zero-transmission route is infeasible, so its higher priority loses;
    // of the two feasible options at one priority, the first identity runs.
    let w = direction::fixed(Method::Deliberative,&[("c11:direct-a",1,1),("c11:direct-b",0,1),
        ("c11:direct-c",2,5)],61);
    let mut p = Pair::new(&w,&BTreeSet::new());
    p.advance(1);
    assert_eq!(direction::ran(&p),["c11:direct-a"]);
    p.advance(2);
    p.replay();
}

#[test]
fn a_free_territory_runs_each_ready_direction_in_pass_order() {
    use isocosm::schema::Method;
    // The half-transmitting route leaves half its offer, which the next
    // ready direction of the same tick then sends on.
    let w = direction::fixed(Method::Reactive,&[("c11:direct-a",0,0),("c11:direct-b",1,0)],161);
    let mut p = Pair::new(&w,&BTreeSet::new());
    p.advance(1);
    assert_eq!(direction::ran(&p),["c11:direct-a","c11:direct-b"]);
    p.advance(2);
    p.replay();
}

#[test]
fn normative_and_unaddressed_direction_refuse_in_the_crowd() {
    use isocosm::schema::Method;
    let w = direction::fixed(Method::Normative,&[("c11:direct-a",1,0)],61);
    let mut crowd = Crowd::for_spreads(&w.probe,w.probe.genesis.dynamics_seed(),&BTreeSet::new()).unwrap();
    assert!(crowd.advance_spreads(1).err().unwrap().contains("882"));
}
